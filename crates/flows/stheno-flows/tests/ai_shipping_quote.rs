use std::sync::Arc;

use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::{ShippingQuoteRequest, ShippingQuoteTool};
use stheno_fulfillment::{
    CreateFulfillmentSetInput, CreateGeoZoneInput, CreateServiceZoneInput,
    CreateShippingOptionInput, CreateShippingProfileInput, FulfillmentService,
};
use stheno_region::{CreateRegionInput, RegionService};

#[tokio::test]
async fn quotes_only_current_eligible_rates_and_checks_region_currency() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let fulfillment = Arc::new(FulfillmentService::new(pool.clone()));
    let regions = Arc::new(RegionService::new(pool));

    let region = regions
        .create_region(CreateRegionInput {
            name: "United States".to_string(),
            currency_code: "usd".to_string(),
            automatic_taxes: Some(false),
            countries: vec!["us".to_string()],
        })
        .await
        .expect("create region");
    let profile = fulfillment
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Default".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("create profile");
    let set = fulfillment
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "Domestic".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("create set");
    let zone = fulfillment
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id,
            name: "United States".to_string(),
            metadata: None,
        })
        .await
        .expect("create zone");
    fulfillment
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
        .expect("create destination");
    let option = fulfillment
        .create_shipping_option(CreateShippingOptionInput {
            name: "Ground".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id,
            shipping_profile_id: profile.id,
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "amount": "12.50" })),
            metadata: None,
        })
        .await
        .expect("create option");

    let tool = ShippingQuoteTool::new(fulfillment, regions);
    let result = tool
        .calculate(ShippingQuoteRequest {
            country_code: "US".to_string(),
            province_code: None,
            city: None,
            postal_code: None,
            currency_code: "USD".to_string(),
            region_id: Some(region.id.to_string()),
            items_count: 1,
            total_weight: None,
        })
        .await
        .expect("quote eligible shipping option");
    assert!(!result.truncated);
    assert_eq!(result.quotes.len(), 1);
    assert_eq!(result.quotes[0].option_id, option.id);
    assert_eq!(result.quotes[0].amount.to_string(), "12.50");
    assert_eq!(result.quotes[0].currency_code, "usd");
    assert!(!result.quotes[0].is_calculated);

    let mismatch = tool
        .calculate(ShippingQuoteRequest {
            currency_code: "EUR".to_string(),
            region_id: Some(region.id.to_string()),
            country_code: "US".to_string(),
            province_code: None,
            city: None,
            postal_code: None,
            items_count: 1,
            total_weight: None,
        })
        .await;
    assert!(matches!(mismatch, Err(stheno_types::Error::Invalid { .. })));
    db.drop().await;
}
