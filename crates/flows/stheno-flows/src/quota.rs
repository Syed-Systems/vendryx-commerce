//! Entitlement quota management, CheckQuotaStep, trial lifecycle, and grandfathering enforcement (ENTITLEMENTS.md §5, §6).

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stheno_db::QuotaCounter;
use stheno_types::{Error, Result};
use stheno_workflow::{Step, StepConfig, StepContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementUsage {
    pub tenant_id: String,
    pub feature: String,
    pub current_count: usize,
    pub limit: usize,
}

#[derive(Clone)]
pub struct CheckQuotaStep {
    counter: QuotaCounter,
    resource: String,
    limit: Option<i64>,
}

impl CheckQuotaStep {
    pub fn new(counter: QuotaCounter, resource: impl Into<String>, limit: Option<i64>) -> Self {
        Self {
            counter,
            resource: resource.into(),
            limit,
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
        // Scoped checkout (ADR-012): `entitlement_usage` is keyed by
        // `(tenant_id, resource)`, so an unscoped connection would take its tenant from
        // an unset setting and violate NOT NULL rather than counting anything.
        let counter = self.counter.clone();
        let resource = self.resource.clone();
        let limit = self.limit;
        let pool = self.counter.pool().clone();

        let outcome = stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move { counter.increment(conn, &resource, limit).await })
        })
        .await;

        match outcome {
            Ok(used) => {
                let mut out = input;
                if let Value::Object(ref mut map) = out {
                    map.insert("quota_used".to_string(), serde_json::json!(used));
                }
                Ok(out)
            }
            Err(Error::Conflict { .. }) => {
                let resource_title = match self.resource.as_str() {
                    "products" => "Product",
                    "users" => "Staff seat",
                    "regions" => "Region",
                    _ => &self.resource,
                };
                let limit_display = self.limit.unwrap_or(0);
                Err(Error::Conflict {
                    code: "not_allowed",
                    message: format!(
                        "{} limit reached ({}). Upgrade to Pro for unlimited {}.",
                        resource_title, limit_display, self.resource
                    ),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        // Compensation must be scoped exactly as the forward step was, or it decrements
        // a different tenant's counter than the one it incremented.
        let counter = self.counter.clone();
        let resource = self.resource.clone();
        let pool = self.counter.pool().clone();

        stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                counter.decrement(conn, &resource).await?;
                Ok(())
            })
        })
        .await
    }
}

pub struct QuotaReconciliationJob {
    counter: QuotaCounter,
}

impl QuotaReconciliationJob {
    pub fn new(counter: QuotaCounter) -> Self {
        Self { counter }
    }

    /// Reconcile actual counted rows against entitlement_usage table and alert if drifted
    /// Reconciles one tenant's counter. The caller establishes which tenant by running
    /// this inside a scope — counters are per tenant, so a nightly job iterates tenants
    /// rather than reconciling a single global row.
    pub async fn reconcile_resource(&self, resource: &str, actual_count: i64) -> Result<bool> {
        let counter = self.counter.clone();
        let resource = resource.to_string();
        let pool = self.counter.pool().clone();

        stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                let recorded = counter.get_usage(conn, &resource).await?;
                if recorded == actual_count {
                    return Ok(false);
                }
                tracing::warn!(
                    "Quota drift detected for {}: recorded={}, actual={}. Correcting...",
                    resource,
                    recorded,
                    actual_count
                );
                counter.set_usage(conn, &resource, actual_count).await?;
                Ok(true)
            })
        })
        .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    Active,
    WarningDay7,
    WarningDay13,
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

        if self.status == TrialStatus::ExpiredToFree {
            return self.status.clone();
        }

        if days >= 14 {
            self.status = TrialStatus::ExpiredToFree;
            self.audit_logs.push(format!(
                "Day {}: Trial expired, downgraded to Free tier",
                days
            ));
        } else if days >= 13 && self.status != TrialStatus::WarningDay13 {
            self.status = TrialStatus::WarningDay13;
            self.audit_logs
                .push("Day 13: Final trial expiration warning emitted".to_string());
        } else if days >= 7 && self.status != TrialStatus::WarningDay7 {
            self.status = TrialStatus::WarningDay7;
            self.audit_logs
                .push("Day 7: First trial expiration warning emitted".to_string());
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
