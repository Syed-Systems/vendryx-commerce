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
    /// The name the `state` column stores. Matches the serde representation, which is the
    /// bare variant name, so a row written by one and read by the other agrees.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionState::NotStarted => "NotStarted",
            TransactionState::Invoking => "Invoking",
            TransactionState::Waiting => "Waiting",
            TransactionState::Compensating => "Compensating",
            TransactionState::Done => "Done",
            TransactionState::Failed => "Failed",
            TransactionState::Reverted => "Reverted",
        }
    }

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
    pub fn as_str(&self) -> &'static str {
        match self {
            StepState::Pending => "Pending",
            StepState::Running => "Running",
            StepState::Completed => "Completed",
            StepState::Failed => "Failed",
            StepState::Compensating => "Compensating",
            StepState::Compensated => "Compensated",
            StepState::Skipped => "Skipped",
        }
    }

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

use diesel::sql_types::Text;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use stheno_db::DbPool;

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    #[serde(alias = "id")]
    pub transaction_id: String,
    pub workflow_name: String,
    pub state: TransactionState,
    pub step_states: HashMap<String, StepState>,
    pub context_payload: Value,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
pub struct WorkflowStore {
    pool: Option<DbPool>,
    executions: Arc<Mutex<HashMap<String, WorkflowExecution>>>,
}

