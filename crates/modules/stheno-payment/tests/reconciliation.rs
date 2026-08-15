use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_payment::{
    AuthorizePaymentSessionInput, CapturePaymentInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService, PspReportItem,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_payment_reconciliation_drift_never_auto_corrects() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("stripe")));
    service.register_provider(offline);

    // 1. Create collection, session, authorize, capture
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(200.00),
            metadata: None,
        })
        .await
        .unwrap();

    let sess = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "stripe".to_string(),
                amount: dec!(200.00),
                currency_code: "usd".to_string(),
                data: Some(serde_json::json!({ "provider_payment_id": "pi_stripe_111" })),
                context: None,
            },
            None,
        )
        .await
        .unwrap();

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: sess.id.clone(),
            context: None,
        })
        .await
        .unwrap();

    let _captured = service
        .capture_payment(CapturePaymentInput {
            payment_id: payment.id.clone(),
            amount: Some(dec!(200.00)),
            created_by: Some("admin".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Prepare PSP settlement report:
    // Item 1: Exactly matches (payment 1, $200 captured)
    // Item 2: Drift: PSP reports $180.00 captured instead of $200.00
    // Item 3: Ghost charge in PSP (unknown in Stheno)
    let psp_report = vec![
        PspReportItem {
            provider_payment_id: "pi_stripe_111".to_string(),
            payment_id: Some(payment.id.clone()),
            amount: dec!(200.00),
            currency: "usd".to_string(),
            status: "captured".to_string(),
            fee: Some(dec!(6.10)),
        },
        PspReportItem {
            provider_payment_id: "pi_stripe_222".to_string(),
            payment_id: Some(payment.id.clone()),
            amount: dec!(180.00), // Discrepancy!
            currency: "usd".to_string(),
            status: "captured".to_string(),
            fee: Some(dec!(5.50)),
        },
        PspReportItem {
            provider_payment_id: "pi_stripe_unknown_999".to_string(),
            payment_id: None,
            amount: dec!(50.00),
            currency: "usd".to_string(),
            status: "captured".to_string(),
            fee: Some(dec!(1.50)),
        },
    ];

    let report = service
        .run_reconciliation("stripe", psp_report)
        .await
        .unwrap();

    assert_eq!(report.total_records_checked, 3);
    assert_eq!(report.matched_count, 1);
    assert_eq!(report.discrepancy_count, 2);

    // Rule PAYMENTS §9: Check that payment ledger did NOT mutate or auto-correct
    let payment_after = service.get_payment(&payment.id).await.unwrap();
    assert_eq!(payment_after.amount, dec!(200.00));
    assert_eq!(payment_after.captures.len(), 1);
    assert_eq!(payment_after.captures[0].amount, dec!(200.00));
}
