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

use crate::entitlements::{
    Entitlements, EntitlementsError, Feature, QuotaKey, Tier, TierConfig, TierDefinition, WindowKey,
};

/// A tenant's plan, as stored. Kept structural rather than importing the platform module
/// so this crate stays free of a database dependency.
#[derive(Debug, Clone)]
pub struct TenantPlan {
    pub tier: String,
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub term_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub quota_overrides: serde_json::Value,
    pub window_overrides: serde_json::Value,
    /// `{"payment_gateway": true, "fraud_score": false}` — an explicit false revokes a
    /// feature the tier grants, an absent key leaves the tier's answer standing.
    pub feature_overrides: serde_json::Value,
}

/// A stored `platform.tier_definition` row.
///
/// Structural rather than the platform module's type, for the same reason [`TenantPlan`]
/// is: this crate stays free of a database dependency.
#[derive(Debug, Clone)]
pub struct TierRow {
    pub tier: String,
    /// `["payment_gateway", "csv_export"]`
    pub features: serde_json::Value,
    /// `{"max_products": 100}` — 0 means unlimited, the same sentinel as the TOML.
    pub quotas: serde_json::Value,
    pub windows: serde_json::Value,
}

/// Lays stored tier definitions over the shipped TOML defaults.
///
/// A row REPLACES that tier wholesale; a tier with no row keeps the TOML definition, so
/// an untouched deployment resolves exactly as it did before this table existed. An
/// unparseable row is skipped rather than applied partially — a half-applied plan is a
/// plan nobody chose, and the TOML default is the safer thing to fall back to.
pub fn overlay_tier_rows(base: &TierConfig, rows: &[TierRow]) -> TierConfig {
    let mut config = base.clone();
    for row in rows {
        let Ok(tier) = row.tier.parse::<Tier>() else {
            continue;
        };
        let Ok(definition) = definition_from_row(row) else {
            continue;
        };
        let carried = match base.definition(tier) {
            Ok(existing) => definition.carrying_terms_of(existing),
            Err(_) => definition,
        };
        config.set(tier, carried);
    }
    config
}

fn definition_from_row(row: &TierRow) -> Result<TierDefinition, EntitlementsError> {
    let features = row
        .features
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.parse::<Feature>())
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let quotas = limits_from(&row.quotas, |k| k.parse::<QuotaKey>())?;
    let windows = limits_from(&row.windows, |k| k.parse::<WindowKey>())?;

    Ok(TierDefinition::from_parts(features, quotas, windows))
}

fn limits_from<K>(
    value: &serde_json::Value,
    parse: impl Fn(&str) -> Result<K, EntitlementsError>,
) -> Result<Vec<(K, u64)>, EntitlementsError> {
    let Some(object) = value.as_object() else {
        return Ok(Vec::new());
    };
    object
        .iter()
        .map(|(key, raw)| {
            let limit = raw
                .as_u64()
                .ok_or_else(|| EntitlementsError::MalformedQuota {
                    key: key.clone(),
                    value: raw.to_string(),
                })?;
            Ok((parse(key)?, limit))
        })
        .collect()
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
    resolve_for_tenant_at(config, plan, chrono::Utc::now())
}

