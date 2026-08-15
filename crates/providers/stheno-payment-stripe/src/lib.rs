//! Hand-rolled typed Stripe Payment Provider on reqwest (PAYMENTS §4, §5, §7, §12).
//! Implements all 18 methods of `PaymentProvider` + `PaymentVault`.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use std::time::Duration;
use subtle::ConstantTimeEq;

use stheno_payment::*;
use stheno_types::{Error, Result};

type HmacSha256 = Hmac<Sha256>;

pub struct StripePaymentProvider {
    id: String,
    api_key: String,
    webhook_secret: String,
    base_url: String,
    is_livemode: bool,
    client: Client,
}

impl StripePaymentProvider {
    pub fn new(
        api_key: &str,
        webhook_secret: &str,
        base_url: Option<&str>,
        is_livemode: bool,
    ) -> Self {
        Self {
            id: "pp_stripe_stripe".to_string(),
            api_key: api_key.to_string(),
            webhook_secret: webhook_secret.to_string(),
            base_url: base_url
                .unwrap_or("https://api.stripe.com")
                .trim_end_matches('/')
                .to_string(),
            is_livemode,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }
}

impl Default for StripePaymentProvider {
    fn default() -> Self {
        Self::new("sk_test_mock", "whsec_mock", None, false)
    }
}

#[async_trait]
impl PaymentProvider for StripePaymentProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput> {
        let minor_amount = to_minor_units(i.amount, &i.currency_code)?;
        let url = format!("{}/v1/payment_intents", self.base_url);

        let params = [
            ("amount", minor_amount.to_string()),
            ("currency", i.currency_code.to_lowercase()),
            ("capture_method", "manual".to_string()),
        ];

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe initiate network error: {}", e).into()))?;

