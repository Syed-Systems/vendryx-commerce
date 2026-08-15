use stheno_types::{Entitlements, Feature, PlanTier};

#[test]
fn test_entitlements_tier_features() {
    let free = Entitlements::for_tier(PlanTier::Free);
    assert!(!free.has(Feature::Promotions));
    assert_eq!(free.max_products, Some(100));

    let pro = Entitlements::for_tier(PlanTier::Pro);
    assert!(pro.has(Feature::Promotions));
    assert!(pro.has(Feature::MultiCurrency));
    assert!(!pro.has(Feature::B2B));

    let enterprise = Entitlements::for_tier(PlanTier::Enterprise);
    assert!(enterprise.has(Feature::B2B));
    assert!(enterprise.has(Feature::Automations));
}

#[test]
fn test_entitlements_fail_closed_boot_validation() {
    let mut invalid_tier = Entitlements::for_tier(PlanTier::Free);
    // Deliberately illegal configuration on free tier
    invalid_tier.active_features.insert(Feature::B2B);

    assert!(invalid_tier.validate_at_boot().is_err());
}
