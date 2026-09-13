//! P30-13: credits, discounts and coupons are invoice adjustments, never negative bills.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{ApplyInvoiceCreditsInput, GenerateInvoiceInput, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::{Error, Money};

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

#[tokio::test]
async fn credits_discounts_and_coupons_reduce_the_invoice_total() {
    let (db, service, invoice_id) = service().await;

    let invoice = service
        .apply_invoice_credits(ApplyInvoiceCreditsInput {
            invoice_id: invoice_id.clone(),
            credit: Money::from_minor_units(20_000, CurrencyCode::Bdt),
            discount: Money::from_minor_units(10_000, CurrencyCode::Bdt),
            coupon: Money::from_minor_units(5_000, CurrencyCode::Bdt),
            reason: "operator goodwill".to_string(),
        })
        .await
        .expect("credits applied");

    assert_eq!(
        invoice.credit_total,
        Money::from_minor_units(35_000, CurrencyCode::Bdt)
    );
    assert_eq!(
        invoice.total,
        Money::from_minor_units(115_000, CurrencyCode::Bdt)
    );
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice_line WHERE kind = 'credit'")
            .await,
        1
    );
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice_line WHERE kind = 'discount'")
            .await,
        2,
        "discount and coupon both persist as discount lines"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_coupon_cannot_make_the_invoice_total_negative() {
    let (db, service, invoice_id) = service().await;

    let err = service
        .apply_invoice_credits(ApplyInvoiceCreditsInput {
            invoice_id,
            credit: Money::from_minor_units(0, CurrencyCode::Bdt),
            discount: Money::from_minor_units(0, CurrencyCode::Bdt),
            coupon: Money::from_minor_units(160_000, CurrencyCode::Bdt),
            reason: "too generous".to_string(),
        })
        .await
        .expect_err("over-crediting is refused");

    assert!(
        matches!(
            err,
            Error::Invalid {
                code: "invoice_total_would_be_negative",
                ..
            }
        ),
        "got {err:?}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice_line WHERE kind IN ('credit', 'discount')")
            .await,
        0,
        "refused adjustments must not leave invoice lines behind"
    );

    db.drop().await;
}
