//! ADR-034: proof submission remains pending verification and cannot reopen a paid invoice.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{GenerateInvoiceInput, PlatformService, UploadPaymentProofInput};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

#[tokio::test]
async fn proof_submission_is_pending_and_paid_invoices_cannot_be_reopened() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));
    let period_start = Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
    let invoice = service
        .generate_invoice(GenerateInvoiceInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            tier: "pro".to_string(),
            period_start,
            period_end,
            amount: Money::from_minor_units(20_000, CurrencyCode::Bdt),
            payable_amount: Some(Money::from_minor_units(20_000, CurrencyCode::Bdt)),
            fx_metadata: None,
        })
        .await
        .expect("invoice");

    let proof = service
        .upload_payment_proof(
            TestDb::TEST_TENANT,
            &invoice.id,
            UploadPaymentProofInput {
                payment_method: "bKash".to_string(),
                transaction_id: "  TXN-2026-001  ".to_string(),
                proof_asset_url: "uploads/payment-proof/001.png".to_string(),
            },
        )
        .await
        .expect("proof submission");
    assert_eq!(proof.status, "pending");
    assert_eq!(
        service.get_invoice(&invoice.id).await.unwrap().status,
        "pending_verification"
    );

    let second = service
        .upload_payment_proof(
            TestDb::TEST_TENANT,
            &invoice.id,
            UploadPaymentProofInput {
                payment_method: "nagad".to_string(),
                transaction_id: "TXN-2026-002".to_string(),
                proof_asset_url: "uploads/payment-proof/002.png".to_string(),
            },
        )
        .await
        .expect_err("one proof may be under review at a time");
    assert!(matches!(
        second,
        stheno_types::Error::Conflict {
            code: "payment_verification_pending",
            ..
        }
    ));

    service
        .mark_invoice_collected(&invoice.id, Utc::now())
        .await
        .expect("mark invoice paid");
    let paid_invoice = service
        .get_invoice(&invoice.id)
        .await
        .expect("paid invoice");
    assert_eq!(paid_invoice.status, "paid");
    let after_paid = service
        .upload_payment_proof(
            TestDb::TEST_TENANT,
            &invoice.id,
            UploadPaymentProofInput {
                payment_method: "bkash".to_string(),
                transaction_id: "TXN-2026-003".to_string(),
                proof_asset_url: "uploads/payment-proof/003.png".to_string(),
            },
        )
        .await
        .expect_err("a paid invoice must never return to verification");
    assert!(matches!(
        after_paid,
        stheno_types::Error::Conflict {
            code: "invoice_already_paid",
            ..
        }
    ));
    assert_eq!(
        service.get_invoice(&invoice.id).await.unwrap().status,
        "paid"
    );

    db.drop().await;
}

#[tokio::test]
async fn proof_submission_rejects_malformed_transaction_data_before_writing() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));
    let period_start = Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
    let invoice = service
        .generate_invoice(GenerateInvoiceInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            tier: "pro".to_string(),
            period_start,
            period_end,
            amount: Money::from_minor_units(20_000, CurrencyCode::Bdt),
            payable_amount: Some(Money::from_minor_units(20_000, CurrencyCode::Bdt)),
            fx_metadata: None,
        })
        .await
        .expect("invoice");

    for (input, expected_code) in [
        (
            UploadPaymentProofInput {
                payment_method: "unknown".to_string(),
                transaction_id: "TXN".to_string(),
                proof_asset_url: "proof.png".to_string(),
            },
            "unsupported_manual_payment_method",
        ),
        (
            UploadPaymentProofInput {
                payment_method: "bkash".to_string(),
                transaction_id: "TX\nN".to_string(),
                proof_asset_url: "proof.png".to_string(),
            },
            "invalid_transaction_id",
        ),
        (
            UploadPaymentProofInput {
                payment_method: "bkash".to_string(),
                transaction_id: "TXN".to_string(),
                proof_asset_url: "".to_string(),
            },
            "invalid_payment_proof_asset",
        ),
    ] {
        let error = service
            .upload_payment_proof(TestDb::TEST_TENANT, &invoice.id, input)
            .await
            .expect_err("malformed proof data must be rejected");
        assert!(matches!(
            error,
            stheno_types::Error::Invalid { code, .. } if code == expected_code
        ));
    }
    assert_eq!(
        service.get_invoice(&invoice.id).await.unwrap().status,
        "open"
    );

    db.drop().await;
}
