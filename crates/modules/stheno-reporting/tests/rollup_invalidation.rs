use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{
    OrderPlacedRollupItem, RecordOrderDeltaInput, RecordOrderPlacedInput, ReportingService,
};

#[tokio::test]
async fn test_rollup_invalidation_on_order_edit_return_refund_and_cancel() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();

    // 1. Initial order: $100.00, 2 units
    service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(100.00),
            units: 2,
            items: vec![OrderPlacedRollupItem {
                product_id: "prod_x".to_string(),
                title: "Jacket".to_string(),
                units: 2,
                total_revenue: dec!(100.00),
            }],
        })
        .await
        .expect("place order");

    // 2. Order Edit: added an item (+ $50.00, + 1 unit, 0 change in order count)
    service
        .record_order_delta(RecordOrderDeltaInput {
            date,
            currency: "USD".to_string(),
            delta_revenue: dec!(50.00),
            delta_units: 1,
            delta_orders: 0,
        })
        .await
        .expect("order edit");

    let rev1 = service
        .get_revenue_over_time(Some(date), Some(date), "USD", None)
        .await
        .unwrap();
    assert_eq!(rev1.total_revenue, dec!(150.00));

    // 3. Return & Refund: returned 1 item (- $50.00, - 1 unit)
    service
        .record_order_delta(RecordOrderDeltaInput {
            date,
            currency: "USD".to_string(),
            delta_revenue: dec!(-50.00),
            delta_units: -1,
            delta_orders: 0,
        })
        .await
        .expect("return & refund");

    let rev2 = service
        .get_revenue_over_time(Some(date), Some(date), "USD", None)
        .await
        .unwrap();
    assert_eq!(rev2.total_revenue, dec!(100.00));

    // 4. Cancellation of order (- $100.00, - 2 units, - 1 order)
    service
        .record_order_delta(RecordOrderDeltaInput {
            date,
            currency: "USD".to_string(),
            delta_revenue: dec!(-100.00),
            delta_units: -2,
            delta_orders: -1,
        })
        .await
        .expect("cancellation");

    let rev3 = service
        .get_revenue_over_time(Some(date), Some(date), "USD", None)
        .await
        .unwrap();
    assert_eq!(rev3.total_revenue, dec!(0.00));

    let orders_rep = service
        .get_orders_over_time(Some(date), Some(date), None)
        .await
        .unwrap();
    assert_eq!(orders_rep.total_orders, 0);

    db.drop().await;
}
