//! Tenant-aware WhatsApp Cloud API delivery for durable notification intents.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use stheno_ai::AiConfigurationService;
use stheno_notification::{
    NotificationProvider, NotificationProviderResult, NotificationService, WhatsAppConsentChecker,
};
use stheno_platform::PlatformService;
use stheno_types::{Error, Result};

#[derive(Clone)]
struct SenderCredentials {
    phone_number_id: String,
    access_token: String,
}

/// Routes Free traffic through Vendryx's central WABA and entitled Pro traffic through the
/// merchant's own configured WABA. It requires server-authored consent and approved-template
/// metadata on the durable intent; neither is inferred from the recipient phone number.
pub struct RoutedWhatsAppNotificationProvider {
    client: reqwest::Client,
    configurations: Arc<AiConfigurationService>,
    platform: Arc<PlatformService>,
    consent: WhatsAppConsentChecker,
    central_sender: Option<SenderCredentials>,
    graph_version: String,
}

impl RoutedWhatsAppNotificationProvider {
    pub fn new(
        configurations: Arc<AiConfigurationService>,
        platform: Arc<PlatformService>,
        notifications: &NotificationService,
    ) -> Self {
        let central_sender = match (
            non_empty_env("WHATSAPP_CENTRAL_PHONE_NUMBER_ID"),
            non_empty_env("WHATSAPP_CENTRAL_ACCESS_TOKEN"),
        ) {
            (Some(phone_number_id), Some(access_token))
                if valid_phone_number_id(&phone_number_id) =>
            {
                Some(SenderCredentials {
                    phone_number_id,
                    access_token,
                })
            }
            _ => None,
        };
        let graph_version = std::env::var("STHENO_META_GRAPH_API_VERSION")
            .ok()
            .filter(|value| valid_graph_version(value))
            .unwrap_or_else(|| "v21.0".to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            configurations,
            platform,
            consent: notifications.whatsapp_consent_checker(),
            central_sender,
            graph_version,
        }
    }

    async fn resolve_sender(&self, tenant_id: &str) -> Result<SenderCredentials> {
        let subscription = self.platform.get_subscription(tenant_id).await?;
        let now = chrono::Utc::now();
        let pro_entitled = match subscription.tier.as_str() {
            "trial" => {
                matches!(subscription.status.as_str(), "active" | "trial")
                    && subscription
                        .trial_ends_at
                        .is_some_and(|ends_at| now < ends_at)
            }
            "pro" => {
                matches!(
                    subscription.status.as_str(),
                    "active" | "grace" | "past_due"
                ) && subscription
                    .term_ends_at
                    .map_or(true, |ends_at| now < ends_at + chrono::Duration::days(7))
            }
            _ => false,
        };

        if pro_entitled {
            let sender = self
                .configurations
                .load_enabled_whatsapp_sender(tenant_id)
                .await?
                .ok_or_else(|| {
                    invalid(
                        "Connect an active WhatsApp Business account to send Pro notifications.",
                    )
                })?;
            if !valid_phone_number_id(&sender.phone_number_id) {
                return Err(invalid(
                    "The configured WhatsApp sender has an invalid phone number ID.",
                ));
            }
            return Ok(SenderCredentials {
                phone_number_id: sender.phone_number_id,
                access_token: sender.access_token,
            });
        }

        self.central_sender
            .clone()
            .ok_or_else(|| invalid("Central WhatsApp notifications are not configured."))
    }

