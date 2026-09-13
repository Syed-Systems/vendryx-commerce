//! P30-02: what a plan costs is data, and it is exact.
//!
//! Two properties are load-bearing here and neither is obvious from the type:
//!
//! 1. **A price is a string in TOML, parsed as a `Decimal`.** TOML has no decimal type, so
//!    `price = 1500.00` would be an `f64` — which [ADR-006](../../../docs/DECISIONS.md#adr-006)
//!    forbids anywhere near money. It would forbid it *silently*, too: `1500.00` survives a
//!    float round trip intact and `1499.95` does not, so the bug would ship on the day
//!    someone discounted a plan.
//!
//! 2. **No price means the plan is not sold, not that it is free.** A billing run must skip
//!    an unpriced plan. If "unpriced" resolved to zero, a plan whose price went missing in
//!    an edit would quietly become free for everyone on it, and the invoices would look
//!    correct.

use rust_decimal::Decimal;
use std::str::FromStr;
use stheno_api_core::{BillingInterval, Tier, TierConfig, TierDefinition};
use stheno_types::currency::CurrencyCode;

const SHIPPED: &str = include_str!("../../../../config/tiers.toml");

fn config(src: &str) -> TierConfig {
    TierConfig::from_toml(src).expect("tier config parses")
}

#[test]
fn the_shipped_pro_plan_has_an_exact_price_in_the_currency_the_business_charges() {
    let config = config(SHIPPED);

    let pricing = config
        .pricing(Tier::Pro)
        .expect("pro pricing resolves")
        .expect("pro is a sold plan");

    assert_eq!(
        pricing.price,
        Decimal::from_str("20.00").unwrap(),
        "the shipped Pro price must survive parsing exactly"
    );
    assert_eq!(
        pricing.currency,
        CurrencyCode::Usd,
        "the platform subscription is priced in USD; merchant checkout money can still be BDT"
    );
    assert_eq!(pricing.interval, BillingInterval::Month);
}

#[test]
fn a_plan_with_no_price_is_not_sold_rather_than_free() {
    let config = config(SHIPPED);

    for tier in [Tier::Free, Tier::Trial] {
        assert!(
            config.pricing(tier).expect("resolves").is_none(),
            "{} must report as not-sold, so a billing run skips it. Reporting zero would \
             make a plan whose price went missing look deliberately free.",
            tier.as_str()
        );
    }
}

#[test]
fn a_price_that_a_float_would_corrupt_survives_intact() {
    // 1499.95 is the case that matters: it does not round trip through f64. If the parser
    // ever goes back to a TOML float, this is the test that notices.
    let config = config(
        r#"
        [free]
        features = []
        [trial]
        features = []
        [pro]
        features = []
        price    = "1499.95"
        currency = "bdt"
        interval = "month"
        "#,
    );

    let pricing = config.pricing(Tier::Pro).unwrap().unwrap();
    assert_eq!(pricing.price, Decimal::from_str("1499.95").unwrap());
    assert_eq!(
        pricing.price.to_string(),
        "1499.95",
        "the decimal must render back exactly as written, with no float artefact"
    );

    // Twelve months of it must be an exact multiple, which f64 cannot promise.
    let year = pricing.price * Decimal::from(12);
    assert_eq!(year, Decimal::from_str("17999.40").unwrap());
}

