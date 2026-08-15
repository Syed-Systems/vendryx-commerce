use rust_decimal_macros::dec;
use stheno_types::{CurrencyCode, Money};

#[test]
fn test_zero_decimal_currencies() {
    let jpy = Money::new(dec!(2500), CurrencyCode::Jpy);
    assert_eq!(jpy.to_minor_units().unwrap(), 2500);
    assert_eq!(
        Money::from_minor_units(2500, CurrencyCode::Jpy).amount,
        dec!(2500)
    );

    let krw = Money::new(dec!(50000), CurrencyCode::Krw);
    assert_eq!(krw.to_minor_units().unwrap(), 50000);
    assert_eq!(
        Money::from_minor_units(50000, CurrencyCode::Krw).amount,
        dec!(50000)
    );
}

#[test]
fn test_standard_two_decimal_currencies() {
    let usd = Money::new(dec!(19.99), CurrencyCode::Usd);
    assert_eq!(usd.to_minor_units().unwrap(), 1999);
    assert_eq!(
        Money::from_minor_units(1999, CurrencyCode::Usd).amount,
        dec!(19.99)
    );

    let eur = Money::new(dec!(100.05), CurrencyCode::Eur);
    assert_eq!(eur.to_minor_units().unwrap(), 10005);
    assert_eq!(
        Money::from_minor_units(10005, CurrencyCode::Eur).amount,
        dec!(100.05)
    );
}

#[test]
fn test_three_decimal_currencies_stripe_rule() {
    // 3-decimal currencies round up to nearest 10 for Stripe compatibility (PAYMENTS §5)
    let kwd = Money::new(dec!(1.234), CurrencyCode::Kwd);
    assert_eq!(kwd.to_minor_units().unwrap(), 1240);

    let bhd = Money::new(dec!(5.551), CurrencyCode::Bhd);
    assert_eq!(bhd.to_minor_units().unwrap(), 5560);

    let omr = Money::new(dec!(10.500), CurrencyCode::Omr);
    assert_eq!(omr.to_minor_units().unwrap(), 10500);
}
