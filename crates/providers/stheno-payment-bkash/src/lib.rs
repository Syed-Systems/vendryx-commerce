//! Typed bKash Tokenized Checkout Payment Provider on reqwest (PAYMENTS §4, §5, §7, §12; docs/providers/PAYMENT_GATEWAYS.md §2).
//! Implements all 18 methods of `PaymentProvider`.

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use stheno_payment::*;
use stheno_types::{Error, Result};

pub struct BkashPaymentProvider {
    id: String,
    base_url: String,
    app_key: String,
    app_secret: String,
    username: String,
    password: String,
    client: Client,
    token_cache: RwLock<Option<(String, DateTime<Utc>)>>,
}

pub struct BkashPaymentProviderFactory;

const BKASH_CREDENTIAL_SCHEMA: &[CredentialField] = &[
    CredentialField {
        key: "base_url",
        label: "Base URL",
        kind: CredentialKind::Url,
        required: true,
    },
    CredentialField {
        key: "app_key",
        label: "App Key",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "app_secret",
        label: "App Secret",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "username",
        label: "Username",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "password",
        label: "Password",
        kind: CredentialKind::Secret,
        required: true,
    },
];

impl PaymentProviderFactory for BkashPaymentProviderFactory {
    fn provider_id(&self) -> &str {
        "pp_bkash_bkash"
    }

