//! Every control-plane write is attributed to the operator who made it.
//!
//! Under [ADR-012](../../../docs/DECISIONS.md#adr-012) this is the one surface that reads
//! and writes across every merchant, and every write it makes is billing-affecting.
//! "Someone moved Acme onto free" is not an answer to an angry merchant; these assert the
//! trail can name who, when, and what the value was before.
//!
//! The last test is the one that matters most: the trail must never contain the
//! credentials it exists to attribute. An audit log holding live operator tokens would be
//! a credential store with strictly worse access control than the credential store.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext, BOOTSTRAP_ACTOR_ID};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;
use tower::ServiceExt;

const BOOTSTRAP: &str = "audit-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let router = platform_routes(PlatformApiContext {
        platform: Arc::new(PlatformService::new(pool)),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
    });
    (db, router)
}

async fn call(
    router: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).expect("request builds"))
        .await
        .expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn issue(router: &axum::Router, name: &str) -> String {
    let (status, body) = call(
        router,
        "POST",
        "/platform/operators",
        BOOTSTRAP,
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "issuing {name}: {body}");
    body["token"].as_str().expect("token").to_string()
}

/// The whole trail, newest first.
async fn trail(router: &axum::Router) -> Vec<Value> {
    let (status, body) = call(router, "GET", "/platform/audit", BOOTSTRAP, None).await;
    assert_eq!(status, StatusCode::OK, "the trail reads: {body}");
    body["entries"].as_array().cloned().unwrap_or_default()
}

fn find<'a>(entries: &'a [Value], action: &str, entity_id: &str) -> &'a Value {
    entries
        .iter()
        .find(|e| e["action"] == action && e["entity_id"] == entity_id)
        .unwrap_or_else(|| {
            panic!("no `{action}` row for {entity_id}; trail was {entries:#?}");
        })
}

