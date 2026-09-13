//! P30-07: subscription collection has its own outbound ledger.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{GenerateInvoiceInput, PlatformService, StartBillingAttemptInput};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

async fn service() -> (TestDb, PlatformService, String) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);
    let invoice = service
        .generate_invoice(GenerateInvoiceInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            tier: "pro".to_string(),
            period_start: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
            amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
            payable_amount: Some(Money::from_minor_units(150_000, CurrencyCode::Bdt)),
            fx_metadata: None,
        })
        .await
        .expect("invoice");
    (db, service, invoice.id)
}

fn attempt(invoice_id: &str) -> StartBillingAttemptInput {
    StartBillingAttemptInput {
        invoice_id: invoice_id.to_string(),
        tenant_id: TestDb::TEST_TENANT.to_string(),
        provider_id: "bkash".to_string(),
        idempotency_key: format!("bill:{invoice_id}:attempt:1"),
        request_hash: "sha256:payload-a".to_string(),
        amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
    }
}

#[tokio::test]
async fn retrying_the_same_collection_attempt_returns_the_existing_row() {
    let (db, service, invoice_id) = service().await;

    let first = service
        .start_billing_attempt(attempt(&invoice_id))
        .await
        .expect("first attempt");
    let second = service
        .start_billing_attempt(attempt(&invoice_id))
        .await
        .expect("retry returns existing attempt");

    assert_eq!(first.id, second.id);
    assert_eq!(first.state, "in_flight");
    assert_eq!(
        db.count("SELECT count(*) FROM platform.billing_attempt")
            .await,
        1,
        "a retry of the same idempotency key must not create a second charge attempt"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_unknown_attempt_is_not_retried_as_a_new_charge() {
    let (db, service, invoice_id) = service().await;

    let first = service
        .start_billing_attempt(attempt(&invoice_id))
        .await
        .expect("first attempt");
    service
        .mark_billing_attempt_unknown(&first.id, "provider timeout after request was sent")
        .await
        .expect("marked unknown");

    let retry = service
        .start_billing_attempt(attempt(&invoice_id))
        .await
        .expect("retry returns unknown attempt");

    assert_eq!(retry.id, first.id);
    assert_eq!(retry.state, "unknown");
    assert_eq!(
        db.count("SELECT count(*) FROM platform.billing_attempt")
            .await,
        1
    );

    db.drop().await;
}

#[tokio::test]
async fn an_idempotency_key_cannot_be_reused_for_a_different_payload() {
    let (db, service, invoice_id) = service().await;

    service
        .start_billing_attempt(attempt(&invoice_id))
        .await
        .expect("first attempt");

    let mut changed = attempt(&invoice_id);
    changed.request_hash = "sha256:payload-b".to_string();
    let err = service
        .start_billing_attempt(changed)
        .await
        .expect_err("different payload under same key is refused");

    assert!(
        matches!(
            err,
            stheno_types::Error::Invalid {
                code: "idempotency_key_reused",
                ..
            }
        ),
        "got {err:?}"
    );

    db.drop().await;
}
