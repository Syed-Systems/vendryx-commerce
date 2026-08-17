use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_tax::{CreateTaxRateInput, CreateTaxRateRuleInput, CreateTaxRegionInput, TaxService};

#[tokio::test]
async fn test_all_4_tax_entities_via_service() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = TaxService::new(pool);

    // 1. TaxRegion — country level
    let us = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "us".to_string(),
            province_code: None,
            parent_id: None,
            provider_id: None,
        })
        .await
        .expect("must create US region");

    assert_eq!(us.country_code, "us");
    assert!(us.province_code.is_none());

    // 2. TaxRegion — province level (hierarchical child)
    let us_ca = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "us".to_string(),
            province_code: Some("ca".to_string()),
            parent_id: Some(us.id.to_string()),
            provider_id: None,
        })
        .await
        .expect("must create US-CA region");

    assert_eq!(us_ca.province_code, Some("ca".to_string()));
    assert_eq!(us_ca.parent_id, Some(us.id.to_string()));

    // 3. TaxRate with rules
    let rate = service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: us_ca.id.to_string(),
            rate: dec!(0.0725),
            code: "CA_STATE".to_string(),
            name: "California State Tax".to_string(),
            is_default: Some(true),
            is_combinable: Some(false),
            rules: Some(vec![CreateTaxRateRuleInput {
                reference: "product_type_id".to_string(),
                reference_id: "ptype_clothing".to_string(),
            }]),
        })
        .await
        .expect("must create rate with rule");

    assert_eq!(rate.rate, dec!(0.0725));
    assert_eq!(rate.code, "CA_STATE");
    assert!(rate.is_default);

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let region_id = {
        let first = TaxService::new(pool.clone());
        let reg = first
            .create_tax_region(CreateTaxRegionInput {
                country_code: "fr".to_string(),
                province_code: None,
                parent_id: None,
                provider_id: None,
            })
            .await
            .expect("create");
        reg.id.to_string()
    };

    let second = TaxService::new(pool);
    let found = second.get_tax_region(&region_id).await.expect("found");
    assert_eq!(found.country_code, "fr");

    db.drop().await;
}