#[test]
fn a_half_written_plan_is_refused_rather_than_guessed_at() {
    // Each of these is a plan someone started pricing and did not finish. Guessing a
    // default for the missing half is how a plan gets sold in the wrong currency.
    let cases = [
        (r#"price = "500.00""#, "a price with no currency"),
        (
            r#"price = "500.00"
               currency = "bdt""#,
            "a price with no interval",
        ),
        (
            r#"currency = "bdt"
               interval = "month""#,
            "a currency and interval with no price",
        ),
        (
            r#"price = "500.00"
               currency = "xyz"
               interval = "month""#,
            "an unknown currency",
        ),
        (
            r#"price = "500.00"
               currency = "bdt"
               interval = "fortnight""#,
            "an interval nothing bills on",
        ),
        (
            r#"price = "-100.00"
               currency = "bdt"
               interval = "month""#,
            "a negative price, which would pay the merchant",
        ),
        (
            r#"price = "not a number"
               currency = "bdt"
               interval = "month""#,
            "a price that is not a decimal",
        ),
    ];

    for (fragment, description) in cases {
        let src = format!(
            "[free]\nfeatures = []\n[trial]\nfeatures = []\n[pro]\nfeatures = []\n{fragment}\n"
        );
        let config = config(&src);
        assert!(
            config.pricing(Tier::Pro).is_err(),
            "{description} must be refused, not resolved to something plausible"
        );
    }
}

#[test]
fn a_free_plan_priced_at_zero_is_sold_and_distinct_from_unpriced() {
    // Free-as-in-zero is a real thing to want: it produces an invoice, which is how a
    // merchant on a promotional plan still shows up in the billing run and in retention.
    let config = config(
        r#"
        [trial]
        features = []
        [free]
        features = []
        price    = "0"
        currency = "bdt"
        interval = "month"
        [pro]
        features = []
        "#,
    );

    let free = config.pricing(Tier::Free).unwrap();
    assert!(
        free.is_some(),
        "a plan priced at zero is sold; only a plan with no price at all is not"
    );
    assert!(free.unwrap().price.is_zero());

    assert!(
        config.pricing(Tier::Pro).unwrap().is_none(),
        "and the unpriced plan next to it is still not sold"
    );
}

#[test]
fn a_control_plane_edit_does_not_take_a_plan_off_sale() {
    // The control plane rewrites features and limits (ADR-016). `carrying_terms_of` is
    // what stops it dropping the fields it does not edit — and price is one of them.
    // Without this, ticking a checkbox in the plan editor would silently unprice Pro.
    let config = config(SHIPPED);
    let shipped = config.definition(Tier::Pro).expect("pro exists");

    let edited = TierDefinition::from_parts([], [], []).carrying_terms_of(shipped);

    let pricing = edited
        .pricing()
        .expect("resolves")
        .expect("an edited plan is still sold");
    assert_eq!(pricing.price, Decimal::from_str("20.00").unwrap());
    assert_eq!(pricing.currency, CurrencyCode::Usd);
    assert_eq!(
        edited.term_months,
        vec![1],
        "and the minimum purchase term survives alongside it"
    );
    assert_eq!(
        edited.discounts, shipped.discounts,
        "and long-term discounts survive alongside it"
    );
}

#[test]
fn the_shipped_pro_plan_discounts_twelve_or_more_months() {
    let config = config(SHIPPED);
    let pro = config.definition(Tier::Pro).expect("pro exists");

    assert_eq!(
        pro.term_months,
        vec![1],
        "one month is the minimum selectable term, not the maximum"
    );
    assert_eq!(pro.discounts.len(), 1);
    assert_eq!(pro.discounts[0].starts_at_months, 12);
    assert_eq!(pro.discounts[0].percent, "10.00");
}

#[test]
fn the_shipped_pro_plan_12_and_24_month_totals_are_exact() {
    let config = config(SHIPPED);
    let pricing = config.pricing(Tier::Pro).unwrap().unwrap();
    let pro = config.definition(Tier::Pro).expect("pro exists");

    let discount_percent = Decimal::from_str(&pro.discounts[0].percent).unwrap();
    let one_hundred = Decimal::from(100);

    // 12 months
    let subtotal_12 = pricing.price * Decimal::from(12);
    let discount_12 = subtotal_12 * (discount_percent / one_hundred);
    let total_12 = subtotal_12 - discount_12;
    assert_eq!(total_12, Decimal::from_str("216.00").unwrap(), "12 months total should be exactly USD 216.00");

    // 24 months
    let subtotal_24 = pricing.price * Decimal::from(24);
    let discount_24 = subtotal_24 * (discount_percent / one_hundred);
    let total_24 = subtotal_24 - discount_24;
    assert_eq!(total_24, Decimal::from_str("432.00").unwrap(), "24 months total should be exactly USD 432.00");
}

#[test]
fn every_interval_the_engine_accepts_round_trips() {
    for interval in BillingInterval::ALL {
        assert_eq!(
            interval.as_str().parse::<BillingInterval>().unwrap(),
            interval
        );
    }
    assert_eq!(BillingInterval::Month.months(), 1);
    assert_eq!(
        BillingInterval::Year.months(),
        12,
        "a yearly plan covers twelve months, which is what places the next period boundary"
    );
}
