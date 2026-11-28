//! Entitlement quota management, CheckQuotaStep, trial lifecycle, and grandfathering enforcement (ENTITLEMENTS.md §5, §6).

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use stheno_db::QuotaCounter;
use stheno_types::{Error, Result};
use stheno_user::{AcceptInviteInput, UserService};
use stheno_workflow::{SagaCoordinator, Step, StepConfig, StepContext};

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

struct AcceptInviteStep {
    users: Arc<UserService>,
}

#[async_trait]
impl Step for AcceptInviteStep {
    fn name(&self) -> &'static str {
        "AcceptInviteStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 0,
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let token = input["token"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_invite_token",
            message: "Missing invite token in workflow input".to_string(),
            field: Some("token".to_string()),
        })?;
        let user = self
            .users
            .accept_invite(AcceptInviteInput {
                token: token.to_string(),
                first_name: input["first_name"].as_str().map(str::to_string),
                last_name: input["last_name"].as_str().map(str::to_string),
            })
            .await?;
        Ok(serde_json::json!({
            "user_id": user.id,
            "email": user.email,
            "role": user.role,
        }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

/// Reserves one staff seat before accepting an invite through the user service.
pub fn build_accept_invite_quota_saga(
    users: Arc<UserService>,
    counter: QuotaCounter,
    limit: Option<i64>,
) -> SagaCoordinator {
    let mut saga = SagaCoordinator::new();
    saga.add_step(Arc::new(CheckQuotaStep::new(counter, "users", limit)));
    saga.add_step(Arc::new(AcceptInviteStep { users }));
    saga
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
    WarningFirst,
    WarningFinal,
    ExpiredToFree,
}

pub struct TrialLifecycleJob {
    pub trial_started_at: DateTime<Utc>,
    pub trial_ends_at: DateTime<Utc>,
    pub status: TrialStatus,
    pub audit_logs: Vec<String>,
}

impl TrialLifecycleJob {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            trial_started_at: started_at,
            trial_ends_at: started_at + Duration::days(7),
            status: TrialStatus::Active,
            audit_logs: Vec::new(),
        }
    }

    /// Builds lifecycle state from the persisted expiry so legacy trials keep their term.
    pub fn with_trial_ends_at(
        started_at: DateTime<Utc>,
        trial_ends_at: DateTime<Utc>,
    ) -> Result<Self> {
        if trial_ends_at <= started_at {
            return Err(Error::Invalid {
                code: "trial_term_invalid",
                message: "Trial end must be later than trial start.".to_string(),
                field: Some("trial_ends_at".to_string()),
            });
        }
        Ok(Self {
            trial_started_at: started_at,
            trial_ends_at,
            status: TrialStatus::Active,
            audit_logs: Vec::new(),
        })
    }

    pub fn evaluate_lifecycle_at(&mut self, as_of: DateTime<Utc>) -> TrialStatus {
        if as_of < self.trial_started_at {
            return self.status.clone();
        }

        let duration_days = self
            .trial_ends_at
            .signed_duration_since(self.trial_started_at)
            .num_days();
        let first_warning_at = self.trial_started_at + Duration::days(duration_days / 2);
        let final_warning_at = self.trial_ends_at - Duration::days(1);
        let next_status = if as_of >= self.trial_ends_at {
            TrialStatus::ExpiredToFree
        } else if as_of >= final_warning_at {
            TrialStatus::WarningFinal
        } else if as_of >= first_warning_at {
            TrialStatus::WarningFirst
        } else {
            TrialStatus::Active
        };
        if trial_status_rank(&next_status) <= trial_status_rank(&self.status) {
            return self.status.clone();
        }

        let days = as_of
            .signed_duration_since(self.trial_started_at)
            .num_days();
        match next_status {
            TrialStatus::WarningFirst => self.audit_logs.push(format!(
                "Day {days}: First trial expiration warning emitted"
            )),
            TrialStatus::WarningFinal => self.audit_logs.push(format!(
                "Day {days}: Final trial expiration warning emitted"
            )),
            TrialStatus::ExpiredToFree => self.audit_logs.push(format!(
                "Day {days}: Trial expired, downgraded to Free tier"
            )),
            TrialStatus::Active => {}
        }
        self.status = next_status;
        self.status.clone()
    }
}

fn trial_status_rank(status: &TrialStatus) -> u8 {
    match status {
        TrialStatus::Active => 0,
        TrialStatus::WarningFirst => 1,
        TrialStatus::WarningFinal => 2,
        TrialStatus::ExpiredToFree => 3,
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
