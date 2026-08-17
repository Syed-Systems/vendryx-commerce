//! Order Edit Confirm Test (P6-04).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_order_edit_confirm_version_bump_and_summary_recompute() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderEditWorkflowEngine::new(order_service.clone(), None);

    // 1. Create order at version 1
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Bob".to_string()),
                last_name: Some("Builder".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("bob@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Hammer".to_string(),
                quantity: 1,
                unit_price: dec!(25.00),
                variant_id: Some("var_hammer".to_string()),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }]),
            shipping_methods: Some(vec![]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(order.version, 1);
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(25.00)
    );

    // 2. Begin edit and add item
    let edit = engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();
    engine
        .add_item(&edit.id, "Nails", 2, dec!(5.00), Some("var_nails"))
        .await
        .unwrap();

    // 3. Confirm edit
    let confirmed_order = engine.confirm_edit(&edit.id, Some("admin")).await.unwrap();

    // 4. Verify version bump and summary recompute
    assert_eq!(confirmed_order.version, 2);
    assert_eq!(
        confirmed_order
            .summary
            .as_ref()
            .unwrap()
            .current_order_total,
        dec!(35.00)
    );

    // 5. Verify version 1 snapshot remains intact when querying historical version
    let v1_order = order_service
        .get_order_at_version(&order.id, 1)
        .await
        .unwrap();
    assert_eq!(v1_order.version, 1);
    assert_eq!(v1_order.items.len(), 1);
    assert_eq!(
        v1_order.summary.as_ref().unwrap().current_order_total,
        dec!(25.00)
    );

    db.drop().await;
}
