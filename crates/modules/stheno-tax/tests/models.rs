use rust_decimal_macros::dec;
use stheno_tax::{CreateTaxRateInput, CreateTaxRateRuleInput, CreateTaxRegionInput, TaxService};

#[tokio::test]
async fn test_all_4_tax_entities_via_service() {
    let service = TaxService::new();

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
}
