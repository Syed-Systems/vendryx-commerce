//! Replay, tenant, and out-of-order contracts for ADR-032 Meta ingress.

use chrono::{TimeZone, Utc};
use stheno_ai::WebhookEventService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn deduplicates_per_tenant_and_suppresses_out_of_order_messages() {
    let db = TestDb::migrated().await;
    let service = WebhookEventService::new(create_pool(&db.url, 5).expect("pool"));
    let later = Utc.with_ymd_and_hms(2026, 10, 23, 10, 0, 0).unwrap();
    let earlier = Utc.with_ymd_and_hms(2026, 10, 23, 9, 0, 0).unwrap();

    assert!(service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("new event"));
    service
        .mark_failed("ten_alpha", "whatsapp", "acct_1", "wamid_new")
        .await
        .expect("release failed attempt");
    assert!(service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("retry failed event"));
    service
        .mark_processed("ten_alpha", "whatsapp", "acct_1", "wamid_new")
        .await
        .expect("record successful handoff");
    assert!(!service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("duplicate event"));
    assert!(!service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_old",
            "sender_1",
            earlier,
        )
        .await
        .expect("out-of-order event"));
    assert!(service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_other",
            "sender_2",
            earlier
        )
        .await
        .expect("independent conversation"));
    assert!(service
        .claim_message(
            "ten_beta",
            "whatsapp",
            "acct_1",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("event identity is tenant scoped"));

    assert!(service
        .claim_message(
            "ten_alpha",
            "messenger",
            "acct_1",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("event identity is channel scoped"));
    assert!(service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_2",
            "wamid_new",
            "sender_1",
            later
        )
        .await
        .expect("event identity is account scoped"));
    assert!(!service
        .claim_message(
            "ten_alpha",
            "whatsapp",
            "acct_1",
            "wamid_old",
            "sender_1",
            later
        )
        .await
        .expect("duplicate message ID remains suppressed"));
    db.drop().await;
}

#[tokio::test]
async fn rejects_untrusted_tenant_and_malformed_event_identifiers() {
    let db = TestDb::migrated().await;
    let service = WebhookEventService::new(create_pool(&db.url, 5).expect("pool"));
    let now = Utc::now();
    assert!(service
        .claim_message("ten';drop", "whatsapp", "acct", "msg", "sender", now)
        .await
        .is_err());
    assert!(service
        .claim_message("ten_alpha", "unsupported", "acct", "msg", "sender", now)
        .await
        .is_err());
    assert!(service
        .claim_message("ten_alpha", "whatsapp", "acct", "\n", "sender", now)
        .await
        .is_err());
    db.drop().await;
}
