use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_orders_over_time_aggregate() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let d1 = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 8, 16).unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d1,
            currency: "EUR".to_string(),
            total_revenue: dec!(100.00),
            units: 2,
            items: vec![],
        })
        .await
        .unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d1,
            currency: "USD".to_string(),
            total_revenue: dec!(50.00),
            units: 1,
            items: vec![],
        })
        .await
        .unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d2,
            currency: "USD".to_string(),
            total_revenue: dec!(200.00),
            units: 4,
            items: vec![],
        })
        .await
        .unwrap();

    let report = service
        .get_orders_over_time(Some(d1), Some(d2), None)
        .await
        .expect("orders report");

    assert_eq!(report.total_orders, 3);
    assert_eq!(report.total_units, 7);
    assert_eq!(report.time_series.len(), 2);
    assert_eq!(report.time_series[0].date, d1);
    assert_eq!(report.time_series[0].order_count, 2);
    assert_eq!(report.time_series[0].units_sold, 3);
    assert_eq!(report.time_series[1].date, d2);
    assert_eq!(report.time_series[1].order_count, 1);
    assert_eq!(report.time_series[1].units_sold, 4);

    db.drop().await;
}
