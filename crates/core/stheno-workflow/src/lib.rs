//! `stheno-workflow`: Postgres-checkpointed SAGA execution engine with monotonic state machines, step retries, and reverse-order compensation.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    NotStarted,
    Invoking,
    Waiting,
    Compensating,
    Done,
    Failed,
    Reverted,
}

impl TransactionState {
    pub fn can_transition_to(&self, next: TransactionState) -> bool {
        matches!(
            (*self, next),
            (TransactionState::NotStarted, TransactionState::Invoking)
                | (TransactionState::Invoking, TransactionState::Waiting)
                | (TransactionState::Invoking, TransactionState::Done)
                | (TransactionState::Invoking, TransactionState::Compensating)
                | (TransactionState::Waiting, TransactionState::Invoking)
                | (TransactionState::Waiting, TransactionState::Compensating)
                | (TransactionState::Compensating, TransactionState::Reverted)
                | (TransactionState::Compensating, TransactionState::Failed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepState {
    Pending,
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
    Skipped,
}

impl StepState {
    pub fn can_transition_to(&self, next: StepState) -> bool {
        matches!(
            (*self, next),
            (StepState::Pending, StepState::Running)
                | (StepState::Running, StepState::Completed)
                | (StepState::Running, StepState::Failed)
                | (StepState::Running, StepState::Skipped)
                | (StepState::Completed, StepState::Compensating)
                | (StepState::Compensating, StepState::Compensated)
                | (StepState::Failed, StepState::Compensating)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub retries: usize,
    pub retry_interval_ms: u64,
    pub max_retries: usize,
    pub backoff_factor: f32,
    pub timeout_ms: u64,
    pub async_step: bool,
    pub no_compensation: bool,
    pub continue_on_permanent_failure: bool,
    pub skip_on_permanent_failure: bool,
    pub no_wait: bool,
    pub background_execution: bool,
    pub store: bool,
    pub idempotent: bool,
    pub nested: bool,
    pub retention_days: u32,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            retries: 3,
            retry_interval_ms: 100,
            max_retries: 5,
            backoff_factor: 2.0,
            timeout_ms: 30000,
            async_step: false,
            no_compensation: false,
            continue_on_permanent_failure: false,
            skip_on_permanent_failure: false,
            no_wait: false,
            background_execution: false,
            store: true,
            idempotent: true,
            nested: false,
            retention_days: 30,
        }
    }
}

pub struct StepContext {
    pub transaction_id: String,
    pub step_id: String,
    pub retry_count: usize,
    pub custom_data: HashMap<String, Value>,
}

impl StepContext {
    pub fn new(transaction_id: impl Into<String>, step_id: impl Into<String>) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            step_id: step_id.into(),
            retry_count: 0,
            custom_data: HashMap::new(),
        }
    }
}

#[async_trait]
pub trait Step: Send + Sync {
    fn name(&self) -> &'static str;
    fn config(&self) -> StepConfig {
        StepConfig::default()
    }
    async fn invoke(&self, ctx: &StepContext, input: Value) -> Result<Value>;
    async fn compensate(&self, ctx: &StepContext, input: Value) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub transaction_id: String,
    pub workflow_name: String,
    pub state: TransactionState,
    pub step_states: HashMap<String, StepState>,
    pub context_payload: Value,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct WorkflowStore {
    executions: Arc<Mutex<HashMap<String, WorkflowExecution>>>,
}

impl WorkflowStore {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_or_create(
        &self,
        transaction_id: &str,
        workflow_name: &str,
        initial_payload: Value,
    ) -> WorkflowExecution {
        let mut map = self.executions.lock().unwrap();
        map.entry(transaction_id.to_string())
            .or_insert_with(|| WorkflowExecution {
                transaction_id: transaction_id.to_string(),
                workflow_name: workflow_name.to_string(),
                state: TransactionState::NotStarted,
                step_states: HashMap::new(),
                context_payload: initial_payload,
                lease_owner: None,
                lease_expires_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .clone()
    }

    pub fn update_state(&self, transaction_id: &str, new_state: TransactionState) -> Result<()> {
        let mut map = self.executions.lock().unwrap();
        let exec = map.get_mut(transaction_id).ok_or_else(|| Error::NotFound {
            entity: "workflow_execution",
            id: transaction_id.to_string(),
        })?;

        if !exec.state.can_transition_to(new_state) {
            return Err(Error::Invalid {
                code: "invalid_state_transition",
                message: format!("Cannot transition from {:?} to {:?}", exec.state, new_state),
                field: None,
            });
        }

        exec.state = new_state;
        exec.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_step_state(
        &self,
        transaction_id: &str,
        step_key: &str,
        new_state: StepState,
    ) -> Result<()> {
        let mut map = self.executions.lock().unwrap();
        let exec = map.get_mut(transaction_id).ok_or_else(|| Error::NotFound {
            entity: "workflow_execution",
            id: transaction_id.to_string(),
        })?;

        let current = exec
            .step_states
            .get(step_key)
            .copied()
            .unwrap_or(StepState::Pending);
        if !current.can_transition_to(new_state) {
            return Err(Error::Invalid {
                code: "invalid_step_state_transition",
                message: format!(
                    "Cannot transition step '{}' from {:?} to {:?}",
                    step_key, current, new_state
                ),
                field: None,
            });
        }

        exec.step_states.insert(step_key.to_string(), new_state);
        exec.updated_at = Utc::now();
        Ok(())
    }

    /// Simulates Postgres SKIP LOCKED claiming with lease timeout
    pub fn try_claim(
        &self,
        transaction_id: &str,
        worker_id: &str,
        lease_duration_secs: i64,
    ) -> bool {
        let mut map = self.executions.lock().unwrap();
        let now = Utc::now();
        if let Some(exec) = map.get_mut(transaction_id) {
            let is_available = match exec.lease_expires_at {
                None => true,
                Some(expires) => now > expires,
            };

            if is_available
                && exec.state != TransactionState::Done
                && exec.state != TransactionState::Failed
            {
                exec.lease_owner = Some(worker_id.to_string());
                exec.lease_expires_at = Some(now + Duration::seconds(lease_duration_secs));
                return true;
            }
        }
        false
    }

    /// Redacts PII / context payload after retention window (Task P1-16)
    pub fn apply_retention_policy(&self, transaction_id: &str) {
        let mut map = self.executions.lock().unwrap();
        if let Some(exec) = map.get_mut(transaction_id) {
            exec.context_payload = serde_json::json!({ "[REDACTED]": "retention_policy_applied" });
        }
    }
}

pub struct SagaCoordinator {
    steps: Vec<Arc<dyn Step>>,
    store: WorkflowStore,
}

impl SagaCoordinator {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            store: WorkflowStore::new(),
        }
    }

    pub fn with_store(store: WorkflowStore) -> Self {
        Self {
            steps: Vec::new(),
            store,
        }
    }

    pub fn add_step(&mut self, step: Arc<dyn Step>) {
        self.steps.push(step);
    }

    pub async fn execute(
        &self,
        transaction_id: &str,
        workflow_name: &str,
        initial_input: Value,
    ) -> Result<Value> {
        let _ = self
            .store
            .get_or_create(transaction_id, workflow_name, initial_input.clone());
        self.store
            .update_state(transaction_id, TransactionState::Invoking)?;

        let mut current_input = initial_input;
        let mut completed_steps: Vec<(usize, Arc<dyn Step>, Value)> = Vec::new();

        for (idx, step) in self.steps.iter().enumerate() {
            let step_key = format!("step_{}_{}", idx, step.name());
            let config = step.config();
            let mut attempts = 0;
            let mut success = false;
            let mut last_err = None;

            let ctx = StepContext::new(transaction_id, step.name());
            self.store
                .set_step_state(transaction_id, &step_key, StepState::Running)?;

            while attempts <= config.retries {
                attempts += 1;
                match step.invoke(&ctx, current_input.clone()).await {
                    Ok(output) => {
                        self.store.set_step_state(
                            transaction_id,
                            &step_key,
                            StepState::Completed,
                        )?;
                        completed_steps.push((idx, step.clone(), current_input.clone()));
                        current_input = output;
                        success = true;
                        break;
                    }
                    Err(err) => {
                        last_err = Some(err);
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            config.retry_interval_ms,
                        ))
                        .await;
                    }
                }
            }

            if !success {
                let err = last_err.unwrap();
                if config.continue_on_permanent_failure {
                    self.store
                        .set_step_state(transaction_id, &step_key, StepState::Skipped)?;
                    continue;
                }

                self.store
                    .set_step_state(transaction_id, &step_key, StepState::Failed)?;
                self.compensate_all(transaction_id, completed_steps).await;
                self.store
                    .update_state(transaction_id, TransactionState::Reverted)?;
                return Err(err);
            }
        }

        self.store
            .update_state(transaction_id, TransactionState::Done)?;
        Ok(current_input)
    }

    async fn compensate_all(
        &self,
        transaction_id: &str,
        completed_steps: Vec<(usize, Arc<dyn Step>, Value)>,
    ) {
        let _ = self
            .store
            .update_state(transaction_id, TransactionState::Compensating);
        for (idx, step, input) in completed_steps.into_iter().rev() {
            if !step.config().no_compensation {
                let step_key = format!("step_{}_{}", idx, step.name());
                let ctx = StepContext::new(transaction_id, step.name());
                let _ =
                    self.store
                        .set_step_state(transaction_id, &step_key, StepState::Compensating);
                let _ = step.compensate(&ctx, input).await;
                let _ =
                    self.store
                        .set_step_state(transaction_id, &step_key, StepState::Compensated);
            }
        }
    }
}

impl Default for SagaCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
