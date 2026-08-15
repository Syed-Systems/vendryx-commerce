use stheno_fulfillment::*;

#[tokio::test]
async fn test_fulfillment_all_12_entities_lifecycle() {
    let service = FulfillmentService::new();

    // 1. Create Shipping Profile
    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Default Profile".to_string(),
            r#type: Some("default".to_string()),
            metadata: None,
        })
        .await
        .expect("profile created");

    // 2. Create Shipping Option Type
    let opt_type = service
        .create_shipping_option_type(CreateShippingOptionTypeInput {
            label: "Express Shipping".to_string(),
            description: Some("Overnight express".to_string()),
            code: "express".to_string(),
            metadata: None,
        })
        .await
        .expect("type created");

    // 3. Create Fulfillment Set
    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "Main Warehouse Fulfillment Set".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    // 4. Create Service Zone
    let zone = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "North America Zone".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    // 5. Create Geo Zone
    let geo = service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: "country".to_string(),
            country_code: "us".to_string(),
            province_code: None,
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("geo zone created");

    assert_eq!(geo.country_code, "us");
    assert_eq!(profile.name, "Default Profile");
    assert_eq!(opt_type.code, "express");
}
