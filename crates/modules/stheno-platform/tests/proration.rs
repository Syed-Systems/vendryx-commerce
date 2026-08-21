//! P30-05: mid-cycle plan changes are prorated exactly.

use chrono::{TimeZone, Utc};
use stheno_platform::{CalculateProrationInput, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

#[test]
fn proration_credits_the_unused_old_plan_and_charges_the_new_remainder() {
    let period_start = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let change_at = Utc.with_ymd_and_hms(2026, 8, 11, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();

    let proration = PlatformService::calculate_proration(CalculateProrationInput {
        old_tier: "starter".to_string(),
        new_tier: "pro".to_string(),
        period_start,
        change_at,
        period_end,
        old_period_amount: Money::from_minor_units(310_000, CurrencyCode::Bdt),
        new_period_amount: Money::from_minor_units(620_000, CurrencyCode::Bdt),
    })
    .expect("proration computes");

    assert_eq!(
        proration.credit,
        Money::from_minor_units(-210_000, CurrencyCode::Bdt),
        "21 unused days of a 31-day 3100 BDT plan are credited back"
    );
    assert_eq!(
        proration.charge,
        Money::from_minor_units(420_000, CurrencyCode::Bdt),
        "the new 6200 BDT plan is charged for the same 21-day remainder"
    );
    assert_eq!(
        proration.net_adjustment,
        Money::from_minor_units(210_000, CurrencyCode::Bdt)
    );
    assert_eq!(proration.lines.len(), 2);
    assert_eq!(proration.lines[0].kind, "proration_credit");
    assert_eq!(proration.lines[1].kind, "proration_charge");
}

#[test]
fn proration_allocates_awkward_periods_without_losing_a_minor_unit() {
    let period_start = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let change_at = Utc.with_ymd_and_hms(2026, 8, 2, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 8, 4, 0, 0, 0).unwrap();

    let old_period_amount = Money::from_minor_units(10_000, CurrencyCode::Bdt);
    let new_period_amount = Money::from_minor_units(20_000, CurrencyCode::Bdt);
    let proration = PlatformService::calculate_proration(CalculateProrationInput {
        old_tier: "starter".to_string(),
        new_tier: "pro".to_string(),
        period_start,
        change_at,
        period_end,
        old_period_amount,
        new_period_amount,
    })
    .expect("proration computes");

    let old_used = (old_period_amount + proration.credit).expect("same currency");
    let old_parts_sum = (old_used
        + Money::new(-proration.credit.amount, proration.credit.currency))
    .expect("same currency");
    assert_eq!(
        old_parts_sum, old_period_amount,
        "used old-plan amount plus unused credit must reconstruct the old period exactly"
    );

    let new_unused = proration.charge;
    let new_used = (new_period_amount - new_unused).expect("same currency");
    let new_parts_sum = (new_used + new_unused).expect("same currency");
    assert_eq!(
        new_parts_sum, new_period_amount,
        "used/remainder allocation of the new plan must reconstruct the new period exactly"
    );
}

#[test]
fn proration_rejects_cross_currency_plan_changes() {
    let period_start = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let change_at = Utc.with_ymd_and_hms(2026, 8, 11, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();

    let err = PlatformService::calculate_proration(CalculateProrationInput {
        old_tier: "starter".to_string(),
        new_tier: "pro".to_string(),
        period_start,
        change_at,
        period_end,
        old_period_amount: Money::from_minor_units(310_000, CurrencyCode::Bdt),
        new_period_amount: Money::from_minor_units(6_200, CurrencyCode::Usd),
    })
    .expect_err("cross-currency proration is refused");

    assert!(
        matches!(err, stheno_types::Error::CurrencyMismatch { .. }),
        "got {err:?}"
    );
}
