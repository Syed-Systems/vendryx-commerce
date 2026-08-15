use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_payment::{
    AuthorizePaymentSessionInput, CapturePaymentInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService, RefundPaymentInput,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_ledger_invariants_overcapture_and_overrefund_prevention() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(100.00),
            metadata: None,
        })
        .await
        .expect("col created");

    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(100.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("session created");

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("authorized");

    // 1. Partial capture: $60 (allowed, remaining $40)
    service
        .capture_payment(CapturePaymentInput {
            payment_id: payment.id.clone(),
            amount: Some(dec!(60.00)),
            created_by: None,
            metadata: None,
        })
        .await
        .expect("capture $60 succeeds");

    // 2. Over-capture: Attempt to capture $50 (60 + 50 = 110 > 100 -> Conflict error E22)
    let cap_err = service
        .capture_payment(CapturePaymentInput {
            payment_id: payment.id.clone(),
            amount: Some(dec!(50.00)),
            created_by: None,
            metadata: None,
        })
        .await
        .unwrap_err();

    match cap_err {
        stheno_types::Error::Conflict { code, .. } => {
            assert_eq!(code, "over_capture_prevented");
        }
        _ => panic!("expected Conflict over_capture_prevented"),
    }

    // 3. Partial refund: $40 on captured $60 (allowed, remaining refundable $20)
    service
        .refund_payment(RefundPaymentInput {
            payment_id: payment.id.clone(),
            amount: dec!(40.00),
            refund_reason_id: None,
            note: None,
            created_by: None,
            metadata: None,
        })
        .await
        .expect("refund $40 succeeds");

    // 4. Over-refund: Attempt to refund $30 (40 + 30 = 70 > 60 captured -> Conflict error E21)
    let ref_err = service
        .refund_payment(RefundPaymentInput {
            payment_id: payment.id.clone(),
            amount: dec!(30.00),
            refund_reason_id: None,
            note: None,
            created_by: None,
            metadata: None,
        })
        .await
        .unwrap_err();

    match ref_err {
        stheno_types::Error::Conflict { code, .. } => {
            assert_eq!(code, "over_refund_prevented");
        }
        _ => panic!("expected Conflict over_refund_prevented"),
    }
}
