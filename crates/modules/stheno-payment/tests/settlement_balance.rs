//! P24-07: Courier fee accounting: sum(remitted) + sum(fees) == sum(collected) over a full cycle,
//! asserted in decimal against a fixture with hand-verified totals (BP8).
//! Verified by: cargo test -p stheno-payment --test settlement_balance

use rust_decimal::Decimal;
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

struct HandVerifiedFixtureItem {
    order_id: &'static str,
    consignment_id: &'static str,
    tracking_code: &'static str,
    collected: Decimal,
    courier_fee: Decimal,
    cod_fee: Decimal,
    remitted: Decimal,
}

#[tokio::test]
async fn test_settlement_balance_decimal_exact_courier_fee_accounting() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // ── Hand-verified fixture: 6 orders across Dhaka, Chittagong, Sylhet ──
    // Formula: remitted + (courier_fee + cod_fee) == collected
    let fixture = vec![
        HandVerifiedFixtureItem {
            order_id: "ord_bal_01",
            consignment_id: "cons_bal_01",
            tracking_code: "trk_bal_01",
            collected: dec!(1450.00),
            courier_fee: dec!(60.00),
            cod_fee: dec!(14.50),
            remitted: dec!(1375.50), // 1375.50 + 60 + 14.50 = 1450.00
        },
        HandVerifiedFixtureItem {
            order_id: "ord_bal_02",
            consignment_id: "cons_bal_02",
            tracking_code: "trk_bal_02",
            collected: dec!(3200.00),
            courier_fee: dec!(100.00),
            cod_fee: dec!(32.00),
            remitted: dec!(3068.00), // 3068.00 + 100 + 32.00 = 3200.00
        },
        HandVerifiedFixtureItem {
            order_id: "ord_bal_03",
            consignment_id: "cons_bal_03",
            tracking_code: "trk_bal_03",
            collected: dec!(850.25),
            courier_fee: dec!(50.00),
            cod_fee: dec!(8.50),
            remitted: dec!(791.75), // 791.75 + 50 + 8.50 = 850.25
        },
        HandVerifiedFixtureItem {
            order_id: "ord_bal_04",
            consignment_id: "cons_bal_04",
            tracking_code: "trk_bal_04",
            collected: dec!(5400.00),
            courier_fee: dec!(130.00),
            cod_fee: dec!(54.00),
            remitted: dec!(5216.00), // 5216.00 + 130 + 54.00 = 5400.00
        },
        HandVerifiedFixtureItem {
            order_id: "ord_bal_05",
            consignment_id: "cons_bal_05",
            tracking_code: "trk_bal_05",
            collected: dec!(2100.75),
            courier_fee: dec!(80.00),
            cod_fee: dec!(21.01),
            remitted: dec!(1999.74), // 1999.74 + 80 + 21.01 = 2100.75
        },
        HandVerifiedFixtureItem {
            order_id: "ord_bal_06",
            consignment_id: "cons_bal_06",
            tracking_code: "trk_bal_06",
            collected: dec!(999.99),
            courier_fee: dec!(60.00),
            cod_fee: dec!(10.00),
            remitted: dec!(929.99), // 929.99 + 60 + 10.00 = 999.99
        },
    ];

    // Hand-calculated expected totals:
    // Total collected = 1450.00 + 3200.00 + 850.25 + 5400.00 + 2100.75 + 999.99 = 14000.99
    let expected_total_collected = dec!(14000.99);
    // Total courier fees = 60 + 100 + 50 + 130 + 80 + 60 = 480.00
    // Total cod fees     = 14.50 + 32.00 + 8.50 + 54.00 + 21.01 + 10.00 = 140.01
    // Total fees         = 620.01
    let expected_total_fees = dec!(620.01);
    // Total remitted     = 1375.50 + 3068.00 + 791.75 + 5216.00 + 1999.74 + 929.99 = 13380.98
    let expected_total_remitted = dec!(13380.98);

    // Assert the arithmetic identity on the hand-calculated totals first:
    assert_eq!(
        expected_total_remitted + expected_total_fees,
        expected_total_collected,
        "Fixture accounting identity violated"
    );

    // 1. Create orders and confirm deliveries with collections
    let mut import_lines = Vec::new();
    for item in &fixture {
        // Individual line balance check
        assert_eq!(
            item.remitted + item.courier_fee + item.cod_fee,
            item.collected,
            "Line {} balance check",
            item.consignment_id
        );

        let col = service
            .create_payment_collection(CreatePaymentCollectionInput {
                currency_code: "BDT".to_string(),
                amount: item.collected,
                metadata: None,
            })
            .await
            .expect("col");

        let ses = service
            .create_payment_session(
                CreatePaymentSessionInput {
                    payment_collection_id: col.id.clone(),
                    provider_id: "pp_system_default".to_string(),
                    amount: item.collected,
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
                delivery_event_id: format!("del_evt_{}", item.consignment_id),
                collected_amount: Some(item.collected),
                courier_id: Some("steadfast".to_string()),
                consignment_id: Some(item.consignment_id.to_string()),
                tracking_code: Some(item.tracking_code.to_string()),
                metadata: None,
            })
            .await
            .expect("confirm delivery");

        import_lines.push(ImportSettlementLineInput {
            consignment_id: item.consignment_id.to_string(),
            tracking_code: Some(item.tracking_code.to_string()),
            order_id: Some(item.order_id.to_string()),
            collected_amount: item.collected,
            courier_fee: item.courier_fee,
            cod_fee: item.cod_fee,
            remitted_amount: item.remitted,
        });
    }

    // 2. Import the settlement report for Steadfast cycle
    let import_out = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "steadfast".to_string(),
            courier_cycle_id: "sf_payout_2026_w36_full".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: import_lines,
            raw_report: None,
        })
        .await
        .expect("import settlement");

    assert_eq!(import_out.imported_lines, 6);
    assert_eq!(import_out.total_collected, expected_total_collected);
    assert_eq!(import_out.total_fees, expected_total_fees);
    assert_eq!(import_out.total_remitted, expected_total_remitted);

    // 3. Core BP8 Assertion: sum(remitted) + sum(fees) == sum(collected)
    assert_eq!(
        import_out.total_remitted + import_out.total_fees,
        import_out.total_collected,
        "Core BP8 Decimal Accounting Invariant: sum(remitted) + sum(fees) must equal sum(collected)"
    );

    // 4. Run reconciliation
    let rec = service
        .run_courier_reconciliation(&import_out.cycle_id)
        .await
        .expect("run reconciliation");

    assert_eq!(rec.state, "clean");
    assert_eq!(rec.matched_count, 6);
    assert_eq!(rec.discrepancy_count, 0);
    assert_eq!(rec.collected_not_remitted_count, 0);
    assert_eq!(rec.remitted_not_matched_count, 0);
    assert_eq!(rec.total_remitted + rec.total_fees, rec.total_collected);
}
