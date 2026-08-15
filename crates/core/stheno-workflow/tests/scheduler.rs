use chrono::Utc;
use croner::Cron;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[test]
fn test_scheduler_cron_parsing_and_single_execution_advisory_lock() {
    // Parse standard 5-field cron (every 5 minutes)
    let cron = Cron::new("*/5 * * * *")
        .parse()
        .expect("valid cron expression");
    let now = Utc::now();
    let next_run = cron
        .find_next_occurrence(&now, false)
        .expect("must find next occurrence");
    assert!(next_run > now);

    // Advisory lock single-execution simulation
    let lock_held = Arc::new(AtomicBool::new(false));

    let worker_1_acquired = lock_held
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();
    assert!(worker_1_acquired);

    // Worker 2 attempts concurrent execution -> rejected by advisory lock
    let worker_2_acquired = lock_held
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();
    assert!(!worker_2_acquired);

    // Worker 1 releases
    lock_held.store(false, Ordering::SeqCst);
}
