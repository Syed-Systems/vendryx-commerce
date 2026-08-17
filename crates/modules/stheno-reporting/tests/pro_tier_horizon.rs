use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_pro_tier_horizon_retrieves_full_history() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let today = Utc::now().date_naive();
    let d_20_days_ago = today - Duration::days(20);
    let d_180_days_ago = today - Duration::days(180);
    let d_365_days_ago = today - Duration::days(365);

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d_20_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(100.00),
            units: 1,
            items: vec![],
        })
        .await
        .unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d_180_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(200.00),
            units: 2,
            items: vec![],
        })
        .await
        .unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d_365_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(300.00),
            units: 3,
            items: vec![],
        })
        .await
        .unwrap();

    // Pro tier has no window limitation (window_days = None)
    let pro_tier_window = None;

    let requested_from = Some(d_365_days_ago);
    let rep = service
        .get_revenue_over_time(requested_from, None, "USD", pro_tier_window)
        .await
        .expect("pro tier report");

    assert_eq!(rep.total_revenue, dec!(600.00));
    assert_eq!(rep.time_series.len(), 3);
    assert_eq!(rep.time_series[0].date, d_365_days_ago);
    assert_eq!(rep.time_series[1].date, d_180_days_ago);
    assert_eq!(rep.time_series[2].date, d_20_days_ago);

    db.drop().await;
}
