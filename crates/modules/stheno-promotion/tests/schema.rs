//! P13-08: the promotion schema exists, with constraints and foreign keys.

use stheno_db::testing::TestDb;

#[tokio::test]
async fn every_promotion_table_exists() {
    let db = TestDb::migrated().await;

    for table in [
        "campaign",
        "campaign_budget",
        "campaign_budget_usage",
        "promotion",
        "application_method",
        "promotion_rule",
        "promotion_rule_value",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'promotion' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "promotion.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in [
        "campaign",
        "campaign_budget",
        "campaign_budget_usage",
        "promotion",
        "application_method",
        "promotion_rule",
        "promotion_rule_value",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'promotion' AND table_name = '{table}' AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(exists, 1, "promotion.{table}.deleted_at is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn promotion_campaign_and_rules_foreign_keys_cascade() {
    let db = TestDb::migrated().await;

    // Seed campaign
    TestDb::execute(
        &db,
        "INSERT INTO promotion.campaign (id, name, campaign_identifier) \
         VALUES ('procamp_1', 'Summer Sale', 'SUMMER2026')",
    )
    .await;

    // Seed promotion referencing campaign
    TestDb::execute(
        &db,
        "INSERT INTO promotion.promotion (id, code, type, campaign_id, status) \
         VALUES ('promo_1', 'SUMMER20', 'standard', 'procamp_1', 'active')",
    )
    .await;

    // Seed application method
    TestDb::execute(
        &db,
        "INSERT INTO promotion.application_method (id, promotion_id, type, target_type, allocation, value) \
         VALUES ('proappmet_1', 'promo_1', 'percentage', 'items', 'across', 20.00)",
    )
    .await;

    // Seed promotion rule
    TestDb::execute(
        &db,
        "INSERT INTO promotion.promotion_rule (id, promotion_id, rule_type, attribute, operator) \
         VALUES ('prorul_1', 'promo_1', 'rules', 'currency_code', 'eq')",
    )
    .await;

    // Seed rule value
    TestDb::execute(
        &db,
        "INSERT INTO promotion.promotion_rule_value (id, promotion_rule_id, value) \
         VALUES ('promorulval_1', 'prorul_1', 'USD')",
    )
    .await;

    // Deleting promotion cascades application_method, rules, and rule_values
    TestDb::execute(&db, "DELETE FROM promotion.promotion WHERE id = 'promo_1'").await;

    assert_eq!(
        db.count(
            "SELECT count(*) FROM promotion.application_method WHERE promotion_id = 'promo_1'"
        )
        .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM promotion.promotion_rule WHERE promotion_id = 'promo_1'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM promotion.promotion_rule_value WHERE promotion_rule_id = 'prorul_1'")
            .await,
        0
    );

    db.drop().await;
}
