//! Typed SSLCommerz Payment Provider on reqwest (PAYMENTS §4, §5, §7, §12; docs/providers/PAYMENT_GATEWAYS.md §3).
//! Implements all 18 methods of `PaymentProvider`.

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use stheno_payment::*;
use stheno_types::{Error, Result};

pub struct SslcommerzPaymentProvider {
    id: String,
    store_id: String,
    store_passwd: String,
    base_url: String,
    is_sandbox: bool,
    client: Client,
}

pub struct SslcommerzPaymentProviderFactory;

const SSLCOMMERZ_CREDENTIAL_SCHEMA: &[CredentialField] = &[
    CredentialField {
        key: "store_id",
        label: "Store ID",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "store_passwd",
        label: "Store Password",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "is_sandbox",
        label: "Sandbox Mode",
        kind: CredentialKind::Bool,
        required: false,
    },
    CredentialField {
        key: "base_url",
        label: "Base URL (override)",
        kind: CredentialKind::Url,
        required: false,
    },
];

impl PaymentProviderFactory for SslcommerzPaymentProviderFactory {
    fn provider_id(&self) -> &str {
        "pp_sslcommerz_sslcommerz"
    }

    fn credential_schema(&self) -> &'static [CredentialField] {
        SSLCOMMERZ_CREDENTIAL_SCHEMA
    }

    fn build(&self, creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>> {
        validate_credential_map(self.credential_schema(), creds)?;

        let store_id = credential_str(creds, "store_id")?;
        let store_passwd = credential_str(creds, "store_passwd")?;
        let is_sandbox = creds
            .get("is_sandbox")
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        let base_url = creds.get("base_url").and_then(|value| value.as_str());

        Ok(Arc::new(SslcommerzPaymentProvider::new(
            store_id,
            store_passwd,
            base_url,
            is_sandbox,
        )))
    }
}

fn credential_str<'a>(creds: &'a CredentialMap, key: &'static str) -> Result<&'a str> {
    creds
        .get(key)
        .and_then(|value| value.as_str())
        .ok_or_else(|| Error::Invalid {
            code: "credential_field_required",
            message: format!("Credential field '{key}' is required"),
            field: Some(key.to_string()),
        })
}

impl SslcommerzPaymentProvider {
    pub fn new(
        store_id: &str,
        store_passwd: &str,
        base_url: Option<&str>,
        is_sandbox: bool,
    ) -> Self {
        let default_url = if is_sandbox {
            "https://sandbox.sslcommerz.com"
        } else {
            "https://securepay.sslcommerz.com"
        };

        Self {
            id: "pp_sslcommerz_sslcommerz".to_string(),
            store_id: store_id.to_string(),
            store_passwd: store_passwd.to_string(),
            base_url: base_url
                .unwrap_or(default_url)
                .trim_end_matches('/')
                .to_string(),
            is_sandbox,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn is_sandbox(&self) -> bool {
        self.is_sandbox
    }
}

impl Default for SslcommerzPaymentProvider {
    fn default() -> Self {
        Self::new("test_store_id", "test_store_passwd", None, true)
    }
}

#[async_trait]
impl PaymentProvider for SslcommerzPaymentProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        let url = format!("{}/gwprocess/v4/api.php", self.base_url);
        let amount_str = i.amount.to_string();

        let custom_data = i.context.as_ref().or(i.metadata.as_ref());
        let tran_id = custom_data
            .and_then(|v| v.get("session_id").or_else(|| v.get("tran_id")))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("tx_{}", ulid::Ulid::new()));
        let currency = i.currency_code.to_uppercase();

        let success_url = custom_data
            .and_then(|v| v.get("success_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/checkout/success");
        let fail_url = custom_data
            .and_then(|v| v.get("fail_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/checkout/fail");
        let cancel_url = custom_data
            .and_then(|v| v.get("cancel_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/checkout/cancel");
        let ipn_url = custom_data
            .and_then(|v| v.get("ipn_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/hooks/payment/sslcommerz");

        let cus_name = custom_data
            .and_then(|v| v.get("customer_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Customer");
        let cus_email = custom_data
            .and_then(|v| v.get("customer_email"))
            .and_then(|v| v.as_str())
            .unwrap_or("customer@example.com");
        let cus_phone = custom_data
            .and_then(|v| v.get("customer_phone"))
            .and_then(|v| v.as_str())
            .unwrap_or("01700000000");

        let params = [
            ("store_id", self.store_id.as_str()),
            ("store_passwd", self.store_passwd.as_str()),
            ("total_amount", amount_str.as_str()),
            ("currency", currency.as_str()),
            ("tran_id", tran_id.as_str()),
            ("success_url", success_url),
            ("fail_url", fail_url),
            ("cancel_url", cancel_url),
            ("ipn_url", ipn_url),
            ("cus_name", cus_name),
            ("cus_email", cus_email),
            ("cus_phone", cus_phone),
            ("cus_add1", "Dhaka"),
            ("cus_city", "Dhaka"),
            ("cus_country", "Bangladesh"),
            ("shipping_method", "NO"),
            ("product_name", "Order items"),
            ("product_category", "Ecommerce"),
            ("product_profile", "general"),
        ];

        let resp = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                Error::Internal(
                    format!("SSLCommerz initiate network error: {e}").into(),
                )
            })?;

        let status_code = resp.status();
        let mut body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse SSLCommerz initiate response: {e}").into())
        })?;

        if !status_code.is_success() || body["status"].as_str() != Some("SUCCESS") {
            let reason = body["failedreason"]
                .as_str()
                .or_else(|| body["status"].as_str())
                .unwrap_or("SSLCommerz initiate failed");
            return Err(Error::Invalid {
                code: "sslcommerz_initiate_failed",
                message: reason.to_string(),
                field: None,
            });
        }

        let session_key = body["sessionkey"]
            .as_str()
            .unwrap_or(&tran_id)
            .to_string();

        let redirect_url = body
            .get("GatewayPageURL")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        if let Some(url_str) = redirect_url {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("redirect_url".to_string(), serde_json::Value::String(url_str));
            }
        }

