//! P24-09: Discrepancy alerting: an unmatched line older than the cycle length raises an alert.
//! Verified by: cargo test -p stheno-payment --test settlement_alerting

use chrono::{Duration, Utc};
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
async fn test_discrepancy_alerting_for_aged_unsettled_lines() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool.clone());
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // ── Setup an order for Line 3 that matches ──
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(1500.00),
            metadata: None,
        })
        .await
        .expect("col");
    let ses = service
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
        .expect("ses");
    let pay = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: ses.id.clone(),
            context: None,
        })
        .await
        .expect("pay");
    service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(pay.id),
            payment_session_id: None,
            delivery_event_id: "del_matched_303".to_string(),
            collected_amount: Some(dec!(1500.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_aged_matched_03".to_string()),
            tracking_code: Some("trk_aged_matched_03".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm delivery");

    // ── Import settlement report with 3 lines ──
    let import_out = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_cycle_alerting_test".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: vec![
                // Line 1: Aged Unmatched (will be backdated to 12 days ago)
                ImportSettlementLineInput {
                    consignment_id: "cons_aged_unmatched_01".to_string(),
                    tracking_code: Some("trk_aged_01".to_string()),
                    order_id: Some("ord_aged_01".to_string()),
                    collected_amount: dec!(2000.00),
                    courier_fee: dec!(80.00),
                    cod_fee: dec!(20.00),
                    remitted_amount: dec!(1900.00),
                },
                // Line 2: Fresh Unmatched (created today)
                ImportSettlementLineInput {
                    consignment_id: "cons_fresh_unmatched_02".to_string(),
                    tracking_code: Some("trk_fresh_02".to_string()),
                    order_id: Some("ord_fresh_02".to_string()),
                    collected_amount: dec!(1000.00),
                    courier_fee: dec!(50.00),
                    cod_fee: dec!(10.00),
                    remitted_amount: dec!(940.00),
                },
                // Line 3: Aged Matched (will be backdated, but matches order)
                ImportSettlementLineInput {
                    consignment_id: "cons_aged_matched_03".to_string(),
                    tracking_code: Some("trk_aged_matched_03".to_string()),
                    order_id: Some("ord_matched_03".to_string()),
                    collected_amount: dec!(1500.00),
                    courier_fee: dec!(60.00),
                    cod_fee: dec!(15.00),
                    remitted_amount: dec!(1425.00),
                },
            ],
            raw_report: None,
        })
        .await
        .expect("import");

    // Run reconciliation so status is computed
    service
        .run_courier_reconciliation(&import_out.cycle_id)
        .await
        .expect("reconciliation");

    // Backdate Line 1 and Line 3 to 12 days ago
    let twelve_days_ago = Utc::now() - Duration::days(12);

    let lines = service
        .list_settlement_lines(&import_out.cycle_id)
        .await
        .expect("list lines");

    let pool_clone = pool.clone();
    stheno_db::tenant::with_tenant_conn(&pool_clone, move |conn| {
        Box::pin(async move {
            use stheno_db::diesel_async::RunQueryDsl;
            for line in lines {
                if line.consignment_id == "cons_aged_unmatched_01"
                    || line.consignment_id == "cons_aged_matched_03"
                {
                    stheno_db::diesel::sql_query(
                        "UPDATE payment.courier_settlement_line SET created_at = $1 WHERE id = $2",
                    )
                    .bind::<stheno_db::diesel::sql_types::Timestamptz, _>(twelve_days_ago)
                    .bind::<stheno_db::diesel::sql_types::Text, _>(&line.id)
                    .execute(conn)
                    .await
                    .expect("backdate line created_at");
                }
            }
            Ok::<(), stheno_types::Error>(())
        })
    })
    .await
    .expect("backdate lines");

    // ── Check discrepancy alerts with cycle length = 7 days ──
    let alerts = service
        .check_unsettled_line_alerts(&import_out.cycle_id, 7)
        .await
        .expect("check alerts succeeds");

    // Exactly 1 alert must be raised:
    // - Line 1 (unmatched, 12 days old >= 7) -> RAISES ALERT
    // - Line 2 (unmatched, 0 days old < 7)   -> NO ALERT
    // - Line 3 (matched, 12 days old)        -> NO ALERT (reconciled)
    assert_eq!(
        alerts.len(),
        1,
        "Only aged unmatched lines exceeding cycle length must raise alerts"
    );

    let alert = &alerts[0];
    assert_eq!(alert.consignment_id, "cons_aged_unmatched_01");
    assert_eq!(alert.tracking_code.as_deref(), Some("trk_aged_01"));
    assert_eq!(alert.order_id.as_deref(), Some("ord_aged_01"));
    assert!(alert.age_days >= 7, "Age must exceed cycle length");
    assert_eq!(alert.alert_level, "critical");
    assert!(alert.message.contains("Unsettled consignment has remained"));
}
