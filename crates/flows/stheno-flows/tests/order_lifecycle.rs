use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderWorkflowEngine;
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService, UpdateOrderInput};

#[tokio::test]
async fn test_order_lifecycle_workflows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderWorkflowEngine::new(order_service.clone());

    // 1. Create order workflow
    let order = engine
        .create_order_flow(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_lifecycle".to_string()),
            sales_channel_id: None,
            email: Some("lifecycle@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Wool Sweater".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(120.00),
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

    assert_eq!(order.status, "pending");

    // 2. Update order workflow
    let updated = engine
        .update_order_flow(
            &order.id,
            UpdateOrderInput {
                email: Some("updated_email@example.com".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("order updated");
    assert_eq!(updated.email, Some("updated_email@example.com".to_string()));

    // 3. Complete order workflow
    let completed = engine
        .complete_order_flow(&order.id)
        .await
        .expect("order completed");
    assert_eq!(completed.status, "completed");

    // 4. Archive order workflow
    let archived = engine
        .archive_order_flow(&order.id)
        .await
        .expect("order archived");
    assert_eq!(archived.status, "archived");

    // 5. Cancel order workflow
    let canceled = engine
        .cancel_order_flow(&order.id)
        .await
        .expect("order canceled");
    assert_eq!(canceled.status, "canceled");
    assert!(canceled.canceled_at.is_some());

    db.drop().await;
}
