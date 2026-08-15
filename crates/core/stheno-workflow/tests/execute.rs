use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use stheno_types::Result;
use stheno_workflow::{SagaCoordinator, Step, StepContext};

struct AddStep;
#[async_trait]
impl Step for AddStep {
    fn name(&self) -> &'static str {
        "add_step"
    }
    async fn invoke(
        &self,
        _ctx: &StepContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let val = input["val"].as_i64().unwrap_or(0);
        Ok(json!({ "val": val + 10 }))
    }
    async fn compensate(&self, _ctx: &StepContext, _input: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_saga_execution_pipeline() {
    let mut saga = SagaCoordinator::new();
    saga.add_step(Arc::new(AddStep));
    saga.add_step(Arc::new(AddStep));

    let result = saga
        .execute("tx_exec_1", "calc_flow", json!({ "val": 5 }))
        .await
        .expect("must execute");

    assert_eq!(result["val"], 25);
}
