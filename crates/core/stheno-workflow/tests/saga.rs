use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepConfig, StepContext};

#[derive(Clone)]
struct StepOne {
    invoked: Arc<AtomicU32>,
    compensated: Arc<AtomicU32>,
}

#[async_trait]
impl Step for StepOne {
    fn name(&self) -> &'static str {
        "StepOne"
    }

    async fn invoke(
        &self,
        _ctx: &StepContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.invoked.fetch_add(1, Ordering::SeqCst);
        let s = input["val"].as_str().unwrap_or("");
        Ok(json!({ "val": format!("processed-{}", s) }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: serde_json::Value) -> Result<()> {
        self.compensated.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct FailingStep {
    attempts: Arc<AtomicU32>,
}

#[async_trait]
impl Step for FailingStep {
    fn name(&self) -> &'static str {
        "FailingStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 2,
            retry_interval_ms: 10,
            ..Default::default()
        }
    }

    async fn invoke(
        &self,
        _ctx: &StepContext,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Err(Error::Internal("Simulated step failure".into()))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_saga_forward_and_backward_compensation() {
    let step1_invoked = Arc::new(AtomicU32::new(0));
    let step1_compensated = Arc::new(AtomicU32::new(0));
    let step2_attempts = Arc::new(AtomicU32::new(0));

    let step1 = Arc::new(StepOne {
        invoked: step1_invoked.clone(),
        compensated: step1_compensated.clone(),
    });

    let step2 = Arc::new(FailingStep {
        attempts: step2_attempts.clone(),
    });

    let mut saga = SagaCoordinator::new();
    saga.add_step(step1);
    saga.add_step(step2);

    let res = saga
        .execute("tx_test_123", "sample_flow", json!({ "val": "item1" }))
        .await;

    assert!(res.is_err());
    assert_eq!(step1_invoked.load(Ordering::SeqCst), 1);
    assert_eq!(step2_attempts.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    assert_eq!(step1_compensated.load(Ordering::SeqCst), 1); // Compensated on failure
}
