//! Per-tenant entitlement resolution (ADR-012).
//!
//! Under ADR-009 a deployment served one merchant, so entitlements were resolved once at
//! boot from `STHENO_TIER`. One process now serves many tenants on different tiers, so
//! the *definitions* still load once from `config/tiers.toml` while the *assignment* is
//! read per tenant from `platform.subscription`.
//!
//! The existing `TierConfig::resolve(tier, env)` already accepts an arbitrary override
//! map keyed `STHENO_QUOTA_*` / `STHENO_WINDOW_*` (ENTITLEMENTS.md 5b). Per-tenant
//! negotiated exceptions are flattened into that same shape, so `resolve` needs no
//! signature change at all.

use std::collections::BTreeMap;

use crate::entitlements::{Entitlements, EntitlementsError, Tier, TierConfig};

/// A tenant's plan, as stored. Kept structural rather than importing the platform module
/// so this crate stays free of a database dependency.
#[derive(Debug, Clone)]
pub struct TenantPlan {
    pub tier: String,
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub term_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub quota_overrides: serde_json::Value,
    pub window_overrides: serde_json::Value,
}

/// Flattens `{"max_products": 250}` into `{"STHENO_QUOTA_MAX_PRODUCTS": "250"}`, the
/// shape `TierConfig::resolve` already understands.
fn flatten(prefix: &str, value: &serde_json::Value, out: &mut BTreeMap<String, String>) {
    let Some(object) = value.as_object() else {
        return;
    };
    for (key, raw) in object {
        let rendered = match raw {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out.insert(format!("{prefix}{}", key.to_ascii_uppercase()), rendered);
    }
}

/// Resolves what this tenant may do.
pub fn resolve_for_tenant(
    config: &TierConfig,
    plan: &TenantPlan,
) -> Result<Entitlements, EntitlementsError> {
    let tier: Tier = plan.tier.parse()?;

    let mut overrides = BTreeMap::new();
    flatten("STHENO_QUOTA_", &plan.quota_overrides, &mut overrides);
    flatten("STHENO_WINDOW_", &plan.window_overrides, &mut overrides);

    Ok(config
        .resolve(tier, &overrides)?
        .with_trial_ends_at(plan.trial_ends_at)
        .with_term_ends_at(plan.term_ends_at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entitlements::{Feature, QuotaKey};

    const TIERS: &str = include_str!("../../../../config/tiers.toml");

    fn plan(tier: &str, quotas: serde_json::Value) -> TenantPlan {
        TenantPlan {
            tier: tier.to_string(),
            trial_ends_at: None,
            term_ends_at: None,
            quota_overrides: quotas,
            window_overrides: serde_json::json!({}),
        }
    }

    #[test]
    fn two_tenants_on_different_tiers_resolve_differently() {
        let config = TierConfig::from_toml(TIERS).unwrap();

        let free = resolve_for_tenant(&config, &plan("free", serde_json::json!({}))).unwrap();
        let pro = resolve_for_tenant(&config, &plan("pro", serde_json::json!({}))).unwrap();

        assert!(!free.has(Feature::PaymentGateway));
        assert!(pro.has(Feature::PaymentGateway));
        assert_eq!(free.quota(QuotaKey::MaxProducts), Some(100));
        assert_eq!(pro.quota(QuotaKey::MaxProducts), None, "pro is unlimited");
    }

    #[test]
    fn a_negotiated_quota_override_is_applied_to_one_tenant_only() {
        // ENTITLEMENTS.md 5b's `STHENO_QUOTA_MAX_PRODUCTS=250`, persisted per tenant
        // instead of set per process.
        let config = TierConfig::from_toml(TIERS).unwrap();

        let standard = resolve_for_tenant(&config, &plan("free", serde_json::json!({}))).unwrap();
        let negotiated = resolve_for_tenant(
            &config,
            &plan("free", serde_json::json!({ "max_products": 250 })),
        )
        .unwrap();

        assert_eq!(standard.quota(QuotaKey::MaxProducts), Some(100));
        assert_eq!(negotiated.quota(QuotaKey::MaxProducts), Some(250));
    }

    #[test]
    fn an_unknown_tier_is_an_error_not_a_silent_downgrade() {
        let config = TierConfig::from_toml(TIERS).unwrap();
        assert!(resolve_for_tenant(&config, &plan("enterprise", serde_json::json!({}))).is_err());
    }
}