    fn credential_schema(&self) -> &'static [CredentialField] {
        BKASH_CREDENTIAL_SCHEMA
    }

    fn build(&self, creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>> {
        validate_credential_map(self.credential_schema(), creds)?;

        let base_url = credential_str(creds, "base_url")?;
        let app_key = credential_str(creds, "app_key")?;
        let app_secret = credential_str(creds, "app_secret")?;
        let username = credential_str(creds, "username")?;
        let password = credential_str(creds, "password")?;

        Ok(Arc::new(BkashPaymentProvider::new(
            base_url,
            app_key,
            app_secret,
            username,
            password,
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

impl BkashPaymentProvider {
    pub fn new(
        base_url: &str,
        app_key: &str,
        app_secret: &str,
        username: &str,
        password: &str,
    ) -> Self {
        Self {
            id: "pp_bkash_bkash".to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            app_key: app_key.to_string(),
            app_secret: app_secret.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            token_cache: RwLock::new(None),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn get_auth_token(&self) -> Result<String> {
        let now = Utc::now();
        {
            let cache = self.token_cache.read().await;
            if let Some((token, expires_at)) = &*cache {
                if *expires_at > now + ChronoDuration::minutes(5) {
                    return Ok(token.clone());
                }
            }
        }

        let mut cache_lock = self.token_cache.write().await;
        if let Some((token, expires_at)) = &*cache_lock {
            if *expires_at > now + ChronoDuration::minutes(5) {
                return Ok(token.clone());
            }
        }

        let url = format!("{}/tokenized/checkout/token/grant", self.base_url);
        let payload = serde_json::json!({
            "app_key": self.app_key,
            "app_secret": self.app_secret,
        });

        let resp = self
            .client
            .post(&url)
            .header("username", &self.username)
            .header("password", &self.password)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("bKash token grant network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse bKash token grant response: {}", e).into())
        })?;

        let id_token = body["id_token"]
            .as_str()
            .ok_or_else(|| Error::Invalid {
                code: "bkash_token_grant_failed",
                message: body["statusMessage"]
                    .as_str()
                    .unwrap_or("Failed to obtain bKash id_token")
                    .to_string(),
                field: None,
            })?
            .to_string();

        let expires_in_sec = body["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = now + ChronoDuration::seconds(expires_in_sec);

        *cache_lock = Some((id_token.clone(), expires_at));
        Ok(id_token)
    }
}

impl Default for BkashPaymentProvider {
    fn default() -> Self {
        Self::new(
            "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
            "test_app_key",
            "test_app_secret",
            "test_username",
            "test_password",
        )
    }
}

#[async_trait]
impl PaymentProvider for BkashPaymentProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        let auth_token = self.get_auth_token().await?;
        let url = format!("{}/tokenized/checkout/payment/create", self.base_url);

        let custom_data = i.context.as_ref().or(i.metadata.as_ref());
        let amount_str = i.amount.to_string();
        let callback_url = custom_data
            .and_then(|v| v.get("callback_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/checkout/callback");
        let payer_ref = custom_data
            .and_then(|v| v.get("payer_reference"))
            .and_then(|v| v.as_str())
            .unwrap_or("01700000000");
        let invoice_num = custom_data
            .and_then(|v| v.get("session_id").or_else(|| v.get("invoice_number")))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("inv_{}", ulid::Ulid::new()));

        let payload = serde_json::json!({
            "mode": "0011",
            "payerReference": payer_ref,
            "callbackURL": callback_url,
            "amount": amount_str,
            "currency": "BDT",
            "intent": "authorization",
            "merchantInvoiceNumber": invoice_num,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth_token)
            .header("X-App-Key", &self.app_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("bKash payment create network error: {}", e).into()))?;

        let mut body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse bKash payment create response: {}", e).into())
        })?;

        let status_code = body["statusCode"].as_str().unwrap_or_default();
        if status_code != "0000" {
            let msg = body["statusMessage"]
                .as_str()
                .or_else(|| body["errorMessage"].as_str())
                .unwrap_or("bKash payment create failed");
            return Err(Error::Invalid {
                code: "bkash_create_failed",
                message: msg.to_string(),
                field: None,
            });
        }

        let payment_id = body["paymentID"]
            .as_str()
            .unwrap_or(&invoice_num)
            .to_string();

        let redirect_url = body
            .get("bkashURL")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        if let Some(url_str) = redirect_url {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("redirect_url".to_string(), serde_json::Value::String(url_str));
            }
        }

        Ok(InitiateOutput {
            id: payment_id,
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
        let auth_token = self.get_auth_token().await?;
        let payment_id = &i.payment_id;
        let url = format!(
            "{}/tokenized/checkout/execute/{}",
            self.base_url, payment_id
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth_token)
            .header("X-App-Key", &self.app_key)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("bKash execute payment network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse bKash execute response: {}", e).into())
        })?;

        let status_code = body["statusCode"].as_str().unwrap_or_default();
        if status_code != "0000" {
            let msg = body["statusMessage"]
                .as_str()
                .or_else(|| body["errorMessage"].as_str())
                .unwrap_or("bKash execute failed");
            return Err(Error::Invalid {
                code: "bkash_execute_failed",
                message: msg.to_string(),
                field: None,
            });
        }

        Ok(CaptureOutput { data: body })
    }

    async fn refund(&self, i: RefundInput) -> Result<RefundOutput> {
        let auth_token = self.get_auth_token().await?;
        let url = format!("{}/tokenized/checkout/payment/refund", self.base_url);

        let custom_data = i.data.unwrap_or_default();
        let trx_id = custom_data
            .get("trxID")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let reason = i.note.as_deref().unwrap_or("Merchant requested refund");

        let payload = serde_json::json!({
            "paymentID": i.payment_id,
            "amount": i.amount.to_string(),
            "trxID": trx_id,
            "sku": "order_item",
            "reason": reason,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth_token)
            .header("X-App-Key", &self.app_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("bKash refund network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse bKash refund response: {}", e).into())
        })?;

        let status_code = body["statusCode"].as_str().unwrap_or_default();
        if status_code != "0000" {
            let msg = body["statusMessage"]
                .as_str()
                .or_else(|| body["errorMessage"].as_str())
                .unwrap_or("bKash refund failed");
            return Err(Error::Invalid {
                code: "bkash_refund_failed",
                message: msg.to_string(),
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
        let auth_token = self.get_auth_token().await?;
        let url = format!(
            "{}/tokenized/checkout/payment/query/{}",
            self.base_url, i.reference_id
        );

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth_token)
            .header("X-App-Key", &self.app_key)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("bKash query payment network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse bKash query response: {}", e).into())
        })?;

        let transaction_status = body["transactionStatus"].as_str().unwrap_or_default();
        let status = match transaction_status {
            "Completed" => PaymentSessionStatus::Authorized,
            "Failed" | "Cancelled" => PaymentSessionStatus::Error,
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
        None
    }

    fn verify_webhook(&self, raw: &[u8], _signature_header: Option<&str>) -> Result<VerifiedEvent> {
        let event_json: serde_json::Value = serde_json::from_slice(raw).map_err(|e| {
            Error::Invalid {
                code: "invalid_webhook_payload",
                message: format!("Could not parse bKash webhook JSON: {}", e),
                field: None,
            }
        })?;

        let event_id = event_json["MessageId"]
            .as_str()
            .or_else(|| event_json["paymentID"].as_str())
            .unwrap_or_default()
            .to_string();

        let event_type = event_json["Type"]
            .as_str()
            .or_else(|| event_json["transactionStatus"].as_str())
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
        let mut msg_obj = &event.payload;
        let parsed_msg: serde_json::Value;

        if let Some(msg_str) = event.payload["Message"].as_str() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg_str) {
                parsed_msg = parsed;
                msg_obj = &parsed_msg;
            }
        }

        let trx_status = msg_obj["transactionStatus"]
            .as_str()
            .or_else(|| event.payload["transactionStatus"].as_str())
            .unwrap_or_default();

        if trx_status.eq_ignore_ascii_case("Completed") {
            return Ok(WebhookAction::Captured);
        }

        Ok(WebhookAction::NotSupported)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            partial_capture: false,
            multi_capture: false,
            partial_refund: true,
            cancel_after_authorize: true,
            vault: false,
            async_settlement: false,
            authorization_ttl: Some(Duration::from_secs(24 * 3600)),
        }
    }
}