    async fn send_template(
        &self,
        tenant_id: &str,
        sender: SenderCredentials,
        to: &str,
        data: Option<&Value>,
    ) -> Result<NotificationProviderResult> {
        let data = data.ok_or_else(|| invalid("WhatsApp template metadata is required."))?;
        let template_name = data
            .get("whatsapp_template")
            .and_then(Value::as_str)
            .filter(|value| valid_template_name(value))
            .ok_or_else(|| invalid("An approved WhatsApp template name is required."))?;
        let language = data
            .get("whatsapp_language")
            .and_then(Value::as_str)
            .filter(|value| valid_language(value))
            .ok_or_else(|| invalid("A valid WhatsApp template language is required."))?;
        let parameters = data
            .get("whatsapp_parameters")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("WhatsApp template parameters are required."))?;
        if parameters.len() > 20
            || parameters.iter().any(|value| {
                value.as_str().map_or(true, |text| {
                    text.len() > 1024 || text.chars().any(char::is_control)
                })
            })
        {
            return Err(invalid(
                "WhatsApp template parameters are outside the allowed bounds.",
            ));
        }
        let e164 = normalize_e164(to)?;
        let body_parameters: Vec<Value> = parameters
            .iter()
            .map(|value| json!({ "type": "text", "text": value }))
            .collect();
        let mut template = json!({
            "name": template_name,
            "language": { "code": language }
        });
        if !body_parameters.is_empty() {
            template["components"] = json!([{
                "type": "body",
                "parameters": body_parameters
            }]);
        }
        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.graph_version, sender.phone_number_id
        );
        if !self.consent.has_consent(tenant_id, to).await? {
            return Err(invalid("WhatsApp consent is absent or has been revoked."));
        }
        let response = self
            .client
            .post(url)
            .bearer_auth(sender.access_token)
            .json(&json!({
                "messaging_product": "whatsapp",
                "to": e164,
                "type": "template",
                "template": template
            }))
            .send()
            .await
            .map_err(meta_error)?;
        let status = response.status();
        if !status.is_success() {
            return Err(meta_status_error(status.as_u16()));
        }
        let response: MetaSendResponse = response.json().await.map_err(meta_error)?;
        let message_id = response
            .messages
            .into_iter()
            .next()
            .map(|message| message.id)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| meta_status_error(200))?;
        Ok(NotificationProviderResult {
            provider_message_id: Some(message_id.clone()),
            metadata: Some(json!({
                "provider": "meta_whatsapp",
                "message_id": message_id,
            })),
        })
    }
}

#[async_trait]
impl NotificationProvider for RoutedWhatsAppNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "meta_whatsapp"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["whatsapp"]
    }

    async fn send(
        &self,
        _to: &str,
        _channel: &str,
        _subject: Option<&str>,
        _body: &str,
        _data: Option<&Value>,
    ) -> Result<NotificationProviderResult> {
        Err(invalid(
            "WhatsApp notifications require a tenant-scoped durable intent.",
        ))
    }

    async fn send_for_tenant(
        &self,
        tenant_id: &str,
        to: &str,
        channel: &str,
        _subject: Option<&str>,
        _body: &str,
        data: Option<&Value>,
        _idempotency_key: Option<&str>,
    ) -> Result<NotificationProviderResult> {
        if channel != "whatsapp" {
            return Err(invalid(
                "The Meta WhatsApp provider only supports WhatsApp.",
            ));
        }
        let sender = self.resolve_sender(tenant_id).await?;
        self.send_template(tenant_id, sender, to, data).await
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn valid_graph_version(value: &str) -> bool {
    value.len() <= 12
        && value.starts_with('v')
        && value[1..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
}

fn valid_template_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_language(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn valid_phone_number_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 32 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn normalize_e164(value: &str) -> Result<String> {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|character| !matches!(character, ' ' | '-' | '(' | ')'))
        .collect();
    let digits = normalized.strip_prefix('+').unwrap_or(&normalized);
    if normalized.starts_with('+')
        && (8..=15).contains(&digits.len())
        && digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        Ok(digits.to_string())
    } else {
        Err(invalid(
            "WhatsApp recipients must be valid E.164 phone numbers.",
        ))
    }
}

#[derive(Deserialize)]
struct MetaSendResponse {
    #[serde(default)]
    messages: Vec<MetaMessage>,
}

#[derive(Deserialize)]
struct MetaMessage {
    id: String,
}

fn invalid(message: &'static str) -> Error {
    Error::Invalid {
        code: "whatsapp_notification_invalid",
        message: message.to_string(),
        field: None,
    }
}

fn meta_error(error: reqwest::Error) -> Error {
    Error::ProviderFailed {
        provider: "meta_whatsapp".to_string(),
        source: Box::new(error),
    }
}

fn meta_status_error(status: u16) -> Error {
    Error::ProviderFailed {
        provider: "meta_whatsapp".to_string(),
        source: Box::new(std::io::Error::other(format!(
            "Meta Graph API returned HTTP {status}"
        ))),
    }
}
