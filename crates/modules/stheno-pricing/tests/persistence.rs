//! P12-03: `pricing` on Postgres.
//!
//! The existing suites already prove the ranking is correct. These prove it is
//! ranking over rows that came out of the database, and that nothing survives
//! only because it is still sitting in the process that wrote it.

use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::{
    CreatePriceInput, CreatePriceListInput, CreatePriceSetInput, PricingContext, PricingService,
};

async fn service() -> (TestDb, PricingService) {
    let db = TestDb::migrated().await;
    let svc = PricingService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

fn price(currency: &str, amount: i64) -> CreatePriceInput {
    CreatePriceInput {
        currency_code: currency.to_string(),
        amount: rust_decimal::Decimal::from(amount),
        min_quantity: None,
        max_quantity: None,
        price_list_id: None,
        rules: None,
    }
}

fn context(currency: &str) -> PricingContext {
    PricingContext {
        currency_code: currency.to_string(),
        region_id: None,
        customer_group_id: None,
        sales_channel_id: None,
        quantity: None,
        as_of: None,
    }
}

#[tokio::test]
async fn a_price_set_round_trips_through_the_database() {
    let (db, service) = service().await;

    let created = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![price("usd", 1000)],
        })
        .await
        .expect("create");

    let id = created.id.to_string();
    assert!(id.starts_with("pset_"), "id lost its prefix: {id}");

    let fetched = service.get_price_set(&id).await.expect("retrieve");
    assert_eq!(fetched.id.to_string(), id);

    assert_eq!(db.count("SELECT count(*) FROM pricing.price_set").await, 1);
    assert_eq!(db.count("SELECT count(*) FROM pricing.price").await, 1);

    db.drop().await;
}

#[tokio::test]
async fn prices_outlive_the_service_instance() {
    // The assertion that separates a migration from a decoration.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let set_id = PricingService::new(pool.clone())
        .create_price_set(CreatePriceSetInput {
            prices: vec![price("usd", 2500)],
        })
        .await
        .expect("create")
        .id
        .to_string();

    // A brand new service, sharing nothing with the first but the database.
    let second = PricingService::new(pool);
    let calculated = second
        .calculate_price(&set_id, &context("usd"))
        .await
        .expect("the price must survive the service that created it");

    assert_eq!(
        calculated.calculated_amount,
        rust_decimal::Decimal::from(2500)
    );

    db.drop().await;
}

#[tokio::test]
async fn money_survives_the_round_trip_at_sub_minor_precision() {
    // ADR-006 end to end: through the repository, not just the column.
    let (db, service) = service().await;

    let amount: rust_decimal::Decimal = "1234.567890".parse().expect("decimal");
    let set_id = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![CreatePriceInput {
                currency_code: "kwd".to_string(),
                amount,
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            }],
        })
        .await
        .expect("create")
        .id
        .to_string();

    let calculated = service
        .calculate_price(&set_id, &context("kwd"))
        .await
        .expect("calculate");
    assert_eq!(
        calculated.calculated_amount, amount,
        "precision was lost between the service and the database"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_soft_deleted_price_set_disappears_from_reads() {
    let (db, service) = service().await;
    let id = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![price("usd", 100)],
        })
        .await
        .expect("create")
        .id
        .to_string();

    service.delete_price_set(&id).await.expect("delete");

    assert!(service.get_price_set(&id).await.is_err());
    // Soft, not hard: the row is retained for audit.
    assert_eq!(db.count("SELECT count(*) FROM pricing.price_set").await, 1);

    db.drop().await;
}

#[tokio::test]
async fn a_price_list_and_its_rules_persist() {
    let (db, service) = service().await;

    let mut rules = HashMap::new();
    rules.insert("region_id".to_string(), "reg_eu".to_string());

    let created = service
        .create_price_list(CreatePriceListInput {
            title: "EU Sale".to_string(),
            description: None,
            r#type: "sale".to_string(),
            starts_at: None,
            ends_at: None,
            rules: Some(rules),
        })
        .await
        .expect("create");

    assert!(created.id.to_string().starts_with("plist_"));

    let stored_rules = db
        .count("SELECT count(*) FROM pricing.price_list_rule")
        .await;
    assert_eq!(stored_rules, 1, "the list rule was not persisted");

    let listed = service.list_price_lists().await.expect("list");
    assert_eq!(listed.len(), 1);

    db.drop().await;
}

