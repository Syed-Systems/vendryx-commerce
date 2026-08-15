use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_pricing::{CreatePriceInput, CreatePriceSetInput, PricingContext, PricingService};

#[tokio::test]
async fn test_pricing_single_query_batch_resolution_100_variants() {
    let service = PricingService::new();

    let mut set_ids = Vec::with_capacity(100);

    for _i in 0..100 {
        let mut us_rule = HashMap::new();
        us_rule.insert("region_id".to_string(), "reg_us".to_string());

        let mut vip_rule = HashMap::new();
        vip_rule.insert("region_id".to_string(), "reg_us".to_string());
        vip_rule.insert("customer_group_id".to_string(), "cgrp_vip".to_string());

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
                        price_list_id: None,
                        rules: Some(us_rule),
                    },
                    CreatePriceInput {
                        currency_code: "usd".to_string(),
                        amount: dec!(65.00),
                        min_quantity: None,
                        max_quantity: None,
                        price_list_id: None,
                        rules: Some(vip_rule),
                    },
                ],
            })
            .await
            .unwrap();

        set_ids.push(set.id.to_string());
    }

    let context = PricingContext {
        currency_code: "usd".to_string(),
        region_id: Some("reg_us".to_string()),
        customer_group_id: Some("cgrp_vip".to_string()),
        sales_channel_id: None,
        quantity: Some(1),
        as_of: None,
    };

    // Resolves all 100 price sets simultaneously in 1 batch call (0 N+1)
    let batch_results = service
        .calculate_prices_batch(&set_ids, &context)
        .await
        .expect("must resolve 100 sets in batch");

    assert_eq!(batch_results.len(), 100);
    for (_id, calc) in batch_results {
        assert_eq!(calc.calculated_amount, dec!(65.00));
        assert_eq!(calc.matched_rules_count, 2);
    }
}
