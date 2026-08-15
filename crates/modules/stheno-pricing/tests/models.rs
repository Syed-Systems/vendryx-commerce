use rust_decimal_macros::dec;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingService};

#[tokio::test]
async fn test_pricing_entity_models_lifecycle() {
    let service = PricingService::new();

    let price_set = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![
                CreatePriceInput {
                    currency_code: "usd".to_string(),
                    amount: dec!(49.99),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
                CreatePriceInput {
                    currency_code: "eur".to_string(),
                    amount: dec!(45.00),
                    min_quantity: None,
                    max_quantity: None,
                    price_list_id: None,
                    rules: None,
                },
            ],
        })
        .await
        .expect("must create price set");

    assert!(!price_set.id.to_string().is_empty());
}