#[tokio::test]
async fn a_plan_move_records_who_moved_them_and_what_they_moved_from() {
    let (db, router) = app().await;
    let ayesha = issue(&router, "Ayesha").await;

    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants",
        &ayesha,
        Some(json!({ "id": "ten_acme", "name": "Acme", "tier": "free" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants/ten_acme/tier",
        &ayesha,
        Some(json!({ "tier": "pro" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = trail(&router).await;

    let created = find(&entries, "tenant.create", "ten_acme");
    assert_eq!(created["actor_name"], "Ayesha");
    assert_eq!(created["diff"]["tier"], "free");

    let moved = find(&entries, "tenant.tier_set", "ten_acme");
    assert_eq!(
        moved["actor_name"], "Ayesha",
        "the trail must name a person, not a secret"
    );
    assert_eq!(
        moved["diff"]["from"], "free",
        "recording only the destination cannot answer 'who took them off Pro'"
    );
    assert_eq!(moved["diff"]["to"], "pro");

    db.drop().await;
}

#[tokio::test]
async fn a_subscription_patch_records_the_row_before_and_after() {
    let (db, router) = app().await;
    let rafi = issue(&router, "Rafi").await;

    call(
        &router,
        "POST",
        "/platform/tenants",
        &rafi,
        Some(json!({ "id": "ten_beta", "name": "Beta", "tier": "free" })),
    )
    .await;

    // A negotiated per-merchant grant: the single most disputable thing an operator can
    // do, because it gives one merchant something their plan does not include.
    let (status, _) = call(
        &router,
        "PATCH",
        "/platform/tenants/ten_beta/subscription",
        &rafi,
        Some(json!({ "feature_overrides": { "payment_gateway": true } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = trail(&router).await;
    let patched = find(&entries, "subscription.patch", "ten_beta");

    assert_eq!(patched["actor_name"], "Rafi");
    assert_eq!(
        patched["diff"]["before"]["feature_overrides"],
        json!({}),
        "without the prior value the trail cannot show what actually changed"
    );
    assert_eq!(
        patched["diff"]["after"]["feature_overrides"]["payment_gateway"],
        true
    );

    db.drop().await;
}

#[tokio::test]
async fn redefining_a_plan_is_recorded_as_what_merchants_now_get() {
    let (db, router) = app().await;
    let ops = issue(&router, "Plan Editor").await;

    let (status, _) = call(
        &router,
        "PUT",
        "/platform/tiers/free",
        &ops,
        Some(json!({ "features": ["csv_export"], "quotas": { "max_products": 25 } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = trail(&router).await;
    let defined = find(&entries, "tier.define", "free");
    assert_eq!(defined["actor_name"], "Plan Editor");
    assert_eq!(defined["entity_type"], "tier");
    assert_eq!(
        defined["diff"]["quotas"]["max_products"], 25,
        "the row records the resolved plan, which is what the router will mount"
    );

    let (status, _) = call(&router, "DELETE", "/platform/tiers/free", &ops, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        find(&trail(&router).await, "tier.reset", "free")["actor_name"],
        "Plan Editor",
        "restoring a shipped default is as billing-affecting as changing one"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_bootstrap_token_is_recorded_as_exactly_as_specific_as_it_is() {
    let (db, router) = app().await;

    call(
        &router,
        "POST",
        "/platform/tenants",
        BOOTSTRAP,
        Some(json!({ "id": "ten_gamma", "name": "Gamma", "tier": "free" })),
    )
    .await;

    let created = find(&trail(&router).await, "tenant.create", "ten_gamma").clone();
    assert_eq!(
        created["actor_id"], BOOTSTRAP_ACTOR_ID,
        "a shared secret cannot name a person, and the trail must not pretend otherwise"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_trail_never_contains_an_operator_token() {
    let (db, router) = app().await;

    // Issuing a credential is the one action whose payload contains a live secret. If any
    // path leaks a token into the trail, it is this one.
    let ayesha = issue(&router, "Ayesha").await;
    let rafi = issue(&router, "Rafi").await;

    call(
        &router,
        "POST",
        "/platform/tenants",
        &ayesha,
        Some(json!({ "id": "ten_delta", "name": "Delta", "tier": "free" })),
    )
    .await;

    let serialised = json!(trail(&router).await).to_string();
    for token in [&ayesha, &rafi, &BOOTSTRAP.to_string()] {
        assert!(
            !serialised.contains(token.as_str()),
            "the audit trail leaked a live credential; a trail holding the tokens it \
             attributes is a credential store with worse access control than the \
             credential store"
        );
    }

    // The redaction is present, so a credential is still identifiable in the trail.
    let entries = trail(&router).await;
    let issued = entries
        .iter()
        .find(|e| e["action"] == "operator.create" && e["diff"]["name"] == "Ayesha")
        .expect("issuing a credential is itself audited");
    assert!(issued["diff"]["redacted_token"]
        .as_str()
        .is_some_and(|r| r.starts_with("op_...")));

    db.drop().await;
}

#[tokio::test]
async fn the_trail_reads_across_tenants_and_narrows_to_one() {
    let (db, router) = app().await;
    let ops = issue(&router, "Ops").await;

    for id in ["ten_one", "ten_two"] {
        call(
            &router,
            "POST",
            "/platform/tenants",
            &ops,
            Some(json!({ "id": id, "name": id, "tier": "free" })),
        )
        .await;
    }

    // The query `audit.audit_log` cannot serve: that table is RLS-forced, so reading two
    // merchants' history in one response is impossible there by construction.
    let all = trail(&router).await;
    assert!(
        all.iter().any(|e| e["entity_id"] == "ten_one")
            && all.iter().any(|e| e["entity_id"] == "ten_two"),
        "the control-plane trail must span tenants; that is why it is not audit.audit_log"
    );

    let (status, body) = call(
        &router,
        "GET",
        "/platform/audit?entity_id=ten_one",
        BOOTSTRAP,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let narrowed = body["entries"].as_array().expect("entries");
    assert!(!narrowed.is_empty());
    assert!(
        narrowed.iter().all(|e| e["entity_id"] == "ten_one"),
        "narrowing to one merchant must not spill another's history: {narrowed:#?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_refused_write_records_nothing() {
    let (db, router) = app().await;
    let ops = issue(&router, "Ops").await;

    // Provisioning onto a plan that does not exist is a 400. An audit row for a change
    // that never happened is worse than no row: it sends someone hunting for a merchant
    // that was never created.
    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants",
        &ops,
        Some(json!({ "id": "ten_ghost", "name": "Ghost", "tier": "enterprise" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // An unauthorised caller likewise leaves nothing behind.
    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants",
        "op_never_issued",
        Some(json!({ "id": "ten_intruder", "name": "Intruder", "tier": "free" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let entries = trail(&router).await;
    assert!(
        !entries
            .iter()
            .any(|e| e["entity_id"] == "ten_ghost" || e["entity_id"] == "ten_intruder"),
        "a write that was refused must leave no trace of having happened: {entries:#?}"
    );

    db.drop().await;
}
