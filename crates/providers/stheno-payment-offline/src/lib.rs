//! Offline / Cash-on-Delivery (COD) Payment Provider (P4-23b).
//! Order placed unpaid, fulfill before payment, manual mark-as-paid, no gateway refund path.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use stheno_payment::*;
use stheno_types::{Error, Result};

pub struct OfflinePaymentProvider {
    id: String,
    manual_instructions: Option<ManualPaymentInstructions>,
}

pub struct OfflinePaymentProviderFactory;
pub struct ManualTransferPaymentProviderFactory;

#[derive(Clone)]
struct ManualPaymentInstructions {
    account: String,
    instructions: String,
}

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

impl PaymentProviderFactory for ManualTransferPaymentProviderFactory {
    fn provider_id(&self) -> &str {
        "pp_manual"
    }

    fn credential_schema(&self) -> &'static [CredentialField] {
        const SCHEMA: &[CredentialField] = &[
            CredentialField {
                key: "payment_account",
                label: "bKash/Nagad/Rocket account",
                kind: CredentialKind::Text,
                required: true,
            },
            CredentialField {
                key: "payment_instructions",
                label: "Manual payment instructions",
                kind: CredentialKind::Text,
                required: true,
            },
        ];
        SCHEMA
    }

    fn build(&self, creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>> {
        validate_credential_map(self.credential_schema(), creds)?;
        let account = creds
            .get("payment_account")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| stheno_types::Error::Invalid {
                code: "manual_payment_account_required",
                message: "A merchant payment account is required.".to_string(),
                field: Some("payment_account".to_string()),
            })?;
        let instructions = creds
            .get("payment_instructions")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| stheno_types::Error::Invalid {
                code: "manual_payment_instructions_required",
                message: "Manual payment instructions are required.".to_string(),
                field: Some("payment_instructions".to_string()),
            })?;
        if account.chars().count() > 80
            || instructions.chars().count() > 500
            || account.chars().any(char::is_control)
            || instructions
                .chars()
                .any(|character| character.is_control() && character != '\n' && character != '\t')
        {
            return Err(stheno_types::Error::Invalid {
                code: "manual_payment_instructions_invalid",
                message:
                    "Manual payment details exceed their safe length or contain control characters."
                        .to_string(),
                field: Some("payment_instructions".to_string()),
            });
        }
        Ok(Arc::new(OfflinePaymentProvider {
            id: "pp_manual".to_string(),
            manual_instructions: Some(ManualPaymentInstructions {
                account: account.to_string(),
                instructions: instructions.to_string(),
            }),
        }))
    }
}

impl OfflinePaymentProvider {
    pub fn new(id: Option<&str>) -> Self {
        Self {
            id: id.unwrap_or("pp_system_default").to_string(),
            manual_instructions: None,
        }
    }
}

impl Default for OfflinePaymentProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

fn validate_manual_currency(is_manual: bool, currency_code: &str) -> Result<()> {
    if is_manual && !currency_code.eq_ignore_ascii_case("bdt") {
        return Err(Error::Invalid {
            code: "manual_payment_currency_unsupported",
            message:
                "bKash, Nagad, Rocket, and these bank-transfer instructions require BDT orders."
                    .to_string(),
            field: Some("currency_code".to_string()),
        });
    }
    Ok(())
}

#[async_trait]
impl PaymentProvider for OfflinePaymentProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        validate_manual_currency(self.manual_instructions.is_some(), &i.currency_code)?;
        let mut data = json!({
            "provider": self.id,
            "amount": i.amount,
            "currency_code": i.currency_code,
            "is_offline": true,
        });
        if let Some(instructions) = &self.manual_instructions {
            data["payment_mode"] = json!("manual_transfer");
            data["payment_account"] = json!(instructions.account);
            data["payment_instructions"] = json!(instructions.instructions);
        }
        Ok(InitiateOutput {
            id: format!("offline_{}", Utc::now().timestamp_micros()),
            status: PaymentSessionStatus::PendingAuthorization,
            data,
        })
    }

    async fn update(&self, i: UpdateInput) -> Result<UpdateOutput> {
        validate_manual_currency(self.manual_instructions.is_some(), &i.currency_code)?;
        let mut data = json!({
            "provider": self.id,
            "amount": i.amount,
            "currency_code": i.currency_code,
            "is_offline": true,
        });
        if let Some(instructions) = &self.manual_instructions {
            data["payment_mode"] = json!("manual_transfer");
            data["payment_account"] = json!(instructions.account);
            data["payment_instructions"] = json!(instructions.instructions);
        }
        Ok(UpdateOutput {
            status: PaymentSessionStatus::PendingAuthorization,
            data,
        })
    }

    async fn delete(&self, _i: DeleteInput) -> Result<()> {
        Ok(())
    }

    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput> {
        // Offline settlement authorizes order placement while the order remains unpaid.
        validate_manual_currency(self.manual_instructions.is_some(), &i.currency_code)?;
        let mut data = json!({
            "provider": self.id,
            "amount": i.amount,
            "currency_code": i.currency_code,
            "authorized_at": Utc::now(),
            "is_offline": true,
        });
        if let Some(instructions) = &self.manual_instructions {
            data["payment_mode"] = json!("manual_transfer");
            data["payment_account"] = json!(&instructions.account);
            data["payment_instructions"] = json!(&instructions.instructions);
        }
        Ok(AuthorizeOutput {
            status: PaymentSessionStatus::Authorized,
            data,
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
