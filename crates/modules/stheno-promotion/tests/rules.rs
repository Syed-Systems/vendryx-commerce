use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{PromotionEvaluationContext, PromotionRuleDTO, PromotionService};

#[tokio::test]
async fn test_rule_engine_all_8_operators() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    let mut context = PromotionEvaluationContext::default();
    context
        .attributes
        .insert("currency_code".to_string(), "usd".to_string());
    context
        .attributes
        .insert("subtotal".to_string(), "100.00".to_string());
    context
        .attributes
        .insert("customer_group".to_string(), "vip".to_string());

    let make_rule = |attr: &str, op: &str, values: &[&str]| PromotionRuleDTO {
        id: "r1".to_string(),
        promotion_id: None,
        application_method_id: None,
        rule_type: "rules".to_string(),
        attribute: attr.to_string(),
        operator: op.to_string(),
        description: None,
        values: values.iter().map(|s| s.to_string()).collect(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 1. eq
    assert!(service.evaluate_rule(&make_rule("currency_code", "eq", &["usd"]), &context));
    assert!(!service.evaluate_rule(&make_rule("currency_code", "eq", &["eur"]), &context));

    // 2. ne
    assert!(service.evaluate_rule(&make_rule("currency_code", "ne", &["eur"]), &context));
    assert!(!service.evaluate_rule(&make_rule("currency_code", "ne", &["usd"]), &context));

    // 3. in
    assert!(service.evaluate_rule(
        &make_rule("customer_group", "in", &["vip", "wholesale"]),
        &context
    ));
    assert!(!service.evaluate_rule(
        &make_rule("customer_group", "in", &["retail", "guest"]),
        &context
    ));

    // 4. nin
    assert!(service.evaluate_rule(
        &make_rule("customer_group", "nin", &["retail", "guest"]),
        &context
    ));
    assert!(!service.evaluate_rule(
        &make_rule("customer_group", "nin", &["vip", "guest"]),
        &context
    ));

    // 5. gt
    assert!(service.evaluate_rule(&make_rule("subtotal", "gt", &["50.00"]), &context));
    assert!(!service.evaluate_rule(&make_rule("subtotal", "gt", &["100.00"]), &context));
    assert!(!service.evaluate_rule(&make_rule("subtotal", "gt", &["150.00"]), &context));

    // 6. gte
    assert!(service.evaluate_rule(&make_rule("subtotal", "gte", &["100.00"]), &context));
    assert!(service.evaluate_rule(&make_rule("subtotal", "gte", &["50.00"]), &context));
    assert!(!service.evaluate_rule(&make_rule("subtotal", "gte", &["100.01"]), &context));

    // 7. lt
    assert!(service.evaluate_rule(&make_rule("subtotal", "lt", &["150.00"]), &context));
    assert!(!service.evaluate_rule(&make_rule("subtotal", "lt", &["100.00"]), &context));

    // 8. lte
    assert!(service.evaluate_rule(&make_rule("subtotal", "lte", &["100.00"]), &context));
    assert!(service.evaluate_rule(&make_rule("subtotal", "lte", &["150.00"]), &context));
    assert!(!service.evaluate_rule(&make_rule("subtotal", "lte", &["99.99"]), &context));
}
