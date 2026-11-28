use chrono::{Duration, Utc};
use stheno_flows::{TrialLifecycleJob, TrialStatus};

#[test]
fn test_trial_lifecycle_warnings_and_expiration_to_free() {
    let started_at = Utc::now();
    let mut job = TrialLifecycleJob::new(started_at);

    // The start instant is active; a clock skew before it cannot expire a trial.
    assert_eq!(job.evaluate_lifecycle_at(started_at), TrialStatus::Active);
    assert_eq!(
        job.evaluate_lifecycle_at(started_at - Duration::nanoseconds(1)),
        TrialStatus::Active
    );

    // Day 3 -> First warning.
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(3)),
        TrialStatus::WarningFirst
    );

    // Day 6 -> Final warning, one full day before expiry.
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(6)),
        TrialStatus::WarningFinal
    );
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(4)),
        TrialStatus::WarningFinal,
        "clock skew must not regress a recorded lifecycle state"
    );

    // Expiry is exclusive: one nanosecond before is still trial; the exact instant is Free.
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(7) - Duration::nanoseconds(1)),
        TrialStatus::WarningFinal
    );
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(7)),
        TrialStatus::ExpiredToFree
    );

    // Idempotent re-evaluation after expiry produces no duplicate audit entry.
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(8)),
        TrialStatus::ExpiredToFree
    );

    // Verify audit logs were written for transitions
    assert_eq!(job.audit_logs.len(), 3);
    assert!(job.audit_logs[0].contains("Day 3"));
    assert!(job.audit_logs[1].contains("Day 6"));
    assert!(job.audit_logs[2].contains("Day 7"));
}

#[test]
fn persisted_legacy_trial_expiry_is_honored_without_shortening() {
    let started_at = Utc::now();
    let ends_at = started_at + Duration::days(14);
    let mut job = TrialLifecycleJob::with_trial_ends_at(started_at, ends_at)
        .expect("existing fourteen-day trial term");

    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(7)),
        TrialStatus::WarningFirst
    );
    assert_eq!(
        job.evaluate_lifecycle_at(ends_at - Duration::nanoseconds(1)),
        TrialStatus::WarningFinal
    );
    assert_eq!(
        job.evaluate_lifecycle_at(ends_at),
        TrialStatus::ExpiredToFree
    );
    assert!(TrialLifecycleJob::with_trial_ends_at(started_at, started_at).is_err());
}
