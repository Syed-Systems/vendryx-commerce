use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_payment::{CreatePaymentCollectionInput, CreatePaymentSessionInput, PaymentService};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_card_testing_abuse_limits() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(50.00),
            metadata: None,
        })
        .await
        .unwrap();

    let ip = "192.168.1.100";

    // 10 session creations succeed
    for _ in 0..10 {
        let res = service
            .create_payment_session(
                CreatePaymentSessionInput {
                    payment_collection_id: col.id.clone(),
                    provider_id: "pp_system_default".to_string(),
                    amount: dec!(50.00),
                    currency_code: "usd".to_string(),
                    data: None,
                    context: None,
                },
                Some(ip),
            )
            .await;
        assert!(res.is_ok());
    }

    // 11th session from the same IP is blocked (P4-27, F10)
    let res_blocked = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(50.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            Some(ip),
        )
        .await;

    assert!(res_blocked.is_err());
    match res_blocked.unwrap_err() {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "card_testing_limit_exceeded");
        }
        _ => panic!("expected Invalid card_testing_limit_exceeded"),
    }
}
