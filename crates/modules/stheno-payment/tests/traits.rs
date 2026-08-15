use rust_decimal_macros::dec;
use stheno_payment::{from_minor_units, to_minor_units};

#[test]
fn test_money_boundary_zero_two_and_three_decimal_currencies() {
    // 1. Zero-decimal: JPY (PAYMENTS §5, E15)
    assert_eq!(to_minor_units(dec!(1000), "JPY").unwrap(), 1000);
    assert_eq!(from_minor_units(1000, "JPY"), dec!(1000));

    // 2. Default 2-decimal: USD
    assert_eq!(to_minor_units(dec!(19.99), "USD").unwrap(), 1999);
    assert_eq!(from_minor_units(1999, "USD"), dec!(19.99));

    // 3. Three-decimal upward rounding: KWD (PAYMENTS §5, E16)
    // 1.234 KWD -> scaled = 1234 -> rounded up to nearest 10 = 1240
    assert_eq!(to_minor_units(dec!(1.234), "KWD").unwrap(), 1240);
    // 1.230 KWD -> 1230
    assert_eq!(to_minor_units(dec!(1.230), "KWD").unwrap(), 1230);
    // 1.231 KWD -> 1240
    assert_eq!(to_minor_units(dec!(1.231), "KWD").unwrap(), 1240);
}
