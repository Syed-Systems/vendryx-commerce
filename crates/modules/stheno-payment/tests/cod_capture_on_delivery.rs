//! P24-03: Delivery confirmation authorises capture on the COD provider — one flow, courier-agnostic (BP6).
//! Verified by: cargo test -p stheno-payment --test cod_capture_on_delivery

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{
    AuthorizePaymentSessionInput, CodDeliveryConfirmationInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService, PaymentSessionStatus,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_cod_capture_on_delivery_courier_agnostic_and_idempotent() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // Test matrix across all major courier providers to prove courier-agnosticism:
    // Pathao, Steadfast, RedX, and Manual
    let couriers = vec![
        (
            "pathao",
            "cons_pathao_001",
            "trk_pathao_991",
            "evt_pathao_del_01",
        ),
        (
            "steadfast",
            "cons_steadfast_002",
            "trk_sf_882",
            "evt_sf_del_02",
        ),
        ("redx", "cons_redx_003", "trk_redx_773", "evt_redx_del_03"),
        ("manual", "cons_manual_004", "trk_man_664", "evt_man_del_04"),
    ];

    for (courier_id, consignment_id, tracking_code, delivery_event_id) in couriers {
        // 1. Order placed with Cash-on-Delivery
        let col = service
            .create_payment_collection(CreatePaymentCollectionInput {
                currency_code: "BDT".to_string(),
                amount: dec!(1500.00),
                metadata: None,
            })
            .await
            .expect("collection created");

        let session = service
            .create_payment_session(
                CreatePaymentSessionInput {
                    payment_collection_id: col.id.clone(),
                    provider_id: "pp_system_default".to_string(),
                    amount: dec!(1500.00),
                    currency_code: "BDT".to_string(),
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
            .expect("payment authorized");

        assert_eq!(payment.captured_amount, dec!(0));
        assert!(payment.captured_at.is_none());

        // 2. Courier delivers parcel -> delivery confirmation authorises capture
        let capture = service
            .confirm_cod_delivery(CodDeliveryConfirmationInput {
                payment_id: Some(payment.id.clone()),
                payment_session_id: None,
                delivery_event_id: delivery_event_id.to_string(),
                collected_amount: Some(dec!(1500.00)),
                courier_id: Some(courier_id.to_string()),
                consignment_id: Some(consignment_id.to_string()),
                tracking_code: Some(tracking_code.to_string()),
                metadata: None,
            })
            .await
            .expect("delivery confirmation capture succeeds");

        assert_eq!(capture.amount, dec!(1500.00));
        assert_eq!(capture.payment_id, payment.id);

        // 3. Verify payment, session, and collection state transitioned to captured
        let updated_payment = service.get_payment(&payment.id).await.expect("payment");
        assert_eq!(updated_payment.captured_amount, dec!(1500.00));
        assert!(updated_payment.captured_at.is_some());

        let updated_col = service
            .get_payment_collection(&col.id)
            .await
            .expect("collection");
        assert_eq!(updated_col.captured_amount, dec!(1500.00));
        assert_eq!(updated_col.status, "captured");

        let updated_session = service
            .get_payment_session(&session.id)
            .await
            .expect("session");
        assert_eq!(updated_session.status, PaymentSessionStatus::Captured);

        // 4. BP6: Courier re-sends delivery event (idempotent replay)
        // Must NOT double capture or double-charge merchant's books
        let duplicate_capture = service
            .confirm_cod_delivery(CodDeliveryConfirmationInput {
                payment_id: Some(payment.id.clone()),
                payment_session_id: None,
                delivery_event_id: delivery_event_id.to_string(),
                collected_amount: Some(dec!(1500.00)),
                courier_id: Some(courier_id.to_string()),
                consignment_id: Some(consignment_id.to_string()),
                tracking_code: Some(tracking_code.to_string()),
                metadata: None,
            })
            .await
            .expect("duplicate delivery replay is idempotent");

        assert_eq!(duplicate_capture.id, capture.id);

        let final_payment = service.get_payment(&payment.id).await.expect("payment");
        assert_eq!(
            final_payment.captured_amount,
            dec!(1500.00),
            "Captured amount must remain 1500, never double-counted"
        );
        let final_col = service
            .get_payment_collection(&col.id)
            .await
            .expect("collection");
        assert_eq!(
            final_col.captured_amount,
            dec!(1500.00),
            "Collection captured amount must remain 1500"
        );
    }
}
