use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{OrderPlacedRollupItem, RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_rollup_incremental_refresh_from_order_placed() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();

    // 1. First order on 2026-08-17
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(120.00),
            units: 2,
            items: vec![OrderPlacedRollupItem {
                product_id: "prod_1".to_string(),
                title: "Sneakers".to_string(),
                units: 2,
                total_revenue: dec!(120.00),
            }],
        })
        .await
        .expect("order 1");

    // 2. Second order on same day
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(80.00),
            units: 1,
            items: vec![OrderPlacedRollupItem {
                product_id: "prod_2".to_string(),
                title: "Hoodie".to_string(),
                units: 1,
                total_revenue: dec!(80.00),
            }],
        })
        .await
        .expect("order 2");

    // Verify revenue over time
    let revenue_rep = service
        .get_revenue_over_time(Some(date), Some(date), "USD", None)
        .await
        .expect("revenue report");
    assert_eq!(revenue_rep.total_revenue, dec!(200.00));
    assert_eq!(revenue_rep.time_series.len(), 1);
    assert_eq!(revenue_rep.time_series[0].revenue, dec!(200.00));

    // Verify orders over time
    let orders_rep = service
        .get_orders_over_time(Some(date), Some(date), None)
        .await
        .expect("orders report");
    assert_eq!(orders_rep.total_orders, 2);
    assert_eq!(orders_rep.total_units, 3);

    db.drop().await;
}
