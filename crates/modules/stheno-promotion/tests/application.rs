use rust_decimal_macros::dec;
use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{
    ApplicationMethodDTO, PromotionDTO, PromotionEvaluationContext, PromotionItemContext,
    PromotionService, PromotionShippingContext,
};

#[tokio::test]
async fn test_application_methods_fixed_and_percentage_across_targets() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    let context = PromotionEvaluationContext {
        attributes: HashMap::new(),
        items: vec![
            PromotionItemContext {
                id: "i1".to_string(),
                product_id: Some("prod_1".to_string()),
                product_category_id: Some("cat_shoes".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 2,
                unit_price: dec!(50.00),
                subtotal: dec!(100.00),
                is_discountable: true,
            },
            PromotionItemContext {
                id: "i2".to_string(),
                product_id: Some("prod_2".to_string()),
                product_category_id: Some("cat_shoes".to_string()),
                product_collection_id: None,
                product_type_id: None,
                quantity: 1,
                unit_price: dec!(100.00),
                subtotal: dec!(100.00),
                is_discountable: true,
            },
        ],
        shipping_methods: vec![PromotionShippingContext {
            id: "sm1".to_string(),
            shipping_option_id: Some("so_express".to_string()),
            amount: dec!(20.00),
        }],
    };

    // 1. Percentage discount across items: 20% off
    let promo_pct_items = PromotionDTO {
        id: "promo_pct".to_string(),
        code: "20PCT_ITEMS".to_string(),
        promotion_type: "standard".to_string(),
        is_automatic: false,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_1".to_string(),
            promotion_id: "promo_pct".to_string(),
            method_type: "percentage".to_string(),
            target_type: "items".to_string(),
            allocation: "across".to_string(),
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

    let adj_pct = service
        .evaluate_promotions(&[promo_pct_items], &context)
        .await;
    let total_pct_discount: rust_decimal::Decimal = adj_pct.iter().map(|a| a.amount).sum();
    // 20% of (100 + 100) = 40.00
    assert_eq!(total_pct_discount, dec!(40.00));

    // 2. Fixed discount on shipping: $15 off
    let promo_ship = PromotionDTO {
        id: "promo_ship".to_string(),
        code: "15OFF_SHIP".to_string(),
        promotion_type: "standard".to_string(),
        is_automatic: false,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_2".to_string(),
            promotion_id: "promo_ship".to_string(),
            method_type: "fixed".to_string(),
            target_type: "shipping_methods".to_string(),
            allocation: "across".to_string(),
            value: dec!(15.00),
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

    let adj_ship = service.evaluate_promotions(&[promo_ship], &context).await;
    assert_eq!(adj_ship.len(), 1);
    assert_eq!(adj_ship[0].shipping_method_id.as_deref(), Some("sm1"));
    assert_eq!(adj_ship[0].amount, dec!(15.00));

    // 3. Fixed discount on order: $30 off distributed across items
    let promo_order = PromotionDTO {
        id: "promo_order".to_string(),
        code: "30OFF_ORDER".to_string(),
        promotion_type: "standard".to_string(),
        is_automatic: false,
        campaign_id: None,
        status: "active".to_string(),
        application_method: Some(ApplicationMethodDTO {
            id: "app_3".to_string(),
            promotion_id: "promo_order".to_string(),
            method_type: "fixed".to_string(),
            target_type: "order".to_string(),
            allocation: "across".to_string(),
            value: dec!(30.00),
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

    let adj_order = service.evaluate_promotions(&[promo_order], &context).await;
    let total_order_discount: rust_decimal::Decimal = adj_order.iter().map(|a| a.amount).sum();
    assert_eq!(total_order_discount, dec!(30.00));
}
