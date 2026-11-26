//! Verified, bounded Meta webhook ingress for WhatsApp, Messenger, and Instagram.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use stheno_ai::WebhookEventService;
use stheno_types::{Error, Result};

const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_EVENTS_PER_DELIVERY: usize = 50;
const MAX_TEXT_BYTES: usize = 4_000;
const RATE_LIMIT_PER_MINUTE: u32 = 60;
const MAX_RATE_LIMIT_KEYS: usize = 4096;
type HmacSha256 = Hmac<Sha256>;

/// Tenant/channel credentials returned only after resolving an opaque webhook path ID.
///
/// Secret fields are deliberately excluded from `Debug` and only used for verification.
#[derive(Clone)]
pub struct MetaWebhookRegistration {
    pub tenant_id: String,
    pub channel: String,
    pub account_id: String,
    pub app_secret: String,
    pub verify_token: String,
    pub access_token: String,
}

/// Resolves a random public webhook identifier to its tenant-owned configuration.
#[async_trait]
pub trait MetaWebhookRegistry: Send + Sync {
    /// Returns `None` for unknown or disabled webhook IDs.
    async fn resolve(&self, webhook_id: &str) -> Result<Option<MetaWebhookRegistration>>;
}

/// Normalized inbound text message passed to the tenant conversation queue.
#[derive(Clone)]
pub struct MetaInboundMessage {
    pub channel: String,
    pub account_id: String,
    pub event_id: String,
    pub sender_id: String,
    pub created_at: DateTime<Utc>,
    pub text: String,
    pub access_token: String,
}

/// Idempotent sink. Implementations enqueue by `(tenant, channel, account, event_id)`.
#[async_trait]
pub trait MetaInboundHandler: Send + Sync {
    /// Persists/enqueues the event without sending an automatic commerce commitment.
    async fn handle(&self, tenant_id: &str, message: MetaInboundMessage) -> Result<()>;
}

/// Ingress dependencies; no tenant secret is stored in Axum router state directly.
#[derive(Clone)]
pub struct MetaWebhookContext {
    registry: Arc<dyn MetaWebhookRegistry>,
    events: Arc<WebhookEventService>,
    handler: Arc<dyn MetaInboundHandler>,
    limiter: Arc<RateLimiter>,
}

impl MetaWebhookContext {
    /// Creates an ingress context with a local per-hook rate window.
    pub fn new(
        registry: Arc<dyn MetaWebhookRegistry>,
        events: Arc<WebhookEventService>,
        handler: Arc<dyn MetaInboundHandler>,
    ) -> Self {
        Self {
            registry,
            events,
            handler,
            limiter: Arc::new(RateLimiter::default()),
        }
    }
}

#[derive(Default)]
struct RateLimiter {
    windows: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    fn allow(&self, key: &str) -> bool {
        let Ok(mut windows) = self.windows.lock() else {
            return false;
        };
        let now = Instant::now();
        if let Some((started, count)) = windows.get_mut(key) {
            if now.duration_since(*started) >= Duration::from_secs(60) {
                *started = now;
                *count = 1;
                return true;
            }
            if *count >= RATE_LIMIT_PER_MINUTE {
                return false;
            }
            *count += 1;
            return true;
        }
        if windows.len() >= MAX_RATE_LIMIT_KEYS {
            windows
                .retain(|_, (started, _)| now.duration_since(*started) < Duration::from_secs(60));
            if windows.len() >= MAX_RATE_LIMIT_KEYS {
                return false;
            }
        }
        windows.insert(key.to_string(), (now, 1));
        true
    }
}

/// Mounts GET verification and POST event routes with a strict body ceiling.
pub fn meta_webhook_routes(context: MetaWebhookContext) -> Router {
    Router::new()
        .route(
            "/webhooks/meta/{webhook_id}",
            get(verify_webhook).post(receive_webhook),
        )
        .route_layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(context)
}

#[derive(Deserialize)]
struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: String,
    #[serde(rename = "hub.verify_token")]
    verify_token: String,
    #[serde(rename = "hub.challenge")]
    challenge: String,
}

