use stheno_region::{CreateRegionInput, RegionService};

#[tokio::test]
async fn test_region_and_country_lifecycle() {
    let service = RegionService::new();

    // 1. Create European Union Region with Germany and France
    let eu_region = service
        .create_region(CreateRegionInput {
            name: "European Union".to_string(),
            currency_code: "eur".to_string(),
            automatic_taxes: Some(true),
            countries: vec!["de".to_string(), "fr".to_string()],
        })
        .await
        .expect("must create EU region");

    assert_eq!(eu_region.name, "European Union");
    assert_eq!(eu_region.currency_code, "eur");
    assert_eq!(eu_region.countries.len(), 2);

    let region_id = eu_region.id.to_string();

    // 2. Retrieve region by id
    let retrieved = service
        .get_region(&region_id)
        .await
        .expect("must get region");
    assert_eq!(retrieved.name, "European Union");
    assert_eq!(retrieved.countries.len(), 2);

    // 3. Resolve country by iso_2 code
    let de = service
        .get_country_by_iso2("de")
        .await
        .expect("must get DE");
    assert_eq!(de.iso_2, "de");

    // 4. List regions
    let list = service.list_regions().await.expect("must list regions");
    assert_eq!(list.len(), 1);
}
