use rust_decimal_macros::dec;
use stheno_payment::*;
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_offline_cod_payment_provider_lifecycle() {
    let provider = OfflinePaymentProvider::new(Some("pp_offline_cod"));

    // 1. Check capabilities (async_settlement: true)
    let caps = provider.capabilities();
    assert!(caps.async_settlement);
    assert!(!caps.vault);

    // 2. Initiate session -> PendingAuthorization
    let init = provider
        .initiate(InitiateInput {
            amount: dec!(75.00),
            currency_code: "usd".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("initiate succeeds");
    assert_eq!(init.status, PaymentSessionStatus::PendingAuthorization);

    // 3. Authorize -> Order placed unpaid
    let auth = provider
        .authorize(AuthorizeInput {
            session_id: init.id,
            amount: dec!(75.00),
            currency_code: "usd".to_string(),
            context: None,
            data: Some(init.data),
        })
        .await
        .expect("authorize succeeds");
    assert_eq!(auth.status, PaymentSessionStatus::Authorized);

    // 4. Capture -> Manual mark-as-paid upon COD delivery
    let cap = provider
        .capture(CaptureInput {
            payment_id: "pay_cod_123".to_string(),
            amount: dec!(75.00),
            currency_code: "usd".to_string(),
            data: Some(auth.data),
        })
        .await
        .expect("manual mark-as-paid capture succeeds");
    assert!(cap.data["captured_at"].is_string());
}
