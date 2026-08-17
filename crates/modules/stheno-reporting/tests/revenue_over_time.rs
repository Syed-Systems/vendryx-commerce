use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_revenue_over_time_aggregate() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let d1 = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 8, 16).unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d1,
            currency: "USD".to_string(),
            total_revenue: dec!(150.00),
            units: 3,
            items: vec![],
        })
        .await
        .unwrap();

    service
        .record_order_placed(RecordOrderPlacedInput {
            date: d2,
            currency: "USD".to_string(),
            total_revenue: dec!(250.00),
            units: 5,
            items: vec![],
        })
        .await
        .unwrap();

    let report = service
        .get_revenue_over_time(Some(d1), Some(d2), "USD", None)
        .await
        .expect("revenue report");

    assert_eq!(report.currency, "USD");
    assert_eq!(report.total_revenue, dec!(400.00));
    assert_eq!(report.time_series.len(), 2);
    assert_eq!(report.time_series[0].date, d1);
    assert_eq!(report.time_series[0].revenue, dec!(150.00));
    assert_eq!(report.time_series[1].date, d2);
    assert_eq!(report.time_series[1].revenue, dec!(250.00));

    db.drop().await;
}
