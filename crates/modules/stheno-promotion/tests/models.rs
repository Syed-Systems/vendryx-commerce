use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{
    CreateApplicationMethodInput, CreateCampaignBudgetInput, CreateCampaignInput,
    CreatePromotionInput, CreatePromotionRuleInput, PromotionService,
};

#[tokio::test]
async fn test_promotion_all_7_entities_lifecycle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    // 1. Campaign & 2. CampaignBudget
    let campaign = service
        .create_campaign(CreateCampaignInput {
            name: "Black Friday 2026".to_string(),
            campaign_identifier: "BF2026".to_string(),
            starts_at: Some(chrono::Utc::now()),
            ends_at: None,
            currency: Some("usd".to_string()),
            budget: Some(CreateCampaignBudgetInput {
                budget_type: "spend".to_string(),
                limit: Some(dec!(10000.00)),
                currency_code: Some("usd".to_string()),
            }),
            metadata: None,
        })
        .await
        .expect("campaign creation succeeds");

    assert_eq!(campaign.campaign_identifier, "BF2026");
    assert!(campaign.budget.is_some());
    assert_eq!(
        campaign.budget.as_ref().unwrap().limit,
        Some(dec!(10000.00))
    );

    // 3. CampaignBudgetUsage (atomic consumption)
    let usage = service
        .consume_campaign_budget(
            &campaign.id,
            dec!(50.00),
            Some("order_123"),
            Some("cart_123"),
        )
        .await
        .expect("budget consumption succeeds");
    assert_eq!(usage.amount, dec!(50.00));

    // Reload campaign to check budget used
    let reloaded_camp = service
        .get_campaign(&campaign.id)
        .await
        .expect("reloaded campaign");
    assert_eq!(reloaded_camp.budget.unwrap().used, dec!(50.00));

    // 4. Promotion, 5. ApplicationMethod, 6. PromotionRule, 7. PromotionRuleValue
    let promo = service
        .create_promotion(CreatePromotionInput {
            code: "SAVE20".to_string(),
            promotion_type: Some("standard".to_string()),
            is_automatic: Some(false),
            campaign_id: Some(campaign.id.clone()),
            status: Some("active".to_string()),
            application_method: Some(CreateApplicationMethodInput {
                method_type: "percentage".to_string(),
                target_type: "items".to_string(),
                allocation: Some("each".to_string()),
                value: dec!(20.00),
                currency_code: Some("usd".to_string()),
                max_quantity: Some(5),
                apply_to_quantity: None,
                buy_rules_min_quantity: None,
                target_rules: Some(vec![CreatePromotionRuleInput {
                    rule_type: Some("target_rules".to_string()),
                    attribute: "product_category_id".to_string(),
                    operator: "in".to_string(),
                    description: Some("Winter category".to_string()),
                    values: vec!["pcat_winter".to_string(), "pcat_apparel".to_string()],
                }]),
                buy_rules: None,
                metadata: None,
            }),
            rules: Some(vec![CreatePromotionRuleInput {
                rule_type: Some("rules".to_string()),
                attribute: "currency_code".to_string(),
                operator: "eq".to_string(),
                description: Some("USD only".to_string()),
                values: vec!["usd".to_string()],
            }]),
            metadata: None,
        })
        .await
        .expect("promotion creation succeeds");

    assert_eq!(promo.code, "SAVE20");
    assert_eq!(promo.status, "active");
    assert!(promo.application_method.is_some());
    assert_eq!(promo.rules.len(), 1);
    assert_eq!(
        promo
            .application_method
            .as_ref()
            .unwrap()
            .target_rules
            .len(),
        1
    );

    // Fetch by code
    let fetched_promo = service
        .get_promotion_by_code("save20")
        .await
        .expect("fetched promotion by code");
    assert_eq!(fetched_promo.id, promo.id);
}
