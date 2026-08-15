use serde_json::json;
use stheno_fulfillment::*;

#[tokio::test]
async fn test_eligible_shipping_options_resolution_by_geo_and_rules() {
    let service = FulfillmentService::new();

    // 1. Setup US East Service Zone
    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "US National Fulfillment".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    let zone_us = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "United States Zone".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone_us.id.clone(),
            r#type: "country".to_string(),
            country_code: "us".to_string(),
            province_code: None,
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("geo zone created");

    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Standard Goods".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("profile created");

    // Option 1: Standard Ground (Available for regular orders only, is_return == false)
    let opt_ground = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Standard US Ground".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone_us.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "rate": "10.00" })),
            metadata: None,
        })
        .await
        .expect("opt ground created");

    service
        .add_shipping_option_rule(CreateShippingOptionRuleInput {
            shipping_option_id: opt_ground.id.clone(),
            attribute: "is_return".to_string(),
            operator: "eq".to_string(),
            value: json!(false),
            metadata: None,
        })
        .await
        .expect("rule added");

    // Option 2: Prepaid Return Shipping (Available for return orders only, is_return == true)
    let opt_return = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Prepaid Return Drop-Off".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone_us.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "rate": "5.00" })),
            metadata: None,
        })
        .await
        .expect("opt return created");

    service
        .add_shipping_option_rule(CreateShippingOptionRuleInput {
            shipping_option_id: opt_return.id.clone(),
            attribute: "is_return".to_string(),
            operator: "eq".to_string(),
            value: json!(true),
            metadata: None,
        })
        .await
        .expect("rule added");

    // 2. Query standard checkout order in US (is_return = false) -> must match opt_ground ONLY
    let checkout_ctx = EligibilityContext {
        country_code: Some("us".to_string()),
        is_return: Some(false),
        ..Default::default()
    };
    let resolved_checkout = service
        .resolve_eligible_shipping_options(&checkout_ctx)
        .await
        .expect("eligible options resolved");

    assert_eq!(resolved_checkout.len(), 1);
    assert_eq!(resolved_checkout[0].name, "Standard US Ground");

    // 3. Query return order in US (is_return = true) -> must match opt_return ONLY
    let return_ctx = EligibilityContext {
        country_code: Some("us".to_string()),
        is_return: Some(true),
        ..Default::default()
    };
    let resolved_return = service
        .resolve_eligible_shipping_options(&return_ctx)
        .await
        .expect("return options resolved");

    assert_eq!(resolved_return.len(), 1);
    assert_eq!(resolved_return[0].name, "Prepaid Return Drop-Off");

    // 4. Query from unsupported country (e.g. "ca") -> must return 0 options
    let ca_ctx = EligibilityContext {
        country_code: Some("ca".to_string()),
        ..Default::default()
    };
    let resolved_ca = service
        .resolve_eligible_shipping_options(&ca_ctx)
        .await
        .expect("ca options resolved");

    assert_eq!(resolved_ca.len(), 0);
}
