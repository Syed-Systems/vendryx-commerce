use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_window_gate_bounds_query_in_sql() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let today = Utc::now().date_naive();
    let d_10_days_ago = today - Duration::days(10);
    let d_45_days_ago = today - Duration::days(45);

    // Recent order (10 days ago): $100
    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d_10_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(100.00),
            units: 1,
            items: vec![],
        })
        .await
        .unwrap();

    // Old order (45 days ago): $500
    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d_45_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(500.00),
            units: 5,
            items: vec![],
        })
        .await
        .unwrap();

    // Query with window_days = 30 -> only includes 10-days-ago order ($100), excludes 45-days-ago ($500)
    let rep_gated = service
        .get_revenue_over_time(None, None, "USD", Some(30))
        .await
        .expect("gated report");
    assert_eq!(rep_gated.total_revenue, dec!(100.00));
    assert_eq!(rep_gated.time_series.len(), 1);
    assert_eq!(rep_gated.time_series[0].date, d_10_days_ago);

    // Query with window_days = None (unlimited) -> includes both ($600)
    let rep_unlimited = service
        .get_revenue_over_time(None, None, "USD", None)
        .await
        .expect("unlimited report");
    assert_eq!(rep_unlimited.total_revenue, dec!(600.00));
    assert_eq!(rep_unlimited.time_series.len(), 2);

    db.drop().await;
}
