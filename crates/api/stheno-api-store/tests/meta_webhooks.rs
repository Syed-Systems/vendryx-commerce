//! Signed Meta webhook contract: tenant/account isolation, replay, ordering, and retries.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use stheno_ai::WebhookEventService;
use stheno_api_store::social_webhook::{
    meta_webhook_routes, MetaInboundHandler, MetaInboundMessage, MetaWebhookContext,
    MetaWebhookRegistration, MetaWebhookRegistry,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_types::{Error, Result};
use tower::ServiceExt;

type HmacSha256 = Hmac<Sha256>;
const HOOK_ID: &str = "opaque-hook-identifier-123";
const SECRET: &str = "meta-app-secret";

struct TestRegistry {
    channel: String,
    account_id: String,
}

#[async_trait::async_trait]
impl MetaWebhookRegistry for TestRegistry {
    async fn resolve(&self, webhook_id: &str) -> Result<Option<MetaWebhookRegistration>> {
        Ok((webhook_id == HOOK_ID).then(|| MetaWebhookRegistration {
            tenant_id: "ten_meta_test".to_string(),
            channel: self.channel.clone(),
            account_id: self.account_id.clone(),
            app_secret: SECRET.to_string(),
            verify_token: "verify-secret".to_string(),
            access_token: "meta-access-token".to_string(),
        }))
    }
}

#[derive(Default)]
struct TestHandler {
    seen: Mutex<Vec<String>>,
    fail_once: AtomicBool,
}

#[async_trait::async_trait]
impl MetaInboundHandler for TestHandler {
    async fn handle(&self, _tenant_id: &str, message: MetaInboundMessage) -> Result<()> {
        if self.fail_once.swap(false, Ordering::SeqCst) {
            return Err(Error::Internal(Box::new(std::io::Error::other(
                "temporary handler outage",
            ))));
        }
        self.seen.lock().expect("seen lock").push(message.event_id);
        Ok(())
    }
}

fn app(db_url: &str, handler: Arc<TestHandler>) -> axum::Router {
    app_for(db_url, handler, "messenger", "page_1")
}

fn app_for(
    db_url: &str,
    handler: Arc<TestHandler>,
    channel: &str,
    account_id: &str,
) -> axum::Router {
    let events = Arc::new(WebhookEventService::new(
        create_pool(db_url, 5).expect("pool"),
    ));
    meta_webhook_routes(MetaWebhookContext::new(
        Arc::new(TestRegistry {
            channel: channel.to_string(),
            account_id: account_id.to_string(),
        }),
        events,
        handler,
    ))
}

fn body_for(event_id: &str, sender: &str, timestamp: i64, account: &str) -> String {
    json!({
        "object":"page",
        "entry":[{
            "id":account,
            "messaging":[{
                "sender":{"id":sender},
                "recipient":{"id":account},
                "timestamp":timestamp,
                "message":{"mid":event_id,"text":"Is the blue lamp available?"}
            }]
        }]
    })
    .to_string()
}

fn signature(body: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(SECRET.as_bytes()).expect("HMAC secret");
    mac.update(body.as_bytes());
    format!("sha256={}", hex(&mac.finalize().into_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

async fn post(app: axum::Router, body: String, signature: &str) -> StatusCode {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/webhooks/meta/{HOOK_ID}"))
            .header("content-type", "application/json")
            .header("x-hub-signature-256", signature)
            .body(Body::from(body))
            .expect("request"),
    )
    .await
    .expect("response")
    .status()
}

#[tokio::test]
async fn signed_messages_are_tenant_checked_deduplicated_and_ordered() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    let app = app(&db.url, handler.clone());

    let newer = body_for("m_new", "buyer_1", 1_800_000_000, "page_1");
    assert_eq!(
        post(app.clone(), newer.clone(), &signature(&newer)).await,
        StatusCode::OK
    );
    assert_eq!(
        post(app.clone(), newer.clone(), &signature(&newer)).await,
        StatusCode::OK
    );
    let older = body_for("m_old", "buyer_1", 1_700_000_000, "page_1");
    assert_eq!(
        post(app.clone(), older.clone(), &signature(&older)).await,
        StatusCode::OK
    );
    assert_eq!(
        handler.seen.lock().expect("seen lock").as_slice(),
        ["m_new"]
    );

    let mismatch = body_for("m_wrong_page", "buyer_1", 1_810_000_000, "page_other");
    assert_eq!(
        post(app, mismatch.clone(), &signature(&mismatch)).await,
        StatusCode::BAD_REQUEST
    );
    db.drop().await;
}

#[tokio::test]
async fn get_verification_requires_matching_token_and_returns_only_challenge() {
    let db = TestDb::migrated().await;
    let app = app(&db.url, Arc::new(TestHandler::default()));
    let valid = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/webhooks/meta/{HOOK_ID}?hub.mode=subscribe&hub.verify_token=verify-secret&hub.challenge=challenge-123"
                ))
                .body(Body::empty())
                .expect("verification request"),
        )
        .await
        .expect("verification response");
    assert_eq!(valid.status(), StatusCode::OK);

    let invalid = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/webhooks/meta/{HOOK_ID}?hub.mode=subscribe&hub.verify_token=wrong&hub.challenge=challenge-123"
                ))
                .body(Body::empty())
                .expect("invalid verification request"),
        )
        .await
        .expect("invalid verification response");
    assert_eq!(invalid.status(), StatusCode::NOT_FOUND);
    db.drop().await;
}

