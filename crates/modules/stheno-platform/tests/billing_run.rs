//! P30-15: the scheduled billing run issues due invoices and is safe to repeat.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{
    BillingPlanPrice, BillingRunInput, NewTenant, PlatformService, SubscriptionPatch,
};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

async fn service() -> (TestDb, PlatformService) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);
    service
        .upsert_fx_rate(
            "USD/BDT",
            rust_decimal::Decimal::new(120, 0),
            "test-fixture",
            "daily",
        )
        .await
        .expect("seed billing FX metadata");
    (db, service)
}

fn pro_price() -> BillingPlanPrice {
    BillingPlanPrice {
        tier: "pro".to_string(),
        amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
        interval_months: 1,
    }
}

#[tokio::test]
async fn billing_run_issues_due_invoices_once() {
    let (db, service) = service().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let first = service
        .run_billing(BillingRunInput {
            now,
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("first billing run");
    let second = service
        .run_billing(BillingRunInput {
            now,
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("repeated billing run");

    assert_eq!(first.invoices_created, 1);
    assert_eq!(second.invoices_created, 0);
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice WHERE tenant_id = 'ten_test'")
            .await,
        1,
        "a repeated scheduled run must not double bill a merchant"
    );

    let sub = service
        .get_subscription(TestDb::TEST_TENANT)
        .await
        .expect("subscription");
    assert_eq!(
        sub.current_period_start,
        Some(Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap())
    );

    db.drop().await;
}

#[tokio::test]
async fn billing_run_skips_unsold_plans() {
    let (db, service) = service().await;
    service
        .create_tenant(NewTenant {
            id: "ten_free_run".to_string(),
            name: "Free Run".to_string(),
            tier: "free".to_string(),
            domain: None,
        })
        .await
        .expect("free tenant");

    let result = service
        .run_billing(BillingRunInput {
            now: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("billing run");

    assert_eq!(result.invoices_created, 1);
    assert!(
        result.unsold_skipped >= 1,
        "the run must report unsold plans it skipped"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice WHERE tenant_id = 'ten_free_run'")
            .await,
        0,
        "an unpriced plan is not sold, not free"
    );

    db.drop().await;
}

#[tokio::test]
async fn collected_renewal_during_grace_extends_term_and_survives_old_grace_deadline() {
    let (db, service) = service().await;
    let term_end = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let renewal_time = term_end + chrono::Duration::days(3);
    service
        .update_subscription(
            TestDb::TEST_TENANT,
            SubscriptionPatch {
                tier: Some("pro".to_string()),
                term_ends_at: Some(Some(term_end)),
                ..Default::default()
            },
        )
        .await
        .expect("configure expired paid term");
    service
        .run_billing(BillingRunInput {
            now: term_end,
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("issue renewal invoice at term boundary");
    service
        .reconcile_expired_subscriptions(term_end)
        .await
        .expect("enter paid-term grace");
    let invoice = service
        .list_invoices_for_tenant(TestDb::TEST_TENANT)
        .await
        .expect("list renewal invoice")
        .into_iter()
        .next()
        .expect("renewal invoice exists");
    let renewed = service
        .mark_invoice_collected(&invoice.id, renewal_time)
        .await
        .expect("collect renewal");
    let new_term_end = service
        .get_subscription(TestDb::TEST_TENANT)
        .await
        .expect("renewed subscription")
        .term_ends_at
        .expect("renewal records paid-through date");
    assert_eq!(renewed.status, "paid");
    assert!(new_term_end > term_end + chrono::Duration::days(7));

    service
        .reconcile_expired_subscriptions(term_end + chrono::Duration::days(7))
        .await
        .expect("old grace deadline is now stale");
    let after_old_deadline = service
        .get_subscription(TestDb::TEST_TENANT)
        .await
        .expect("subscription remains renewed");
    assert_eq!(after_old_deadline.tier, "pro");
    assert_eq!(after_old_deadline.status, "active");
    assert_eq!(after_old_deadline.term_ends_at, Some(new_term_end));

    db.drop().await;
}
