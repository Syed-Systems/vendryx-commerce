//! P9-14e: each tier boots and mounts exactly its declared routes, no more.
//!
//! This is the drift catcher (ENTITLEMENTS.md 8). If a new route group is added
//! unconditionally that should have been gated, or a tier quietly gains a feature,
//! these assertions fail rather than shipping a leak.

use std::collections::BTreeMap;
use stheno_api_core::{
    EntitlementsError, Feature, QuotaKey, QuotaState, Tier, TierConfig, WindowKey,
};
use stheno_api_router::mounted_features;

const TIERS: &str = include_str!("../../../../config/tiers.toml");

fn config() -> TierConfig {
    TierConfig::from_toml(TIERS).expect("config/tiers.toml must parse")
}

fn resolve(tier: Tier) -> stheno_api_core::Entitlements {
    config()
        .resolve(tier, &BTreeMap::new())
        .expect("tier must resolve")
}

#[test]
fn every_tier_boots() {
    for tier in [Tier::Trial, Tier::Free, Tier::Pro] {
        let ent = resolve(tier);
        assert_eq!(ent.tier(), tier);
    }
}

#[test]
fn each_tier_mounts_exactly_its_declared_features() {
    let base = vec![
        Feature::AnalyticsBasic,
        Feature::CsvExport,
        Feature::SpreadsheetImport,
        Feature::WebPush,
    ];

    let mut trial = base.clone();
    trial.sort();
    assert_eq!(mounted_features(&resolve(Tier::Trial)), trial);

    let mut free = base.clone();
    free.sort();
    assert_eq!(mounted_features(&resolve(Tier::Free)), free);

    let mut pro = base;
    pro.extend([
        Feature::PaymentGateway,
        Feature::CourierDispatch,
        Feature::FraudScore,
        Feature::CustomDomain,
    ]);
    pro.sort();
    assert_eq!(mounted_features(&resolve(Tier::Pro)), pro);
}

#[test]
fn paid_only_features_never_leak_into_free_or_trial() {
    for tier in [Tier::Trial, Tier::Free] {
        let ent = resolve(tier);
        for paid in [
            Feature::PaymentGateway,
            Feature::CourierDispatch,
            Feature::FraudScore,
            Feature::CustomDomain,
        ] {
            assert!(!ent.has(paid), "{:?} must not expose {:?}", tier, paid);
        }
    }
}

#[test]
fn quotas_match_the_tier_table() {
    // Zero in config means unlimited, surfaced as None.
    let trial = resolve(Tier::Trial);
    assert_eq!(trial.quota(QuotaKey::MaxProducts), None);
    assert_eq!(trial.quota(QuotaKey::MaxStaffSeats), Some(10));

    let free = resolve(Tier::Free);
    assert_eq!(free.quota(QuotaKey::MaxProducts), Some(100));
    assert_eq!(free.quota(QuotaKey::MaxStaffSeats), Some(3));
    assert_eq!(free.window(WindowKey::AnalyticsWindowDays), Some(30));

    let pro = resolve(Tier::Pro);
    assert_eq!(pro.quota(QuotaKey::MaxProducts), None);
    assert_eq!(pro.window(WindowKey::AnalyticsWindowDays), None);
}

#[test]
fn creation_is_rejected_exactly_at_the_limit_not_at_limit_plus_one() {
    let free = resolve(Tier::Free);
    assert!(
        free.may_create(QuotaKey::MaxProducts, 99),
        "99 used, one left"
    );
    assert!(
        !free.may_create(QuotaKey::MaxProducts, 100),
        "at the limit, creation must be blocked"
    );
    assert!(!free.may_create(QuotaKey::MaxProducts, 101));

    // Unlimited never blocks.
    assert!(resolve(Tier::Pro).may_create(QuotaKey::MaxProducts, 1_000_000));
}

#[test]
fn downgrade_puts_an_over_limit_quota_into_grace() {
    // ENTITLEMENTS.md 6.4: 412 products on a limit of 100 stay live; only creates block.
    let free = resolve(Tier::Free);
    assert_eq!(
        free.quota_state(QuotaKey::MaxProducts, 412),
        QuotaState::Grace
    );
    assert!(!free.may_create(QuotaKey::MaxProducts, 412));
    assert_eq!(free.quota_state(QuotaKey::MaxProducts, 50), QuotaState::Ok);
}

#[test]
fn env_overrides_apply_on_top_of_tier_defaults() {
    let env = BTreeMap::from([("STHENO_QUOTA_MAX_PRODUCTS".to_string(), "250".to_string())]);
    let ent = config().resolve(Tier::Free, &env).unwrap();
    assert_eq!(ent.quota(QuotaKey::MaxProducts), Some(250));
    // Untouched quotas keep their tier default.
    assert_eq!(ent.quota(QuotaKey::MaxStaffSeats), Some(3));
}

#[test]
fn an_unknown_feature_key_aborts_rather_than_silently_granting() {
    let bad = r#"
[free]
features = ["spreadsheet_import", "teleportation"]
quotas   = { max_products = 100 }
"#;
    let err = TierConfig::from_toml(bad)
        .unwrap()
        .resolve(Tier::Free, &BTreeMap::new())
        .unwrap_err();
    assert_eq!(
        err,
        EntitlementsError::UnknownFeature("teleportation".to_string())
    );
}

#[test]
fn a_malformed_quota_aborts_rather_than_silently_denying() {
    let bad = r#"
[free]
features = []
quotas   = { max_products = "lots" }
"#;
    let err = TierConfig::from_toml(bad)
        .unwrap()
        .resolve(Tier::Free, &BTreeMap::new())
        .unwrap_err();
    assert!(matches!(err, EntitlementsError::MalformedQuota { .. }));
}

#[test]
fn an_unknown_quota_key_aborts() {
    let bad = r#"
[free]
features = []
quotas   = { max_unicorns = 3 }
"#;
    let err = TierConfig::from_toml(bad)
        .unwrap()
        .resolve(Tier::Free, &BTreeMap::new())
        .unwrap_err();
    assert_eq!(
        err,
        EntitlementsError::UnknownQuota("max_unicorns".to_string())
    );
}