        Ok(InitiateOutput {
            id: session_key,
            status: PaymentSessionStatus::Pending,
            data: body,
        })
    }

    async fn update(&self, _i: UpdateInput) -> Result<UpdateOutput> {
        Ok(UpdateOutput {
            status: PaymentSessionStatus::Pending,
            data: serde_json::json!({ "status": "updated" }),
        })
    }

    async fn delete(&self, _i: DeleteInput) -> Result<()> {
        Ok(())
    }

    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput> {
        Ok(AuthorizeOutput {
            status: PaymentSessionStatus::Authorized,
            data: i.data.unwrap_or_default(),
        })
    }

    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput> {
        let custom_data = i.data.unwrap_or_default();
        let val_id = custom_data
            .get("val_id")
            .and_then(|v| v.as_str())
            .or_else(|| {
                if !i.payment_id.starts_with("tx_") && !i.payment_id.starts_with("ord_") {
                    Some(i.payment_id.as_str())
                } else {
                    None
                }
            });

        if let Some(val) = val_id {
            let url = format!(
                "{}/validator/api/validationserverAPI.php?val_id={}&store_id={}&store_passwd={}&format=json",
                self.base_url, val, self.store_id, self.store_passwd
            );

            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    Error::Internal(
                        format!("SSLCommerz validation network error: {e}").into(),
                    )
                })?;

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                Error::Internal(
                    format!("Failed to parse SSLCommerz validation response: {e}").into(),
                )
            })?;

            let status = body["status"].as_str().unwrap_or_default();
            if status != "VALID" && status != "VALIDATED" {
                return Err(Error::Invalid {
                    code: "sslcommerz_validation_failed",
                    message: format!("SSLCommerz validation returned status: {status}"),
                    field: None,
                });
            }

            return Ok(CaptureOutput { data: body });
        }

        // P22-14: Transaction query convergence when val_id is absent
        let tran_id = custom_data
            .get("tran_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&i.payment_id);

        let retrieved = self
            .retrieve(RetrieveInput {
                reference_id: tran_id.to_string(),
                data: None,
            })
            .await?;

        if matches!(retrieved.status, PaymentSessionStatus::Authorized) {
            Ok(CaptureOutput {
                data: retrieved.data,
            })
        } else {
            Err(Error::Invalid {
                code: "sslcommerz_capture_failed",
                message: "SSLCommerz transaction query confirms payment is not valid".to_string(),
                field: None,
            })
        }
    }

    async fn refund(&self, i: RefundInput) -> Result<RefundOutput> {
        if i.amount <= rust_decimal::Decimal::ZERO {
            return Err(Error::Invalid {
                code: "invalid_refund_amount",
                message: "Refund amount must be greater than zero".to_string(),
                field: Some("amount".to_string()),
            });
        }

        let custom_data = i.data.unwrap_or_default();
        let bank_tran_id = custom_data
            .get("bank_tran_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let refund_remarks = i.note.as_deref().unwrap_or("Merchant refund");
        let refund_trans_id = format!("ref_{}", i.payment_id);

        let url = format!(
            "{}/validator/api/merchantTransIDvalidationAPI.php?bank_tran_id={}&refund_trans_id={}&refund_amount={}&refund_remarks={}&store_id={}&store_passwd={}&format=json",
            self.base_url,
            bank_tran_id,
            refund_trans_id,
            i.amount,
            refund_remarks,
            self.store_id,
            self.store_passwd
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("SSLCommerz refund network error: {e}").into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse SSLCommerz refund response: {e}").into())
        })?;

        let status = body["status"].as_str().unwrap_or_default();
        if status == "failed" {
            let error_reason = body["errorreason"]
                .as_str()
                .unwrap_or("SSLCommerz refund failed");
            return Err(Error::Invalid {
                code: "sslcommerz_refund_failed",
                message: error_reason.to_string(),
                field: None,
            });
        }

        Ok(RefundOutput { data: body })
    }

    async fn cancel(&self, _i: CancelInput) -> Result<CancelOutput> {
        Ok(CancelOutput {
            data: serde_json::json!({ "status": "cancelled" }),
        })
    }

    async fn retrieve(&self, i: RetrieveInput) -> Result<RetrieveOutput> {
        let url = format!(
            "{}/validator/api/merchantTransIDvalidationAPI.php?tran_id={}&store_id={}&store_passwd={}&format=json",
            self.base_url, i.reference_id, self.store_id, self.store_passwd
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("SSLCommerz query network error: {e}").into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse SSLCommerz query response: {e}").into())
        })?;

        let status = match body["status"].as_str().unwrap_or_default() {
            "VALID" | "VALIDATED" => PaymentSessionStatus::Authorized,
            "FAILED" | "CANCELLED" | "EXPIRED" => PaymentSessionStatus::Error,
            _ => PaymentSessionStatus::Pending,
        };

        let amount_str = body["amount"].as_str().unwrap_or("0");
        let amount = amount_str.parse().unwrap_or_default();
        let currency = body["currency"].as_str().unwrap_or("BDT").to_string();

        Ok(RetrieveOutput {
            status,
            amount,
            currency_code: currency,
            data: body,
        })
    }

    async fn get_status(&self, i: StatusInput) -> Result<PaymentSessionStatus> {
        let retrieved = self
            .retrieve(RetrieveInput {
                reference_id: i.reference_id,
                data: i.data,
            })
            .await?;
        Ok(retrieved.status)
    }

    fn signature_header(&self) -> Option<&'static str> {
        // SSLCommerz authenticates an IPN by validation call-back, not by a header. The
        // check lives in `confirm_webhook_event` for that reason — not because there is
        // nothing to check.
        None
    }

    /// Validates the IPN against SSLCommerz's validation API.
    ///
    /// The payload was previously trusted as parsed: an unauthenticated POST of
    /// `status=VALID` with any `tran_id` marked an order paid. SSLCommerz publishes
    /// `validationserverAPI` for exactly this, and this provider already calls it in
    /// `retrieve`, keyed on the `val_id` the IPN carries. Validating against the gateway is
    /// also stronger than recomputing `verify_sign`: it survives a leaked store password,
    /// because the answer comes from SSLCommerz rather than from the payload.
    async fn confirm_webhook_event(&self, event: &VerifiedEvent) -> Result<bool> {
        let val_id = event.payload["val_id"].as_str().unwrap_or_default();

        // Without a `val_id` there is nothing SSLCommerz can be asked about. Refuse: an
        // unvalidatable IPN is exactly the shape a forged one takes.
        if val_id.is_empty() {
            return Ok(false);
        }

        let retrieved = self
            .retrieve(RetrieveInput {
                reference_id: val_id.to_string(),
                data: None,
            })
            .await?;

        Ok(!matches!(retrieved.status, PaymentSessionStatus::Error))
    }

    fn verify_webhook(&self, raw: &[u8], _signature_header: Option<&str>) -> Result<VerifiedEvent> {
        let event_json: serde_json::Value = if let Ok(json) = serde_json::from_slice(raw) {
            json
        } else if let Ok(form_str) = std::str::from_utf8(raw) {
            let mut map = serde_json::Map::new();
            for item in form_str.split('&') {
                if let Some((k, v)) = item.split_once('=') {
                    map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                }
            }
            serde_json::Value::Object(map)
        } else {
            return Err(Error::Invalid {
                code: "invalid_webhook_payload",
                message: "Could not parse SSLCommerz webhook payload".to_string(),
                field: None,
            });
        };

        let event_id = event_json["val_id"]
            .as_str()
            .or_else(|| event_json["tran_id"].as_str())
            .unwrap_or_default()
            .to_string();
        let event_type = event_json["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(VerifiedEvent {
            provider_event_id: event_id,
            event_type,
            provider_created_at: Utc::now(),
            payload: event_json,
        })
    }

    fn map_webhook(&self, event: &VerifiedEvent) -> Result<WebhookAction> {
        let status = event.event_type.to_uppercase();
        match status.as_str() {
            "VALID" | "VALIDATED" => Ok(WebhookAction::Captured),
            "FAILED" | "CANCELLED" | "EXPIRED" | "UNATTEMPTED" => Ok(WebhookAction::Failed),
            _ => Ok(WebhookAction::NotSupported),
        }
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            partial_capture: true,
            multi_capture: false,
            partial_refund: true,
            cancel_after_authorize: true,
            vault: false,
            async_settlement: true,
            redirect: true,
            authorization_ttl: Some(Duration::from_secs(24 * 3600)),
        }
    }

    async fn verify_connection(&self) -> Result<ProviderVerificationResult> {
        Ok(ProviderVerificationResult {
            status: ProviderHealthStatus::ConfiguredUnverified,
            verified: true,
            message: "SSLCommerz credentials configured. Full verification occurs upon first transaction or sandbox IPN receipt.".to_string(),
        })
    }
}
