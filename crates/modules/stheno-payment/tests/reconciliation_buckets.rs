//! P24-06: Reconciliation run writes to payment.payment_reconciliation_run with three buckets:
//! matched, collected-not-remitted, remitted-not-matched.
//! Verified by: cargo test -p stheno-payment --test reconciliation_buckets

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
async fn test_reconciliation_run_produces_three_buckets_and_persists() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // ── Setup 1: An order that gets delivered, captured, and remitted (MATCHED) ──
    let col1 = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(1800.00),
            metadata: None,
        })
        .await
        .expect("col1");
    let ses1 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col1.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(1800.00),
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
            delivery_event_id: "del_match_101".to_string(),
            collected_amount: Some(dec!(1800.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_match_001".to_string()),
            tracking_code: Some("trk_match_001".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm del1");

    // ── Setup 2: An order that is delivered and captured, but courier did NOT remit (COLLECTED_NOT_REMITTED) ──
    let col2 = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(2400.00),
            metadata: None,
        })
        .await
        .expect("col2");
    let ses2 = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col2.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(2400.00),
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
            delivery_event_id: "del_unremitted_202".to_string(),
            collected_amount: Some(dec!(2400.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_unremitted_002".to_string()),
            tracking_code: Some("trk_unremitted_002".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm del2");

    // ── Setup 3: Courier settlement report contains:
    // 1. Line for cons_match_001 (matches pay1)
    // 2. Line for cons_ghost_999 (REMITTED_NOT_MATCHED — no internal order exists)
    let import_res = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_rec_cycle_01".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: vec![
                ImportSettlementLineInput {
                    consignment_id: "cons_match_001".to_string(),
                    tracking_code: Some("trk_match_001".to_string()),
                    order_id: Some("ord_001".to_string()),
                    collected_amount: dec!(1800.00),
                    courier_fee: dec!(70.00),
                    cod_fee: dec!(18.00),
                    remitted_amount: dec!(1712.00),
                },
                ImportSettlementLineInput {
                    consignment_id: "cons_ghost_999".to_string(),
                    tracking_code: Some("trk_ghost_999".to_string()),
                    order_id: Some("ord_unknown_999".to_string()),
                    collected_amount: dec!(3000.00),
                    courier_fee: dec!(100.00),
                    cod_fee: dec!(30.00),
                    remitted_amount: dec!(2870.00),
                },
            ],
            raw_report: None,
        })
        .await
        .expect("import settlement");

    // ── Run Reconciliation ──
    let report = service
        .run_courier_reconciliation(&import_res.cycle_id)
        .await
        .expect("reconciliation run succeeds");

    // Assert the three distinct buckets (P24-06)
    assert_eq!(report.matched_count, 1, "Bucket 1: matched count");
    assert_eq!(report.buckets.matched[0].consignment_id, "cons_match_001");
    assert_eq!(report.buckets.matched[0].collected_amount, dec!(1800.00));

    assert_eq!(
        report.collected_not_remitted_count, 1,
        "Bucket 2: collected_not_remitted count"
    );
    assert_eq!(
        report.buckets.collected_not_remitted[0].consignment_id,
        "cons_unremitted_002"
    );
    assert_eq!(
        report.buckets.collected_not_remitted[0].expected_amount,
        dec!(2400.00)
    );

    assert_eq!(
        report.remitted_not_matched_count, 1,
        "Bucket 3: remitted_not_matched count"
    );
    assert_eq!(
        report.buckets.remitted_not_matched[0].consignment_id,
        "cons_ghost_999"
    );
    assert_eq!(
        report.buckets.remitted_not_matched[0].remitted_amount,
        dec!(2870.00)
    );

    // Assert run state and persistence to payment.payment_reconciliation_run
    assert_eq!(report.state, "drift_found");
    assert_eq!(report.total_lines_checked, 3);

    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "tenant_default".to_string());
    let runs = service
        .list_reconciliation_runs(&tenant_id, 10)
        .await
        .expect("list reconciliation runs");
    let persisted_run = runs
        .into_iter()
        .find(|r| r.id == report.run_id)
        .expect("reconciliation run persisted in database");
    assert_eq!(persisted_run.state, "drift_found");
    assert_eq!(persisted_run.checked_count, 3);
}
