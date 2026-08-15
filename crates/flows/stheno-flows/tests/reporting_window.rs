use chrono::{Duration, Utc};
use std::sync::Arc;
use stheno_flows::platform::PlatformReportingWorkflow;
use stheno_order::OrderService;

#[tokio::test]
async fn test_reporting_window_horizon_gate() {
    let order_service = Arc::new(OrderService::new());
    // Tier allows maximum 30 days analytics window
    let reporting = PlatformReportingWorkflow::new(order_service, 30);

    // 1. Valid request within 30 days
    let valid_start = Utc::now() - Duration::days(20);
    let valid_report = reporting.generate_report(Some(valid_start), None).await;
    assert!(valid_report.is_ok());

    // 2. Invalid request exceeding 30 days horizon (e.g. 60 days)
    let invalid_start = Utc::now() - Duration::days(60);
    let invalid_report = reporting.generate_report(Some(invalid_start), None).await;

    assert!(invalid_report.is_err());
    let err = invalid_report.err().unwrap();
    match err {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "reporting_window_exceeded");
        }
        other => panic!("Expected Error::Invalid, got {:?}", other),
    }
}
