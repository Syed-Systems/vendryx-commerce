use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{
    CreatePriceInput, CreatePriceListInput, CreatePriceSetInput, PricingContext, PricingService,
};

#[tokio::test]
async fn test_pricing_fixture_corpus_medusa_v2_parity() {
    let db = TestDb::migrated().await;
    let service = PricingService::new(create_pool(&db.url, 5).expect("pool"));
    let now = Utc::now();

    // 1. Create Sale List & Override List
    let sale_list = service
        .create_price_list(CreatePriceListInput {
            title: "Corpus Sale".to_string(),
            description: None,
            r#type: "sale".to_string(),
            starts_at: Some(now - Duration::days(1)),
            ends_at: Some(now + Duration::days(10)),
            rules: None,
        })
        .await
        .unwrap();

    let override_list = service
        .create_price_list(CreatePriceListInput {
            title: "Corpus VIP Contract".to_string(),
            description: None,
            r#type: "override".to_string(),
            starts_at: Some(now - Duration::days(1)),
            ends_at: Some(now + Duration::days(30)),
            rules: None,
        })
        .await
        .unwrap();

    let mut us_rule = HashMap::new();
    us_rule.insert("region_id".to_string(), "reg_us".to_string());

    let mut vip_rule = HashMap::new();
    vip_rule.insert("region_id".to_string(), "reg_us".to_string());
    vip_rule.insert("customer_group_id".to_string(), "cgrp_vip".to_string());

    let set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                // 1. Base USD ($100)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(100.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                // 2. Quantity tier (10+ units -> $90)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(90.00),
                    min_quantity: Some(10),
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                // 3. US Region ($85)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(85.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: Some(us_rule),
                },
                // 4. US VIP ($70)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(70.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: Some(vip_rule),
                },
                // 5. Sale Price ($75)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(75.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: Some(sale_list.id.to_string()),
                    rules: None,
                },
                // 6. Override Price ($55)
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(55.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: Some(override_list.id.to_string()),
                    rules: None,
                },
            ],
        })
        .await
        .unwrap();

    let set_id = set.id.to_string();

    // Case A: Override Price List wins ($55)
    let calc_override = service
        .calculate_price(
            &set_id,
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: Some("reg_us".to_string()),
                customer_group_id: Some("cgrp_vip".to_string()),
                sales_channel_id: None,
                quantity: Some(1),
                as_of: Some(now),
            },
        )
        .await
        .unwrap();
    assert_eq!(calc_override.calculated_amount, dec!(55.00));
    assert_eq!(calc_override.price_list_type.as_deref(), Some("override"));

    // Case B: Base fallback without region or list ($100 for 1 unit, $90 for 10 units)
    let calc_tier = service
        .calculate_price(
            &set_id,
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(15),
                as_of: Some(now + Duration::days(40)), // past override expiry
            },
        )
        .await
        .unwrap();
    assert_eq!(calc_tier.calculated_amount, dec!(90.00));

    db.drop().await;
}
