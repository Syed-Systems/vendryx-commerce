use rust_decimal_macros::dec;
use stheno_tax::compute_tax_inclusive_reverse;

/// P3-14: Tax-inclusive reverse computation must be exact at every decimal precision.
/// For a €119 tax-inclusive item at 19% VAT:
///   subtotal = 119 / 1.19 = 100.00 (exactly)
///   tax      = 119 - 100 = 19.00 (exactly)
///   total    = 119.00 = subtotal + tax

#[test]
fn test_tax_inclusive_reverse_eur_19_pct() {
    let breakdown = compute_tax_inclusive_reverse(dec!(119.00), dec!(0.19), 2);
    assert_eq!(breakdown.subtotal, dec!(100.00));
    assert_eq!(breakdown.tax_total, dec!(19.00));
    assert_eq!(breakdown.total, dec!(119.00));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_reverse_usd_7_25_pct() {
    // $107.25 inclusive at 7.25%
    let breakdown = compute_tax_inclusive_reverse(dec!(107.25), dec!(0.0725), 2);
    assert_eq!(breakdown.subtotal, dec!(100.00));
    assert_eq!(breakdown.tax_total, dec!(7.25));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_reverse_jpy_zero_decimal() {
    // ¥1100 inclusive at 10% (JPY has 0 decimal places)
    let breakdown = compute_tax_inclusive_reverse(dec!(1100), dec!(0.10), 0);
    assert_eq!(breakdown.subtotal, dec!(1000));
    assert_eq!(breakdown.tax_total, dec!(100));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_reverse_bhd_three_decimal() {
    // BHD 1.050 inclusive at 5% (BHD has 3 decimal places)
    let breakdown = compute_tax_inclusive_reverse(dec!(1.050), dec!(0.05), 3);
    assert_eq!(breakdown.subtotal, dec!(1.000));
    assert_eq!(breakdown.tax_total, dec!(0.050));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_reverse_rounding_cent() {
    // $10.00 inclusive at 20% → subtotal = 8.33, tax = 1.67
    let breakdown = compute_tax_inclusive_reverse(dec!(10.00), dec!(0.20), 2);
    assert_eq!(breakdown.subtotal, dec!(8.33));
    assert_eq!(breakdown.tax_total, dec!(1.67));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_reverse_zero_rate() {
    let breakdown = compute_tax_inclusive_reverse(dec!(50.00), dec!(0), 2);
    assert_eq!(breakdown.subtotal, dec!(50.00));
    assert_eq!(breakdown.tax_total, dec!(0.00));
    assert_eq!(breakdown.subtotal + breakdown.tax_total, breakdown.total);
}

#[test]
fn test_tax_inclusive_proptest_invariant_100_amounts() {
    let rates = [
        dec!(0.05),
        dec!(0.0725),
        dec!(0.10),
        dec!(0.19),
        dec!(0.20),
        dec!(0.25),
    ];
    let precisions = [0u32, 2, 3];

    for rate in &rates {
        for precision in &precisions {
            for cents in 1..=100 {
                let total = rust_decimal::Decimal::new(cents * 100, *precision);
                let breakdown = compute_tax_inclusive_reverse(total, *rate, *precision);
                // Invariant: subtotal + tax_total == total (exactly)
                assert_eq!(
                    breakdown.subtotal + breakdown.tax_total,
                    breakdown.total,
                    "Invariant failed for total={total}, rate={rate}, precision={precision}"
                );
            }
        }
    }
}
