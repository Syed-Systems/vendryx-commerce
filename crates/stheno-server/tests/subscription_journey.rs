//! The full subscription journey, end to end against a real database.
//!
//! One test, deliberately. Every step depends on the state the previous one left, and the
//! thing actually being proven is that a plan edit made in the control plane reaches a
//! merchant's request path without a restart — which cannot be asserted from a step in
//! isolation.
//!
//! It also cannot be split because it sets `STHENO_PLATFORM_TOKEN`, which is
//! process-global. Being the only test in this binary is what makes that safe; a second
//! `#[tokio::test]` here would race it (see the note in `boot_pool.rs`).
//!
//! Covers, in order:
//!   1. Provisioning a merchant onto a plan, and onto a plan that does not exist.
//!   2. The control plane refusing an unauthenticated caller — with a 404, not a 401.
//!   3. Reading the shipped plans.
//!   4. A merchant's own view of its entitlements, resolved through the Host header.
//!   5. Redefining a plan, and that landing on the merchant's next request.
//!   6. Rejecting an unknown feature key at the write boundary rather than storing it.
//!   7. Restoring the shipped default.
//!   8. A per-merchant feature grant, and that it applies to that merchant only.
//!   9. A per-merchant quota override.
//!  10. A per-merchant revocation of a feature the plan grants.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_server::TestApp;
use tower::ServiceExt;

const TOKEN: &str = "journey-operator-token";
const ACME: &str = "acme.localhost";
const RIVAL: &str = "rival.localhost";

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// A control-plane call carrying the operator token.
async fn operator(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {TOKEN}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    call(app, builder.body(body).unwrap()).await
}

/// Registers a staff user for the merchant on `host` and returns their admin JWT.
///
/// The Host header matters on the way in too: registration runs inside the tenant scope
/// it resolves, so a user created without it would belong to the wrong merchant.
async fn staff_token(app: &axum::Router, host: &str, email: &str) -> String {
    let (status, body) = call(
        app,
        Request::builder()
            .method("POST")
            .uri("/auth/user/emailpass/register")
            .header("host", host)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "email": email, "password": "Sup3rSecret!" }).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert!(
        status.is_success(),
        "staff registration for {host} failed: {status} {body}"
    );
    body["token"]
        .as_str()
        .expect("registration issues a token")
        .to_string()
}

