//! P13-04: Compensation stack is persisted to Postgres and recoverable after process restart.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_types::{Error, Result};
use stheno_workflow::{
    SagaCoordinator, Step, StepConfig, StepContext, StepState, TransactionState, WorkflowStore,
};

static STEP1_COMPENSATED: AtomicUsize = AtomicUsize::new(0);
static STEP2_COMPENSATED: AtomicUsize = AtomicUsize::new(0);

struct InventoryStep;
#[async_trait]
impl Step for InventoryStep {
    fn name(&self) -> &'static str {
        "inventory_reserve"
    }
    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        Ok(input)
    }
    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        STEP1_COMPENSATED.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct PaymentStep;
#[async_trait]
impl Step for PaymentStep {
    fn name(&self) -> &'static str {
        "payment_auth"
    }
    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        Ok(input)
    }
    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        STEP2_COMPENSATED.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct FailingStep;
#[async_trait]
impl Step for FailingStep {
    fn name(&self) -> &'static str {
        "order_create"
    }
    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 0,
            ..StepConfig::default()
        }
    }
    async fn invoke(&self, _ctx: &StepContext, _input: Value) -> Result<Value> {
        Err(Error::Internal(
            "Simulated crash at step 3".to_string().into(),
        ))
    }
    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn compensation_stack_persists_and_recovers_across_restarts() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");

    let tx_id = "tx_comp_durability_1";
    let store1 = WorkflowStore::from_pool(pool.clone());

    let mut coordinator = SagaCoordinator::with_store(store1);
    coordinator.add_step(Arc::new(InventoryStep));
    coordinator.add_step(Arc::new(PaymentStep));
    coordinator.add_step(Arc::new(FailingStep));

    let result = coordinator
        .execute(tx_id, "checkout_saga", json!({ "cart_id": "c123" }))
        .await;
    assert!(result.is_err(), "Coordinator execution must fail at step 3");

    // Both steps must have been compensated
    assert_eq!(
        STEP2_COMPENSATED.load(Ordering::SeqCst),
        1,
        "PaymentStep must be compensated"
    );
    assert_eq!(
        STEP1_COMPENSATED.load(Ordering::SeqCst),
        1,
        "InventoryStep must be compensated"
    );

    // Drop coordinator and store1 to simulate shutdown/restart
    drop(coordinator);

    // Restart with a fresh store connected to the same database
    let store2 = WorkflowStore::from_pool(pool);
    let execution = store2
        .get_execution_async(tx_id)
        .await
        .expect("query execution")
        .expect("execution must exist in DB");

    assert_eq!(
        execution.state,
        TransactionState::Reverted,
        "Workflow in DB must be Reverted"
    );
    assert_eq!(
        execution.step_states.get("step_0_inventory_reserve"),
        Some(&StepState::Compensated),
        "Step 0 must be marked Compensated in DB"
    );
    assert_eq!(
        execution.step_states.get("step_1_payment_auth"),
        Some(&StepState::Compensated),
        "Step 1 must be marked Compensated in DB"
    );
    assert_eq!(
        execution.step_states.get("step_2_order_create"),
        Some(&StepState::Failed),
        "Step 2 must be marked Failed in DB"
    );

    db.drop().await;
}
