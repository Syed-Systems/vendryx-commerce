use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::FulfillmentWorkflowEngine;
use stheno_fulfillment::{
    CreateFulfillmentInput, CreateFulfillmentItemInput, CreateServiceZoneInput,
    CreateShippingOptionInput, CreateShippingProfileInput, FulfillmentService,
};
use stheno_fulfillment_manual::ManualFulfillmentProvider;

#[tokio::test]
async fn test_all_8_fulfillment_workflows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let ful_service = Arc::new(FulfillmentService::new(pool));
    ful_service.register_provider(Arc::new(ManualFulfillmentProvider::new()));

    let engine = FulfillmentWorkflowEngine::new(ful_service.clone());

    // 1. Setup Fulfillment Set & Shipping Profile
    let set = ful_service
        .create_fulfillment_set(stheno_fulfillment::CreateFulfillmentSetInput {
            name: "Central Hub".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    let profile = ful_service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Default".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("profile created");

    // Flow 1: Create Service Zone workflow
    let zone = engine
        .create_service_zone_flow(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "Global Zone".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    // Flow 2: Create Shipping Option workflow
    let option = engine
        .create_shipping_option_flow(CreateShippingOptionInput {
            name: "Standard Ground".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("option created");

    // Flow 3: Create Fulfillment workflow
    let ful = engine
        .create_fulfillment_flow(CreateFulfillmentInput {
            location_id: Some("loc_1".to_string()),
            provider_id: "manual".to_string(),
            shipping_option_id: Some(option.id.clone()),
            delivery_address: None,
            items: vec![CreateFulfillmentItemInput {
                title: "Silk Scarf".to_string(),
                sku: Some("SKU-SILK".to_string()),
                barcode: None,
                line_item_id: "li_silk".to_string(),
                inventory_item_id: None,
                quantity: 1,
                metadata: None,
            }],
            labels: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("fulfillment created");

    assert_eq!(ful.items.len(), 1);

    // Flow 4: Create Shipment workflow
    let shipment = engine
        .create_shipment_flow(&ful.id)
        .await
        .expect("shipment created");
    assert!(shipment.shipped_at.is_some());

    // Flow 5: Mark fulfillment delivered workflow
    let delivered = engine
        .mark_fulfillment_delivered_flow(&ful.id)
        .await
        .expect("delivered");
    assert!(delivered.delivered_at.is_some());

    // Flow 6: Create return fulfillment workflow
    let ret_ful = engine
        .create_return_fulfillment_flow(CreateFulfillmentInput {
            location_id: Some("loc_return".to_string()),
            provider_id: "manual".to_string(),
            shipping_option_id: Some(option.id.clone()),
            delivery_address: None,
            items: vec![CreateFulfillmentItemInput {
                title: "Silk Scarf".to_string(),
                sku: Some("SKU-SILK".to_string()),
                barcode: None,
                line_item_id: "li_silk".to_string(),
                inventory_item_id: None,
                quantity: 1,
                metadata: None,
            }],
            labels: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("return fulfillment created");

    assert_eq!(ret_ful.provider_id, "manual");

    // Flow 7: Cancel fulfillment workflow
    let canceled = engine
        .cancel_fulfillment_flow(&ret_ful.id)
        .await
        .expect("canceled");
    assert!(canceled.canceled_at.is_some());

    // Flow 8: Mark fulfillment shipped workflow
    let mark_shipped = engine
        .mark_fulfillment_shipped_flow(&ful.id)
        .await
        .expect("shipped");
    assert!(mark_shipped.shipped_at.is_some());

    db.drop().await;
}
