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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_types::currency::CurrencyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Trial,
    Free,
    Pro,
}

impl Tier {
    /// Every plan there is. The control plane renders one editor per entry, so a new
    /// tier appears in the UI by adding it here rather than by editing a list in
    /// TypeScript that would drift out of step with the enum.
    pub const ALL: [Tier; 3] = [Tier::Trial, Tier::Free, Tier::Pro];

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
    CustomTheme,
    Branding,
}

impl Feature {
    /// Every gateable feature. The control plane needs the whole universe, not just the
    /// ones a tier happens to hold, or a feature could never be switched *on* from the UI.
    pub const ALL: [Feature; 10] = [
        Feature::SpreadsheetImport,
        Feature::CsvExport,
        Feature::WebPush,
        Feature::AnalyticsBasic,
        Feature::PaymentGateway,
        Feature::CourierDispatch,
        Feature::FraudScore,
        Feature::CustomDomain,
        Feature::CustomTheme,
        Feature::Branding,
    ];

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
            Feature::CustomTheme => "custom_theme",
            Feature::Branding => "branding",
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
    pub const ALL: [QuotaKey; 2] = [QuotaKey::MaxProducts, QuotaKey::MaxStaffSeats];

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
    pub const ALL: [WindowKey; 1] = [WindowKey::AnalyticsWindowDays];

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
    /// Price as an exact decimal **string** — see [`TierDefinition::pricing`] for why
    /// it is not a TOML float. `None` means the plan is not sold, which is not the
    /// same as free.
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub interval: Option<String>,
}

impl TierDefinition {
    /// Builds a definition from already-validated keys.
    ///
    /// Takes the enums rather than strings on purpose: by the time a definition exists,
    /// an unknown feature or quota key must already have been rejected at the API
    /// boundary. This constructor makes that a type-level guarantee instead of a comment.
    pub fn from_parts(
        features: impl IntoIterator<Item = Feature>,
        quotas: impl IntoIterator<Item = (QuotaKey, u64)>,
        windows: impl IntoIterator<Item = (WindowKey, u64)>,
    ) -> Self {
        Self {
            duration_days: None,
            starts_on: None,
            features: features
                .into_iter()
                .map(|f| f.as_str().to_string())
                .collect(),
            quotas: quotas
                .into_iter()
                .map(|(k, v)| (k.as_str().to_string(), toml::Value::Integer(v as i64)))
                .collect(),
            windows: windows
                .into_iter()
                .map(|(k, v)| (k.as_str().to_string(), toml::Value::Integer(v as i64)))
                .collect(),
            term_months: Vec::new(),
            price: None,
            currency: None,
            interval: None,
        }
    }

    /// Keeps the trial's own mechanics (duration, start trigger) and the Pro commitment
    /// terms, which the control plane does not edit and must not silently drop.
    pub fn carrying_terms_of(mut self, base: &TierDefinition) -> Self {
        self.duration_days = base.duration_days;
        self.starts_on = base.starts_on.clone();
        self.term_months = base.term_months.clone();
        // Pricing is carried too. A control-plane edit rewrites features and limits;
        // silently dropping the price would take the plan off sale as a side effect
        // of ticking a checkbox.
        self.price = base.price.clone();
        self.currency = base.currency.clone();
        self.interval = base.interval.clone();
        self
    }
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

    /// Replaces one tier's definition. Used to lay a stored `platform.tier_definition`
    /// row over the shipped TOML defaults; wholesale replacement, never a merge
    /// (see the migration for why a merge is unsafe).
    pub fn set(&mut self, tier: Tier, definition: TierDefinition) {
        self.tiers.insert(tier.as_str().to_string(), definition);
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

impl std::str::FromStr for Feature {
    type Err = EntitlementsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_feature(s)
    }
}

impl std::str::FromStr for QuotaKey {
    type Err = EntitlementsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_quota_key(s)
    }
}

impl std::str::FromStr for WindowKey {
    type Err = EntitlementsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_window_key(s)
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
        "custom_theme" => Ok(Feature::CustomTheme),
        "branding" => Ok(Feature::Branding),
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

