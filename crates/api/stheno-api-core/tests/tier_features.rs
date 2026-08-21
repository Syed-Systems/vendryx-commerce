//! P35-01: custom_theme and branding features in config/tiers.toml.

use stheno_api_core::{Tier, TierConfig};

const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

#[test]
fn tier_features_include_custom_theme_and_branding_in_pro() {
    let config = TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses");
    let pro_tier = config.definition(Tier::Pro).expect("pro tier exists");

    assert!(pro_tier.features.contains(&"custom_theme".to_string()));
    assert!(pro_tier.features.contains(&"branding".to_string()));

    let free_tier = config.definition(Tier::Free).expect("free tier exists");
    assert!(!free_tier.features.contains(&"custom_theme".to_string()));
    assert!(!free_tier.features.contains(&"branding".to_string()));
}
