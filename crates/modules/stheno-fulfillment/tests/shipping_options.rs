use serde_json::json;
use stheno_fulfillment::*;

#[tokio::test]
async fn test_shipping_options_rules_types_and_profiles() {
    let service = FulfillmentService::new();

    // 1. Setup profile and service zone
    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Fragile Goods".to_string(),
            r#type: Some("custom".to_string()),
            metadata: None,
        })
        .await
        .expect("profile created");

    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "West Coast Hub".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    let zone = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "California Metro".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    let opt_type = service
        .create_shipping_option_type(CreateShippingOptionTypeInput {
            label: "Same-Day Delivery".to_string(),
            description: Some("Courier same-day".to_string()),
            code: "same-day".to_string(),
            metadata: None,
        })
        .await
        .expect("type created");

    // 2. Create Shipping Option
    let option = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Same-Day Courier Delivery".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: Some(opt_type.id.clone()),
            data: Some(json!({ "rate": "25.00" })),
            metadata: None,
        })
        .await
        .expect("option created");

    // 3. Add Rule: Not allowed for returns (is_return == false)
    let rule = service
        .add_shipping_option_rule(CreateShippingOptionRuleInput {
            shipping_option_id: option.id.clone(),
            attribute: "is_return".to_string(),
            operator: "eq".to_string(),
            value: json!(false),
            metadata: None,
        })
        .await
        .expect("rule added");

    assert_eq!(option.name, "Same-Day Courier Delivery");
    assert_eq!(rule.attribute, "is_return");
}
