use chrono::{Duration, Utc};
use stheno_flows::{TrialLifecycleJob, TrialStatus};

#[test]
fn test_trial_lifecycle_warnings_and_expiration_to_free() {
    let started_at = Utc::now();
    let mut job = TrialLifecycleJob::new(started_at);

    // Day 10 -> Active
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(10)),
        TrialStatus::Active
    );

    // Day 24 -> WarningDay23
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(24)),
        TrialStatus::WarningDay23
    );

    // Day 29 -> WarningDay29
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(29)),
        TrialStatus::WarningDay29
    );

    // Day 30 -> ExpiredToFree
    assert_eq!(
        job.evaluate_lifecycle_at(started_at + Duration::days(30)),
        TrialStatus::ExpiredToFree
    );

    // Verify audit logs were written for all transitions
    assert_eq!(job.audit_logs.len(), 3);
    assert!(job.audit_logs[0].contains("Day 23"));
    assert!(job.audit_logs[1].contains("Day 29"));
    assert!(job.audit_logs[2].contains("Day 30"));
}
