//! Runtime entitlements: what a deployment is allowed to do (ENTITLEMENTS.md).
//!
//! This type lives in the API layer on purpose. Domain modules must never see it —
//! a gate is a mount-point decision, never an `if` inside a service
//! (ENTITLEMENTS.md 6.1, enforced by `scripts/check-no-domain-gates.sh`).
//!
//! `0` is the single sentinel for "unlimited" across both quotas and windows, so a
//! missing or zeroed value can never accidentally read as a restriction.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Trial,
    Free,
    Pro,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Trial => "trial",
            Tier::Free => "free",
            Tier::Pro => "pro",
        }
    }
}

impl std::str::FromStr for Tier {
    type Err = EntitlementsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "trial" => Ok(Tier::Trial),
            "free" => Ok(Tier::Free),
            "pro" => Ok(Tier::Pro),
            other => Err(EntitlementsError::UnknownTier(other.to_string())),
        }
    }
}

/// Gateable features. Core commerce is deliberately absent: it is never gateable
/// (ENTITLEMENTS.md 4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    SpreadsheetImport,
    CsvExport,
    WebPush,
    AnalyticsBasic,
    PaymentGateway,
    CourierDispatch,
    FraudScore,
    CustomDomain,
}

impl Feature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Feature::SpreadsheetImport => "spreadsheet_import",
            Feature::CsvExport => "csv_export",
            Feature::WebPush => "web_push",
            Feature::AnalyticsBasic => "analytics_basic",
            Feature::PaymentGateway => "payment_gateway",
            Feature::CourierDispatch => "courier_dispatch",
            Feature::FraudScore => "fraud_score",
            Feature::CustomDomain => "custom_domain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaKey {
    MaxProducts,
    MaxStaffSeats,
}

impl QuotaKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuotaKey::MaxProducts => "max_products",
            QuotaKey::MaxStaffSeats => "max_staff_seats",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowKey {
    AnalyticsWindowDays,
}

impl WindowKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowKey::AnalyticsWindowDays => "analytics_window_days",
        }
    }
}

/// Quota state as reported by `GET /admin/entitlements`.
///
/// `Grace` is the grandfathered case (ENTITLEMENTS.md 6.4): `used > limit` after a
/// downgrade. Existing data stays live and editable; only new creates are blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaState {
    Ok,
    Grace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementsError {
    UnknownTier(String),
    UnknownFeature(String),
    UnknownQuota(String),
    UnknownWindow(String),
    MalformedQuota { key: String, value: String },
    MissingTier(String),
    Parse(String),
    Io(String),
}

impl std::fmt::Display for EntitlementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntitlementsError::UnknownTier(t) => write!(f, "unknown tier `{t}`"),
            EntitlementsError::UnknownFeature(k) => write!(f, "unknown feature key `{k}`"),
            EntitlementsError::UnknownQuota(k) => write!(f, "unknown quota key `{k}`"),
            EntitlementsError::UnknownWindow(k) => write!(f, "unknown window key `{k}`"),
            EntitlementsError::MalformedQuota { key, value } => {
                write!(f, "malformed quota `{key}` = `{value}`")
            }
            EntitlementsError::MissingTier(t) => write!(f, "tier `{t}` is not defined in config"),
            EntitlementsError::Parse(m) => write!(f, "tier config parse error: {m}"),
            EntitlementsError::Io(m) => write!(f, "tier config read error: {m}"),
        }
    }
}

impl std::error::Error for EntitlementsError {}

/// One tier as written in `config/tiers.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TierDefinition {
    #[serde(default)]
    pub duration_days: Option<u64>,
    #[serde(default)]
    pub starts_on: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub quotas: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub windows: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub term_months: Vec<u32>,
}

/// The parsed contents of `config/tiers.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TierConfig {
    #[serde(flatten)]
    tiers: BTreeMap<String, TierDefinition>,
}

