//! Offline / Cash-on-Delivery (COD) Payment Provider (P4-23b).
//! Order placed unpaid, fulfill before payment, manual mark-as-paid, no gateway refund path.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use stheno_payment::*;
use stheno_types::Result;

pub struct OfflinePaymentProvider {
    id: String,
}

pub struct OfflinePaymentProviderFactory;

impl PaymentProviderFactory for OfflinePaymentProviderFactory {
    fn provider_id(&self) -> &str {
        "pp_system_default"
    }

    fn credential_schema(&self) -> &'static [CredentialField] {
        &[]
    }

    fn build(&self, creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>> {
        validate_credential_map(self.credential_schema(), creds)?;
        Ok(Arc::new(OfflinePaymentProvider::default()))
    }
}

impl OfflinePaymentProvider {
    pub fn new(id: Option<&str>) -> Self {
        Self {
            id: id.unwrap_or("pp_system_default").to_string(),
        }
    }
}

impl Default for OfflinePaymentProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl PaymentProvider for OfflinePaymentProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        Ok(InitiateOutput {
            id: format!("offline_{}", Utc::now().timestamp_micros()),
            status: PaymentSessionStatus::PendingAuthorization,
            data: json!({
                "provider": self.id,
                "amount": i.amount,
                "currency_code": i.currency_code,
                "is_offline": true,
            }),
        })
    }

    async fn update(&self, i: UpdateInput) -> Result<UpdateOutput> {
        Ok(UpdateOutput {
            status: PaymentSessionStatus::PendingAuthorization,
            data: json!({
                "provider": self.id,
                "amount": i.amount,
                "currency_code": i.currency_code,
                "is_offline": true,
            }),
        })
    }

    async fn delete(&self, _i: DeleteInput) -> Result<()> {
        Ok(())
    }

    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput> {
        // Offline / COD authorizes immediately so order placement proceeds unpaid
        Ok(AuthorizeOutput {
            status: PaymentSessionStatus::Authorized,
            data: json!({
                "provider": self.id,
                "amount": i.amount,
                "authorized_at": Utc::now(),
                "is_offline": true,
            }),
        })
    }

    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput> {
        // Manual mark-as-paid when cash is collected
        Ok(CaptureOutput {
            data: json!({
                "provider": self.id,
                "captured_amount": i.amount,
                "captured_at": Utc::now(),
                "is_offline": true,
            }),
        })
    }

    async fn refund(&self, i: RefundInput) -> Result<RefundOutput> {
        // Manual mark-as-refunded
        Ok(RefundOutput {
            data: json!({
                "provider": self.id,
                "refunded_amount": i.amount,
                "refunded_at": Utc::now(),
                "reason": i.reason,
                "is_offline": true,
            }),
        })
    }

    async fn cancel(&self, _i: CancelInput) -> Result<CancelOutput> {
        Ok(CancelOutput {
            data: json!({
                "provider": self.id,
                "canceled_at": Utc::now(),
                "is_offline": true,
            }),
        })
    }

    async fn retrieve(&self, i: RetrieveInput) -> Result<RetrieveOutput> {
        Ok(RetrieveOutput {
            status: PaymentSessionStatus::Authorized,
            amount: rust_decimal_macros::dec!(0),
            currency_code: "usd".to_string(),
            data: json!({
                "reference_id": i.reference_id,
                "is_offline": true,
            }),
        })
    }

    async fn get_status(&self, _i: StatusInput) -> Result<PaymentSessionStatus> {
        Ok(PaymentSessionStatus::Authorized)
    }

    fn verify_webhook(&self, _raw: &[u8], _sig: Option<&str>) -> Result<VerifiedEvent> {
        Ok(VerifiedEvent {
            provider_event_id: format!("offline_evt_{}", Utc::now().timestamp_micros()),
            event_type: "offline.manual_event".to_string(),
            provider_created_at: Utc::now(),
            payload: json!({ "offline": true }),
        })
    }

    fn map_webhook(&self, _event: &VerifiedEvent) -> Result<WebhookAction> {
        Ok(WebhookAction::NotSupported)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            partial_capture: true,
            multi_capture: false,
            partial_refund: true,
            cancel_after_authorize: true,
            vault: false,
            async_settlement: true, // Offline payment settles when collected
            redirect: false,
            authorization_ttl: None,
        }
    }
}
