use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use stheno_pricing::{
    CreatePriceInput, CreatePriceListInput, CreatePriceSetInput, PricingContext, PricingService,
};

#[tokio::test]
async fn test_price_lists_sale_and_override_precedence_and_date_windows() {
    let service = PricingService::new();
    let now = Utc::now();

    // 1. Create Active Sale List
    let sale_list = service
        .create_price_list(CreatePriceListInput {
            title: "Summer Flash Sale".to_string(),
            description: Some("20% off items".to_string()),
            r#type: "sale".to_string(),
            starts_at: Some(now - Duration::days(2)),
            ends_at: Some(now + Duration::days(5)),
            rules: None,
        })
        .await
        .expect("must create sale list");

    // 2. Create Active Override List (e.g. VIP B2B Contract)
    let override_list = service
        .create_price_list(CreatePriceListInput {
            title: "Executive Override Contract".to_string(),
            description: Some("Custom fixed pricing".to_string()),
            r#type: "override".to_string(),
            starts_at: Some(now - Duration::days(1)),
            ends_at: Some(now + Duration::days(30)),
            rules: None,
        })
        .await
        .expect("must create override list");

    // 3. Create Price Set with Base ($100), Sale ($80), and Override ($60)
    let set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(100.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(80.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: Some(sale_list.id.to_string()),
                    rules: None,
                },
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(60.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: Some(override_list.id.to_string()),
                    rules: None,
                },
            ],
        })
        .await
        .expect("must create price set");

    // Override ($60) takes precedence over Sale ($80) and Base ($100)
    let calc = service
        .calculate_price(
            &set.id.to_string(),
            &PricingContext {
                currency_code: "usd".to_string(),
                region_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(1),
                as_of: Some(now),
            },
        )
        .await
        .unwrap();

    assert_eq!(calc.calculated_amount, dec!(60.00));
    assert_eq!(calc.original_amount, dec!(100.00));
    assert_eq!(calc.price_list_type.as_deref(), Some("override"));
}
