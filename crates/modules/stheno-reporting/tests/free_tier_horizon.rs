use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_free_tier_horizon_cannot_retrieve_older_than_30_days() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let today = Utc::now().date_naive();
    let d_20_days_ago = today - Duration::days(20);
    let d_60_days_ago = today - Duration::days(60);

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
            date: d_60_days_ago,
            currency: "USD".to_string(),
            total_revenue: dec!(500.00),
            units: 5,
            items: vec![],
        })
        .await
        .unwrap();

    // Free tier enforces window_days = 30
    let free_tier_window = Some(30);

    // Even if merchant requests from 90 days ago, Free tier clamps to 30 days
    let requested_from = Some(today - Duration::days(90));
    let rep = service
        .get_revenue_over_time(requested_from, None, "USD", free_tier_window)
        .await
        .expect("free tier report");

    assert_eq!(rep.total_revenue, dec!(100.00));
    assert_eq!(rep.time_series.len(), 1);
    assert_eq!(rep.time_series[0].date, d_20_days_ago);

    db.drop().await;
}
