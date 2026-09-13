use rust_decimal::Decimal;
use std::str::FromStr;
use stheno_platform::calculator::{CalculatorError, SubscriptionInvoiceCalculator};
use stheno_types::currency::CurrencyCode;

#[test]
fn calculation_rejects_invalid_months() {
    let base_price = Decimal::from_str("20.00").unwrap();
    let percent = Some(Decimal::from_str("10.00").unwrap());
    let starts_at = Some(12);

    let res = SubscriptionInvoiceCalculator::calculate(base_price, percent, starts_at, 0, "US", None);
    assert_eq!(res.unwrap_err(), CalculatorError::InvalidMonths);

    let res2 = SubscriptionInvoiceCalculator::calculate(base_price, percent, starts_at, -5, "US", None);
    assert_eq!(res2.unwrap_err(), CalculatorError::InvalidMonths);
}

#[test]
fn calculation_works_for_usd_merchant() {
    let base_price = Decimal::from_str("20.00").unwrap();
    let percent = Some(Decimal::from_str("10.00").unwrap());
    let starts_at = Some(12);
    
    // 1 month
    let res1 = SubscriptionInvoiceCalculator::calculate(base_price, percent, starts_at, 1, "US", None).unwrap();
    assert_eq!(res1.subtotal_usd, Decimal::from_str("20.00").unwrap());
    assert_eq!(res1.discount_usd, Decimal::from_str("0.00").unwrap());
    assert_eq!(res1.total_usd, Decimal::from_str("20.00").unwrap());
    assert_eq!(res1.presentment_currency, CurrencyCode::Usd);
    assert_eq!(res1.payable_amount, Decimal::from_str("20.00").unwrap());

    // 12 months (10% discount)
    let res12 = SubscriptionInvoiceCalculator::calculate(base_price, percent, starts_at, 12, "US", None).unwrap();
    assert_eq!(res12.subtotal_usd, Decimal::from_str("240.00").unwrap());
    assert_eq!(res12.discount_usd, Decimal::from_str("24.00").unwrap());
    assert_eq!(res12.total_usd, Decimal::from_str("216.00").unwrap());
    assert_eq!(res12.presentment_currency, CurrencyCode::Usd);
    assert_eq!(res12.payable_amount, Decimal::from_str("216.00").unwrap());
}

#[test]
fn calculation_works_for_bdt_merchant() {
    let base_price = Decimal::from_str("20.00").unwrap();
    let percent = Some(Decimal::from_str("10.00").unwrap());
    let starts_at = Some(12);

    // 6 months (no discount)
    let res6 = SubscriptionInvoiceCalculator::calculate(base_price, percent, starts_at, 6, "BD", None).unwrap();
    assert_eq!(res6.subtotal_usd, Decimal::from_str("120.00").unwrap());
    assert_eq!(res6.discount_usd, Decimal::from_str("0.00").unwrap());
    assert_eq!(res6.total_usd, Decimal::from_str("120.00").unwrap());
    assert_eq!(res6.presentment_currency, CurrencyCode::Bdt);
    
    // 120.00 USD * 120 (stub rate) = 14400.00 BDT
    assert_eq!(res6.payable_amount, Decimal::from_str("14400").unwrap());
}
