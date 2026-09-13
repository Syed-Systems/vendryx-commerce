//! P30-04: generating an invoice is idempotent on `(tenant_id, period_start)`.
//!
//! The billing run may execute on two nodes at once. The database unique constraint is
//! the guard, and the service must turn the losing insert into "return the invoice that
//! already exists" rather than a conflict.

use chrono::{TimeZone, Utc};
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{GenerateInvoiceInput, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

fn invoice_input() -> GenerateInvoiceInput {
    GenerateInvoiceInput {
        tenant_id: TestDb::TEST_TENANT.to_string(),
        tier: "pro".to_string(),
        period_start: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
        period_end: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
        payable_amount: Some(Money::from_minor_units(150_000, CurrencyCode::Bdt)),
        fx_metadata: None,
    }
}

#[tokio::test]
async fn running_the_same_billing_period_twice_returns_the_existing_invoice() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    let first = service
        .generate_invoice(invoice_input())
        .await
        .expect("first invoice");
    let second = service
        .generate_invoice(invoice_input())
        .await
        .expect("second invoice returns the existing row");

    assert_eq!(first.id, second.id);
    assert_eq!(first.tenant_id, TestDb::TEST_TENANT);
    assert_eq!(first.tier, "pro");
    assert_eq!(
        first.total,
        Money::from_minor_units(150_000, CurrencyCode::Bdt)
    );
    assert_eq!(first.status, "open");

    assert_eq!(
        db.count(
            "SELECT count(*) FROM platform.invoice \
             WHERE tenant_id = 'ten_test' \
               AND period_start = '2026-08-01T00:00:00Z'::timestamptz"
        )
        .await,
        1,
        "one billing period must have one invoice row"
    );
    assert_eq!(
        db.count(
            "SELECT count(*) FROM platform.invoice_line l \
             JOIN platform.invoice i ON i.id = l.invoice_id \
             WHERE i.tenant_id = 'ten_test' \
               AND i.period_start = '2026-08-01T00:00:00Z'::timestamptz \
               AND l.kind = 'subscription'"
        )
        .await,
        1,
        "the invoice must have exactly one base subscription line"
    );

    db.drop().await;
}

#[tokio::test]
async fn concurrent_billing_runs_create_one_invoice() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let service = Arc::new(PlatformService::new(pool));

    let mut handles = Vec::new();
    for _ in 0..16 {
        let service = Arc::clone(&service);
        handles.push(tokio::spawn(async move {
            service.generate_invoice(invoice_input()).await
        }));
    }

    let mut invoice_ids = Vec::new();
    for handle in handles {
        let invoice = handle.await.expect("task joins").expect("invoice");
        invoice_ids.push(invoice.id);
    }

    invoice_ids.sort();
    invoice_ids.dedup();
    assert_eq!(
        invoice_ids.len(),
        1,
        "all concurrent callers must converge on the same invoice id"
    );
    assert_eq!(
        db.count(
            "SELECT count(*) FROM platform.invoice \
             WHERE tenant_id = 'ten_test' \
               AND period_start = '2026-08-01T00:00:00Z'::timestamptz"
        )
        .await,
        1,
        "the unique index must be the idempotency guarantee under concurrency"
    );

    db.drop().await;
}
