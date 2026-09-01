//! A callback that carries no signature must be confirmed with the gateway, not believed.
//!
//! bKash and SSLCommerz return `None` from `signature_header`, which the trait documents as
//! "a claim that the body is self-authenticating, never a licence to skip verification".
//! Both then parsed the payload and returned it unchecked, so an unauthenticated POST of
//! `{"paymentID": "...", "transactionStatus": "Completed"}` to the public webhook URL was
//! enough to have an order treated as paid.
//!
//! Both now confirm against their own status API. This asserts the seam those two hang on —
//! that `receive_webhook` refuses an event the provider will not confirm — without needing a
//! live gateway: the provider here simply says no.

use async_trait::async_trait;
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::*;
use stheno_types::Result;

/// A gateway that parses anything and vouches for nothing — the shape bKash and SSLCommerz
/// had before they were taught to ask.
struct UnconfirmingProvider;

#[async_trait]
impl PaymentProvider for UnconfirmingProvider {
    fn id(&self) -> &str {
        "pp_unconfirming"
    }

    fn signature_header(&self) -> Option<&'static str> {
        None
    }

    async fn confirm_webhook_event(&self, _event: &VerifiedEvent) -> Result<bool> {
        Ok(false)
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        Ok(InitiateOutput {
            id: "unconfirming_1".to_string(),
            status: PaymentSessionStatus::PendingAuthorization,
            data: serde_json::json!({ "amount": i.amount }),
        })
    }

    async fn update(&self, _i: UpdateInput) -> Result<UpdateOutput> {
        Ok(UpdateOutput {
            status: PaymentSessionStatus::PendingAuthorization,
            data: serde_json::json!({}),
        })
    }

    async fn delete(&self, _i: DeleteInput) -> Result<()> {
        Ok(())
    }

    async fn authorize(&self, _i: AuthorizeInput) -> Result<AuthorizeOutput> {
        Ok(AuthorizeOutput {
            status: PaymentSessionStatus::Authorized,
            data: serde_json::json!({}),
        })
    }

    async fn capture(&self, _i: CaptureInput) -> Result<CaptureOutput> {
        Ok(CaptureOutput {
            data: serde_json::json!({}),
        })
    }

    async fn refund(&self, _i: RefundInput) -> Result<RefundOutput> {
        Ok(RefundOutput {
            data: serde_json::json!({}),
        })
    }

    async fn cancel(&self, _i: CancelInput) -> Result<CancelOutput> {
        Ok(CancelOutput {
            data: serde_json::json!({}),
        })
    }

    async fn retrieve(&self, _i: RetrieveInput) -> Result<RetrieveOutput> {
        Ok(RetrieveOutput {
            status: PaymentSessionStatus::Authorized,
            amount: rust_decimal_macros::dec!(0),
            currency_code: "usd".to_string(),
            data: serde_json::json!({}),
        })
    }

    async fn get_status(&self, _i: StatusInput) -> Result<PaymentSessionStatus> {
        Ok(PaymentSessionStatus::Authorized)
    }

    fn verify_webhook(&self, raw: &[u8], _sig: Option<&str>) -> Result<VerifiedEvent> {
        let payload: serde_json::Value =
            serde_json::from_slice(raw).unwrap_or_else(|_| serde_json::json!({}));
        Ok(VerifiedEvent {
            provider_event_id: "unconfirming_evt_1".to_string(),
            event_type: "unconfirming.event".to_string(),
            provider_created_at: chrono::Utc::now(),
            payload,
        })
    }

    fn map_webhook(&self, _event: &VerifiedEvent) -> Result<WebhookAction> {
        Ok(WebhookAction::NotSupported)
    }
}

#[tokio::test]
async fn an_unsigned_webhook_the_gateway_will_not_confirm_is_refused() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    service.register_provider(Arc::new(UnconfirmingProvider));

    // Exactly the forgery the old code accepted: well-formed, unsigned, and claiming success.
    let forged = br#"{"paymentID":"TR0011abc","transactionStatus":"Completed"}"#;

    let result = service
        .receive_webhook("pp_unconfirming", forged, None)
        .await;

    let err = result.expect_err("an unconfirmed event must not be accepted");
    let code = match &err {
        stheno_types::Error::Invalid { code, .. } => *code,
        other => panic!("expected an invalid-signature refusal, got {other:?}"),
    };
    assert_eq!(
        code, "webhook_signature_invalid",
        "the refusal must read as a signature failure so the hooks route answers 401 and \
         the gateway stops retrying a payload that will never be accepted"
    );
}

/// The default stays permissive for providers that genuinely verify in `verify_webhook`:
/// Stripe checks an HMAC there, and the offline provider has no gateway to ask. Changing
/// the default to "refuse" would have broken both.
#[tokio::test]
async fn a_provider_that_verifies_for_itself_is_still_accepted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    service.register_provider(Arc::new(stheno_payment_offline::OfflinePaymentProvider::new(
        Some("pp_self_verifying"),
    )));

    let event = service
        .receive_webhook("pp_self_verifying", br#"{"event":"manual"}"#, None)
        .await
        .expect("a self-authenticating provider is unaffected by the confirmation step");

    assert_eq!(event.state, "received");
    let _ = dec!(1);
}
