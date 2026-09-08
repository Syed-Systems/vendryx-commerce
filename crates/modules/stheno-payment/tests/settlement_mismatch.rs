//! P24-08: A deliberately mismatched settlement file produces a discrepancy row, not a silent balance.
//! Verified by: cargo test -p stheno-payment --test settlement_mismatch

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{
    AuthorizePaymentSessionInput, CodDeliveryConfirmationInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, ImportSettlementLineInput, ImportSettlementReportInput,
    PaymentService,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_deliberate_settlement_mismatches_produce_discrepancy_rows_not_silent_balance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // ── Case 1: Order captured for 2000 BDT, but courier report claims collected only 1600 BDT (Short collection) ──
    let col1 = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(2000.00),
            metadata: None,
        })
        .await
        .expect("col1");
    let ses1 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col1.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(2000.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("ses1");
    let pay1 = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: ses1.id.clone(),
            context: None,
        })
        .await
        .expect("pay1");

    service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(pay1.id.clone()),
            payment_session_id: None,
            delivery_event_id: "del_mismatch_evt_01".to_string(),
            collected_amount: Some(dec!(2000.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_mismatch_amt_001".to_string()),
            tracking_code: Some("trk_mismatch_amt_001".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm del1");

    // ── Case 2: Order captured for 1500 BDT, courier collected 1500 BDT, but remitted 1300 BDT with fee of only 80 BDT (120 BDT unexplainable deduction) ──
    let col2 = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(1500.00),
            metadata: None,
        })
        .await
        .expect("col2");
    let ses2 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col2.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(1500.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("ses2");
    let pay2 = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: ses2.id.clone(),
            context: None,
        })
        .await
        .expect("pay2");

    service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(pay2.id.clone()),
            payment_session_id: None,
            delivery_event_id: "del_mismatch_evt_02".to_string(),
            collected_amount: Some(dec!(1500.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_mismatch_fee_002".to_string()),
            tracking_code: Some("trk_mismatch_fee_002".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm del2");

    // ── Import settlement report containing both deliberate mismatches ──
    let import_res = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_mismatch_cycle_2026".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: vec![
                ImportSettlementLineInput {
                    consignment_id: "cons_mismatch_amt_001".to_string(),
                    tracking_code: Some("trk_mismatch_amt_001".to_string()),
                    order_id: Some("ord_mismatch_01".to_string()),
                    collected_amount: dec!(1600.00), // Mismatch: captured was 2000.00
                    courier_fee: dec!(70.00),
                    cod_fee: dec!(16.00),
                    remitted_amount: dec!(1514.00),
                },
                ImportSettlementLineInput {
                    consignment_id: "cons_mismatch_fee_002".to_string(),
                    tracking_code: Some("trk_mismatch_fee_002".to_string()),
                    order_id: Some("ord_mismatch_02".to_string()),
                    collected_amount: dec!(1500.00),
                    courier_fee: dec!(65.00),
                    cod_fee: dec!(15.00),
                    remitted_amount: dec!(1300.00), // Mismatch: 1300 + 80 = 1380 != 1500
                },
            ],
            raw_report: None,
        })
        .await
        .expect("import succeeds");

    // ── Run Reconciliation ──
    let rec = service
        .run_courier_reconciliation(&import_res.cycle_id)
        .await
        .expect("reconciliation run succeeds");

    // Assert that the mismatches produced discrepancy rows, NEVER a silent balance
    assert_eq!(rec.state, "drift_found");
    assert_eq!(rec.matched_count, 0, "No lines should silently match");
    assert_eq!(rec.discrepancy_count, 2, "Both lines must be flagged as discrepancies");

    let disc_amt = rec
        .buckets
        .discrepant
        .iter()
        .find(|d| d.consignment_id == "cons_mismatch_amt_001")
        .expect("amount mismatch discrepancy found");
    assert_eq!(disc_amt.expected_amount, dec!(2000.00));
    assert_eq!(disc_amt.collected_amount, dec!(1600.00));
    assert!(disc_amt
        .reason
        .as_ref()
        .unwrap()
        .contains("Collected amount mismatch"));

    let disc_fee = rec
        .buckets
        .discrepant
        .iter()
        .find(|d| d.consignment_id == "cons_mismatch_fee_002")
        .expect("fee mismatch discrepancy found");
    assert!(disc_fee
        .reason
        .as_ref()
        .unwrap()
        .contains("Fee accounting mismatch"));

    // Verify lines in database are flagged as discrepant
    let lines = service
        .list_settlement_lines(&import_res.cycle_id)
        .await
        .expect("list lines");
    for line in lines {
        assert_eq!(line.reconciliation_status, "discrepant");
        assert!(line.discrepancy_reason.is_some());
    }

    // Cycle status must be discrepant, not reconciled
    let cycle = service
        .get_settlement_cycle(&import_res.cycle_id)
        .await
        .expect("get cycle");
    assert_eq!(cycle.status, "discrepant");
}