        let status_code = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe initiate response: {}", e).into())
        })?;

        if !status_code.is_success() {
            return Err(Error::Invalid {
                code: "stripe_initiate_failed",
                message: body["error"]["message"]
                    .as_str()
                    .unwrap_or("Stripe initiate failed")
                    .to_string(),
                field: None,
            });
        }

        let intent_id = body["id"].as_str().unwrap_or_default().to_string();
        let stripe_status = body["status"].as_str().unwrap_or("requires_payment_method");
        let status = map_stripe_status_to_session(stripe_status);

        Ok(InitiateOutput {
            id: intent_id,
            status,
            data: body,
        })
    }

    async fn update(&self, i: UpdateInput) -> Result<UpdateOutput> {
        let minor_amount = to_minor_units(i.amount, &i.currency_code)?;
        let url = format!("{}/v1/payment_intents/{}", self.base_url, i.session_id);

        let params = [("amount", minor_amount.to_string())];

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe update network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe update response: {}", e).into())
        })?;

        let stripe_status = body["status"].as_str().unwrap_or("requires_payment_method");
        let status = map_stripe_status_to_session(stripe_status);

        Ok(UpdateOutput { status, data: body })
    }

    async fn delete(&self, i: DeleteInput) -> Result<()> {
        let url = format!(
            "{}/v1/payment_intents/{}/cancel",
            self.base_url, i.session_id
        );
        let _ = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await;
        Ok(())
    }

    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput> {
        let url = format!(
            "{}/v1/payment_intents/{}/confirm",
            self.base_url, i.session_id
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe authorize network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe authorize response: {}", e).into())
        })?;

        let stripe_status = body["status"].as_str().unwrap_or("requires_payment_method");
        let status = map_stripe_status_to_session(stripe_status);

        Ok(AuthorizeOutput { status, data: body })
    }

    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput> {
        let minor_amount = to_minor_units(i.amount, &i.currency_code)?;
        let url = format!(
            "{}/v1/payment_intents/{}/capture",
            self.base_url, i.payment_id
        );

        let params = [("amount_to_capture", minor_amount.to_string())];

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe capture network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe capture response: {}", e).into())
        })?;

        Ok(CaptureOutput { data: body })
    }

    async fn refund(&self, i: RefundInput) -> Result<RefundOutput> {
        let minor_amount = to_minor_units(i.amount, &i.currency_code)?;
        let url = format!("{}/v1/refunds", self.base_url);

        let mut params = vec![
            ("payment_intent", i.payment_id),
            ("amount", minor_amount.to_string()),
        ];
        if let Some(note) = &i.note {
            params.push(("reason", note.clone()));
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe refund network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe refund response: {}", e).into())
        })?;

        Ok(RefundOutput { data: body })
    }

    async fn cancel(&self, i: CancelInput) -> Result<CancelOutput> {
        let url = format!(
            "{}/v1/payment_intents/{}/cancel",
            self.base_url, i.payment_id
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe cancel network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe cancel response: {}", e).into())
        })?;

        Ok(CancelOutput { data: body })
    }

    async fn retrieve(&self, i: RetrieveInput) -> Result<RetrieveOutput> {
        let url = format!("{}/v1/payment_intents/{}", self.base_url, i.reference_id);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Stripe retrieve network error: {}", e).into()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe retrieve response: {}", e).into())
        })?;

        let stripe_status = body["status"].as_str().unwrap_or("requires_payment_method");
        let status = map_stripe_status_to_session(stripe_status);
        let currency = body["currency"].as_str().unwrap_or("usd").to_string();
        let minor_amount = body["amount"].as_i64().unwrap_or(0);
        let amount = from_minor_units(minor_amount, &currency);

        Ok(RetrieveOutput {
            status,
            amount,
            currency_code: currency,
            data: body,
        })
    }

    async fn get_status(&self, i: StatusInput) -> Result<PaymentSessionStatus> {
        let ret = self
            .retrieve(RetrieveInput {
                reference_id: i.reference_id,
                data: i.data,
            })
            .await?;
        Ok(ret.status)
    }

    /// Webhook Signature Verification with 5-minute tolerance & livemode check (PAYMENTS §7.1, E10, E13, E14).
    fn verify_webhook(&self, raw: &[u8], signature_header: Option<&str>) -> Result<VerifiedEvent> {
        let sig_header = signature_header.ok_or_else(|| Error::Invalid {
            code: "missing_signature_header",
            message: "Missing Stripe-Signature header".to_string(),
            field: None,
        })?;

        // Parse t=timestamp,v1=signature
        let mut timestamp: Option<i64> = None;
        let mut signatures = Vec::new();

        for item in sig_header.split(',') {
            let parts: Vec<&str> = item.trim().splitn(2, '=').collect();
            if parts.len() == 2 {
                match parts[0] {
                    "t" => timestamp = parts[1].parse::<i64>().ok(),
                    "v1" => signatures.push(parts[1]),
                    _ => {}
                }
            }
        }

        let ts = timestamp.ok_or_else(|| Error::Invalid {
            code: "invalid_signature_timestamp",
            message: "Stripe-Signature header missing timestamp".to_string(),
            field: None,
        })?;

        // 5-minute tolerance check (PAYMENTS §7.1, E10)
        let now_epoch = Utc::now().timestamp();
        if (now_epoch - ts).abs() > 300 {
            return Err(Error::Invalid {
                code: "webhook_timestamp_out_of_tolerance",
                message: format!(
                    "Webhook timestamp {} outside 300s window (now: {})",
                    ts, now_epoch
                ),
                field: None,
            });
        }

        // Verify HMAC-SHA256 signature
        let signed_payload = format!("{}.{}", ts, std::str::from_utf8(raw).unwrap_or_default());
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| Error::Internal(format!("Invalid HMAC secret key: {}", e).into()))?;
        mac.update(signed_payload.as_bytes());
        let expected_sig = hex::encode(mac.finalize().into_bytes());

        let sig_matched = signatures
            .iter()
            .any(|&sig| sig.as_bytes().ct_eq(expected_sig.as_bytes()).into());

        if !sig_matched {
            return Err(Error::Invalid {
                code: "invalid_webhook_signature",
                message: "Stripe webhook signature mismatch".to_string(),
                field: None,
            });
        }

        // Parse event JSON
        let payload: serde_json::Value =
            serde_json::from_slice(raw).map_err(|e| Error::Invalid {
                code: "invalid_json_payload",
                message: format!("Malformed webhook JSON: {}", e),
                field: None,
            })?;

        // Livemode check (PAYMENTS §7.5, E13)
        if let Some(event_livemode) = payload.get("livemode").and_then(|v| v.as_bool()) {
            if event_livemode != self.is_livemode {
                return Err(Error::Invalid {
                    code: "webhook_livemode_mismatch",
                    message: format!(
                        "Event livemode {} does not match server livemode {}",
                        event_livemode, self.is_livemode
                    ),
                    field: None,
                });
            }
        }

        let event_id = payload["id"].as_str().unwrap_or_default().to_string();
        let event_type = payload["type"].as_str().unwrap_or_default().to_string();
        let provider_created_at = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now);

        Ok(VerifiedEvent {
            provider_event_id: event_id,
            event_type,
            provider_created_at,
            payload,
        })
    }

    fn map_webhook(&self, event: &VerifiedEvent) -> Result<WebhookAction> {
        match event.event_type.as_str() {
            "payment_intent.amount_capturable_updated" => Ok(WebhookAction::Authorized),
            "payment_intent.succeeded" => Ok(WebhookAction::Captured),
            "payment_intent.payment_failed" => Ok(WebhookAction::Failed),
            "payment_intent.canceled" => Ok(WebhookAction::Canceled),
            "payment_intent.requires_action" => Ok(WebhookAction::RequiresMore),
            "charge.refunded" => Ok(WebhookAction::Captured),
            _ => Ok(WebhookAction::NotSupported),
        }
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            partial_capture: true,
            multi_capture: false,
            partial_refund: true,
            cancel_after_authorize: true,
            vault: true,
            async_settlement: false,
            authorization_ttl: Some(Duration::from_secs(7 * 24 * 3600)),
        }
    }
}

