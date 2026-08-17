use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{
    ApplicationMethodDTO, PromotionDTO, PromotionEvaluationContext, PromotionItemContext,
    PromotionRuleDTO, PromotionService,
};

#[tokio::test]
async fn test_buyget_promotion_evaluation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    // Buy 2 Coffee Bags (cat_coffee), Get 1 Mug (cat_mug) at 100% off (free)
    let promo_buyget = PromotionDTO {
        id: "promo_bogo".to_string(),
        code: "BUY2_GET_MUG".to_string(),
        promotion_type: "buyget".to_string(),
        is_automatic: true,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_bogo".to_string(),
            promotion_id: "promo_bogo".to_string(),
            method_type: "percentage".to_string(), // 100% off
            target_type: "items".to_string(),
            allocation: "each".to_string(),
            value: dec!(100.00), // 100% off
            currency_code: None,
            max_quantity: None,
            apply_to_quantity: Some(1),      // Get 1
            buy_rules_min_quantity: Some(2), // Buy 2
            target_rules: vec![PromotionRuleDTO {
                id: "tr1".to_string(),
                promotion_id: None,
                application_method_id: Some("app_bogo".to_string()),
                rule_type: "target_rules".to_string(),
                attribute: "product_category_id".to_string(),
                operator: "eq".to_string(),
                description: None,
                values: vec!["cat_mug".to_string()],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }],
            buy_rules: vec![PromotionRuleDTO {
                id: "br1".to_string(),
                promotion_id: None,
                application_method_id: Some("app_bogo".to_string()),
                rule_type: "buy_rules".to_string(),
                attribute: "product_category_id".to_string(),
                operator: "eq".to_string(),
                description: None,
                values: vec!["cat_coffee".to_string()],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }],
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }),
        rules: vec![],
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Case 1: Meets buy condition (2x coffee) + contains 1x mug
    let context_eligible = PromotionEvaluationContext {
        attributes: HashMap::new(),
        items: vec![
            PromotionItemContext {
                id: "item_coffee".to_string(),
                product_id: Some("prod_espresso".to_string()),
                product_category_id: Some("cat_coffee".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 2,
                unit_price: dec!(18.00),
                subtotal: dec!(36.00),
                is_discountable: true,
            },
            PromotionItemContext {
                id: "item_mug".to_string(),
                product_id: Some("prod_ceramic_mug".to_string()),
                product_category_id: Some("cat_mug".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 1,
                unit_price: dec!(12.00),
                subtotal: dec!(12.00),
                is_discountable: true,
            },
        ],
        shipping_methods: vec![],
    };

    let adj1 = service
        .evaluate_promotions(std::slice::from_ref(&promo_buyget), &context_eligible)
        .await;
    assert_eq!(adj1.len(), 1);
    assert_eq!(adj1[0].line_item_id.as_deref(), Some("item_mug"));
    assert_eq!(adj1[0].amount, dec!(12.00)); // 100% of $12 mug

    // Case 2: Only 1x coffee (does not meet min buy quantity of 2)
    let context_ineligible = PromotionEvaluationContext {
        attributes: HashMap::new(),
        items: vec![
            PromotionItemContext {
                id: "item_coffee_1".to_string(),
                product_id: Some("prod_espresso".to_string()),
                product_category_id: Some("cat_coffee".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 1,
                unit_price: dec!(18.00),
                subtotal: dec!(18.00),
                is_discountable: true,
            },
            PromotionItemContext {
                id: "item_mug".to_string(),
                product_id: Some("prod_ceramic_mug".to_string()),
                product_category_id: Some("cat_mug".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 1,
                unit_price: dec!(12.00),
                subtotal: dec!(12.00),
                is_discountable: true,
            },
        ],
        shipping_methods: vec![],
    };

    let adj2 = service
        .evaluate_promotions(&[promo_buyget], &context_ineligible)
        .await;
    assert!(adj2.is_empty());
}
