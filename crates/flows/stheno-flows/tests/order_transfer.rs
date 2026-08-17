use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderWorkflowEngine;
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_order_transfer_request_accept_decline_cancel() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderWorkflowEngine::new(order_service.clone());

    // 1. Create order owned by cus_alice
    let order = engine
        .create_order_flow(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_alice".to_string()),
            sales_channel_id: None,
            email: Some("alice@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Leather Belt".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(45.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: None,
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: None,
                variant_barcode: None,
                variant_title: None,
                variant_option_values: None,
                metadata: None,
            }]),
            shipping_methods: None,
            is_draft_order: None,
            metadata: None,
        })
        .await
        .expect("order created");

    assert_eq!(order.customer_id, Some("cus_alice".to_string()));

    // 2. Request transfer to cus_bob
    let req = engine
        .request_order_transfer(&order.id, "cus_bob")
        .await
        .expect("transfer requested");

    assert_eq!(req.status, "pending");
    assert_eq!(req.target_customer_id, "cus_bob");

    // 3. Accept transfer -> order ownership moves to cus_bob
    let transferred_order = engine
        .accept_order_transfer(&order.id)
        .await
        .expect("transfer accepted");

    assert_eq!(transferred_order.customer_id, Some("cus_bob".to_string()));

    // 4. Request another transfer to cus_charlie and decline it
    let req2 = engine
        .request_order_transfer(&order.id, "cus_charlie")
        .await
        .expect("transfer 2 requested");
    assert_eq!(req2.status, "pending");

    let declined_req = engine
        .decline_order_transfer(&order.id)
        .await
        .expect("transfer declined");
    assert_eq!(declined_req.status, "declined");

    // 5. Request another transfer to cus_dan and cancel it
    let req3 = engine
        .request_order_transfer(&order.id, "cus_dan")
        .await
        .expect("transfer 3 requested");
    assert_eq!(req3.status, "pending");

    let canceled_req = engine
        .cancel_order_transfer(&order.id)
        .await
        .expect("transfer canceled");
    assert_eq!(canceled_req.status, "canceled");

    db.drop().await;
}
