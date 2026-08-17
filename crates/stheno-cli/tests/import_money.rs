use rust_decimal_macros::dec;
use stheno_cli::money::convert_medusa_money_to_stheno;
use stheno_types::currency::CurrencyCode;

#[test]
fn test_medusa_money_conversion_usd() {
    // 1500 minor units in USD should be $15.00
    let amount = 1500;
    let money = convert_medusa_money_to_stheno(amount, CurrencyCode::Usd).unwrap();

    assert_eq!(money.amount, dec!(15.00));
    assert_eq!(money.to_minor_units().unwrap(), amount);
}

#[test]
fn test_medusa_money_conversion_jpy() {
    // 1500 minor units in JPY should be 1500 (0 decimal places)
    let amount = 1500;
    let money = convert_medusa_money_to_stheno(amount, CurrencyCode::Jpy).unwrap();

    assert_eq!(money.amount, dec!(1500));
    assert_eq!(money.to_minor_units().unwrap(), amount);
}

#[test]
fn test_medusa_money_conversion_kwd() {
    // 1240 minor units in KWD should be 1.240 (3 decimal places, Stripe rounding rule)
    // Note: 1240 is used because KWD requires the last digit to be 0 per Stripe rules
    let amount = 1240;
    let money = convert_medusa_money_to_stheno(amount, CurrencyCode::Kwd).unwrap();

    assert_eq!(money.amount, dec!(1.240));
    assert_eq!(money.to_minor_units().unwrap(), amount);
}

#[test]
fn test_medusa_money_conversion_precision_loss_assertion() {
    // A malformed Medusa money value that violates Stripe rules for 3-decimal currencies
    // KWD must end in 0. So 1234 should theoretically be invalid, but let's test how our conversion handles it.
    // Our conversion takes 1234, becomes 1.234 KWD.
    // When re-converted to minor units, it becomes 1240 per the Stripe rule in to_minor_units().
    // So 1240 != 1234, causing an error.
    let amount = 1234;
    let result = convert_medusa_money_to_stheno(amount, CurrencyCode::Kwd);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Precision lost"),
        "Expected precision loss error, got: {}",
        err_msg
    );
}
