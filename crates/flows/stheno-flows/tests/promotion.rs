use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::PromotionWorkflows;
use stheno_promotion::*;

#[tokio::test]
async fn test_promotion_workflows_lifecycle_and_campaign_budget() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let promo_service = Arc::new(PromotionService::new(pool));
    let workflows = PromotionWorkflows::new(promo_service);

    // 1. Create Promotion Workflow
    let promo = workflows
        .create_promotion(CreatePromotionInput {
            code: "FLOW20".to_string(),
            promotion_type: Some("standard".to_string()),
            is_automatic: Some(false),
            campaign_id: None,
            status: Some("active".to_string()),
            application_method: Some(CreateApplicationMethodInput {
                method_type: "percentage".to_string(),
                target_type: "items".to_string(),
                allocation: Some("across".to_string()),
                value: dec!(20),
                currency_code: None,
                max_quantity: None,
                apply_to_quantity: None,
                buy_rules_min_quantity: None,
                target_rules: None,
                buy_rules: None,
                metadata: None,
            }),
            rules: Some(vec![]),
            metadata: None,
        })
        .await
        .expect("promotion created");

    assert_eq!(promo.code, "FLOW20");

    // 2. Validate Promo Code Workflow
    let fetched = workflows
        .validate_promo_code("flow20", Some("cart_123"), Some("127.0.0.1"))
        .await
        .expect("validated promo code");

    assert_eq!(fetched.id, promo.id);

    // 3. Create Campaign and Budget Workflow
    let campaign = workflows
        .create_campaign(CreateCampaignInput {
            name: "Summer Sale 2026".to_string(),
            campaign_identifier: "CAMP-SUMMER-2026".to_string(),
            starts_at: None,
            ends_at: None,
            currency: Some("USD".to_string()),
            budget: None,
            metadata: None,
        })
        .await
        .expect("campaign created");

    let budget = workflows
        .create_campaign_budget(
            &campaign.id,
            CreateCampaignBudgetInput {
                budget_type: "spend".to_string(),
                limit: Some(dec!(500.00)),
                currency_code: Some("USD".to_string()),
            },
        )
        .await
        .expect("budget created");

    assert_eq!(budget.limit, Some(dec!(500.00)));

    // 4. Consume Budget Workflow
    let usage = workflows
        .consume_campaign_budget(&campaign.id, dec!(75.00), Some("ord_1"), Some("cart_123"))
        .await
        .expect("budget consumed");

    assert_eq!(usage.amount, dec!(75.00));
}
