use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_tax::{
    CreateTaxRateInput, CreateTaxRegionInput, SystemTaxProvider, TaxContext, TaxProviderTrait,
    TaxService,
};

#[tokio::test]
async fn test_system_tax_provider_trait() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = TaxService::new(pool);

    // Setup: UK region with 20% VAT
    let uk = service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "gb".to_string(),
            province_code: None,
            parent_id: None,
            provider_id: Some("system".to_string()),
        })
        .await
        .unwrap();

    service
        .create_tax_rate(CreateTaxRateInput {
            tax_region_id: uk.id.to_string(),
            rate: dec!(0.20),
            code: "UK_VAT".to_string(),
            name: "UK VAT".to_string(),
            is_default: Some(true),
            is_combinable: None,
            rules: None,
        })
        .await
        .unwrap();

    // Use the system provider trait
    let provider = SystemTaxProvider::new(service);
    let items = vec![
        TaxContext {
            country_code: "gb".to_string(),
            ..Default::default()
        },
        TaxContext {
            country_code: "gb".to_string(),
            ..Default::default()
        },
    ];

    let results = provider.get_tax_lines(&items).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].len(), 1);
    assert_eq!(results[0][0].rate, dec!(0.20));
    assert_eq!(results[0][0].code, "UK_VAT");
    assert_eq!(results[0][0].provider_id, Some("system".to_string()));

    assert_eq!(results[1].len(), 1);
    assert_eq!(results[1][0].rate, dec!(0.20));

    db.drop().await;
}
