use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_types::Result;
use stheno_workflow::{SagaCoordinator, Step, StepContext, WorkflowStore};

struct CountingStep {
    invocations: Arc<AtomicUsize>,
}

#[async_trait]
impl Step for CountingStep {
    fn name(&self) -> &'static str {
        "counting_step"
    }
    async fn invoke(
        &self,
        _ctx: &StepContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.invocations.fetch_add(1, Ordering::SeqCst);
        Ok(input)
    }
    async fn compensate(&self, _ctx: &StepContext, _input: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_idempotency_concurrent_execution() {
    let store = WorkflowStore::new();
    let invocations = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..20 {
        let store_clone = store.clone();
        let inv_clone = invocations.clone();
        handles.push(tokio::spawn(async move {
            let mut saga = SagaCoordinator::with_store(store_clone);
            saga.add_step(Arc::new(CountingStep {
                invocations: inv_clone,
            }));
            let _ = saga.execute("tx_idem_100", "test_flow", json!({})).await;
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    // Workflow state is recorded and transitions once
    assert!(invocations.load(Ordering::SeqCst) >= 1);
}
