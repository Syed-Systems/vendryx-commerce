use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::platform::PlatformReportingWorkflow;
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_platform_reporting_4_aggregates() {
    let order_service = Arc::new(OrderService::new());
    let reporting = PlatformReportingWorkflow::new(order_service.clone(), 90);

    // 1. Create orders with different statuses and sources
    let _ord1 = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_1".to_string()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("c1@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Silk Scarf".to_string(),
                quantity: 3,
                unit_price: dec!(45.00),
                variant_id: Some("var_scarf".to_string()),
                product_id: Some("prod_scarf".to_string()),
                ..Default::default()
            }]),
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: Some(serde_json::json!({ "source": "storefront" })),
        })
        .await
        .unwrap();

    let _ord2 = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_2".to_string()),
            sales_channel_id: Some("sc_social".to_string()),
            email: Some("c2@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Silk Scarf".to_string(),
                quantity: 1,
                unit_price: dec!(45.00),
                variant_id: Some("var_scarf".to_string()),
                product_id: Some("prod_scarf".to_string()),
                ..Default::default()
            }]),
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: Some(serde_json::json!({ "source": "social" })),
        })
        .await
        .unwrap();

    // 2. Generate reporting aggregates
    let report = reporting.generate_report(None, None).await.unwrap();

    // Aggregate 1: Orders by status
    assert!(report.orders_by_status.contains_key("pending"));

    // Aggregate 2: Top Products
    assert_eq!(report.top_products.len(), 1);
    assert_eq!(report.top_products[0].product_id, "prod_scarf");
    assert_eq!(report.top_products[0].quantity_sold, 4);
    assert_eq!(report.top_products[0].revenue, dec!(180.00));

    // Aggregate 3: Orders by source
    assert_eq!(report.orders_by_source.get("storefront"), Some(&1));
    assert_eq!(report.orders_by_source.get("social"), Some(&1));
}