/// A merchant-facing call. The Host header selects the tenant (ADR-012); the staff JWT is
/// what gets past the admin guard.
async fn as_merchant(
    app: &axum::Router,
    host: &str,
    token: &str,
    path: &str,
) -> (StatusCode, Value) {
    call(
        app,
        Request::builder()
            .uri(path)
            .header("host", host)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await
}

fn features_of(entitlements: &Value) -> Vec<String> {
    entitlements["features"]
        .as_array()
        .expect("features is an array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn quota_limit(entitlements: &Value, key: &str) -> Option<u64> {
    entitlements["quotas"]
        .as_array()
        .expect("quotas is an array")
        .iter()
        .find(|q| q["key"] == key)
        .and_then(|q| q["limit"].as_u64())
}

#[tokio::test]
async fn an_operator_can_control_every_plan_and_every_subscription() {
    std::env::set_var("STHENO_PLATFORM_TOKEN", TOKEN);
    let harness = TestApp::spawn().await;
    let app = harness.app.clone();

    // -- 1. Provisioning ----------------------------------------------------------------
    let (status, _) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({ "id": "ten_acme", "name": "Acme", "tier": "free", "domain": ACME })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "acme provisions");

    let (status, _) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({ "id": "ten_rival", "name": "Rival", "tier": "free", "domain": RIVAL })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "rival provisions");

    // A merchant provisioned onto a plan that does not exist would have every request fall
    // back to the least-privileged entitlements — a dead store that looks alive.
    let (status, body) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({ "id": "ten_ghost", "name": "Ghost", "tier": "enterprise" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unknown tier is rejected");
    assert_eq!(body["type"], "invalid_data");

    let acme_token = staff_token(&app, ACME, "owner@acme.test").await;
    let rival_token = staff_token(&app, RIVAL, "owner@rival.test").await;

    // A merchant reaching /admin/* without a staff token gets nothing, whatever its plan.
    let (status, _) = call(
        &app,
        Request::builder()
            .uri("/admin/entitlements")
            .header("host", ACME)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // -- 2. The control plane is closed without the token --------------------------------
    let (status, _) = call(
        &app,
        Request::builder()
            .uri("/platform/tiers")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "404, not 401: an unauthorised caller is not told the control plane exists"
    );

    let (status, _) = call(
        &app,
        Request::builder()
            .uri("/platform/tiers")
            .header("authorization", "Bearer wrong-token")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "a wrong token looks the same");

    // -- 3. The plans as shipped ---------------------------------------------------------
    let (status, plans) = operator(&app, "GET", "/platform/tiers", None).await;
    assert_eq!(status, StatusCode::OK);

    let tiers = plans["tiers"].as_array().expect("tiers is an array");
    assert_eq!(tiers.len(), 3, "trial, free, pro");
    let free = tiers.iter().find(|t| t["tier"] == "free").unwrap();
    assert_eq!(free["source"], "default", "untouched plans read as shipped");
    assert_eq!(free["quotas"]["max_products"], 100);
    assert!(
        !features_of(free).contains(&"payment_gateway".to_string()),
        "free does not include the gateway by default"
    );

    // The catalog is what the editor renders from: every gate the engine can apply, not
    // just the ones some plan happens to hold.
    let catalog = &plans["catalog"];
    assert!(catalog["features"]
        .as_array()
        .unwrap()
        .contains(&json!("payment_gateway")));
    assert!(catalog["quotas"]
        .as_array()
        .unwrap()
        .contains(&json!("max_products")));

    // -- 4. What the merchant sees -------------------------------------------------------
    let (status, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(acme["tier"], "free");
    assert!(!features_of(&acme).contains(&"payment_gateway".to_string()));
    assert_eq!(quota_limit(&acme, "max_products"), Some(100));

    // -- 5. Redefine the plan, live ------------------------------------------------------
    let (status, saved) = operator(
        &app,
        "PUT",
        "/platform/tiers/free",
        Some(json!({
            "features": ["csv_export", "web_push", "analytics_basic", "payment_gateway"],
            "quotas": { "max_products": 250, "max_staff_seats": 3 },
            "windows": { "analytics_window_days": 30 }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["tier"]["source"], "stored");

    // The payoff: no restart, no cache to invalidate. The merchant's very next request
    // resolves against the new definition.
    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert!(
        features_of(&acme).contains(&"payment_gateway".to_string()),
        "the plan edit reached the merchant without a restart"
    );
    assert_eq!(quota_limit(&acme, "max_products"), Some(250));
    assert!(
        !features_of(&acme).contains(&"spreadsheet_import".to_string()),
        "replacement, not merge: the saved plan omitted spreadsheet_import"
    );

    // Every merchant on the plan moves together.
    let (_, rival) = as_merchant(&app, RIVAL, &rival_token, "/admin/entitlements").await;
    assert!(features_of(&rival).contains(&"payment_gateway".to_string()));

    // Medusa-shaped feature flags follow, so the dashboard hides and shows the same set.
    let (status, flags) = as_merchant(&app, ACME, &acme_token, "/admin/feature-flags").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(flags["feature_flags"]["payment_gateway"], json!(true));

    // -- 6. A typo never reaches the table -----------------------------------------------
    let (status, body) = operator(
        &app,
        "PUT",
        "/platform/tiers/free",
        Some(json!({ "features": ["payment_gatway"], "quotas": {}, "windows": {} })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["message"].as_str().unwrap().contains("payment_gatway"),
        "the error names the key that was wrong, got: {body}"
    );

    let (status, body) = operator(
        &app,
        "PUT",
        "/platform/tiers/free",
        Some(json!({ "features": [], "quotas": { "max_widgets": 5 }, "windows": {} })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap().contains("max_widgets"));

    // The rejected writes changed nothing.
    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert!(features_of(&acme).contains(&"payment_gateway".to_string()));

    // -- 7. Back to the shipped default --------------------------------------------------
    let (status, restored) = operator(&app, "DELETE", "/platform/tiers/free", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["tier"]["source"], "default");

    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert!(
        !features_of(&acme).contains(&"payment_gateway".to_string()),
        "restoring the default took the gateway back"
    );
    assert_eq!(quota_limit(&acme, "max_products"), Some(100));

    // -- 8. One merchant, one exception --------------------------------------------------
    let (status, patched) = operator(
        &app,
        "PATCH",
        "/platform/tenants/ten_acme/subscription",
        Some(json!({ "feature_overrides": { "payment_gateway": true } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(patched["effective"]["features"]
        .as_array()
        .unwrap()
        .contains(&json!("payment_gateway")));

    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert!(features_of(&acme).contains(&"payment_gateway".to_string()));

    let (_, rival) = as_merchant(&app, RIVAL, &rival_token, "/admin/entitlements").await;
    assert!(
        !features_of(&rival).contains(&"payment_gateway".to_string()),
        "a negotiated grant is for one merchant, not the plan"
    );

    // -- 9. A negotiated limit -----------------------------------------------------------
    let (status, _) = operator(
        &app,
        "PATCH",
        "/platform/tenants/ten_acme/subscription",
        Some(json!({ "quota_overrides": { "max_products": 500 } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    assert_eq!(quota_limit(&acme, "max_products"), Some(500));
    let (_, rival) = as_merchant(&app, RIVAL, &rival_token, "/admin/entitlements").await;
    assert_eq!(quota_limit(&rival, "max_products"), Some(100));

    // A patch that names one field leaves the rest alone — the grant from step 8 survives.
    let (_, sub) = operator(
        &app,
        "GET",
        "/platform/tenants/ten_acme/subscription",
        None,
    )
    .await;
    assert_eq!(sub["subscription"]["feature_overrides"]["payment_gateway"], json!(true));

    // -- 10. Upgrade, then revoke one thing ----------------------------------------------
    let (status, _) = operator(
        &app,
        "PATCH",
        "/platform/tenants/ten_acme/subscription",
        Some(json!({
            "tier": "pro",
            "status": "active",
            "feature_overrides": { "fraud_score": false },
            "term_ends_at": "2027-01-01T12:00:00Z"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, acme) = as_merchant(&app, ACME, &acme_token, "/admin/entitlements").await;
    let acme_features = features_of(&acme);
    assert_eq!(acme["tier"], "pro");
    assert!(acme_features.contains(&"courier_dispatch".to_string()), "pro grants it");
    assert!(
        !acme_features.contains(&"fraud_score".to_string()),
        "an explicit false revokes a feature the plan grants"
    );
    assert_eq!(acme["term_ends_at"], "2027-01-01T12:00:00+00:00");

    // A status outside the stored CHECK constraint is a 400 naming the allowed set, not a
    // 500 from Postgres three layers down.
    let (status, body) = operator(
        &app,
        "PATCH",
        "/platform/tenants/ten_acme/subscription",
        Some(json!({ "status": "lapsed" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap().contains("active"));

    harness.teardown().await;
}
