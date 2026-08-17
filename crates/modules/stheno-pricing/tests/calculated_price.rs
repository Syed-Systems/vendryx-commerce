use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingContext, PricingService};

#[tokio::test]
async fn test_calculated_price_ranking_and_batch_evaluation() {
    let db = TestDb::migrated().await;
    let service = PricingService::new(create_pool(&db.url, 5).expect("pool"));

    // 1. Create Price Set 1 (T-Shirt) with Base ($25) and US Region ($20)
    let mut us_rule = HashMap::new();
    us_rule.insert("region_id".to_string(), "reg_us".to_string());

    let set1 = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(25.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(20.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: Some(us_rule),
                },
            ],
        })
        .await
        .expect("must create set 1");

    // 2. Create Price Set 2 (Hoodie) with Base ($60)
    let set2 = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![CreatePriceInput {
                currency_code: "usd".to_string(),
                amount: dec!(60.00),
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            }],
        })
        .await
        .expect("must create set 2");

    let context = PricingContext {
        currency_code: "usd".to_string(),
        region_id: Some("reg_us".to_string()),
        customer_group_id: None,
        sales_channel_id: None,
        quantity: Some(1),
        as_of: None,
    };

    let set1_id = set1.id.to_string();
    let set2_id = set2.id.to_string();

    let batch = service
        .calculate_prices_batch(&[set1_id.clone(), set2_id.clone()], &context)
        .await
        .expect("must calculate batch");

    assert_eq!(batch.len(), 2);
    // Set 1 gets US region price ($20)
    assert_eq!(batch.get(&set1_id).unwrap().calculated_amount, dec!(20.00));

    // Set 2 gets base price ($60)
    assert_eq!(batch.get(&set2_id).unwrap().calculated_amount, dec!(60.00));

    db.drop().await;
}