#[async_trait]
impl PaymentVault for StripePaymentProvider {
    async fn create_account_holder(
        &self,
        i: CreateAccountHolderInput,
    ) -> Result<AccountHolderOutput> {
        let url = format!("{}/v1/customers", self.base_url);
        let mut params = Vec::new();
        if let Some(email) = &i.email {
            params.push(("email", email.as_str()));
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe create customer network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe customer response: {}", e).into())
        })?;

        let id = body["id"].as_str().unwrap_or_default().to_string();
        Ok(AccountHolderOutput {
            external_id: id,
            email: i.email,
            data: Some(body),
        })
    }

    async fn update_account_holder(
        &self,
        i: UpdateAccountHolderInput,
    ) -> Result<AccountHolderOutput> {
        let url = format!("{}/v1/customers/{}", self.base_url, i.external_id);
        let mut params = Vec::new();
        if let Some(email) = &i.email {
            params.push(("email", email.as_str()));
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe update customer network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe update customer: {}", e).into())
        })?;

        Ok(AccountHolderOutput {
            external_id: i.external_id,
            email: i.email,
            data: Some(body),
        })
    }

    async fn retrieve_account_holder(
        &self,
        i: RetrieveAccountHolderInput,
    ) -> Result<AccountHolderOutput> {
        let url = format!("{}/v1/customers/{}", self.base_url, i.external_id);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe retrieve customer network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe retrieve customer: {}", e).into())
        })?;

        let email = body["email"].as_str().map(|s| s.to_string());
        Ok(AccountHolderOutput {
            external_id: i.external_id,
            email,
            data: Some(body),
        })
    }

    async fn delete_account_holder(&self, i: DeleteAccountHolderInput) -> Result<()> {
        let url = format!("{}/v1/customers/{}", self.base_url, i.external_id);
        let _ = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await;
        Ok(())
    }

    async fn list_payment_methods(&self, i: ListMethodsInput) -> Result<Vec<PaymentMethod>> {
        let url = format!(
            "{}/v1/customers/{}/payment_methods",
            self.base_url, i.account_holder_id
        );

        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe list payment methods network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe list payment methods: {}", e).into())
        })?;

        let mut list = Vec::new();
        if let Some(items) = body["data"].as_array() {
            for item in items {
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let mtype = item["type"].as_str().unwrap_or("card").to_string();
                let card = &item["card"];
                let last4 = card["last4"].as_str().map(|s| s.to_string());
                let exp_month = card["exp_month"].as_i64().map(|n| n as i32);
                let exp_year = card["exp_year"].as_i64().map(|n| n as i32);
                let brand = card["brand"].as_str().map(|s| s.to_string());

                list.push(PaymentMethod {
                    id,
                    method_type: mtype,
                    last4,
                    exp_month,
                    exp_year,
                    brand,
                    data: Some(item.clone()),
                });
            }
        }
        Ok(list)
    }

    async fn save_payment_method(&self, i: SaveMethodInput) -> Result<PaymentMethod> {
        let url = format!(
            "{}/v1/payment_methods/{}/attach",
            self.base_url, i.payment_method_token
        );
        let params = [("customer", i.account_holder_id)];

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                Error::Internal(format!("Stripe save payment method network error: {}", e).into())
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Stripe attach method: {}", e).into())
        })?;

        let id = body["id"].as_str().unwrap_or_default().to_string();
        let mtype = body["type"].as_str().unwrap_or("card").to_string();
        let card = &body["card"];
        let last4 = card["last4"].as_str().map(|s| s.to_string());
        let exp_month = card["exp_month"].as_i64().map(|n| n as i32);
        let exp_year = card["exp_year"].as_i64().map(|n| n as i32);
        let brand = card["brand"].as_str().map(|s| s.to_string());

        Ok(PaymentMethod {
            id,
            method_type: mtype,
            last4,
            exp_month,
            exp_year,
            brand,
            data: Some(body),
        })
    }

    async fn delete_payment_method(&self, i: DeleteMethodInput) -> Result<()> {
        let url = format!(
            "{}/v1/payment_methods/{}/detach",
            self.base_url, i.payment_method_id
        );
        let _ = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await;
        Ok(())
    }
}

fn map_stripe_status_to_session(stripe_status: &str) -> PaymentSessionStatus {
    match stripe_status {
        "requires_payment_method" => PaymentSessionStatus::Pending,
        "requires_confirmation" | "processing" => PaymentSessionStatus::PendingAuthorization,
        "requires_action" => PaymentSessionStatus::RequiresMore,
        "requires_capture" => PaymentSessionStatus::Authorized,
        "succeeded" => PaymentSessionStatus::Captured,
        "canceled" => PaymentSessionStatus::Canceled,
        _ => PaymentSessionStatus::Error,
    }
}
