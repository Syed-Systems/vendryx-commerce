use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use stheno_types::{CurrencyCode, Money};

#[test]
fn test_allocation_proportional_exact() {
    let m = Money::new(dec!(100.00), CurrencyCode::Usd);
    let ratios = vec![dec!(1), dec!(2), dec!(3)];
    let allocated = m.allocate_proportionally(&ratios).unwrap();

    assert_eq!(allocated.len(), 3);
    let sum: Decimal = allocated.iter().map(|p| p.amount).sum();
    assert_eq!(sum, dec!(100.00));
}

// Special precision currencies list (22 special currencies per PRD & PAYMENTS.md)
const SPECIAL_CURRENCIES: [CurrencyCode; 22] = [
    // Zero decimal (16)
    CurrencyCode::Bif,
    CurrencyCode::Clp,
    CurrencyCode::Djf,
    CurrencyCode::Gnf,
    CurrencyCode::Jpy,
    CurrencyCode::Kmf,
    CurrencyCode::Krw,
    CurrencyCode::Mga,
    CurrencyCode::Pyg,
    CurrencyCode::Rwf,
    CurrencyCode::Ugx,
    CurrencyCode::Vnd,
    CurrencyCode::Vuv,
    CurrencyCode::Xaf,
    CurrencyCode::Xof,
    CurrencyCode::Xpf,
    // Three decimal (6)
    CurrencyCode::Bhd,
    CurrencyCode::Iqd,
    CurrencyCode::Jod,
    CurrencyCode::Kwd,
    CurrencyCode::Omr,
    CurrencyCode::Tnd,
];

#[test]
fn test_allocation_across_all_22_special_currencies() {
    for currency in SPECIAL_CURRENCIES {
        let amount = Money::new(dec!(1234.567), currency);
        let ratios = vec![dec!(1), dec!(1), dec!(1), dec!(1), dec!(1)];
        let allocated = amount.allocate_proportionally(&ratios).unwrap();

        let exp = currency.minor_unit_exponent();
        let multiplier = match exp {
            0 => Decimal::from(1),
            3 => Decimal::from(1000),
            _ => Decimal::from(100),
        };
        let rounded_total = (amount.amount * multiplier).round() / multiplier;

        let sum: Decimal = allocated.iter().map(|p| p.amount).sum();
        assert_eq!(
            sum, rounded_total,
            "Allocation sum mismatch for currency {currency:?}"
        );
    }
}

proptest! {
    #[test]
    fn proptest_allocation_sum_equals_original(
        amount_units in 1i64..1_000_000,
        r1 in 1u32..100,
        r2 in 1u32..100,
        r3 in 1u32..100,
    ) {
        let m = Money::new(Decimal::from(amount_units) / Decimal::from(100), CurrencyCode::Usd);
        let ratios = vec![Decimal::from(r1), Decimal::from(r2), Decimal::from(r3)];
        let allocated = m.allocate_proportionally(&ratios).unwrap();
        let sum: Decimal = allocated.iter().map(|p| p.amount).sum();
        prop_assert_eq!(sum, m.amount);
    }
}
