use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;

#[tokio::test]
async fn test_eligible_shipping_options_resolution_by_geo_and_rules() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

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

    // Option 3: VIP Member Express (Available only for customer_id == "cus_vip")
    let opt_vip = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "VIP Free Overnight".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone_us.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "rate": "0.00" })),
            metadata: None,
        })
        .await
        .expect("opt vip created");

    service
        .add_shipping_option_rule(CreateShippingOptionRuleInput {
            shipping_option_id: opt_vip.id.clone(),
            attribute: "customer_id".to_string(),
            operator: "eq".to_string(),
            value: json!("cus_vip"),
            metadata: None,
        })
        .await
        .expect("rule added");

    // ── Test 1: Standard Checkout in US (Regular order) ─────────────────────
    let regular_eligible = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            customer_id: Some("cus_regular".to_string()),
            is_return: Some(false),
            ..Default::default()
        })
        .await
        .expect("resolved regular");

    assert_eq!(regular_eligible.len(), 1);
    assert_eq!(regular_eligible[0].name, "Standard US Ground");

    // ── Test 2: Return Flow in US ──────────────────────────────────────────
    let return_eligible = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            customer_id: Some("cus_regular".to_string()),
            is_return: Some(true),
            ..Default::default()
        })
        .await
        .expect("resolved return");

    assert_eq!(return_eligible.len(), 1);
    assert_eq!(return_eligible[0].name, "Prepaid Return Drop-Off");

    // ── Test 3: VIP Customer Regular Checkout in US ────────────────────────
    let vip_eligible = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            customer_id: Some("cus_vip".to_string()),
            is_return: Some(false),
            ..Default::default()
        })
        .await
        .expect("resolved vip");

    assert_eq!(vip_eligible.len(), 2);
    let names: Vec<_> = vip_eligible.iter().map(|o| o.name.as_str()).collect();
    assert!(names.contains(&"Standard US Ground"));
    assert!(names.contains(&"VIP Free Overnight"));

    // ── Test 4: Destination outside US (e.g., Canada) ──────────────────────
    let ca_eligible = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("ca".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolved ca");

    assert_eq!(ca_eligible.len(), 0);

    db.drop().await;
}