/// Resolves entitlements at an explicit instant so expiry behavior is deterministic in
/// tests and identical for every request handled at the same time.
pub fn resolve_for_tenant_at(
    config: &TierConfig,
    plan: &TenantPlan,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Entitlements, EntitlementsError> {
    let tier: Tier = effective_tier(plan, now)?;

    let mut overrides = BTreeMap::new();
    flatten("STHENO_QUOTA_", &plan.quota_overrides, &mut overrides);
    flatten("STHENO_WINDOW_", &plan.window_overrides, &mut overrides);

    let mut resolved = config
        .resolve(tier, &overrides)?
        .with_trial_ends_at(plan.trial_ends_at)
        .with_term_ends_at(plan.term_ends_at);

    // Feature overrides land last, so a negotiated grant survives the tier's answer and a
    // revocation survives an upgrade. An unknown key is an error rather than a shrug: a
    // typo in a negotiated exception must not read as "no exception".
    if let Some(object) = plan.feature_overrides.as_object() {
        for (key, enabled) in object {
            let feature: Feature = key.parse()?;
            resolved.set_feature(feature, enabled.as_bool().unwrap_or(false));
        }
    }

    Ok(resolved)
}

fn effective_tier(
    plan: &TenantPlan,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Tier, EntitlementsError> {
    let assigned: Tier = plan.tier.parse()?;
    // A trial without a persisted end must never become an indefinite Pro grant.
    let expired_trial = assigned == Tier::Trial
        && match plan.trial_ends_at {
            None => true,
            Some(end) => now >= end,
        };
    let expired_paid_term = assigned == Tier::Pro
        && plan
            .term_ends_at
            .and_then(|end| end.checked_add_signed(chrono::Duration::days(7)))
            .is_some_and(|grace_end| now >= grace_end);

    if expired_trial || expired_paid_term {
        Ok(Tier::Free)
    } else {
        Ok(assigned)
    }
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
            feature_overrides: serde_json::json!({}),
        }
    }

    #[test]
    fn expired_trial_and_paid_term_resolve_to_free_at_exact_boundaries() {
        let config = TierConfig::from_toml(TIERS).unwrap();
        let trial_end = chrono::DateTime::parse_from_rfc3339("2026-10-10T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let mut trial = plan("trial", serde_json::json!({}));
        trial.trial_ends_at = Some(trial_end);

        let before_trial_end = resolve_for_tenant_at(
            &config,
            &trial,
            trial_end - chrono::Duration::nanoseconds(1),
        )
        .unwrap();
        let at_trial_end = resolve_for_tenant_at(&config, &trial, trial_end).unwrap();
        assert_eq!(before_trial_end.tier(), Tier::Trial);
        assert_eq!(at_trial_end.tier(), Tier::Free);
        assert!(!at_trial_end.has(Feature::PaymentGateway));

        let term_end = trial_end;
        let mut pro = plan("pro", serde_json::json!({}));
        pro.term_ends_at = Some(term_end);
        let grace_end = term_end + chrono::Duration::days(7);

        let at_term_end = resolve_for_tenant_at(&config, &pro, term_end).unwrap();
        let before_grace_end =
            resolve_for_tenant_at(&config, &pro, grace_end - chrono::Duration::nanoseconds(1))
                .unwrap();
        let at_grace_end = resolve_for_tenant_at(&config, &pro, grace_end).unwrap();
        assert_eq!(at_term_end.tier(), Tier::Pro);
        assert_eq!(before_grace_end.tier(), Tier::Pro);
        assert_eq!(at_grace_end.tier(), Tier::Free);
        assert!(!at_grace_end.has(Feature::PaymentGateway));
        assert_eq!(at_grace_end.quota(QuotaKey::MaxProducts), Some(100));
    }

    #[test]
    fn missing_trial_expiry_fails_closed_but_missing_paid_term_stays_assigned() {
        let config = TierConfig::from_toml(TIERS).unwrap();
        let now = chrono::Utc::now();
        let trial =
            resolve_for_tenant_at(&config, &plan("trial", serde_json::json!({})), now).unwrap();
        let pro = resolve_for_tenant_at(&config, &plan("pro", serde_json::json!({})), now).unwrap();
        assert_eq!(trial.tier(), Tier::Free);
        assert_eq!(pro.tier(), Tier::Pro);
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

    fn row(tier: &str, features: serde_json::Value, quotas: serde_json::Value) -> TierRow {
        TierRow {
            tier: tier.to_string(),
            features,
            quotas,
            windows: serde_json::json!({}),
        }
    }

    #[test]
    fn a_stored_tier_row_replaces_that_tier_and_leaves_the_others_alone() {
        let base = TierConfig::from_toml(TIERS).unwrap();

        // The operator sells the payment gateway on Free and halves the product limit.
        let config = overlay_tier_rows(
            &base,
            &[row(
                "free",
                serde_json::json!(["csv_export", "payment_gateway"]),
                serde_json::json!({ "max_products": 50 }),
            )],
        );

        let free = resolve_for_tenant(&config, &plan("free", serde_json::json!({}))).unwrap();
        assert!(free.has(Feature::PaymentGateway));
        assert!(
            !free.has(Feature::WebPush),
            "replacement, not merge: web_push was not in the row"
        );
        assert_eq!(free.quota(QuotaKey::MaxProducts), Some(50));

        // Pro had no row, so it still resolves from the shipped TOML.
        let pro = resolve_for_tenant(&config, &plan("pro", serde_json::json!({}))).unwrap();
        assert!(pro.has(Feature::WebPush));
        assert_eq!(pro.quota(QuotaKey::MaxProducts), None);
    }

    #[test]
    fn a_malformed_row_leaves_the_shipped_default_standing() {
        let base = TierConfig::from_toml(TIERS).unwrap();

        // Nothing here is applied partially: a bad feature name discards the whole row.
        let config = overlay_tier_rows(
            &base,
            &[row(
                "free",
                serde_json::json!(["csv_export", "teleportation"]),
                serde_json::json!({ "max_products": 5 }),
            )],
        );

        let free = resolve_for_tenant(&config, &plan("free", serde_json::json!({}))).unwrap();
        assert_eq!(free.quota(QuotaKey::MaxProducts), Some(100), "TOML default");
        assert!(free.has(Feature::WebPush));
    }

    #[test]
    fn a_per_tenant_feature_override_grants_and_revokes() {
        let config = TierConfig::from_toml(TIERS).unwrap();

        let mut granted = plan("free", serde_json::json!({}));
        granted.feature_overrides = serde_json::json!({ "payment_gateway": true });
        let granted = resolve_for_tenant(&config, &granted).unwrap();
        assert!(
            granted.has(Feature::PaymentGateway),
            "negotiated grant on Free"
        );

        let mut revoked = plan("pro", serde_json::json!({}));
        revoked.feature_overrides = serde_json::json!({ "fraud_score": false });
        let revoked = resolve_for_tenant(&config, &revoked).unwrap();
        assert!(!revoked.has(Feature::FraudScore));
        assert!(
            revoked.has(Feature::PaymentGateway),
            "only the named one moves"
        );
    }

    #[test]
    fn an_unknown_feature_override_is_an_error_not_a_shrug() {
        let config = TierConfig::from_toml(TIERS).unwrap();
        let mut broken = plan("free", serde_json::json!({}));
        broken.feature_overrides = serde_json::json!({ "payment_gatway": true });
        assert!(resolve_for_tenant(&config, &broken).is_err());
    }
}
