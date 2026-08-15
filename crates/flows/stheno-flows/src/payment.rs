use std::sync::Arc;
use stheno_payment::*;
use stheno_types::Result;

pub struct PaymentWorkflows {
    payment_service: Arc<PaymentService>,
}

impl PaymentWorkflows {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    // 1. create_payment_collection_workflow
    pub async fn create_payment_collection(
        &self,
        input: CreatePaymentCollectionInput,
    ) -> Result<PaymentCollectionDTO> {
        self.payment_service.create_payment_collection(input).await
    }

    // 2. get_payment_collection_workflow
    pub async fn get_payment_collection(&self, id: &str) -> Result<PaymentCollectionDTO> {
        self.payment_service.get_payment_collection(id).await
    }

    // 3. create_payment_session_workflow
    pub async fn create_payment_session(
        &self,
        input: CreatePaymentSessionInput,
        client_ip: Option<&str>,
    ) -> Result<PaymentSessionDTO> {
        self.payment_service
            .create_payment_session(input, client_ip)
            .await
    }

    // 4. get_payment_session_workflow
    pub async fn get_payment_session(&self, id: &str) -> Result<PaymentSessionDTO> {
        self.payment_service.get_payment_session(id).await
    }

    // 5. authorize_payment_session_workflow
    pub async fn authorize_payment_session(
        &self,
        input: AuthorizePaymentSessionInput,
    ) -> Result<PaymentDTO> {
        self.payment_service.authorize_payment_session(input).await
    }

    // 6. get_payment_workflow
    pub async fn get_payment(&self, id: &str) -> Result<PaymentDTO> {
        self.payment_service.get_payment(id).await
    }

    // 7. capture_payment_workflow (under row lock)
    pub async fn capture_payment(&self, input: CapturePaymentInput) -> Result<CaptureDTO> {
        self.payment_service.capture_payment(input).await
    }

    // 8. refund_payment_workflow (under row lock)
    pub async fn refund_payment(&self, input: RefundPaymentInput) -> Result<RefundDTO> {
        self.payment_service.refund_payment(input).await
    }

    // 9. receive_webhook_workflow (with replay guard and convergence reducer)
    pub async fn receive_webhook(
        &self,
        provider_id: &str,
        raw_body: &[u8],
        signature_header: Option<&str>,
    ) -> Result<PaymentWebhookEventDTO> {
        self.payment_service
            .receive_webhook(provider_id, raw_body, signature_header)
            .await
    }

    // 10. outbound_idempotent_call_workflow
    #[allow(clippy::too_many_arguments)]
    pub async fn idempotent_call<F, Fut, T>(
        &self,
        idempotency_key: &str,
        provider_id: &str,
        operation: &str,
        resource_type: &str,
        resource_id: &str,
        payload: &serde_json::Value,
        call_fn: F,
    ) -> Result<T>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    {
        self.payment_service
            .idempotent_call(
                idempotency_key,
                provider_id,
                operation,
                resource_type,
                resource_id,
                payload,
                call_fn,
            )
            .await
    }
}
