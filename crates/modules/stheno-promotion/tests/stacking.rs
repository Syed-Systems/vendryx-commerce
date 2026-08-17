use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{
    ApplicationMethodDTO, PromotionDTO, PromotionEvaluationContext, PromotionItemContext,
    PromotionService,
};

#[tokio::test]
async fn test_deterministic_stacking_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    // Item: $100 subtotal
    let context = PromotionEvaluationContext {
        attributes: HashMap::new(),
        items: vec![PromotionItemContext {
            id: "item_stack".to_string(),
            product_id: Some("prod_100".to_string()),
            product_category_id: None,
            product_collection_id: None,
            product_type_id: None,
            quantity: 1,
            unit_price: dec!(100.00),
            subtotal: dec!(100.00),
            is_discountable: true,
        }],
        shipping_methods: vec![],
    };

    // Promo A: $20 Fixed off Item
    let promo_a = PromotionDTO {
        id: "promo_a".to_string(),
        code: "FIXED20".to_string(),
        promotion_type: "standard".to_string(),
        is_automatic: false,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_a".to_string(),
            promotion_id: "promo_a".to_string(),
            method_type: "fixed".to_string(),
            target_type: "items".to_string(),
            allocation: "each".to_string(),
            value: dec!(20.00),
            currency_code: None,
            max_quantity: None,
            apply_to_quantity: None,
            buy_rules_min_quantity: None,
            target_rules: vec![],
            buy_rules: vec![],
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }),
        rules: vec![],
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Promo B: 10% off Item
    let promo_b = PromotionDTO {
        id: "promo_b".to_string(),
        code: "PCT10".to_string(),
        promotion_type: "standard".to_string(),
        is_automatic: false,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_b".to_string(),
            promotion_id: "promo_b".to_string(),
            method_type: "percentage".to_string(),
            target_type: "items".to_string(),
            allocation: "each".to_string(),
            value: dec!(10.00), // 10%
            currency_code: None,
            max_quantity: None,
            apply_to_quantity: None,
            buy_rules_min_quantity: None,
            target_rules: vec![],
            buy_rules: vec![],
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }),
        rules: vec![],
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Evaluate in reverse order [promo_b, promo_a] to test that sorting normalizes to Fixed first
    let adj = service
        .evaluate_promotions(&[promo_b, promo_a], &context)
        .await;

    assert_eq!(adj.len(), 2);
    // 1st applied: FIXED20 -> $20 discount on $100 -> remaining balance = $80
    assert_eq!(adj[0].code, "FIXED20");
    assert_eq!(adj[0].amount, dec!(20.00));

    // 2nd applied: PCT10 -> 10% of remaining $80 = $8.00
    assert_eq!(adj[1].code, "PCT10");
    assert_eq!(adj[1].amount, dec!(8.00));
}