#[tokio::test]
async fn whatsapp_phone_account_is_checked_before_message_handoff() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    let app = app_for(&db.url, handler.clone(), "whatsapp", "phone_1");
    let payload = json!({
        "object":"whatsapp_business_account",
        "entry":[{
            "id":"waba_1",
            "changes":[{
                "field":"messages",
                "value":{
                    "metadata":{"phone_number_id":"phone_1"},
                    "messages":[{
                        "from":"buyer_3",
                        "id":"wamid_3",
                        "timestamp":"1800000000",
                        "type":"text",
                        "text":{"body":"Hello from WhatsApp"}
                    }]
                }
            }]
        }]
    })
    .to_string();
    assert_eq!(
        post(app.clone(), payload.clone(), &signature(&payload)).await,
        StatusCode::OK
    );
    assert_eq!(
        handler.seen.lock().expect("seen lock").as_slice(),
        ["wamid_3"]
    );

    let wrong_account = payload.replace("phone_1", "phone_other");
    assert_eq!(
        post(app, wrong_account.clone(), &signature(&wrong_account)).await,
        StatusCode::BAD_REQUEST
    );
    db.drop().await;
}

#[tokio::test]
async fn instagram_message_payloads_use_the_linked_account_id() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    let app = app_for(&db.url, handler.clone(), "instagram", "ig_account_1");
    let payload = json!({
        "object":"instagram",
        "entry":[{
            "id":"ig_account_1",
            "messaging":[{
                "sender":{"id":"ig_buyer"},
                "recipient":{"id":"ig_account_1"},
                "timestamp":1800000000000_i64,
                "message":{"mid":"ig_mid_1","text":"Hello on Instagram"}
            }]
        }]
    })
    .to_string();
    assert_eq!(
        post(app, payload.clone(), &signature(&payload)).await,
        StatusCode::OK
    );
    assert_eq!(
        handler.seen.lock().expect("seen lock").as_slice(),
        ["ig_mid_1"]
    );
    db.drop().await;
}

#[tokio::test]
async fn bad_signatures_are_rejected_and_failed_handoffs_are_retryable() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    handler.fail_once.store(true, Ordering::SeqCst);
    let app = app(&db.url, handler.clone());
    let body = body_for("m_retry", "buyer_2", 1_800_000_000, "page_1");

    assert_eq!(
        post(app.clone(), body.clone(), "sha256=00").await,
        StatusCode::UNAUTHORIZED
    );
    let signature = signature(&body);
    assert_eq!(
        post(app.clone(), body.clone(), &signature).await,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(post(app, body.clone(), &signature).await, StatusCode::OK);
    assert_eq!(
        handler.seen.lock().expect("seen lock").as_slice(),
        ["m_retry"]
    );
    db.drop().await;
}

#[tokio::test]
async fn malformed_unsupported_and_oversized_deliveries_fail_closed() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    let app = app(&db.url, handler.clone());
    let malformed = "not-json".to_string();
    assert_eq!(
        post(app.clone(), malformed.clone(), &signature(&malformed)).await,
        StatusCode::BAD_REQUEST
    );
    let unsupported = json!({"object":"unsupported","entry":[]}).to_string();
    assert_eq!(
        post(app.clone(), unsupported.clone(), &signature(&unsupported)).await,
        StatusCode::BAD_REQUEST
    );

    let oversized = "x".repeat(65 * 1024);
    let oversized_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/webhooks/meta/{HOOK_ID}"))
                .header("x-hub-signature-256", signature(&oversized))
                .body(Body::from(oversized))
                .expect("oversized request"),
        )
        .await
        .expect("body limit response");
    assert_eq!(oversized_response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(handler.seen.lock().expect("seen lock").is_empty());
    db.drop().await;
}

#[tokio::test]
async fn webhook_rate_limit_returns_too_many_requests_without_handoff() {
    let db = TestDb::migrated().await;
    let handler = Arc::new(TestHandler::default());
    let app = app(&db.url, handler.clone());
    let body = body_for("m_rate_limit", "buyer_rate_limit", 1_800_000_000, "page_1");
    let signature = signature(&body);

    for _ in 0..60 {
        assert_eq!(
            post(app.clone(), body.clone(), &signature).await,
            StatusCode::OK
        );
    }

    assert_eq!(
        post(app, body, &signature).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        handler.seen.lock().expect("seen lock").as_slice(),
        ["m_rate_limit"]
    );
    db.drop().await;
}
