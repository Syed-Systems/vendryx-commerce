use chrono::{Duration, Utc};
use stheno_flows::AnalyticsWindowGate;

#[test]
fn test_window_gate_clamps_reporting_queries_to_plan_limit() {
    let now = Utc::now();

    // Merchant on Free tier requests 180 days of analytics data, but tier limit is 30 days
    let requested_180_days_ago = now - Duration::days(180);
    let clamped_start =
        AnalyticsWindowGate::clamp_analytics_window_at(requested_180_days_ago, 30, now);

    let allowed_diff = (now - clamped_start).num_days();
    assert_eq!(
        allowed_diff, 30,
        "Reporting query window must be clamped to 30 days"
    );

    // Query within allowed window (e.g. 15 days ago) is kept intact
    let requested_15_days_ago = now - Duration::days(15);
    let intact_start =
        AnalyticsWindowGate::clamp_analytics_window_at(requested_15_days_ago, 30, now);
    assert_eq!(intact_start, requested_15_days_ago);
}
