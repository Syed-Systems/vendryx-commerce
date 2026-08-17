use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{OrderPlacedRollupItem, RecordOrderPlacedInput, ReportingService};

#[tokio::test]
async fn test_top_products_aggregate() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();

    // Order 1: 5 units of Shoes ($500), 2 units of Shirt ($100)
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(600.00),
            units: 7,
            items: vec![
                OrderPlacedRollupItem {
                    product_id: "prod_shoes".to_string(),
                    title: "Shoes".to_string(),
                    units: 5,
                    total_revenue: dec!(500.00),
                },
                OrderPlacedRollupItem {
                    product_id: "prod_shirt".to_string(),
                    title: "Shirt".to_string(),
                    units: 2,
                    total_revenue: dec!(100.00),
                },
            ],
        })
        .await
        .unwrap();

    // Order 2: 1 unit of Watch ($1000)
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(1000.00),
            units: 1,
            items: vec![OrderPlacedRollupItem {
                product_id: "prod_watch".to_string(),
                title: "Watch".to_string(),
                units: 1,
                total_revenue: dec!(1000.00),
            }],
        })
        .await
        .unwrap();

    let report = service
        .get_top_products(Some(date), Some(date), 2, None)
        .await
        .expect("top products report");

    assert_eq!(report.top_products.len(), 2);
    // Highest revenue: Watch ($1000)
    assert_eq!(report.top_products[0].product_id, "prod_watch");
    assert_eq!(report.top_products[0].total_revenue, dec!(1000.00));
    // 2nd highest: Shoes ($500)
    assert_eq!(report.top_products[1].product_id, "prod_shoes");
    assert_eq!(report.top_products[1].total_revenue, dec!(500.00));

    db.drop().await;
}
