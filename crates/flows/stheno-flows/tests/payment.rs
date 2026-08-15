use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::PaymentWorkflows;
use stheno_payment::*;
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_payment_workflows_collection_and_capture_lifecycle() {
    let payment_service = Arc::new(PaymentService::new());
    let offline_provider = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    payment_service.register_provider(offline_provider);

    let workflows = PaymentWorkflows::new(payment_service);

    // 1. Create Payment Collection Workflow
    let collection = workflows
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "USD".to_string(),
            amount: dec!(150.00),
            metadata: None,
        })
        .await
        .expect("payment collection created");

    assert_eq!(collection.amount, dec!(150.00));
    assert_eq!(collection.status, "not_paid");

    // 2. Create Payment Session Workflow
    let session = workflows
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: collection.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(150.00),
                currency_code: "USD".to_string(),
                context: None,
                data: None,
            },
            Some("127.0.0.1"),
        )
        .await
        .expect("payment session created");

    assert_eq!(session.amount, dec!(150.00));

    // 3. Authorize Payment Session Workflow
    let payment = workflows
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("payment authorized");

    assert_eq!(payment.amount, dec!(150.00));

    // 4. Capture Payment Workflow
    let capture = workflows
        .capture_payment(CapturePaymentInput {
            payment_id: payment.id.clone(),
            amount: Some(dec!(100.00)),
            created_by: Some("admin_user".to_string()),
            metadata: None,
        })
        .await
        .expect("payment captured");

    assert_eq!(capture.amount, dec!(100.00));

    // 5. Refund Payment Workflow
    let refund = workflows
        .refund_payment(RefundPaymentInput {
            payment_id: payment.id.clone(),
            amount: dec!(40.00),
            refund_reason_id: None,
            note: Some("Partial refund".to_string()),
            created_by: Some("admin_user".to_string()),
            metadata: None,
        })
        .await
        .expect("payment refunded");

    assert_eq!(refund.amount, dec!(40.00));
}
