//! Shipping quotes must fail closed for missing or unsupported destination constraints.

use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;

async fn option_for_geo(
    service: &FulfillmentService,
    profile_id: &str,
    name: &str,
    geo_type: &str,
    province_code: Option<&str>,
    city: Option<&str>,
    postal_pattern: Option<&str>,
) -> ShippingOptionDTO {
    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: format!("{name} set"),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("create fulfillment set");
    let zone = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id,
            name: format!("{name} zone"),
            metadata: None,
        })
        .await
        .expect("create service zone");
    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: geo_type.to_string(),
            country_code: "us".to_string(),
            province_code: province_code.map(str::to_string),
            city: city.map(str::to_string),
            postal_expression: postal_pattern.map(|pattern| json!({ "pattern": pattern })),
            metadata: None,
        })
        .await
        .expect("create geo zone");
    service
        .create_shipping_option(CreateShippingOptionInput {
            name: name.to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id,
            shipping_profile_id: profile_id.to_string(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(json!({ "amount": "5.00" })),
            metadata: None,
        })
        .await
        .expect("create shipping option")
}

#[tokio::test]
async fn geo_constraints_and_unknown_option_rules_fail_closed() {
    let db = TestDb::migrated().await;
    let service = FulfillmentService::new(create_pool(&db.url, 5).expect("pool"));
    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Default".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("create profile");

    let country = option_for_geo(
        &service,
        &profile.id,
        "Country",
        "country",
        None,
        None,
        None,
    )
    .await;
    let province = option_for_geo(
        &service,
        &profile.id,
        "Province",
        "province",
        Some("ca"),
        None,
        None,
    )
    .await;
    let city = option_for_geo(
        &service,
        &profile.id,
        "City",
        "city",
        Some("ca"),
        Some("Los Angeles"),
        None,
    )
    .await;
    let postal = option_for_geo(
        &service,
        &profile.id,
        "Postal",
        "zip",
        None,
        None,
        Some("90*"),
    )
    .await;

    let country_only = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolve country only");
    assert_eq!(
        country_only
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec![country.id.as_str()]
    );

    let province_match = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            province_code: Some("CA".to_string()),
            city: Some("Los Angeles".to_string()),
            postal_code: Some("90001".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolve detailed destination");
    let ids = province_match
        .iter()
        .map(|option| option.id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&country.id.as_str()));
    assert!(ids.contains(&province.id.as_str()));
    assert!(ids.contains(&city.id.as_str()));
    assert!(ids.contains(&postal.id.as_str()));

    let missing_postal = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            province_code: Some("CA".to_string()),
            city: Some("Los Angeles".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolve without postal code");
    assert!(!missing_postal.iter().any(|option| option.id == postal.id));

    let wrong_province = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            province_code: Some("NY".to_string()),
            city: Some("Los Angeles".to_string()),
            postal_code: Some("90001".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolve mismatched province");
    assert!(!wrong_province.iter().any(|option| option.id == city.id));

    service
        .add_shipping_option_rule(CreateShippingOptionRuleInput {
            shipping_option_id: country.id.clone(),
            attribute: "unrecognized_rule".to_string(),
            operator: "eq".to_string(),
            value: json!("anything"),
            metadata: None,
        })
        .await
        .expect("add rule that resolver must fail closed on");
    let unsupported_rule = service
        .resolve_eligible_shipping_options(&EligibilityContext {
            country_code: Some("us".to_string()),
            ..Default::default()
        })
        .await
        .expect("resolve unsupported option rule");
    assert!(!unsupported_rule
        .iter()
        .any(|option| option.id == country.id));

    db.drop().await;
}
