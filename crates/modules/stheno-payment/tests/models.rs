use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_payment::{
    AuthorizePaymentSessionInput, CapturePaymentInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService, RefundPaymentInput,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_payment_all_entities_lifecycle() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // 1. PaymentCollection
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(150.00),
            metadata: None,
        })
        .await
        .expect("collection created");

    assert_eq!(col.amount, dec!(150.00));
    assert_eq!(col.status, "not_paid");

    // 2. PaymentSession
    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(150.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            Some("127.0.0.1"),
        )
        .await
        .expect("session created");

    assert_eq!(session.payment_collection_id, col.id);

    // 3. Authorize -> Creates Payment (created ONLY on successful authorization)
    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("authorized and payment created");

    assert_eq!(payment.amount, dec!(150.00));

    // 4. Capture (Partial: $100)
    let cap1 = service
        .capture_payment(CapturePaymentInput {
            payment_id: payment.id.clone(),
            amount: Some(dec!(100.00)),
            created_by: Some("admin_1".to_string()),
            metadata: None,
        })
        .await
        .expect("capture succeeds");
    assert_eq!(cap1.amount, dec!(100.00));

    // 5. Refund (Partial: $30)
    let ref1 = service
        .refund_payment(RefundPaymentInput {
            payment_id: payment.id.clone(),
            amount: dec!(30.00),
            refund_reason_id: Some("customer_request".to_string()),
            note: Some("partial return".to_string()),
            created_by: Some("admin_1".to_string()),
            metadata: None,
        })
        .await
        .expect("refund succeeds");
    assert_eq!(ref1.amount, dec!(30.00));

    // Reload collection to verify aggregate amounts
    let reloaded_col = service
        .get_payment_collection(&col.id)
        .await
        .expect("reloaded collection");
    assert_eq!(reloaded_col.authorized_amount, dec!(150.00));
    assert_eq!(reloaded_col.captured_amount, dec!(100.00));
    assert_eq!(reloaded_col.refunded_amount, dec!(30.00));
}
