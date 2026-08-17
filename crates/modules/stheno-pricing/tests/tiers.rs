use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingContext, PricingService};

#[tokio::test]
async fn test_pricing_quantity_tiers_resolution() {
    let db = TestDb::migrated().await;
    let service = PricingService::new(create_pool(&db.url, 5).expect("pool"));

    let set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                // 1 - 9 units: $50
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(50.00),
                    min_quantity: Some(1),
                    max_quantity: Some(9),
                    price_list_id: None,
                    rules: None,
                },
                // 10 - 49 units: $40
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(40.00),
                    min_quantity: Some(10),
                    max_quantity: Some(49),
                    price_list_id: None,
                    rules: None,
                },
                // 50+ units: $30
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(30.00),
                    min_quantity: Some(50),
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
            ],
        })
        .await
        .expect("must create tiered price set");

    // Quantity 5 -> $50
    let calc_5 = service
        .calculate_price(
            &set.id.to_string(),
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(5),
                as_of: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(calc_5.calculated_amount, dec!(50.00));

    // Quantity 25 -> $40
    let calc_25 = service
        .calculate_price(
            &set.id.to_string(),
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(25),
                as_of: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(calc_25.calculated_amount, dec!(40.00));

    // Quantity 100 -> $30
    let calc_100 = service
        .calculate_price(
            &set.id.to_string(),
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(100),
                as_of: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(calc_100.calculated_amount, dec!(30.00));

    db.drop().await;
}
