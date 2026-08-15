use rust_decimal_macros::dec;
use stheno_types::{CurrencyCode, Money};

#[test]
fn test_money_creation_and_ops() {
    let m1 = Money::new(dec!(25.00), CurrencyCode::Usd);
    let m2 = Money::new(dec!(10.00), CurrencyCode::Usd);

    let sum = (m1 + m2).expect("same currency addition must succeed");
    assert_eq!(sum.amount, dec!(35.00));
    assert_eq!(sum.currency, CurrencyCode::Usd);

    let diff = (m1 - m2).expect("same currency subtraction must succeed");
    assert_eq!(diff.amount, dec!(15.00));

    let scaled = m1 * dec!(2);
    assert_eq!(scaled.amount, dec!(50.00));
}

#[test]
fn test_money_currency_mismatch_fails() {
    let usd = Money::new(dec!(10.00), CurrencyCode::Usd);
    let eur = Money::new(dec!(10.00), CurrencyCode::Eur);

    assert!((usd + eur).is_err());
    assert!((usd - eur).is_err());
}
