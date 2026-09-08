//! P23-02: stheno-fulfillment-pathao crate, registered, options and serviceable-area lookup.

use serde_json::json;
use stheno_fulfillment::FulfillmentProvider;
use stheno_fulfillment_pathao::{PathaoConfig, PathaoFulfillmentProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn provider_registered_and_options_honest() {
    let provider = PathaoFulfillmentProvider::default();
    assert_eq!(provider.identifier(), "pathao");

    let options = provider
        .get_fulfillment_options()
        .await
        .expect("fulfillment options");

    assert_eq!(options.len(), 4);
    assert!(options.iter().any(|o| o.id == "pathao-standard" && !o.is_return));
    assert!(options.iter().any(|o| o.id == "pathao-sub-city" && !o.is_return));
    assert!(options.iter().any(|o| o.id == "pathao-outside-city" && !o.is_return));
    assert!(options.iter().any(|o| o.id == "pathao-return" && o.is_return));
}

#[tokio::test]
async fn serviceable_area_hierarchy_lookup_wiremock() {
    let mock_server = MockServer::start().await;

    // 1. Mock token
    Mock::given(method("POST"))
        .and(path("/aladdin/api/v1/issue-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "mock_token_pathao_123",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // 2. Mock cities
    Mock::given(method("GET"))
        .and(path("/aladdin/api/v1/countries/1/city-list"))
        .and(header("Authorization", "Bearer mock_token_pathao_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "data": [
                    { "city_id": 1, "city_name": "Dhaka" },
                    { "city_id": 2, "city_name": "Chittagong" }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // 3. Mock zones
    Mock::given(method("GET"))
        .and(path("/aladdin/api/v1/cities/1/zone-list"))
        .and(header("Authorization", "Bearer mock_token_pathao_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "data": [
                    { "zone_id": 101, "zone_name": "Gulshan", "city_id": 1 },
                    { "zone_id": 102, "zone_name": "Dhanmondi", "city_id": 1 }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // 4. Mock areas
    Mock::given(method("GET"))
        .and(path("/aladdin/api/v1/zones/101/area-list"))
        .and(header("Authorization", "Bearer mock_token_pathao_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "data": [
                    { "area_id": 1001, "area_name": "Gulshan-1", "zone_id": 101 },
                    { "area_id": 1002, "area_name": "Gulshan-2", "zone_id": 101 }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let provider = PathaoFulfillmentProvider::new(PathaoConfig {
        base_url: mock_server.uri(),
        ..PathaoConfig::default()
    });

    let cities = provider.list_cities().await.expect("list cities");
    assert_eq!(cities.len(), 2);
    assert_eq!(cities[0].city_name, "Dhaka");

    let zones = provider.list_zones(1).await.expect("list zones");
    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].zone_name, "Gulshan");

    let areas = provider.list_areas(101).await.expect("list areas");
    assert_eq!(areas.len(), 2);
    assert_eq!(areas[0].area_name, "Gulshan-1");

    assert!(provider.is_serviceable_area(1, 101, Some(1001)).await.expect("serviceable check"));
    assert!(!provider.is_serviceable_area(999, 101, None).await.expect("invalid city check"));
}
