//! Draft Order to Live Order Conversion Test (P6-16).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::DraftOrderWorkflowEngine;
use stheno_order::{CreateLineItemInput, OrderService};

#[tokio::test]
async fn test_draft_to_live_order_conversion() {
    let order_service = Arc::new(OrderService::new());
    let engine = DraftOrderWorkflowEngine::new(order_service.clone());

    // 1. Create a draft order
    let draft = engine
        .create_draft_order(
            Some("cust_conv_01"),
            Some("conv@example.com"),
            Some("reg_us"),
            "usd",
            vec![CreateLineItemInput {
                title: "Customized Desk".to_string(),
                quantity: 1,
                unit_price: dec!(450.00),
                variant_id: Some("var_desk".to_string()),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }],
            vec![],
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(draft.status, "draft");

    // 2. Convert draft to live order
    let live_order = engine.convert_draft_order(&draft.id).await.unwrap();

    assert_eq!(live_order.status, "pending");
    assert_eq!(live_order.payment_status, "not_paid");
    assert_eq!(live_order.fulfillment_status, "not_fulfilled");
    assert_eq!(
        live_order.summary.unwrap().current_order_total,
        dec!(450.00)
    );
}
