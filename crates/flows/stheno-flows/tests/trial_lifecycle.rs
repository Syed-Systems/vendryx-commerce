use chrono::{Duration, Utc};
use stheno_flows::{TrialLifecycleJob, TrialStatus};

#[test]
fn test_trial_lifecycle_warnings_and_expiration_to_free() {
    let started_at = Utc::now();
    let mut job = TrialLifecycleJob::new(started_at);

    // Day 3 -> Active
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(3)),
        TrialStatus::Active
    );

    // Day 7 -> WarningDay7 (7 days remaining)
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(7)),
        TrialStatus::WarningDay7
    );

    // Day 13 -> WarningDay13 (1 day remaining / final warning)
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(13)),
        TrialStatus::WarningDay13
    );

    // Day 14 -> ExpiredToFree
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(14)),
        TrialStatus::ExpiredToFree
    );

    // Idempotent re-evaluation at Day 15 -> still ExpiredToFree, no new duplicate logs
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(15)),
        TrialStatus::ExpiredToFree
    );

    // Verify audit logs were written for transitions
    assert_eq!(job.audit_logs.len(), 3);
    assert!(job.audit_logs[0].contains("Day 7"));
    assert!(job.audit_logs[1].contains("Day 13"));
    assert!(job.audit_logs[2].contains("Day 14"));
}
