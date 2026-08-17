use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_average_order_value_aggregate() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();

    // Order 1: $100.00
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(100.00),
            units: 1,
            items: vec![],
        })
        .await
        .unwrap();

    // Order 2: $200.00
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(200.00),
            units: 2,
            items: vec![],
        })
        .await
        .unwrap();

    let report = service
        .get_average_order_value(Some(date), Some(date), "USD", None)
        .await
        .expect("AOV report");

    assert_eq!(report.currency, "USD");
    assert_eq!(report.order_count, 2);
    assert_eq!(report.total_revenue, dec!(300.00));
    assert_eq!(report.average_order_value, dec!(150.00)); // 300 / 2 = 150

    db.drop().await;
}
