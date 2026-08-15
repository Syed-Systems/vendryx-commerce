use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepContext};

struct StatefulStep {
    name: &'static str,
    fail_on_invoke: bool,
    was_compensated: Arc<AtomicBool>,
}

#[async_trait]
impl Step for StatefulStep {
    fn name(&self) -> &'static str {
        self.name
    }
    async fn invoke(
        &self,
        _ctx: &StepContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        if self.fail_on_invoke {
            return Err(Error::Invalid {
                code: "injected_failure",
                message: "injected step failure".to_string(),
                field: None,
            });
        }
        Ok(input)
    }
    async fn compensate(&self, _ctx: &StepContext, _input: serde_json::Value) -> Result<()> {
        self.was_compensated.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_compensation_completeness_at_each_step_position() {
    // Step 1 succeeds, Step 2 succeeds, Step 3 fails -> Steps 2 & 1 compensated in reverse order
    let comp1 = Arc::new(AtomicBool::new(false));
    let comp2 = Arc::new(AtomicBool::new(false));
    let comp3 = Arc::new(AtomicBool::new(false));

    let mut saga = SagaCoordinator::new();
    saga.add_step(Arc::new(StatefulStep {
        name: "step1",
        fail_on_invoke: false,
        was_compensated: comp1.clone(),
    }));
    saga.add_step(Arc::new(StatefulStep {
        name: "step2",
        fail_on_invoke: false,
        was_compensated: comp2.clone(),
    }));
    saga.add_step(Arc::new(StatefulStep {
        name: "step3",
        fail_on_invoke: true, // Injected failure at position 3
        was_compensated: comp3.clone(),
    }));

    let res = saga.execute("tx_comp_matrix", "test_flow", json!({})).await;
    assert!(res.is_err());

    // Verified: step 1 and step 2 are compensated, step 3 never completed so not compensated
    assert!(comp1.load(Ordering::SeqCst));
    assert!(comp2.load(Ordering::SeqCst));
    assert!(!comp3.load(Ordering::SeqCst));
}
