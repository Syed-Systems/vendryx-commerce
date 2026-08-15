use rust_decimal_macros::dec;
use stheno_tax::{
    CreateTaxRateInput, CreateTaxRateRuleInput, CreateTaxRegionInput, TaxContext, TaxService,
};

#[tokio::test]
async fn test_tax_rate_resolution_hierarchy_rules_combinable() {
    let service = TaxService::new();

    // 1. Create US country-level region
    let us = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "us".to_string(),
            province_code: None,
            parent_id: None,
            provider_id: None,
        })
        .await
        .unwrap();

    // 2. Create US-CA province-level region (child of US)
    let us_ca = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "us".to_string(),
            province_code: Some("ca".to_string()),
            parent_id: Some(us.id.to_string()),
            provider_id: None,
        })
        .await
        .unwrap();

    // 3. Create US default rate: 0% (federal, no sales tax)
    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: us.id.to_string(),
            rate: dec!(0),
            code: "US_FED".to_string(),
            name: "US Federal".to_string(),
            is_default: Some(true),
            is_combinable: None,
            rules: None,
        })
        .await
        .unwrap();

    // 4. Create CA default rate: 7.25%
    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: us_ca.id.to_string(),
            rate: dec!(0.0725),
            code: "CA_STATE".to_string(),
            name: "California State Tax".to_string(),
            is_default: Some(true),
            is_combinable: None,
            rules: None,
        })
        .await
        .unwrap();

    // 5. Create a specific rate for digital goods in CA: 0% (exempt)
    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: us_ca.id.to_string(),
            rate: dec!(0),
            code: "CA_DIGITAL_EXEMPT".to_string(),
            name: "CA Digital Goods Exemption".to_string(),
            is_default: Some(false),
            is_combinable: None,
            rules: Some(vec![CreateTaxRateRuleInput {
                reference: "product_type_id".to_string(),
                reference_id: "ptype_digital".to_string(),
            }]),
        })
        .await
        .unwrap();

    // Case A: Physical goods in CA -> 7.25%
    let lines_physical = service
        .resolve_tax_lines(&TaxContext {
            country_code: "us".to_string(),
            province_code: Some("ca".to_string()),
            product_type_id: Some("ptype_physical".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(lines_physical.len(), 1);
    assert_eq!(lines_physical[0].rate, dec!(0.0725));
    assert_eq!(lines_physical[0].code, "CA_STATE");

    // Case B: Digital goods in CA -> 0% (specific rule match)
    let lines_digital = service
        .resolve_tax_lines(&TaxContext {
            country_code: "us".to_string(),
            province_code: Some("ca".to_string()),
            product_type_id: Some("ptype_digital".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(lines_digital.len(), 1);
    assert_eq!(lines_digital[0].rate, dec!(0));
    assert_eq!(lines_digital[0].code, "CA_DIGITAL_EXEMPT");

    // Case C: No province specified for US -> falls to US default 0%
    let lines_us_generic = service
        .resolve_tax_lines(&TaxContext {
            country_code: "us".to_string(),
            province_code: None,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(lines_us_generic.len(), 1);
    assert_eq!(lines_us_generic[0].rate, dec!(0));
    assert_eq!(lines_us_generic[0].code, "US_FED");
}

#[tokio::test]
async fn test_tax_rate_combinable_rates() {
    let service = TaxService::new();

    // Create region
    let de = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "de".to_string(),
            province_code: None,
            parent_id: None,
            provider_id: None,
        })
        .await
        .unwrap();

    // Default VAT 19%
    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: de.id.to_string(),
            rate: dec!(0.19),
            code: "DE_VAT".to_string(),
            name: "German VAT".to_string(),
            is_default: Some(true),
            is_combinable: None,
            rules: None,
        })
        .await
        .unwrap();

    // Combinable environmental surcharge 0.5%
    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: de.id.to_string(),
            rate: dec!(0.005),
            code: "DE_ENV".to_string(),
            name: "Environmental Surcharge".to_string(),
            is_default: Some(false),
            is_combinable: Some(true),
            rules: None,
        })
        .await
        .unwrap();

    let lines = service
        .resolve_tax_lines(&TaxContext {
            country_code: "de".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    // Should get both: default VAT + combinable surcharge
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].rate, dec!(0.19));
    assert_eq!(lines[1].rate, dec!(0.005));
}
