use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingContext, PricingService};

#[tokio::test]
async fn test_price_rules_count_and_specificity_ranking() {
    let db = TestDb::migrated().await;
    let service = PricingService::new(create_pool(&db.url, 5).expect("pool"));

    let mut general_rules = HashMap::new();
    general_rules.insert("region_id".to_string(), "reg_us".to_string());

    let mut specific_rules = HashMap::new();
    specific_rules.insert("region_id".to_string(), "reg_us".to_string());
    specific_rules.insert("customer_group_id".to_string(), "cgrp_vip".to_string());

    let set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                // Base USD: $100 (0 rules)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(100.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                // US Region USD: $90 (1 rule)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(90.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: Some(general_rules),
                },
                // US VIP Customer USD: $75 (2 rules - higher specificity)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(75.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: Some(specific_rules),
                },
            ],
        })
        .await
        .expect("must create price set");

    // 1. VIP Customer in US gets $75 (2 matching rules win over 1 rule and base)
    let vip_calc = service
        .calculate_price(
            &set.id.to_string(),
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: Some("reg_us".to_string()),
                customer_group_id: Some("cgrp_vip".to_string()),
                sales_channel_id: None,
                quantity: Some(1),
                as_of: None,
            },
        )
        .await
        .expect("must calculate price");

    assert_eq!(vip_calc.calculated_amount, dec!(75.00));
    assert_eq!(vip_calc.matched_rules_count, 2);

    // 2. Regular Customer in US gets $90 (1 matching rule wins over base)
    let regular_calc = service
        .calculate_price(
            &set.id.to_string(),
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
        .expect("must calculate price");

    assert_eq!(regular_calc.calculated_amount, dec!(90.00));
    assert_eq!(regular_calc.matched_rules_count, 1);

    db.drop().await;
}
