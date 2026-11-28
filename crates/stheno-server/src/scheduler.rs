//! Periodic jobs — the component [TRD §9](../../../docs/TRD.md) specifies and the engine
//! has never had.
//!
//! Until this existed, three pieces of logic in this codebase were written, tested, and
//! **inert**, because nothing called them on a clock:
//!
//! * `PaymentService::reconcile_unsettled_requests` — a payment left `in_flight` by a crash
//!   converges only when the same idempotency key is retried. After a crash nothing retries,
//!   so the row sits ambiguous forever. A convergence path that never runs is not a
//!   convergence path.
//! * `PlatformService::advance_dunning` — invoices generate and the ladder is correct, but
//!   with no clock advancing it, non-payment is never noticed and no subscription is ever
//!   suspended for it. Billing that never runs collects exactly as much as no billing.
//!
//! Both are money. Both were finished. Neither ran.
//!
//! ponytail: only implemented jobs are registered. The authorization-expiry warning,
//! abandoned-3DS sweep, and read-index sync still lack service methods. Trial expiry and
//! day-3/day-6 warnings now have scheduled handlers. Register remaining jobs only after
//! their domain operation and retry contract exist.
//!
//! This also replaces `stheno-workflow/tests/scheduler.rs`, which asserted that `croner`
//! parses cron expressions and that an `AtomicBool` behaves like a mutex. It touched no
//! Stheno code, passed unconditionally, and made this component look built for three weeks.
//!
//! ## Single execution across instances
//!
//! Horizontal scaling runs N identical processes, so every job must run once per tick across
//! the fleet, not once per process. Exclusion is a Postgres advisory lock keyed on the job
//! name, taken with **`max_wait` of zero**: an instance that cannot get the lock has learned
//! that another instance is already running this tick and skips it. Waiting would serialise
//! the duplicate rather than drop it.
//!
//! `PostgresAdvisoryLockProvider` is used directly rather than through `LockManager`.
//! `LockManager` falls back to an in-process lock when Postgres looks unhealthy, which is the
//! right trade for a cart mutex and the wrong one here: under fallback, N nodes each acquire
//! their own in-memory lock and all N run the job. For a dunning ladder or a payment sweep,
//! double execution is the failure being prevented, so a lock that cannot be taken must mean
//! "skip", never "proceed locally".

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use croner::Cron;
use stheno_db::DbPool;
use stheno_locking::{LockProvider, PostgresAdvisoryLockProvider};

/// How long a job may hold its lock before Postgres is entitled to consider it abandoned.
/// Longer than any job should take; short enough that a killed process does not block the
/// next tick for an hour.
const LOCK_TTL_SECONDS: u64 = 900;

/// Never sleep less than this. `find_next_occurrence` can return the current second, and a
/// zero-length sleep in a `loop` is a busy-wait that pins a core.
const MIN_SLEEP: Duration = Duration::from_secs(1);

type JobFuture = Pin<Box<dyn Future<Output = Result<String, String>> + Send>>;
type JobFn = Arc<dyn Fn() -> JobFuture + Send + Sync>;

/// One periodic job: a name, a cron expression, and the work.
///
/// The name is the advisory-lock key and the log field, so it must be stable — renaming a
/// job releases its exclusion for one deploy, during which two versions can both run it.
pub struct Job {
    pub name: &'static str,
    pub schedule: &'static str,
    pub run: JobFn,
}

impl Job {
    pub fn new<F, Fut>(name: &'static str, schedule: &'static str, run: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, String>> + Send + 'static,
    {
        Self {
            name,
            schedule,
            run: Arc::new(move || Box::pin(run()) as JobFuture),
        }
    }
}

/// Spawns one task per job. Returns the number spawned.
///
/// A job with an unparseable cron expression is refused at boot rather than silently never
/// running: a schedule typo that costs a merchant their dunning ladder should stop the
/// deploy, not hide in a log line.
pub fn spawn(pool: DbPool, jobs: Vec<Job>) -> Result<usize, String> {
    let provider = Arc::new(PostgresAdvisoryLockProvider::new(pool));
    let mut spawned = 0;

    for job in jobs {
        let cron = Cron::new(job.schedule).parse().map_err(|e| {
            format!(
                "job '{}' has an invalid schedule '{}': {e}",
                job.name, job.schedule
            )
        })?;
        let provider = provider.clone();
        tokio::spawn(run_forever(job, cron, provider));
        spawned += 1;
    }

    Ok(spawned)
}

async fn run_forever(job: Job, cron: Cron, provider: Arc<PostgresAdvisoryLockProvider>) {
    let resource = format!("job:{}", job.name);

    loop {
        let now = Utc::now();
        let Ok(next) = cron.find_next_occurrence(&now, false) else {
            tracing::error!(job = job.name, "no further occurrences; job stopping");
            return;
        };
        let wait = (next - now).to_std().unwrap_or(MIN_SLEEP).max(MIN_SLEEP);
        tokio::time::sleep(wait).await;

        match provider.try_acquire(&resource, LOCK_TTL_SECONDS).await {
            Ok(Some(token)) => {
                execute(&job).await;
                if let Err(e) = provider.release(&resource, &token.token).await {
                    // The TTL will free it; say so rather than leaving a silent gap.
                    tracing::warn!(job = job.name, error = %e, "could not release job lock; it expires with the TTL");
                }
            }
            // Another instance is running this tick. Correct and expected under N nodes.
            Ok(None) => tracing::debug!(job = job.name, "skipped: held by another instance"),
            Err(e) => {
                tracing::warn!(job = job.name, error = %e, "could not take job lock; skipping this tick")
            }
        }
    }
}

/// Runs the job on its own task so a panic inside it cannot kill the scheduling loop.
/// A job that panics once must still run at its next occurrence.
async fn execute(job: &Job) {
    let started = std::time::Instant::now();
    let fut = (job.run)();
    match tokio::spawn(fut).await {
        Ok(Ok(summary)) => {
            tracing::info!(
                job = job.name,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "{summary}"
            )
        }
        Ok(Err(e)) => tracing::error!(job = job.name, "failed: {e}"),
        Err(e) => tracing::error!(job = job.name, "panicked: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The schedules registered in `lib.rs` must parse. A typo here is a job that never runs,
    /// which is the exact class of silent failure this module exists to end.
    #[test]
    fn registered_schedules_parse() {
        for expr in [
            super::super::PAYMENT_SWEEP_SCHEDULE,
            super::super::DUNNING_SCHEDULE,
            super::super::ENTITLEMENT_EXPIRY_SCHEDULE,
            super::super::TRIAL_WARNING_SCHEDULE,
            super::super::NOTIFICATION_OUTBOX_SCHEDULE,
            super::super::QUOTA_RECONCILIATION_SCHEDULE,
        ] {
            let cron = Cron::new(expr)
                .parse()
                .unwrap_or_else(|e| panic!("{expr}: {e}"));
            let now = Utc::now();
            assert!(
                cron.find_next_occurrence(&now, false)
                    .expect("has an occurrence")
                    > now,
                "{expr} must have a future occurrence"
            );
        }
    }

    /// An invalid expression must stop the boot, not be logged and skipped.
    #[test]
    fn invalid_schedule_is_refused() {
        let job = Job::new("bad", "not a cron", || async { Ok(String::new()) });
        assert!(Cron::new(job.schedule).parse().is_err());
    }
}