/// Reads one execution on a caller-supplied connection.
///
/// A free function rather than a method so the transactional bodies below can read on the
/// connection they already hold. Calling `get_execution_async` from inside one of them
/// would take a SECOND connection from the same pool while the first is still in a
/// transaction — N concurrent workflows against a pool of N then deadlock, each holding
/// one connection and waiting for another only a peer can release.
async fn read_execution(
    conn: &mut diesel_async::AsyncPgConnection,
    transaction_id: &str,
) -> Result<Option<WorkflowExecution>> {
    let rows: Vec<JsonRow> = diesel::sql_query(
        "SELECT row_to_json(e)::text AS value FROM workflow.execution e \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind::<diesel::sql_types::Text, _>(transaction_id)
    .load(conn)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;

    match rows.into_iter().next() {
        Some(row) => serde_json::from_str(&row.value)
            .map(Some)
            .map_err(|e| Error::Database(e.to_string())),
        None => Ok(None),
    }
}

impl WorkflowStore {
    pub fn new() -> Self {
        Self {
            pool: None,
            executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_pool(pool: DbPool) -> Self {
        Self {
            pool: Some(pool),
            executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn in_memory() -> Self {
        Self::new()
    }

    pub async fn get_or_create_async(
        &self,
        transaction_id: &str,
        workflow_name: &str,
        initial_payload: Value,
    ) -> Result<WorkflowExecution> {
        let Some(pool) = &self.pool else {
            return Ok(self.get_or_create(transaction_id, workflow_name, initial_payload));
        };

        let transaction_id = transaction_id.to_string();
        let workflow_name = workflow_name.to_string();

        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                if let Some(exec) = read_execution(conn, &transaction_id).await? {
                    return Ok(exec);
                }

                let now = Utc::now();
                let exec = WorkflowExecution {
                    transaction_id: transaction_id.clone(),
                    workflow_name,
                    state: TransactionState::NotStarted,
                    step_states: HashMap::new(),
                    context_payload: initial_payload,
                    lease_owner: None,
                    lease_expires_at: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };
                let json =
                    serde_json::to_string(&exec).map_err(|e| Error::Database(e.to_string()))?;
                // tenant_id is deliberately absent from the column list: it carries a
                // DEFAULT of current_setting('app.tenant_id'), which the scope established
                // by with_tenant_conn is what populates.
                let insert_sql =
                    "INSERT INTO workflow.execution AS e (id, workflow_name, state, step_states, context_payload, created_at, updated_at) \
                     SELECT COALESCE(x.id, ($1::json->>'transaction_id')), x.workflow_name, x.state, x.step_states, x.context_payload, x.created_at, x.updated_at \
                     FROM json_populate_record(NULL::workflow.execution, $1::json) AS x \
                     ON CONFLICT (id) DO UPDATE SET updated_at = e.updated_at \
                     RETURNING row_to_json(e)::text AS value";
                let inserted: Vec<JsonRow> = diesel::sql_query(insert_sql)
                    .bind::<diesel::sql_types::Text, _>(&json)
                    .load(&mut *conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                match inserted.into_iter().next() {
                    Some(row) => serde_json::from_str(&row.value)
                        .map_err(|e| Error::Database(e.to_string())),
                    None => Ok(exec),
                }
            })
        })
        .await
    }

    pub fn get_or_create(
        &self,
        transaction_id: &str,
        workflow_name: &str,
        initial_payload: Value,
    ) -> WorkflowExecution {
        let mut map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
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
                deleted_at: None,
            })
            .clone()
    }

    pub async fn list_executions_async(&self) -> Result<Vec<WorkflowExecution>> {
        let Some(pool) = &self.pool else {
            return Ok(self.list_executions());
        };

        // No tenant predicate in the SQL: RLS supplies it. Adding one by hand would be a
        // second, weaker copy of the rule that could drift out of step with the policy.
        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                let rows: Vec<JsonRow> = diesel::sql_query(
                    "SELECT row_to_json(e)::text AS value FROM workflow.execution e \
                     WHERE deleted_at IS NULL ORDER BY created_at DESC",
                )
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                rows.into_iter()
                    .map(|row| {
                        serde_json::from_str::<WorkflowExecution>(&row.value)
                            .map_err(|e| Error::Database(e.to_string()))
                    })
                    .collect()
            })
        })
        .await
    }

    /// All known executions, newest first. Read-only; used by admin observability.
    pub fn list_executions(&self) -> Vec<WorkflowExecution> {
        let map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
        let mut all: Vec<WorkflowExecution> = map.values().cloned().collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.created_at));
        all
    }

    pub async fn get_execution_async(
        &self,
        transaction_id: &str,
    ) -> Result<Option<WorkflowExecution>> {
        let Some(pool) = &self.pool else {
            return Ok(self.get_execution(transaction_id));
        };

        let transaction_id = transaction_id.to_string();
        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move { read_execution(conn, &transaction_id).await })
        })
        .await
    }

    pub fn get_execution(&self, transaction_id: &str) -> Option<WorkflowExecution> {
        self.executions.lock().unwrap_or_else(|e| e.into_inner()).get(transaction_id).cloned()
    }

    pub async fn update_state_async(
        &self,
        transaction_id: &str,
        new_state: TransactionState,
    ) -> Result<()> {
        let Some(pool) = &self.pool else {
            return self.update_state(transaction_id, new_state);
        };

        let transaction_id = transaction_id.to_string();
        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                // Read and write on one connection inside one transaction: the guard and
                // the write it guards are now atomic, where two checkouts left a window in
                // which a peer could move the state between them.
                let current = read_execution(conn, &transaction_id)
                    .await?
                    .ok_or_else(|| Error::NotFound {
                        entity: "workflow_execution",
                        id: transaction_id.clone(),
                    })?;

                if !current.state.can_transition_to(new_state) {
                    return Err(Error::Invalid {
                        code: "invalid_state_transition",
                        message: format!(
                            "Cannot transition from {:?} to {:?}",
                            current.state, new_state
                        ),
                        field: None,
                    });
                }

                diesel::sql_query(
                    "UPDATE workflow.execution SET state = $1, updated_at = now() \
                     WHERE id = $2 AND deleted_at IS NULL",
                )
                .bind::<diesel::sql_types::Text, _>(new_state.as_str())
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())
            })
        })
        .await
    }

    /// Replaces an execution's `context_payload`.
    ///
    /// This is the only way to checkpoint a *value*. `update_state_async` moves the
    /// machine between states and `set_step_state_async` records which named step is
    /// running, but a caller that needs to remember "how far did I get" — a resume
    /// offset, an id created partway through — had nowhere to put it. `CompleteCartWorkflowEngine`
    /// kept exactly such a map in memory because of that gap.
    ///
    /// The whole payload is replaced rather than merged. A partial update would need a
    /// read-modify-write, and the read would have to happen on the caller's connection or
    /// it takes a second one from the pool mid-transaction (see `read_execution`).
    pub async fn update_payload_async(
        &self,
        transaction_id: &str,
        payload: Value,
    ) -> Result<()> {
        let Some(pool) = &self.pool else {
            let mut state = self
                .executions
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(exec) = state.get_mut(transaction_id) {
                exec.context_payload = payload;
                exec.updated_at = chrono::Utc::now();
            }
            return Ok(());
        };

        let transaction_id = transaction_id.to_string();
        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "UPDATE workflow.execution SET context_payload = $1, updated_at = now() \
                     WHERE id = $2 AND deleted_at IS NULL",
                )
                .bind::<diesel::sql_types::Json, _>(&payload)
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())
            })
        })
        .await
    }

    pub fn update_state(&self, transaction_id: &str, new_state: TransactionState) -> Result<()> {
        let mut map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
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

    pub async fn set_step_state_async(
        &self,
        transaction_id: &str,
        step_key: &str,
        new_state: StepState,
    ) -> Result<()> {
        let Some(pool) = &self.pool else {
            return self.set_step_state(transaction_id, step_key, new_state);
        };

        let transaction_id = transaction_id.to_string();
        let step_key = step_key.to_string();

        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                let current = read_execution(conn, &transaction_id)
                    .await?
                    .ok_or_else(|| Error::NotFound {
                        entity: "workflow_execution",
                        id: transaction_id.clone(),
                    })?;

                let current_step = current
                    .step_states
                    .get(&step_key)
                    .copied()
                    .unwrap_or(StepState::Pending);

                if !current_step.can_transition_to(new_state) {
                    return Err(Error::Invalid {
                        code: "invalid_step_state_transition",
                        message: format!(
                            "Cannot transition step '{step_key}' from \
                            {current_step:?} to {new_state:?}"
                        ),
                        field: None,
                    });
                }

                let state_str = new_state.as_str();
                let step_id = format!("{transaction_id}_{step_key}");
                let step_name = step_key.split('_').nth(2).unwrap_or(&step_key).to_string();

                diesel::sql_query(
                    "UPDATE workflow.execution \
                     SET step_states = jsonb_set(step_states, ARRAY[$1], to_jsonb($2::text)), \
                         updated_at = now() \
                     WHERE id = $3 AND deleted_at IS NULL",
                )
                .bind::<diesel::sql_types::Text, _>(&step_key)
                .bind::<diesel::sql_types::Text, _>(state_str)
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                // The step_state row is the per-step audit trail; the jsonb column above is
                // what the coordinator reads. Both are written in the one transaction, so a
                // failure cannot leave them disagreeing.
                diesel::sql_query(
                    "INSERT INTO workflow.step_state \
                         (id, transaction_id, step_key, step_name, state, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, now()) \
                     ON CONFLICT (transaction_id, step_key) \
                     DO UPDATE SET state = $5, updated_at = now()",
                )
                .bind::<diesel::sql_types::Text, _>(&step_id)
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .bind::<diesel::sql_types::Text, _>(&step_key)
                .bind::<diesel::sql_types::Text, _>(&step_name)
                .bind::<diesel::sql_types::Text, _>(state_str)
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())
            })
        })
        .await
    }

    pub fn set_step_state(
        &self,
        transaction_id: &str,
        step_key: &str,
        new_state: StepState,
    ) -> Result<()> {
        let mut map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
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
                    "Cannot transition step '{step_key}' from {current:?} to {new_state:?}"
                ),
                field: None,
            });
        }

        exec.step_states.insert(step_key.to_string(), new_state);
        exec.updated_at = Utc::now();
        Ok(())
    }

    pub async fn try_claim_async(
        &self,
        transaction_id: &str,
        worker_id: &str,
        lease_duration_secs: i64,
    ) -> Result<bool> {
        let Some(pool) = &self.pool else {
            return Ok(self.try_claim(transaction_id, worker_id, lease_duration_secs));
        };

        let transaction_id = transaction_id.to_string();
        let worker_id = worker_id.to_string();

        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                // The interval is built from a bound integer rather than interpolated, so a
                // lease duration can never reach the parser as text.
                let count = diesel::sql_query(
                    "UPDATE workflow.execution \
                     SET lease_owner = $1, \
                         lease_expires_at = now() + make_interval(secs => $2::double precision), \
                         updated_at = now() \
                     WHERE id = $3 \
                       AND state NOT IN ('Done', 'Failed', 'Reverted') \
                       AND (lease_expires_at IS NULL OR lease_expires_at < now() OR lease_owner = $1) \
                       AND deleted_at IS NULL",
                )
                .bind::<diesel::sql_types::Text, _>(&worker_id)
                .bind::<diesel::sql_types::BigInt, _>(lease_duration_secs)
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                if count == 0 {
                    return Ok(false);
                }

                diesel::sql_query(
                    "INSERT INTO workflow.lease \
                         (id, transaction_id, owner_id, lease_expires_at, renewed_at, updated_at) \
                     VALUES ($1, $2, $3, now() + make_interval(secs => $4::double precision), now(), now()) \
                     ON CONFLICT (transaction_id) DO UPDATE \
                     SET owner_id = $3, \
                         lease_expires_at = now() + make_interval(secs => $4::double precision), \
                         renewed_at = now(), updated_at = now()",
                )
                .bind::<diesel::sql_types::Text, _>(format!("lease_{transaction_id}"))
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .bind::<diesel::sql_types::Text, _>(&worker_id)
                .bind::<diesel::sql_types::BigInt, _>(lease_duration_secs)
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(true)
            })
        })
        .await
    }

    /// Simulates Postgres SKIP LOCKED claiming with lease timeout
    pub fn try_claim(
        &self,
        transaction_id: &str,
        worker_id: &str,
        lease_duration_secs: i64,
    ) -> bool {
        let mut map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
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

    pub async fn apply_retention_policy_async(&self, transaction_id: &str) -> Result<()> {
        let Some(pool) = &self.pool else {
            self.apply_retention_policy(transaction_id);
            return Ok(());
        };

        let transaction_id = transaction_id.to_string();
        stheno_db::tenant::with_tenant_conn(pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "UPDATE workflow.execution \
                     SET context_payload = '{\"[REDACTED]\": \"retention_policy_applied\"}'::jsonb, \
                         updated_at = now() \
                     WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind::<diesel::sql_types::Text, _>(&transaction_id)
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
    }

    /// Redacts PII / context payload after retention window (Task P1-16)
    pub fn apply_retention_policy(&self, transaction_id: &str) {
        let mut map = self.executions.lock().unwrap_or_else(|e| e.into_inner());
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
            .get_or_create_async(transaction_id, workflow_name, initial_input.clone())
            .await?;
        self.store
            .update_state_async(transaction_id, TransactionState::Invoking)
            .await?;

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
                .set_step_state_async(transaction_id, &step_key, StepState::Running)
                .await?;

            while attempts <= config.retries {
                attempts += 1;
                match step.invoke(&ctx, current_input.clone()).await {
                    Ok(output) => {
                        self.store
                            .set_step_state_async(transaction_id, &step_key, StepState::Completed)
                            .await?;
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
                        .set_step_state_async(transaction_id, &step_key, StepState::Skipped)
                        .await?;
                    continue;
                }

                self.store
                    .set_step_state_async(transaction_id, &step_key, StepState::Failed)
                    .await?;
                self.compensate_all(transaction_id, completed_steps).await;
                self.store
                    .update_state_async(transaction_id, TransactionState::Reverted)
                    .await?;
                return Err(err);
            }
        }

        self.store
            .update_state_async(transaction_id, TransactionState::Done)
            .await?;
        Ok(current_input)
    }

    async fn compensate_all(
        &self,
        transaction_id: &str,
        completed_steps: Vec<(usize, Arc<dyn Step>, Value)>,
    ) {
        let _ = self
            .store
            .update_state_async(transaction_id, TransactionState::Compensating)
            .await;
        for (idx, step, input) in completed_steps.into_iter().rev() {
            if !step.config().no_compensation {
                let step_key = format!("step_{}_{}", idx, step.name());
                let ctx = StepContext::new(transaction_id, step.name());
                let _ = self
                    .store
                    .set_step_state_async(transaction_id, &step_key, StepState::Compensating)
                    .await;
                let _ = step.compensate(&ctx, input).await;
                let _ = self
                    .store
                    .set_step_state_async(transaction_id, &step_key, StepState::Compensated)
                    .await;
            }
        }
    }
}

impl Default for SagaCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
