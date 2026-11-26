//! P24-05: Settlement report import per courier, idempotent on re-import (BP7).
//! Verified by: cargo test -p stheno-payment --test settlement_import

use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{ImportSettlementLineInput, ImportSettlementReportInput, PaymentService};

#[tokio::test]
async fn test_settlement_import_idempotent_and_partial_overlap() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);

    let initial_lines = vec![
        ImportSettlementLineInput {
            consignment_id: "cons_pth_001".to_string(),
            tracking_code: Some("trk_pth_001".to_string()),
            order_id: Some("ord_001".to_string()),
            collected_amount: dec!(1200.00),
            courier_fee: dec!(60.00),
            cod_fee: dec!(12.00),
            remitted_amount: dec!(1128.00),
        },
        ImportSettlementLineInput {
            consignment_id: "cons_pth_002".to_string(),
            tracking_code: Some("trk_pth_002".to_string()),
            order_id: Some("ord_002".to_string()),
            collected_amount: dec!(2500.00),
            courier_fee: dec!(80.00),
            cod_fee: dec!(25.00),
            remitted_amount: dec!(2395.00),
        },
        ImportSettlementLineInput {
            consignment_id: "cons_pth_003".to_string(),
            tracking_code: Some("trk_pth_003".to_string()),
            order_id: Some("ord_003".to_string()),
            collected_amount: dec!(800.00),
            courier_fee: dec!(50.00),
            cod_fee: dec!(8.00),
            remitted_amount: dec!(742.00),
        },
    ];

    // 1. First import of Pathao settlement cycle
    let res1 = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_inv_2026_w36".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: initial_lines.clone(),
            raw_report: None,
        })
        .await
        .expect("initial import succeeds");

    assert_eq!(res1.imported_lines, 3);
    assert_eq!(res1.skipped_duplicate_lines, 0);
    assert!(!res1.is_reimport);
    assert_eq!(res1.total_collected, dec!(4500.00));
    assert_eq!(res1.total_fees, dec!(235.00));
    assert_eq!(res1.total_remitted, dec!(4265.00));

    let cycle = service
        .get_settlement_cycle(&res1.cycle_id)
        .await
        .expect("get cycle");
    assert_eq!(cycle.total_collected, dec!(4500.00));
    assert_eq!(cycle.total_fees, dec!(235.00));
    assert_eq!(cycle.total_remitted, dec!(4265.00));

    // 2. BP7: Re-uploading identical file must be an idempotent NO-OP
    // Must NOT double-count collected or remitted amounts
    let res2 = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_inv_2026_w36".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: initial_lines.clone(),
            raw_report: None,
        })
        .await
        .expect("re-import succeeds");

    assert_eq!(res2.imported_lines, 0);
    assert_eq!(res2.skipped_duplicate_lines, 3);
    assert!(res2.is_reimport);
    assert_eq!(res2.total_collected, dec!(4500.00));
    assert_eq!(res2.total_fees, dec!(235.00));
    assert_eq!(res2.total_remitted, dec!(4265.00));

    let cycle_after_reimport = service
        .get_settlement_cycle(&res1.cycle_id)
        .await
        .expect("get cycle");
    assert_eq!(cycle_after_reimport.total_collected, dec!(4500.00));
    assert_eq!(cycle_after_reimport.total_fees, dec!(235.00));
    assert_eq!(cycle_after_reimport.total_remitted, dec!(4265.00));

    // 3. BP7: Partially overlapping file (2 already seen lines + 2 new lines)
    let overlapping_lines = vec![
        initial_lines[0].clone(), // already imported (cons_pth_001)
        initial_lines[1].clone(), // already imported (cons_pth_002)
        ImportSettlementLineInput {
            consignment_id: "cons_pth_004".to_string(),
            tracking_code: Some("trk_pth_004".to_string()),
            order_id: Some("ord_004".to_string()),
            collected_amount: dec!(1000.00),
            courier_fee: dec!(60.00),
            cod_fee: dec!(10.00),
            remitted_amount: dec!(930.00),
        },
        ImportSettlementLineInput {
            consignment_id: "cons_pth_005".to_string(),
            tracking_code: Some("trk_pth_005".to_string()),
            order_id: Some("ord_005".to_string()),
            collected_amount: dec!(1500.00),
            courier_fee: dec!(70.00),
            cod_fee: dec!(15.00),
            remitted_amount: dec!(1415.00),
        },
    ];

    let res3 = service
        .import_settlement_report(ImportSettlementReportInput {
            provider_id: "pathao".to_string(),
            courier_cycle_id: "pathao_inv_2026_w36".to_string(),
            cycle_start: None,
            cycle_end: None,
            currency: Some("BDT".to_string()),
            lines: overlapping_lines,
            raw_report: None,
        })
        .await
        .expect("overlapping import succeeds");

    assert_eq!(res3.imported_lines, 2);
    assert_eq!(res3.skipped_duplicate_lines, 2);
    assert!(res3.is_reimport);
    // Total lines in cycle is now 5: 4500 + 1000 + 1500 = 7000
    assert_eq!(res3.total_collected, dec!(7000.00));
    // Fees: 235 + 70 + 85 = 390
    assert_eq!(res3.total_fees, dec!(390.00));
    // Remitted: 4265 + 930 + 1415 = 6610
    assert_eq!(res3.total_remitted, dec!(6610.00));

    let cycle_after_overlap = service
        .get_settlement_cycle(&res1.cycle_id)
        .await
        .expect("get cycle");
    assert_eq!(cycle_after_overlap.total_collected, dec!(7000.00));
    assert_eq!(cycle_after_overlap.total_fees, dec!(390.00));
    assert_eq!(cycle_after_overlap.total_remitted, dec!(6610.00));

    let lines_in_db = service
        .list_settlement_lines(&res1.cycle_id)
        .await
        .expect("list lines");
    assert_eq!(lines_in_db.len(), 5);
}
