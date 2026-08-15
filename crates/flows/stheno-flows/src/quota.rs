//! Entitlement quota management, CheckQuotaStep, trial lifecycle, and grandfathering enforcement (ENTITLEMENTS.md §5).

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stheno_types::{Error, Result};
use stheno_workflow::{Step, StepConfig, StepContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementUsage {
    pub tenant_id: String,
    pub feature: String,
    pub current_count: usize,
    pub limit: usize,
}

#[derive(Clone, Default)]
pub struct QuotaStore {
    usages: Arc<Mutex<HashMap<String, EntitlementUsage>>>,
}

impl QuotaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_quota(&self, tenant_id: &str, feature: &str, limit: usize, initial_count: usize) {
        let key = format!("{}::{}", tenant_id, feature);
        let mut map = self.usages.lock().unwrap();
        map.insert(
            key,
            EntitlementUsage {
                tenant_id: tenant_id.to_string(),
                feature: feature.to_string(),
                current_count: initial_count,
                limit,
            },
        );
    }

    /// Atomically increments usage if under limit (P2-21b, P2-21c)
    pub fn try_increment(&self, tenant_id: &str, feature: &str, count: usize) -> Result<usize> {
        let key = format!("{}::{}", tenant_id, feature);
        let mut map = self.usages.lock().unwrap();
        let usage = map.get_mut(&key).ok_or_else(|| Error::NotFound {
            entity: "entitlement_usage",
            id: key.clone(),
        })?;

        if usage.current_count + count > usage.limit {
            return Err(Error::Conflict {
                code: "quota_exceeded",
                message: format!(
                    "Quota limit reached for {}: current {}, limit {}, requested {}",
                    feature, usage.current_count, usage.limit, count
                ),
            });
        }

        usage.current_count += count;
        Ok(usage.current_count)
    }

    pub fn get_count(&self, tenant_id: &str, feature: &str) -> usize {
        let key = format!("{}::{}", tenant_id, feature);
        let map = self.usages.lock().unwrap();
        map.get(&key).map(|u| u.current_count).unwrap_or(0)
    }

    /// Grandfathering test support (P2-21e): sets a new lower limit without mutating current_count
    pub fn downgrade_limit(&self, tenant_id: &str, feature: &str, new_limit: usize) {
        let key = format!("{}::{}", tenant_id, feature);
        let mut map = self.usages.lock().unwrap();
        if let Some(usage) = map.get_mut(&key) {
            usage.limit = new_limit;
        }
    }
}

pub struct CheckQuotaStep {
    quota_store: QuotaStore,
    tenant_id: String,
    feature: String,
}

impl CheckQuotaStep {
    pub fn new(
        quota_store: QuotaStore,
        tenant_id: impl Into<String>,
        feature: impl Into<String>,
    ) -> Self {
        Self {
            quota_store,
            tenant_id: tenant_id.into(),
            feature: feature.into(),
        }
    }
}

#[async_trait]
impl Step for CheckQuotaStep {
    fn name(&self) -> &'static str {
        "CheckQuotaStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 0,
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let requested = input["count"].as_u64().unwrap_or(1) as usize;
        self.quota_store
            .try_increment(&self.tenant_id, &self.feature, requested)?;
        Ok(input)
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    Active,
    WarningDay23,
    WarningDay29,
    ExpiredToFree,
}

pub struct TrialLifecycleJob {
    pub trial_started_at: DateTime<Utc>,
    pub status: TrialStatus,
    pub audit_logs: Vec<String>,
}

impl TrialLifecycleJob {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            trial_started_at: started_at,
            status: TrialStatus::Active,
            audit_logs: Vec::new(),
        }
    }

    pub fn evaluate_lifecycle_at(&mut self, as_of: DateTime<Utc>) -> TrialStatus {
        let age = as_of.signed_duration_since(self.trial_started_at);
        let days = age.num_days();

        if days >= 30 && self.status != TrialStatus::ExpiredToFree {
            self.status = TrialStatus::ExpiredToFree;
            self.audit_logs.push(format!(
                "Day {}: Trial expired, downgraded to Free tier",
                days
            ));
        } else if days >= 29 && self.status != TrialStatus::WarningDay29 {
            self.status = TrialStatus::WarningDay29;
            self.audit_logs
                .push("Day 29: Final trial expiration warning emitted".to_string());
        } else if days >= 23 && self.status != TrialStatus::WarningDay23 {
            self.status = TrialStatus::WarningDay23;
            self.audit_logs
                .push("Day 23: First trial expiration warning emitted".to_string());
        }

        self.status.clone()
    }
}

pub struct AnalyticsWindowGate;

impl AnalyticsWindowGate {
    /// Clamps requested time window to max allowed window days for tier (P2-21f)
    pub fn clamp_analytics_window(
        requested_start: DateTime<Utc>,
        max_window_days: u32,
    ) -> DateTime<Utc> {
        Self::clamp_analytics_window_at(requested_start, max_window_days, Utc::now())
    }

    pub fn clamp_analytics_window_at(
        requested_start: DateTime<Utc>,
        max_window_days: u32,
        reference_now: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let earliest_allowed = reference_now - Duration::days(max_window_days as i64);

        if requested_start < earliest_allowed {
            earliest_allowed
        } else {
            requested_start
        }
    }
}
