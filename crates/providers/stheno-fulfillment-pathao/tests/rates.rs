use rust_decimal_macros::dec;
use serde_json::json;
use stheno_fulfillment::{CalculatePriceContext, FulfillmentProvider};
use stheno_fulfillment_pathao::PathaoFulfillmentProvider;

#[tokio::test]
async fn pathao_tier_rates_exact_decimal() {
    let provider = PathaoFulfillmentProvider::default();

    // 1. Inside city tier - base 60 BDT
    let rate_inside = provider
        .calculate_price(
            Some(&json!({ "tier": "inside_city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(0.5)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate inside rate");
    assert_eq!(rate_inside, dec!(60.00));

    // Inside city with overweight: 2.0kg -> 1.5kg extra, ceils to 2 * 15 = 30 -> 90 BDT
    let rate_inside_heavy = provider
        .calculate_price(
            Some(&json!({ "id": "pathao-standard" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 2,
                total_weight: Some(dec!(2.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate inside heavy rate");
    assert_eq!(rate_inside_heavy, dec!(90.00));

    // 2. Sub-city tier - base 100 BDT
    let rate_sub = provider
        .calculate_price(
            Some(&json!({ "id": "pathao-sub-city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(1.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate sub-city rate");
    assert_eq!(rate_sub, dec!(100.00));

    // Sub-city overweight: 3.0kg -> 2.0kg extra * 20 = 40 -> 140 BDT
    let rate_sub_heavy = provider
        .calculate_price(
            Some(&json!({ "tier": "sub_city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 3,
                total_weight: Some(dec!(3.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate sub-city heavy rate");
    assert_eq!(rate_sub_heavy, dec!(140.00));

    // 3. Outside city tier - base 130 BDT
    let rate_outside = provider
        .calculate_price(
            Some(&json!({ "id": "pathao-outside-city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(0.8)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate outside rate");
    assert_eq!(rate_outside, dec!(130.00));

    // Outside city overweight: 2.5kg -> 1.5kg extra -> ceil to 2 * 25 = 50 -> 180 BDT
    let rate_outside_heavy = provider
        .calculate_price(
            Some(&json!({ "tier": "outside_city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 2,
                total_weight: Some(dec!(2.5)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("calculate outside heavy rate");
    assert_eq!(rate_outside_heavy, dec!(180.00));

    // 4. Explicit option rate override
    let rate_custom = provider
        .calculate_price(
            Some(&json!({ "rate": "75.00" })),
            None,
            None,
        )
        .await
        .expect("custom rate");
    assert_eq!(rate_custom, dec!(75.00));
}
