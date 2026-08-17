use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;

#[tokio::test]
async fn test_fulfillment_sets_service_zones_and_geo_zones() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    // 1. Create fulfillment set
    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "EU Fulfillment Center".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    // 2. Create service zone
    let zone = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "DACH Region".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    // 3. Create Geo Zones for Germany (country), Bavaria (province), Berlin (city), and zip pattern
    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: "country".to_string(),
            country_code: "de".to_string(),
            province_code: None,
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("country geo created");

    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: "province".to_string(),
            country_code: "de".to_string(),
            province_code: Some("by".to_string()),
            city: None,
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("province geo created");

    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: "city".to_string(),
            country_code: "de".to_string(),
            province_code: None,
            city: Some("Berlin".to_string()),
            postal_expression: None,
            metadata: None,
        })
        .await
        .expect("city geo created");

    service
        .create_geo_zone(CreateGeoZoneInput {
            service_zone_id: zone.id.clone(),
            r#type: "zip".to_string(),
            country_code: "de".to_string(),
            province_code: None,
            city: None,
            postal_expression: Some(json!({ "pattern": "10***" })),
            metadata: None,
        })
        .await
        .expect("zip geo created");

    // 4. Retrieve complete hierarchy
    let retrieved_set = service
        .get_fulfillment_set(&set.id)
        .await
        .expect("set loaded");
    assert_eq!(retrieved_set.service_zones.len(), 1);
    assert_eq!(retrieved_set.service_zones[0].geo_zones.len(), 4);

    db.drop().await;
}
