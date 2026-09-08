//! P24-04: A returned COD order produces no capture and no settlement expectation (BP6).
//! Verified by: cargo test -p stheno-payment --test cod_return_no_capture

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{
    AuthorizePaymentSessionInput, CodDeliveryConfirmationInput, CodReturnInput,
    CreatePaymentCollectionInput, CreatePaymentSessionInput, PaymentService, PaymentSessionStatus,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_cod_return_before_delivery_produces_no_capture() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // 1. Order placed with Cash-on-Delivery
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(2200.00),
            metadata: None,
        })
        .await
        .expect("collection");

    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(2200.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("session");

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("payment authorized");

    // 2. Parcel returned by customer (undeliverable / refused) BEFORE any delivery capture
    let return_res = service
        .process_cod_return(CodReturnInput {
            payment_id: Some(payment.id.clone()),
            payment_session_id: None,
            return_event_id: "ret_evt_101".to_string(),
            reason: Some("Customer rejected parcel".to_string()),
            courier_id: Some("pathao".to_string()),
            metadata: None,
        })
        .await
        .expect("process_cod_return succeeds");

    assert_eq!(return_res.status, "canceled");
    assert!(!return_res.was_captured);
    assert_eq!(return_res.refunded_amount, dec!(0));

    // Verify payment and session are canceled
    let updated_payment = service.get_payment(&payment.id).await.expect("payment");
    assert!(updated_payment.canceled_at.is_some());
    assert_eq!(updated_payment.captured_amount, dec!(0));

    let updated_session = service
        .get_payment_session(&session.id)
        .await
        .expect("session");
    assert_eq!(updated_session.status, PaymentSessionStatus::Canceled);

    // 3. BP6: Ugly ordering — delayed delivery event arrives AFTER return was registered
    // Must be rejected; no capture must be written!
    let delivery_res = service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(payment.id.clone()),
            payment_session_id: None,
            delivery_event_id: "delayed_del_evt_202".to_string(),
            collected_amount: Some(dec!(2200.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_ret_001".to_string()),
            tracking_code: Some("trk_ret_001".to_string()),
            metadata: None,
        })
        .await;

    assert!(
        delivery_res.is_err(),
        "Delivery confirmation on a returned COD order must be rejected"
    );

    // Captured amount must remain 0
    let final_payment = service.get_payment(&payment.id).await.expect("payment");
    assert_eq!(final_payment.captured_amount, dec!(0));
    assert!(final_payment.captured_at.is_none());

    let final_col = service
        .get_payment_collection(&col.id)
        .await
        .expect("collection");
    assert_eq!(final_col.captured_amount, dec!(0));
    assert_eq!(final_col.status, "canceled");
}

#[tokio::test]
async fn test_cod_return_after_delivery_reverses_capture() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // 1. Order placed with Cash-on-Delivery
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(3500.00),
            metadata: None,
        })
        .await
        .expect("collection");

    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(3500.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("session");

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("payment authorized");

    // 2. Delivery confirmation arrived first -> captured
    service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(payment.id.clone()),
            payment_session_id: None,
            delivery_event_id: "del_first_evt_301".to_string(),
            collected_amount: Some(dec!(3500.00)),
            courier_id: Some("steadfast".to_string()),
            consignment_id: Some("cons_sf_ret_002".to_string()),
            tracking_code: Some("trk_sf_ret_002".to_string()),
            metadata: None,
        })
        .await
        .expect("delivery capture succeeds");

    let captured_payment = service.get_payment(&payment.id).await.expect("payment");
    assert_eq!(captured_payment.captured_amount, dec!(3500.00));

    // 3. Parcel physically returned afterwards (delivered -> returned normal sequence, BP6)
    let return_res = service
        .process_cod_return(CodReturnInput {
            payment_id: Some(payment.id.clone()),
            payment_session_id: None,
            return_event_id: "ret_after_del_evt_402".to_string(),
            reason: Some("Doorstep return after inspection".to_string()),
            courier_id: Some("steadfast".to_string()),
            metadata: None,
        })
        .await
        .expect("return processing succeeds");

    assert!(return_res.was_captured);
    assert_eq!(return_res.refunded_amount, dec!(3500.00));

    let final_col = service
        .get_payment_collection(&col.id)
        .await
        .expect("collection");
    assert_eq!(final_col.status, "canceled");
    assert_eq!(
        final_col.refunded_amount,
        dec!(3500.00),
        "Collection refunded amount must equal captured amount, leaving net 0 revenue"
    );

    let final_payment = service.get_payment(&payment.id).await.expect("payment");
    assert!(final_payment.canceled_at.is_some());
}