    /// Grants or revokes one feature on an already-resolved value.
    ///
    /// This is how a per-tenant override lands (ENTITLEMENTS.md 5b, persisted). It is
    /// applied during resolution, before the router reads the value, so §6.1 still holds:
    /// the gate remains a mount-point decision, not a branch in domain code.
    pub fn set_feature(&mut self, feature: Feature, enabled: bool) {
        if enabled {
            self.features.insert(feature);
        } else {
            self.features.remove(&feature);
        }
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

/// How often a plan is billed.
///
/// Two values, because two are what `config/tiers.toml` can express and a third would be
/// a plan nobody sells. `term_months` on Pro is a *commitment length*, not a billing
/// interval — a merchant on a 12-month term is still invoiced monthly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingInterval {
    Month,
    Year,
}

impl BillingInterval {
    pub const ALL: [BillingInterval; 2] = [BillingInterval::Month, BillingInterval::Year];

    pub fn as_str(&self) -> &'static str {
        match self {
            BillingInterval::Month => "month",
            BillingInterval::Year => "year",
        }
    }

    /// How many months one interval covers. Used to place the next period boundary.
    pub fn months(&self) -> u32 {
        match self {
            BillingInterval::Month => 1,
            BillingInterval::Year => 12,
        }
    }
}

impl std::str::FromStr for BillingInterval {
    type Err = EntitlementsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "month" => Ok(BillingInterval::Month),
            "year" => Ok(BillingInterval::Year),
            other => Err(EntitlementsError::Parse(format!(
                "unknown billing interval `{other}`; expected month or year"
            ))),
        }
    }
}

/// What a plan costs, when it is sold at all.
///
/// A plan with no price is **not free** — it is *not sold*. Free is a price of zero, and
/// the difference matters: an invoice run must skip a plan that is not sold rather than
/// invoice it for nothing, and a plan that is meant to cost money but whose price went
/// missing must not silently become free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierPricing {
    pub price: Decimal,
    pub currency: CurrencyCode,
    pub interval: BillingInterval,
}

impl TierDefinition {
    /// The plan's price, or `None` when the plan is not sold.
    ///
    /// # Why the TOML value is a string
    ///
    /// TOML has no decimal type. `price = 1500.00` parses as an **f64**, which
    /// [ADR-006](../../../docs/DECISIONS.md#adr-006) forbids anywhere near money — and it
    /// would be forbidden here silently, because 1500.00 survives a float round trip and
    /// 1499.95 does not. Writing it as `price = "1500.00"` keeps the value exact from the
    /// file to the invoice, and makes the constraint visible in the config rather than
    /// buried in a deserializer.
    pub fn pricing(&self) -> Result<Option<TierPricing>, EntitlementsError> {
        let Some(raw) = self.price.as_deref() else {
            // No price and no currency is a plan that is not sold. A currency without a
            // price is a half-written plan, and that is worth refusing loudly.
            if self.currency.is_some() || self.interval.is_some() {
                return Err(EntitlementsError::Parse(
                    "a plan with a currency or interval must also have a price".to_string(),
                ));
            }
            return Ok(None);
        };

        let price = Decimal::from_str_exact(raw.trim()).map_err(|e| {
            EntitlementsError::Parse(format!("price `{raw}` is not an exact decimal: {e}"))
        })?;

        if price.is_sign_negative() {
            return Err(EntitlementsError::Parse(format!(
                "price `{raw}` is negative; a plan cannot pay the merchant"
            )));
        }

        let currency = self
            .currency
            .as_deref()
            .ok_or_else(|| EntitlementsError::Parse("a priced plan needs a currency".to_string()))?
            .parse::<CurrencyCode>()
            .map_err(|e| EntitlementsError::Parse(e.to_string()))?;

        let interval = self
            .interval
            .as_deref()
            .ok_or_else(|| EntitlementsError::Parse("a priced plan needs an interval".to_string()))?
            .parse::<BillingInterval>()?;

        Ok(Some(TierPricing {
            price,
            currency,
            interval,
        }))
    }
}

impl TierConfig {
    /// What this tier costs on this deployment, or `None` when it is not sold.
    pub fn pricing(&self, tier: Tier) -> Result<Option<TierPricing>, EntitlementsError> {
        self.definition(tier)?.pricing()
    }
}
