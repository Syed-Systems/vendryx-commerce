//! P24-12: End-to-end COD: order → delivered → captured → settled from courier input alone, decimal-exact.
//! Verified by: ./scripts/cod-lifecycle.sh and cargo test -p stheno-payment --test cod_lifecycle

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{
    AuthorizePaymentSessionInput, CodDeliveryConfirmationInput, CodReturnInput,
    CreatePaymentCollectionInput, CreatePaymentSessionInput, ImportSettlementLineInput,
    ImportSettlementReportInput, PaymentService, PaymentSessionStatus,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_end_to_end_cod_lifecycle_delivered_captured_settled() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 1: Order with COD payment collection placed
    // ─────────────────────────────────────────────────────────────────────────────
    let order_amount = dec!(2750.00);
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: order_amount,
            metadata: None,
        })
        .await
        .expect("payment collection created");

    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: order_amount,
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("payment session created");

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("session authorized");

    assert_eq!(payment.amount, order_amount);
    assert_eq!(payment.captured_amount, dec!(0.00));
    assert!(payment.captured_at.is_none());

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 2: Courier delivers consignment -> delivery confirmation authorises capture
    // Courier input alone drives the capture on the COD provider (BP6)
    // ─────────────────────────────────────────────────────────────────────────────
    let courier_id = "pathao";
    let consignment_id = "cons_e2e_pathao_101";
    let tracking_code = "trk_e2e_101";
    let delivery_event_id = "evt_pathao_delivered_101";

    let capture = service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(payment.id.clone()),
            payment_session_id: None,
            delivery_event_id: delivery_event_id.to_string(),
            collected_amount: Some(order_amount),
            courier_id: Some(courier_id.to_string()),
            consignment_id: Some(consignment_id.to_string()),
            tracking_code: Some(tracking_code.to_string()),
            metadata: None,
        })
        .await
        .expect("delivery confirmation capture succeeds");

    assert_eq!(capture.amount, order_amount);
    assert_eq!(capture.payment_id, payment.id);

    // Verify payment and collection states have transitioned to captured
    let updated_payment = service.get_payment(&payment.id).await.expect("payment");
    assert_eq!(updated_payment.captured_amount, order_amount);
    assert!(updated_payment.captured_at.is_some());

    let updated_col = service
        .get_payment_collection(&col.id)
        .await
        .expect("collection");
    assert_eq!(updated_col.captured_amount, order_amount);
    assert_eq!(updated_col.status, "captured");

    let updated_session = service
        .get_payment_session(&session.id)
        .await
        .expect("session");
    assert_eq!(updated_session.status, PaymentSessionStatus::Captured);

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 3: Courier settlement report imported (BP7, BP8)
    // Courier payout breakdown:
    // Collected: 2750.00 BDT
    // Courier delivery fee: 100.00 BDT
    // COD fee (1%): 27.50 BDT
    // Remitted: 2622.50 BDT
    // Total fees: 127.50 BDT
    // Identity: 2622.50 + 127.50 == 2750.00 (Decimal exact)
    // ─────────────────────────────────────────────────────────────────────────────
    let courier_fee = dec!(100.00);
    let cod_fee = dec!(27.50);
    let total_fees = dec!(127.50);
    let remitted_amount = dec!(2622.50);
    assert_eq!(remitted_amount + total_fees, order_amount);

    let import_out = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: courier_id.to_string(),
            courier_cycle_id: "pathao_payout_2026_w36".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: vec![ImportSettlementLineInput {
                consignment_id: consignment_id.to_string(),
                tracking_code: Some(tracking_code.to_string()),
                order_id: Some("ord_e2e_101".to_string()),
                collected_amount: order_amount,
                courier_fee,
                cod_fee,
                remitted_amount,
            }],
            raw_report: None,
        })
        .await
        .expect("import settlement report");

    assert_eq!(import_out.imported_lines, 1);
    assert_eq!(import_out.total_collected, order_amount);
    assert_eq!(import_out.total_fees, total_fees);
    assert_eq!(import_out.total_remitted, remitted_amount);

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 4: Reconciliation run executed -> transitions to settled
    // Decimal exact balance: sum(remitted) + sum(fees) == sum(collected)
    // ─────────────────────────────────────────────────────────────────────────────
    let rec = service
        .run_courier_reconciliation(&import_out.cycle_id)
        .await
        .expect("reconciliation run");

    assert_eq!(rec.state, "clean");
    assert_eq!(rec.matched_count, 1);
    assert_eq!(rec.discrepancy_count, 0);
    assert_eq!(rec.collected_not_remitted_count, 0);
    assert_eq!(rec.remitted_not_matched_count, 0);
    assert_eq!(rec.buckets.matched[0].consignment_id, consignment_id);
    assert_eq!(rec.buckets.matched[0].collected_amount, order_amount);
    assert_eq!(rec.buckets.matched[0].remitted_amount, remitted_amount);
    assert_eq!(rec.buckets.matched[0].fee_amount, total_fees);
    assert_eq!(rec.total_remitted + rec.total_fees, rec.total_collected);

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 5: Deliberately mismatched settlement file produces a discrepancy row,
    // NOT a silent balance (P24-08)
    // ─────────────────────────────────────────────────────────────────────────────
    let mismatch_consignment = "cons_e2e_mismatch_102";
    let mismatch_col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(1000.00),
            metadata: None,
        })
        .await
        .expect("mismatch collection");
    let mismatch_ses = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: mismatch_col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(1000.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("mismatch session");
    let mismatch_pay = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: mismatch_ses.id,
            context: None,
        })
        .await
        .expect("mismatch pay");

    service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(mismatch_pay.id),
            payment_session_id: None,
            delivery_event_id: "del_evt_mismatch_102".to_string(),
            collected_amount: Some(dec!(1000.00)),
            courier_id: Some(courier_id.to_string()),
            consignment_id: Some(mismatch_consignment.to_string()),
            tracking_code: Some("trk_mismatch_102".to_string()),
            metadata: None,
        })
        .await
        .expect("mismatch delivery");

    // Courier remits less than expected without balanced fee
    let mismatch_import = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: courier_id.to_string(),
            courier_cycle_id: "pathao_payout_2026_w36_mismatch".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: vec![ImportSettlementLineInput {
                consignment_id: mismatch_consignment.to_string(),
                tracking_code: Some("trk_mismatch_102".to_string()),
                order_id: Some("ord_mismatch_102".to_string()),
                collected_amount: dec!(800.00), // Expected 1000.00
                courier_fee: dec!(50.00),
                cod_fee: dec!(8.00),
                remitted_amount: dec!(742.00),
            }],
            raw_report: None,
        })
        .await
        .expect("import mismatch report");

    let mismatch_rec = service
        .run_courier_reconciliation(&mismatch_import.cycle_id)
        .await
        .expect("run mismatch reconciliation");

    assert_eq!(mismatch_rec.state, "drift_found");
    assert_eq!(mismatch_rec.discrepancy_count, 1);
    assert_eq!(
        mismatch_rec.buckets.discrepant[0].consignment_id,
        mismatch_consignment
    );
    assert!(mismatch_rec.buckets.discrepant[0]
        .reason
        .as_ref()
        .unwrap()
        .contains("Collected amount mismatch"));

    // ─────────────────────────────────────────────────────────────────────────────
    // STAGE 6: Returned COD order produces no capture and no settlement expectation (P24-04)
    // ─────────────────────────────────────────────────────────────────────────────
    let return_col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(1500.00),
            metadata: None,
        })
        .await
        .expect("return collection");
    let return_ses = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: return_col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(1500.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("return session");
    let return_pay = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: return_ses.id,
            context: None,
        })
        .await
        .expect("return pay");

    let return_dto = service
        .process_cod_return(CodReturnInput {
            payment_id: Some(return_pay.id.clone()),
            payment_session_id: None,
            return_event_id: "ret_evt_e2e_103".to_string(),
            courier_id: Some("pathao".to_string()),
            reason: Some("Customer canceled at doorstep".to_string()),
            metadata: None,
        })
        .await
        .expect("process return");

    assert_eq!(return_dto.status, "canceled");
    assert!(!return_dto.was_captured);
    assert_eq!(return_dto.refunded_amount, dec!(0.00));

    let final_return_pay = service
        .get_payment(&return_pay.id)
        .await
        .expect("get payment");
    assert_eq!(final_return_pay.captured_amount, dec!(0.00));
    assert!(final_return_pay.captured_at.is_none());
}
