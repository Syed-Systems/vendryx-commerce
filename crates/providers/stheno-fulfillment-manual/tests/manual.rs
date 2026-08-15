use stheno_fulfillment::{CreateFulfillmentItemInput, FulfillmentProvider};
use stheno_fulfillment_manual::ManualFulfillmentProvider;

#[tokio::test]
async fn test_manual_fulfillment_provider_lifecycle() {
    let provider = ManualFulfillmentProvider::new();

    assert_eq!(provider.identifier(), "manual");

    // 1. Get options
    let options = provider.get_fulfillment_options().await.expect("options");
    assert_eq!(options.len(), 3);
    assert!(options.iter().any(|o| o.id == "manual-standard"));
    assert!(options
        .iter()
        .any(|o| o.id == "manual-return" && o.is_return));

    // 2. Validate option
    let is_valid = provider.validate_option(None).await.expect("validated");
    assert!(is_valid);

    // 3. Create fulfillment with labels
    let items = vec![CreateFulfillmentItemInput {
        title: "Denim Jacket".to_string(),
        sku: Some("SKU-JKT".to_string()),
        barcode: None,
        line_item_id: "li_1".to_string(),
        inventory_item_id: Some("inv_1".to_string()),
        quantity: 1,
        metadata: None,
    }];

    let response = provider
        .create_fulfillment(
            Some(&serde_json::json!({ "tracking_number": "TRK-MANUAL-001" })),
            &items,
            None,
        )
        .await
        .expect("fulfillment created");

    assert_eq!(response.labels.len(), 1);
    assert_eq!(response.labels[0].tracking_number, "TRK-MANUAL-001");
}
