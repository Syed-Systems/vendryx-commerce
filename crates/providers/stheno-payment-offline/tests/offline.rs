use rust_decimal_macros::dec;
use stheno_payment::*;
use stheno_payment_offline::{ManualTransferPaymentProviderFactory, OfflinePaymentProvider};

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
    assert!(init.data["payment_mode"].is_null());

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

#[tokio::test]
async fn manual_transfer_provider_exposes_configured_instructions_not_cod() {
    let mut credentials = CredentialMap::new();
    credentials.insert(
        "payment_account".to_string(),
        serde_json::json!("01700000000"),
    );
    credentials.insert(
        "payment_instructions".to_string(),
        serde_json::json!("Use Send Money and enter your order reference."),
    );
    let factory = ManualTransferPaymentProviderFactory;
    let provider = factory.build(&credentials).expect("valid merchant details");
    let session = provider
        .initiate(InitiateInput {
            amount: dec!(1000),
            currency_code: "bdt".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("manual session");

    assert_eq!(session.data["payment_mode"], "manual_transfer");
    assert_eq!(session.data["payment_account"], "01700000000");
    assert_eq!(session.data["currency_code"], "bdt");
    let authorized = provider
        .authorize(AuthorizeInput {
            session_id: session.id,
            amount: dec!(1000),
            currency_code: "bdt".to_string(),
            context: None,
            data: Some(session.data),
        })
        .await
        .expect("manual authorization");
    assert_eq!(authorized.data["payment_mode"], "manual_transfer");
    assert_eq!(authorized.data["payment_account"], "01700000000");
    let wrong_currency = provider
        .initiate(InitiateInput {
            amount: dec!(25),
            currency_code: "usd".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect_err("manual transfer cannot quote a non-BDT total");
    assert!(wrong_currency
        .to_string()
        .contains("manual_payment_currency_unsupported"));
}
