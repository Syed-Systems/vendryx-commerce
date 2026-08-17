use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingContext, PricingService};

#[tokio::test]
async fn test_price_preferences_tax_inclusive_resolution() {
    let db = TestDb::migrated().await;
    let service = PricingService::new(create_pool(&db.url, 5).expect("pool"));

    // 1. Set EUR in EU region to be tax-inclusive
    service
        .set_price_preference("region_id", "reg_eu", true)
        .await
        .expect("must set pref");

    // 2. Set USD in US region to be tax-exclusive
    service
        .set_price_preference("region_id", "reg_us", false)
        .await
        .expect("must set pref");

    // 3. Create price set
    let set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                CreatePriceInput {
                    currency_code: "eur".to_string(),
                    amount: dec!(50.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(50.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
            ],
        })
        .await
        .expect("must create price set");

    let set_id = set.id.to_string();

    // 4. Calculate for EU -> tax_inclusive = true
    let calc_eu = service
        .calculate_price(
            &set_id,
            &PricingContext {
                currency_code: "eur".to_string(),
                region_id: Some("reg_eu".to_string()),
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(1),
                as_of: None,
            },
        )
        .await
        .expect("must calculate EU price");

    assert!(calc_eu.is_tax_inclusive, "EU price must be tax-inclusive");

    // 5. Calculate for US -> tax_inclusive = false
    let calc_us = service
        .calculate_price(
            &set_id,
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: Some("reg_us".to_string()),
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(1),
                as_of: None,
            },
        )
        .await
        .expect("must calculate US price");

    assert!(!calc_us.is_tax_inclusive, "US price must be tax-exclusive");

    db.drop().await;
}
