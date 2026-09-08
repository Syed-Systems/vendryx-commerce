//! P21-07: Document and type the InitiateOutput.data.redirect_url convention;
//! offline declares none, a fixture redirect provider declares one.

use async_trait::async_trait;
use rust_decimal_macros::dec;
use stheno_payment::{
    AuthorizeInput, AuthorizeOutput, CancelInput, CancelOutput, CaptureInput, CaptureOutput,
    DeleteInput, InitiateInput, InitiateOutput, PaymentProvider, PaymentSessionStatus,
    ProviderCapabilities, RefundInput, RefundOutput, UpdateInput, UpdateOutput, WebhookAction,
};
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_types::Result;

struct FixtureRedirectProvider;

#[async_trait]
impl PaymentProvider for FixtureRedirectProvider {
    fn id(&self) -> &str {
        "pp_fixture_redirect"
    }

    async fn initiate(&self, _input: InitiateInput) -> Result<InitiateOutput> {
        Ok(InitiateOutput {
            id: "sess_redirect_123".to_string(),
            status: PaymentSessionStatus::Pending,
            data: serde_json::json!({
                "redirect_url": "https://gateway.example.com/checkout/sess_redirect_123",
                "payment_id": "pay_123"
            }),
        })
    }

    async fn update(&self, _input: UpdateInput) -> Result<UpdateOutput> {
        unimplemented!()
    }

    async fn delete(&self, _input: DeleteInput) -> Result<()> {
        unimplemented!()
    }

    async fn authorize(&self, _input: AuthorizeInput) -> Result<AuthorizeOutput> {
        unimplemented!()
    }

    async fn capture(&self, _input: CaptureInput) -> Result<CaptureOutput> {
        unimplemented!()
    }

    async fn refund(&self, _input: RefundInput) -> Result<RefundOutput> {
        unimplemented!()
    }

    async fn cancel(&self, _input: CancelInput) -> Result<CancelOutput> {
        unimplemented!()
    }

    async fn retrieve(&self, _input: stheno_payment::RetrieveInput) -> Result<stheno_payment::RetrieveOutput> {
        unimplemented!()
    }

    async fn get_status(&self, _input: stheno_payment::StatusInput) -> Result<PaymentSessionStatus> {
        unimplemented!()
    }

    fn verify_webhook(&self, _raw: &[u8], _signature_header: Option<&str>) -> Result<stheno_payment::VerifiedEvent> {
        Ok(stheno_payment::VerifiedEvent {
            provider_event_id: "evt_123".to_string(),
            event_type: "payment.succeeded".to_string(),
            provider_created_at: chrono::Utc::now(),
            payload: serde_json::Value::Null,
        })
    }

    fn map_webhook(&self, _event: &stheno_payment::VerifiedEvent) -> Result<WebhookAction> {
        Ok(WebhookAction::NotSupported)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            redirect: true,
            ..Default::default()
        }
    }
}

#[tokio::test]
async fn offline_provider_initiate_declares_no_redirect_url() {
    let offline = OfflinePaymentProvider::new(None);
    let out = offline
        .initiate(InitiateInput {
            amount: dec!(50.00),
            currency_code: "USD".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("offline initiate succeeds");

    assert_eq!(out.redirect_url(), None);
    assert!(!offline.capabilities().redirect);
}

#[tokio::test]
async fn redirect_provider_initiate_declares_redirect_url() {
    let redirect_provider = FixtureRedirectProvider;
    let out = redirect_provider
        .initiate(InitiateInput {
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("redirect initiate succeeds");

    assert_eq!(
        out.redirect_url(),
        Some("https://gateway.example.com/checkout/sess_redirect_123")
    );
    assert!(redirect_provider.capabilities().redirect);
}
