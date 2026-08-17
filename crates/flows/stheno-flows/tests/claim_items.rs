//! Claim Items with Evidence Images Test (P6-11).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::ClaimWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_claim_begin_and_add_items_with_images() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = ClaimWorkflowEngine::new(order_service.clone());

    // 1. Create order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Claimer".to_string()),
                last_name: Some("User".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("claimer@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Smartphone".to_string(),
                quantity: 1,
                unit_price: dec!(799.00),
                variant_id: Some("var_phone".to_string()),
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

    let item_id = order.items[0].id.clone();

    // 2. Begin claim
    let claim = engine
        .begin_claim(&order.id, "replace", None, Some("cust_01"))
        .await
        .unwrap();

    assert_eq!(claim.status, "open");
    assert_eq!(claim.r#type, "replace");

    // 3. Add claim item with evidence photos
    let claim_item = engine
        .add_claim_item(
            &claim.id,
            &item_id,
            1,
            "production_failure",
            Some(vec![
                "https://cdn.example.com/claims/screen_defect_1.jpg".to_string(),
                "https://cdn.example.com/claims/screen_defect_2.jpg".to_string(),
            ]),
        )
        .await
        .unwrap();

    assert_eq!(claim_item.quantity, 1);
    assert_eq!(claim_item.reason, "production_failure");
    assert_eq!(claim_item.images.len(), 2);
    assert_eq!(
        claim_item.images[0].url,
        "https://cdn.example.com/claims/screen_defect_1.jpg"
    );

    db.drop().await;
}
