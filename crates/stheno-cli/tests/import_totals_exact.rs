use rust_decimal_macros::dec;
use stheno_cli::money::{reconcile_order_financials, OrderFinancialSnapshot};
use stheno_types::currency::CurrencyCode;

#[test]
fn test_import_order_totals_exact_usd() {
    let snapshot = OrderFinancialSnapshot {
        currency: CurrencyCode::Usd,
        line_item_subtotal_minor: 12500,           // $125.00
        shipping_total_minor: 1000,                // $10.00
        tax_total_minor: 1250,                     // $12.50
        discount_total_minor: 2000,                // $20.00
        expected_order_total_minor: 12750,         // $127.50 ($125 + $10 + $12.50 - $20)
        payment_captures_minor: vec![10000, 2750], // $100.00 + $27.50 = $127.50
        refunds_minor: vec![2500],                 // $25.00
    };

    let result = reconcile_order_financials(&snapshot).expect("Reconciliation must succeed");

    assert!(result.is_exact_match);
    assert_eq!(result.order_total.amount, dec!(127.50));
    assert_eq!(result.total_captured.amount, dec!(127.50));
    assert_eq!(result.total_refunded.amount, dec!(25.00));
    assert_eq!(result.net_settled.amount, dec!(102.50));
}

#[test]
fn test_import_order_totals_exact_zero_decimal_jpy() {
    let snapshot = OrderFinancialSnapshot {
        currency: CurrencyCode::Jpy,
        line_item_subtotal_minor: 15000,   // ¥15,000
        shipping_total_minor: 500,         // ¥500
        tax_total_minor: 1550,             // ¥1,550
        discount_total_minor: 1000,        // ¥1,000
        expected_order_total_minor: 16050, // ¥16,050
        payment_captures_minor: vec![16050],
        refunds_minor: vec![3000],
    };

    let result =
        reconcile_order_financials(&snapshot).expect("Reconciliation must succeed for JPY");

    assert!(result.is_exact_match);
    assert_eq!(result.order_total.amount, dec!(16050));
    assert_eq!(result.total_captured.amount, dec!(16050));
    assert_eq!(result.total_refunded.amount, dec!(3000));
    assert_eq!(result.net_settled.amount, dec!(13050));
}

#[test]
fn test_import_order_totals_mismatch_detected() {
    let snapshot = OrderFinancialSnapshot {
        currency: CurrencyCode::Usd,
        line_item_subtotal_minor: 10000,
        shipping_total_minor: 500,
        tax_total_minor: 500,
        discount_total_minor: 0,
        expected_order_total_minor: 11001, // 1 cent off from 11000
        payment_captures_minor: vec![11000],
        refunds_minor: vec![],
    };

    let result = reconcile_order_financials(&snapshot);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Order totals mismatch") || err.contains("reconciliation_mismatch"));
}
