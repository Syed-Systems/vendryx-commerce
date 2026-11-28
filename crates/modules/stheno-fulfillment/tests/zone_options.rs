//! P23-12: Shipping-option configuration per courier zone expressed through existing shipping options.

use rust_decimal_macros::dec;
use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_fulfillment_pathao::PathaoFulfillmentProvider;
use stheno_fulfillment_redx::RedxFulfillmentProvider;
use stheno_fulfillment_steadfast::SteadfastFulfillmentProvider;

#[tokio::test]
async fn courier_zone_options_configuration_and_resolution() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    // Register all three courier providers
    service.register_provider(Arc::new(PathaoFulfillmentProvider::default()));
    service.register_provider(Arc::new(SteadfastFulfillmentProvider::default()));
    service.register_provider(Arc::new(RedxFulfillmentProvider::default()));

    // Seed dummy provider rows in Postgres table to satisfy foreign key constraints
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES
            ('pathao', 'Pathao Courier'),
            ('steadfast', 'Steadfast Courier'),
            ('redx', 'RedX Courier')
         ON CONFLICT DO NOTHING",
    )
    .await;

    // 1. Create fulfillment set for Bangladesh Central Warehouse
    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "BD Central Logistics Hub".to_string(),
            r#type: "shipping".to_string(),
            metadata: Some(json!({ "country": "bd" })),
        })
        .await
        .expect("fulfillment set created");

    // 2. Create Service Zones for the three courier pricing tiers
    let zone_inside = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "Inside Dhaka Metro".to_string(),
            metadata: Some(json!({ "tier": "inside_city" })),
        })
        .await
        .expect("inside zone created");

    let zone_sub = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "Dhaka Sub-City".to_string(),
            metadata: Some(json!({ "tier": "sub_city" })),
        })
        .await
        .expect("sub-city zone created");

    let zone_outside = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "Outside Dhaka / Nationwide".to_string(),
            metadata: Some(json!({ "tier": "outside_city" })),
        })
        .await
        .expect("outside zone created");

    // 3. Attach Geo Zones
    // Inside Dhaka: country = bd, city = Dhaka
    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone_inside.id.clone(),
            r#type: "city".to_string(),
            country_code: "bd".to_string(),
            province_code: None,
            city: Some("dhaka".to_string()),
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("inside geo zone created");

    // Sub-city: country = bd, province = sub_dhaka (e.g. Savar, Gazipur)
    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone_sub.id.clone(),
            r#type: "province".to_string(),
            country_code: "bd".to_string(),
            province_code: Some("sub_dhaka".to_string()),
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("sub geo zone created");

    // Outside Dhaka: country = bd, province = outside
    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone_outside.id.clone(),
            r#type: "province".to_string(),
            country_code: "bd".to_string(),
            province_code: Some("outside".to_string()),
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("outside geo zone created");

    // 4. Create Shipping Profile
    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "General Goods".to_string(),
            r#type: Some("standard".to_string()),
            metadata: None,
        })
        .await
        .expect("shipping profile created");

    // 5. Create Shipping Options for Inside Dhaka (Pathao, Steadfast, RedX)
    let opt_pathao_inside = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Pathao Express (Inside Dhaka)".to_string(),
            price_type: "calculated".to_string(),
            service_zone_id: zone_inside.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "pathao".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "tier": "inside_city", "id": "pathao-standard" })),
            metadata: None,
        })
        .await
        .expect("pathao inside option created");

    let opt_steadfast_outside = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Steadfast Parcel (Outside Dhaka)".to_string(),
            price_type: "calculated".to_string(),
            service_zone_id: zone_outside.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "steadfast".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "tier": "outside_city", "id": "steadfast-outside-city" })),
            metadata: None,
        })
        .await
        .expect("steadfast outside option created");

    let opt_redx_sub = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "RedX Delivery (Dhaka Sub-City)".to_string(),
            price_type: "calculated".to_string(),
            service_zone_id: zone_sub.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "redx".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "tier": "sub_city", "id": "redx-sub-city" })),
            metadata: None,
        })
        .await
        .expect("redx sub-city option created");

    // 6. Test Option Resolution for Inside Dhaka
    let ctx_inside = EligibilityContext {
        country_code: Some("bd".to_string()),
        city: Some("dhaka".to_string()),
        ..Default::default()
    };
    let resolved_inside = service
        .resolve_eligible_shipping_options(&ctx_inside)
        .await
        .expect("resolve inside options");
    assert!(resolved_inside.iter().any(|o| o.id == opt_pathao_inside.id));

    // 7. Calculate shipping price for resolved inside option
    let calc_ctx = CalculatePriceContext {
        items_count: 1,
        total_weight: Some(dec!(0.5)),
        currency_code: "bdt".to_string(),
    };
    let price_inside = service
        .calculate_shipping_price(&opt_pathao_inside.id, Some(&calc_ctx))
        .await
        .expect("calculate inside price");
    assert_eq!(price_inside, dec!(60.00));

    // 8. Test Option Resolution for Outside Dhaka
    let ctx_outside = EligibilityContext {
        country_code: Some("bd".to_string()),
        province_code: Some("outside".to_string()),
        ..Default::default()
    };
    let resolved_outside = service
        .resolve_eligible_shipping_options(&ctx_outside)
        .await
        .expect("resolve outside options");
    assert!(resolved_outside
        .iter()
        .any(|o| o.id == opt_steadfast_outside.id));

    let price_outside = service
        .calculate_shipping_price(&opt_steadfast_outside.id, Some(&calc_ctx))
        .await
        .expect("calculate outside price");
    assert_eq!(price_outside, dec!(130.00));

    // 9. Test Option Resolution for Sub-City
    let ctx_sub = EligibilityContext {
        country_code: Some("bd".to_string()),
        province_code: Some("sub_dhaka".to_string()),
        ..Default::default()
    };
    let resolved_sub = service
        .resolve_eligible_shipping_options(&ctx_sub)
        .await
        .expect("resolve sub options");
    assert!(resolved_sub.iter().any(|o| o.id == opt_redx_sub.id));

    let price_sub = service
        .calculate_shipping_price(&opt_redx_sub.id, Some(&calc_ctx))
        .await
        .expect("calculate sub price");
    assert_eq!(price_sub, dec!(100.00));

    db.drop().await;
}
