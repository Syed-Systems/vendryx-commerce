//! P30-06: Bangladesh VAT is computed from the invoice subtotal.

use rust_decimal::Decimal;
use stheno_platform::{CalculateVatInput, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

#[test]
fn bd_vat_is_fifteen_percent_of_the_invoice_subtotal() {
    let vat = PlatformService::calculate_vat(CalculateVatInput {
        taxable_subtotal: Money::from_minor_units(150_000, CurrencyCode::Bdt),
        rate: Decimal::new(15, 2),
    })
    .expect("vat computes");

    assert_eq!(vat.kind, "tax");
    assert_eq!(vat.jurisdiction, "BD");
    assert_eq!(vat.mushak_form, "Mushak-6.3");
    assert_eq!(vat.rate, Decimal::new(15, 2));
    assert_eq!(
        vat.amount,
        Money::from_minor_units(22_500, CurrencyCode::Bdt)
    );
}

#[test]
fn vat_uses_the_invoice_subtotal_not_rounded_per_line_tax() {
    let vat = PlatformService::calculate_vat(CalculateVatInput {
        taxable_subtotal: Money::from_minor_units(10_001, CurrencyCode::Bdt),
        rate: Decimal::new(15, 2),
    })
    .expect("vat computes");

    assert_eq!(
        vat.amount,
        Money::from_minor_units(1_500, CurrencyCode::Bdt),
        "15% of the invoice subtotal rounds once to 15.00 BDT; taxing two 50.005 BDT \
         lines separately would over-collect"
    );
}

#[test]
fn vat_refuses_negative_subtotals() {
    let err = PlatformService::calculate_vat(CalculateVatInput {
        taxable_subtotal: Money::from_minor_units(-10_000, CurrencyCode::Bdt),
        rate: Decimal::new(15, 2),
    })
    .expect_err("negative taxable base is refused");

    assert!(
        matches!(
            err,
            stheno_types::Error::Invalid {
                code: "negative_taxable_subtotal",
                ..
            }
        ),
        "got {err:?}"
    );
}