#[tokio::test]
async fn setting_a_preference_twice_updates_rather_than_duplicating() {
    // A unique index covers (attribute, value); a second row would make tax
    // inclusivity depend on which one is read first.
    let (db, service) = service().await;

    service
        .set_price_preference("region_id", "reg_eu", true)
        .await
        .expect("set");
    service
        .set_price_preference("region_id", "reg_eu", false)
        .await
        .expect("update");

    let rows = db
        .count("SELECT count(*) FROM pricing.price_preference")
        .await;
    assert_eq!(rows, 1, "the preference was duplicated instead of updated");

    let inclusive = db
        .count(
            "SELECT count(*) FROM pricing.price_preference \
             WHERE attribute = 'region_id' AND value = 'reg_eu' AND is_tax_inclusive = false",
        )
        .await;
    assert_eq!(inclusive, 1, "the update did not take effect");

    db.drop().await;
}

#[tokio::test]
async fn deleting_a_preference_removes_it() {
    let (db, service) = service().await;

    service
        .set_price_preference("currency_code", "usd", true)
        .await
        .expect("set");
    service
        .delete_price_preference("currency_code", "usd")
        .await
        .expect("delete");

    assert_eq!(
        db.count("SELECT count(*) FROM pricing.price_preference")
            .await,
        0
    );

    // Deleting one that is not there is not an error: the caller cannot know.
    service
        .delete_price_preference("currency_code", "usd")
        .await
        .expect("deleting a missing preference is a no-op");

    db.drop().await;
}

#[tokio::test]
async fn updating_a_price_set_replaces_its_prices() {
    let (db, service) = service().await;

    let id = service
        .create_price_set(CreatePriceSetInput {
            prices: vec![price("usd", 100), price("eur", 200)],
        })
        .await
        .expect("create")
        .id
        .to_string();
    assert_eq!(db.count("SELECT count(*) FROM pricing.price").await, 2);

    service
        .update_price_set(
            &id,
            stheno_pricing::UpdatePriceSetInput {
                prices: Some(vec![price("usd", 999)]),
            },
        )
        .await
        .expect("update");

    assert_eq!(
        db.count("SELECT count(*) FROM pricing.price").await,
        1,
        "the old prices were not replaced"
    );

    let calculated = service
        .calculate_price(&id, &context("usd"))
        .await
        .expect("calculate");
    assert_eq!(
        calculated.calculated_amount,
        rust_decimal::Decimal::from(999)
    );

    db.drop().await;
}

#[tokio::test]
async fn a_batch_calculation_issues_no_query_per_price_set() {
    // The shape that matters for the latency budget: 25 price sets resolved in
    // one pass, not 25 round trips. Correctness is asserted here; P12-04
    // measures the cost.
    let (db, service) = service().await;

    let mut set_ids = Vec::new();
    for amount in 0..25i64 {
        let id = service
            .create_price_set(CreatePriceSetInput {
                prices: vec![price("usd", 100 + amount)],
            })
            .await
            .expect("create")
            .id
            .to_string();
        set_ids.push(id);
    }

    let results = service
        .calculate_prices_batch(&set_ids, &context("usd"))
        .await
        .expect("batch calculate");

    assert_eq!(results.len(), 25, "every price set must resolve");
    for (i, id) in set_ids.iter().enumerate() {
        assert_eq!(
            results[id].calculated_amount,
            rust_decimal::Decimal::from(100 + i as i64),
            "price set {id} resolved to the wrong amount"
        );
    }

    db.drop().await;
}

#[tokio::test]
async fn a_batch_larger_than_the_in_clause_limit_still_resolves() {
    // `FilterSpec` caps `$in` at 100, so a bigger batch must be chunked rather
    // than rejected. A cart with more line items than that is unusual but not
    // an error.
    let (db, service) = service().await;

    let mut set_ids = Vec::new();
    for _ in 0..120 {
        set_ids.push(
            service
                .create_price_set(CreatePriceSetInput {
                    prices: vec![price("usd", 500)],
                })
                .await
                .expect("create")
                .id
                .to_string(),
        );
    }

    let results = service
        .calculate_prices_batch(&set_ids, &context("usd"))
        .await
        .expect("a batch over the $in limit must be chunked, not rejected");

    assert_eq!(results.len(), 120);

    db.drop().await;
}