impl TierConfig {
    pub fn from_toml(src: &str) -> Result<Self, EntitlementsError> {
        toml::from_str(src).map_err(|e| EntitlementsError::Parse(e.to_string()))
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, EntitlementsError> {
        let src = std::fs::read_to_string(path.as_ref())
            .map_err(|e| EntitlementsError::Io(format!("{}: {e}", path.as_ref().display())))?;
        Self::from_toml(&src)
    }

    pub fn definition(&self, tier: Tier) -> Result<&TierDefinition, EntitlementsError> {
        self.tiers
            .get(tier.as_str())
            .ok_or_else(|| EntitlementsError::MissingTier(tier.as_str().to_string()))
    }

    /// Resolves a tier into an immutable `Entitlements` value.
    ///
    /// Resolution order is tier defaults, then env overrides (ENTITLEMENTS.md 5b).
    /// An unknown key or a malformed quota is an error, never a silent grant or deny.
    pub fn resolve(
        &self,
        tier: Tier,
        env: &BTreeMap<String, String>,
    ) -> Result<Entitlements, EntitlementsError> {
        let def = self.definition(tier)?;

        let mut features = BTreeSet::new();
        for raw in &def.features {
            features.insert(parse_feature(raw)?);
        }

        let mut quotas = BTreeMap::new();
        for (key, value) in &def.quotas {
            quotas.insert(parse_quota_key(key)?, parse_limit(key, value)?);
        }

        let mut windows = BTreeMap::new();
        for (key, value) in &def.windows {
            windows.insert(parse_window_key(key)?, parse_limit(key, value)?);
        }

        // Env overrides are additive and explicit: STHENO_QUOTA_MAX_PRODUCTS=250
        for (name, value) in env {
            if let Some(suffix) = name.strip_prefix("STHENO_QUOTA_") {
                let key = suffix.to_ascii_lowercase();
                let parsed =
                    value
                        .parse::<u64>()
                        .map_err(|_| EntitlementsError::MalformedQuota {
                            key: key.clone(),
                            value: value.clone(),
                        })?;
                quotas.insert(parse_quota_key(&key)?, parsed);
            } else if let Some(suffix) = name.strip_prefix("STHENO_WINDOW_") {
                let key = suffix.to_ascii_lowercase();
                let parsed =
                    value
                        .parse::<u64>()
                        .map_err(|_| EntitlementsError::MalformedQuota {
                            key: key.clone(),
                            value: value.clone(),
                        })?;
                windows.insert(parse_window_key(&key)?, parsed);
            }
        }

        Ok(Entitlements {
            tier,
            trial_ends_at: None,
            term_ends_at: None,
            features,
            quotas,
            windows,
        })
    }
}

fn parse_feature(raw: &str) -> Result<Feature, EntitlementsError> {
    match raw {
        "spreadsheet_import" => Ok(Feature::SpreadsheetImport),
        "csv_export" => Ok(Feature::CsvExport),
        "web_push" => Ok(Feature::WebPush),
        "analytics_basic" => Ok(Feature::AnalyticsBasic),
        "payment_gateway" => Ok(Feature::PaymentGateway),
        "courier_dispatch" => Ok(Feature::CourierDispatch),
        "fraud_score" => Ok(Feature::FraudScore),
        "custom_domain" => Ok(Feature::CustomDomain),
        other => Err(EntitlementsError::UnknownFeature(other.to_string())),
    }
}

fn parse_quota_key(raw: &str) -> Result<QuotaKey, EntitlementsError> {
    match raw {
        "max_products" => Ok(QuotaKey::MaxProducts),
        "max_staff_seats" => Ok(QuotaKey::MaxStaffSeats),
        other => Err(EntitlementsError::UnknownQuota(other.to_string())),
    }
}

fn parse_window_key(raw: &str) -> Result<WindowKey, EntitlementsError> {
    match raw {
        "analytics_window_days" => Ok(WindowKey::AnalyticsWindowDays),
        other => Err(EntitlementsError::UnknownWindow(other.to_string())),
    }
}

/// Limits must be non-negative integers. A float, a string, or a negative number is
/// malformed and aborts startup rather than being coerced.
fn parse_limit(key: &str, value: &toml::Value) -> Result<u64, EntitlementsError> {
    match value.as_integer() {
        Some(n) if n >= 0 => Ok(n as u64),
        _ => Err(EntitlementsError::MalformedQuota {
            key: key.to_string(),
            value: value.to_string(),
        }),
    }
}

/// An immutable, resolved statement of what this deployment may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entitlements {
    tier: Tier,
    trial_ends_at: Option<DateTime<Utc>>,
    term_ends_at: Option<DateTime<Utc>>,
    features: BTreeSet<Feature>,
    quotas: BTreeMap<QuotaKey, u64>,
    windows: BTreeMap<WindowKey, u64>,
}

impl Entitlements {
    pub fn tier(&self) -> Tier {
        self.tier
    }

    pub fn trial_ends_at(&self) -> Option<DateTime<Utc>> {
        self.trial_ends_at
    }

    pub fn term_ends_at(&self) -> Option<DateTime<Utc>> {
        self.term_ends_at
    }

    pub fn with_trial_ends_at(mut self, at: Option<DateTime<Utc>>) -> Self {
        self.trial_ends_at = at;
        self
    }

    pub fn with_term_ends_at(mut self, at: Option<DateTime<Utc>>) -> Self {
        self.term_ends_at = at;
        self
    }

    pub fn has(&self, feature: Feature) -> bool {
        self.features.contains(&feature)
    }

    pub fn features(&self) -> impl Iterator<Item = Feature> + '_ {
        self.features.iter().copied()
    }

    /// `None` means unlimited. The `0` sentinel is normalised away here so callers
    /// never have to remember what zero means.
    pub fn quota(&self, key: QuotaKey) -> Option<u64> {
        match self.quotas.get(&key) {
            Some(0) | None => None,
            Some(n) => Some(*n),
        }
    }

    pub fn window(&self, key: WindowKey) -> Option<u64> {
        match self.windows.get(&key) {
            Some(0) | None => None,
            Some(n) => Some(*n),
        }
    }

    pub fn quota_keys(&self) -> impl Iterator<Item = QuotaKey> + '_ {
        self.quotas.keys().copied()
    }

    pub fn window_keys(&self) -> impl Iterator<Item = WindowKey> + '_ {
        self.windows.keys().copied()
    }

    /// Whether one more of `key` may be created given current usage.
    ///
    /// This is the read side only. The authoritative check is the atomic counter
    /// update in `CheckQuotaStep` (ENTITLEMENTS.md 6.2) — read-then-write overruns
    /// under concurrency, so never gate a create on this alone.
    pub fn may_create(&self, key: QuotaKey, used: u64) -> bool {
        match self.quota(key) {
            None => true,
            Some(limit) => used < limit,
        }
    }

    pub fn quota_state(&self, key: QuotaKey, used: u64) -> QuotaState {
        match self.quota(key) {
            Some(limit) if used > limit => QuotaState::Grace,
            _ => QuotaState::Ok,
        }
    }
}