async fn verify_webhook(
    State(context): State<MetaWebhookContext>,
    Path(webhook_id): Path<String>,
    Query(query): Query<VerifyQuery>,
) -> std::result::Result<(StatusCode, String), StatusCode> {
    if !valid_webhook_id(&webhook_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    if !context.limiter.allow(&webhook_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    let registration = resolve_registration(&context, &webhook_id).await?;
    if query.mode != "subscribe"
        || query.verify_token.len() > 4096
        || query.challenge.is_empty()
        || query.challenge.len() > 256
        || !verify_token(&registration.verify_token, &webhook_id, &query.verify_token)
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok((StatusCode::OK, query.challenge))
}

async fn receive_webhook(
    State(context): State<MetaWebhookContext>,
    Path(webhook_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> std::result::Result<(StatusCode, Json<Value>), StatusCode> {
    if !valid_webhook_id(&webhook_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    if body.len() > MAX_BODY_BYTES || !context.limiter.allow(&webhook_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    let registration = resolve_registration(&context, &webhook_id).await?;
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    verify_signature(&registration.app_secret, signature, &body)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let payload: Value = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut messages =
        parse_messages(&registration, &payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    for message in &mut messages {
        message.access_token = registration.access_token.clone();
    }
    for message in messages {
        let claimed = context
            .events
            .claim_message(
                &registration.tenant_id,
                &message.channel,
                &message.account_id,
                &message.event_id,
                &message.sender_id,
                message.created_at,
            )
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if !claimed {
            continue;
        }
        match context
            .handler
            .handle(&registration.tenant_id, message.clone())
            .await
        {
            Ok(()) => context
                .events
                .mark_processed(
                    &registration.tenant_id,
                    &message.channel,
                    &message.account_id,
                    &message.event_id,
                )
                .await
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?,
            Err(_) => {
                let _ = context
                    .events
                    .mark_failed(
                        &registration.tenant_id,
                        &message.channel,
                        &message.account_id,
                        &message.event_id,
                    )
                    .await;
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
        }
    }
    Ok((StatusCode::OK, Json(serde_json::json!({"received": true}))))
}

async fn resolve_registration(
    context: &MetaWebhookContext,
    webhook_id: &str,
) -> std::result::Result<MetaWebhookRegistration, StatusCode> {
    if !valid_webhook_id(webhook_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    let registration = context
        .registry
        .resolve(webhook_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .ok_or(StatusCode::NOT_FOUND)?;
    stheno_db::tenant::validate(&registration.tenant_id).map_err(|_| StatusCode::NOT_FOUND)?;
    if !matches!(
        registration.channel.as_str(),
        "whatsapp" | "messenger" | "instagram"
    ) || registration.account_id.trim().is_empty()
        || registration.account_id.len() > 128
        || registration.app_secret.trim().is_empty()
        || registration.app_secret.len() > 4096
        || registration.verify_token.trim().is_empty()
        || registration.verify_token.len() > 4096
        || registration.access_token.trim().is_empty()
        || registration.access_token.len() > 8192
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(registration)
}

fn valid_webhook_id(webhook_id: &str) -> bool {
    (16..=128).contains(&webhook_id.len())
        && webhook_id
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'-' || value == b'_')
}

fn verify_token(configured: &str, webhook_id: &str, supplied: &str) -> bool {
    let Ok(mut expected) = HmacSha256::new_from_slice(configured.as_bytes()) else {
        return false;
    };
    expected.update(webhook_id.as_bytes());
    let expected = expected.finalize().into_bytes();
    let Ok(mut actual) = HmacSha256::new_from_slice(supplied.as_bytes()) else {
        return false;
    };
    actual.update(webhook_id.as_bytes());
    actual.verify_slice(&expected).is_ok()
}

fn verify_signature(secret: &str, signature: &str, body: &[u8]) -> Result<()> {
    let encoded = signature
        .strip_prefix("sha256=")
        .ok_or_else(signature_error)?;
    if encoded.len() != 64 {
        return Err(signature_error());
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in encoded.as_bytes().chunks_exact(2).enumerate() {
        decoded[index] = (hex(pair[0]).ok_or_else(signature_error)? << 4)
            | hex(pair[1]).ok_or_else(signature_error)?;
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| signature_error())?;
    mac.update(body);
    mac.verify_slice(&decoded).map_err(|_| signature_error())
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn signature_error() -> Error {
    Error::Unauthorized("invalid Meta webhook signature".to_string())
}

fn parse_messages(
    registration: &MetaWebhookRegistration,
    payload: &Value,
) -> Result<Vec<MetaInboundMessage>> {
    let object = payload
        .get("object")
        .and_then(Value::as_str)
        .ok_or_else(invalid_payload)?;
    let expected_object = match registration.channel.as_str() {
        "whatsapp" => "whatsapp_business_account",
        "messenger" => "page",
        "instagram" => "instagram",
        _ => return Err(invalid_payload()),
    };
    if object != expected_object {
        return Err(invalid_payload());
    }
    let entries = payload
        .get("entry")
        .and_then(Value::as_array)
        .ok_or_else(invalid_payload)?;
    if entries.is_empty() || entries.len() > 20 {
        return Err(invalid_payload());
    }
    let mut messages = Vec::new();
    for entry in entries {
        let entry_id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(invalid_payload)?;
        if registration.channel != "whatsapp" && entry_id != registration.account_id {
            return Err(invalid_payload());
        }
        if registration.channel == "whatsapp" {
            let changes = entry
                .get("changes")
                .and_then(Value::as_array)
                .ok_or_else(invalid_payload)?;
            if changes.len() > MAX_EVENTS_PER_DELIVERY {
                return Err(invalid_payload());
            }
            for change in changes {
                let value = change.get("value").ok_or_else(invalid_payload)?;
                let phone_id = value
                    .pointer("/metadata/phone_number_id")
                    .and_then(Value::as_str);
                if phone_id != Some(&registration.account_id) {
                    return Err(invalid_payload());
                }
                if let Some(batch) = value.get("messages").and_then(Value::as_array) {
                    for item in batch {
                        messages.push(parse_whatsapp_message(registration, item)?);
                        if messages.len() > MAX_EVENTS_PER_DELIVERY {
                            return Err(invalid_payload());
                        }
                    }
                }
            }
        } else if let Some(batch) = entry.get("messaging").and_then(Value::as_array) {
            if batch.len() > MAX_EVENTS_PER_DELIVERY {
                return Err(invalid_payload());
            }
            for item in batch {
                let Some(message) = item.get("message") else {
                    continue;
                };
                if message.get("is_echo").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                let Some(text) = message.get("text").and_then(Value::as_str) else {
                    continue;
                };
                messages.push(MetaInboundMessage {
                    channel: registration.channel.clone(),
                    account_id: registration.account_id.clone(),
                    event_id: required_id(message.get("mid"))?,
                    sender_id: required_id(item.pointer("/sender/id"))?,
                    created_at: parse_timestamp(item.get("timestamp"))?,
                    text: validate_text(text)?,
                    access_token: String::new(),
                });
                if messages.len() > MAX_EVENTS_PER_DELIVERY {
                    return Err(invalid_payload());
                }
            }
        }
    }
    Ok(messages)
}

fn parse_whatsapp_message(
    registration: &MetaWebhookRegistration,
    item: &Value,
) -> Result<MetaInboundMessage> {
    let text = item
        .pointer("/text/body")
        .and_then(Value::as_str)
        .ok_or_else(invalid_payload)?;
    let sender_id = required_id(item.get("from"))?;
    Ok(MetaInboundMessage {
        channel: registration.channel.clone(),
        account_id: registration.account_id.clone(),
        event_id: required_id(item.get("id"))?,
        sender_id,
        created_at: parse_timestamp(item.get("timestamp"))?,
        text: validate_text(text)?,
        access_token: String::new(),
    })
}

fn required_id(value: Option<&Value>) -> Result<String> {
    let value = value.and_then(Value::as_str).ok_or_else(invalid_payload)?;
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(invalid_payload());
    }
    Ok(value.to_string())
}

fn parse_timestamp(value: Option<&Value>) -> Result<DateTime<Utc>> {
    let timestamp = value
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_str()?.parse::<i64>().ok())
        })
        .ok_or_else(invalid_payload)?;
    let seconds = if timestamp > 100_000_000_000 {
        timestamp / 1000
    } else {
        timestamp
    };
    Utc.timestamp_opt(seconds, 0)
        .single()
        .ok_or_else(invalid_payload)
}

fn validate_text(text: &str) -> Result<String> {
    if text.trim().is_empty()
        || text.len() > MAX_TEXT_BYTES
        || text.chars().any(|value| value == '\0')
    {
        return Err(invalid_payload());
    }
    Ok(text.to_string())
}

fn invalid_payload() -> Error {
    Error::Invalid {
        code: "invalid_meta_webhook_payload",
        message: "Meta webhook payload is malformed or unsupported.".to_string(),
        field: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_verification_uses_exact_body_and_rejects_bad_hex() {
        let body = br#"{"object":"page"}"#;
        let mut mac = HmacSha256::new_from_slice(b"test-secret").expect("HMAC key");
        mac.update(body);
        let signature = format!(
            "sha256={}",
            mac.finalize()
                .into_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        assert!(verify_signature("test-secret", &signature, body).is_ok());
        assert!(verify_signature("test-secret", &signature, b"changed").is_err());
        assert!(verify_signature("test-secret", "sha256=not-hex", body).is_err());
    }

    #[test]
    fn per_hook_rate_limit_is_bounded_and_fails_closed_at_the_limit() {
        let limiter = RateLimiter::default();
        for _ in 0..RATE_LIMIT_PER_MINUTE {
            assert!(limiter.allow("hook-a"));
        }
        assert!(!limiter.allow("hook-a"));
        assert!(limiter.allow("hook-b"));
        let capacity = RateLimiter::default();
        for index in 0..MAX_RATE_LIMIT_KEYS {
            assert!(capacity.allow(&format!("hook-{index}")));
        }
        assert!(!capacity.allow("hook-overflow"));
    }
}
