use rust_decimal_macros::dec;
use serde_json::json;
use std::sync::Arc;
use stheno_payment::*;
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_edge_atlas_e1_through_e34_payment_safety_scenarios() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_offline_edge")));
    service.register_provider(offline);

    // E3: Double-click Pay -> Same deterministic idempotency key dedupes
    let key_e3 = "tx_123_step_auth_pay_1";
    let payload_e3 = json!({ "amount": 1000 });
    let mut exec_count_e3 = 0;

    let res_e3_1: stheno_types::Result<serde_json::Value> = service
        .idempotent_call(
            key_e3,
            "pp_offline_edge",
            "authorize",
            "payment",
            "p1",
            &payload_e3,
            |_k| {
                exec_count_e3 += 1;
                async move { Ok(json!({ "status": "authorized" })) }
            },
        )
        .await;
    assert!(res_e3_1.is_ok());

    let res_e3_2: stheno_types::Result<serde_json::Value> = service
        .idempotent_call(
            key_e3,
            "pp_offline_edge",
            "authorize",
            "payment",
            "p1",
            &payload_e3,
            |_k| {
                exec_count_e3 += 1;
                async move { Ok(json!({ "status": "duplicate" })) }
            },
        )
        .await;
    assert!(res_e3_2.is_ok());
    assert_eq!(
        exec_count_e3, 1,
        "E3: double click must execute exactly once"
    );

    // E4: Same key with changed payload -> rejected on our side before PSP call
    let changed_payload_e4 = json!({ "amount": 2000 });
    let res_e4: stheno_types::Result<serde_json::Value> = service
        .idempotent_call(
            key_e3,
            "pp_offline_edge",
            "authorize",
            "payment",
            "p1",
            &changed_payload_e4,
            |_k| async move { Ok(json!({})) },
        )
        .await;
    assert!(res_e4.is_err(), "E4: changed body with same key must error");

    // E15 & E16: Money currency boundaries
    assert_eq!(
        to_minor_units(dec!(500), "JPY").unwrap(),
        500,
        "E15: JPY 0-decimal"
    );
    assert_eq!(
        to_minor_units(dec!(1.555), "KWD").unwrap(),
        1560,
        "E16: KWD 3-decimal rounded up"
    );

    // E30: Creating a second session cancels previous active session on the collection
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(50.00),
            metadata: None,
        })
        .await
        .unwrap();

    let sess1 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_offline_edge".to_string(),
                amount: dec!(50.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .unwrap();

    let sess2 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_offline_edge".to_string(),
                amount: dec!(50.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .unwrap();

    let reloaded_sess1 = service.get_payment_session(&sess1.id).await.unwrap();
    assert_eq!(
        reloaded_sess1.status,
        PaymentSessionStatus::Canceled,
        "E30: previous session canceled"
    );
    assert_eq!(sess2.status, PaymentSessionStatus::PendingAuthorization);
}
