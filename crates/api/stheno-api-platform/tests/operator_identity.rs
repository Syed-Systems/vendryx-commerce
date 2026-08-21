//! The control plane knows who is calling it, and can stop knowing one of them.
//!
//! `/platform/*` is the one surface that reads and writes across every merchant, and until
//! now it was gated on a single shared secret. These assert the three things a shared
//! secret cannot do: name the caller, revoke one holder without revoking the rest, and
//! keep a credential unrecoverable after it is issued.
//!
//! The bootstrap token stays valid throughout on purpose — see
//! `an_operator_cannot_lock_everyone_out_by_revoking_the_last_credential`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;
use tower::ServiceExt;

const BOOTSTRAP: &str = "bootstrap-secret-for-this-test";

/// The shipped plans. Read from the repo rather than hand-rolled so these tests cannot
/// pass against a tier config the deployment does not have.
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
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
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

/// Issues a credential using the bootstrap token, returning `(id, plaintext token)`.
async fn issue(router: &axum::Router, name: &str) -> (String, String) {
    let (status, body) = call(
        router,
        "POST",
        "/platform/operators",
        Some(BOOTSTRAP),
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "issuing {name}: {body}");
    (
        body["operator"]["id"].as_str().expect("id").to_string(),
        body["token"].as_str().expect("token").to_string(),
    )
}

#[tokio::test]
async fn an_issued_credential_authenticates_and_names_its_holder() {
    let (db, router) = app().await;

    let (id, token) = issue(&router, "Ayesha").await;
    assert!(
        token.starts_with("op_"),
        "an issued token must be recognisable as one if it turns up in a log, got {token:?}"
    );

    let (status, body) = call(&router, "GET", "/platform/operators", Some(&token), None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the credential just issued must open the surface it was issued for"
    );
    assert_eq!(body["caller"]["id"], id);
    assert_eq!(
        body["caller"]["name"], "Ayesha",
        "the control plane must be able to say who is calling, which is the whole point"
    );
    assert_eq!(
        body["caller"]["bootstrap"], false,
        "a named operator is not the shared secret"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_plaintext_token_is_returned_once_and_never_again() {
    let (db, router) = app().await;

    let (id, token) = issue(&router, "Rafi").await;

    let (_, listing) = call(&router, "GET", "/platform/operators", Some(BOOTSTRAP), None).await;
    let serialised = listing.to_string();
    assert!(
        !serialised.contains(&token),
        "the listing leaked the plaintext token; only its SHA-256 is stored, and only a \
         redaction may ever be rendered"
    );

    let row = listing["operators"]
        .as_array()
        .expect("operators is an array")
        .iter()
        .find(|o| o["id"] == id.as_str())
        .expect("the issued credential is listed");
    assert!(
        row["redacted_token"]
            .as_str()
            .is_some_and(|r| r.starts_with("op_...")),
        "a credential still has to be identifiable in the UI, got {:?}",
        row["redacted_token"]
    );

    db.drop().await;
}

#[tokio::test]
async fn a_revoked_credential_stops_working_and_the_others_do_not() {
    let (db, router) = app().await;

    let (doomed_id, doomed) = issue(&router, "Leaver").await;
    let (_, kept) = issue(&router, "Stayer").await;

    // Both work before the revocation, so the assertion afterwards is about the
    // revocation and not about one of them never having worked.
    for token in [&doomed, &kept] {
        let (status, _) = call(&router, "GET", "/platform/tenants", Some(token), None).await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, _) = call(
        &router,
        "DELETE",
        &format!("/platform/operators/{doomed_id}"),
        Some(BOOTSTRAP),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "revocation succeeds");

    let (status, _) = call(&router, "GET", "/platform/tenants", Some(&doomed), None).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "a revoked credential must be as good as one that was never issued — and 404, \
         not 401, so it is not even told the surface exists"
    );

    let (status, _) = call(&router, "GET", "/platform/tenants", Some(&kept), None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "revoking one holder must not revoke the rest; that is the entire reason this is \
         not one shared secret"
    );

    // A revoked credential stays listed. One that vanished would be indistinguishable
    // from one that was never issued, which is the opposite of an access review.
    let (_, listing) = call(&router, "GET", "/platform/operators", Some(BOOTSTRAP), None).await;
    let row = listing["operators"]
        .as_array()
        .expect("array")
        .iter()
        .find(|o| o["id"] == doomed_id.as_str())
        .expect("a revoked credential is still listed");
    assert_eq!(row["revoked"], true);

    db.drop().await;
}

#[tokio::test]
async fn an_operator_cannot_lock_everyone_out_by_revoking_the_last_credential() {
    let (db, router) = app().await;

    let (id, _) = issue(&router, "Only Operator").await;
    let (status, _) = call(
        &router,
        "DELETE",
        &format!("/platform/operators/{id}"),
        Some(BOOTSTRAP),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(&router, "GET", "/platform/operators", Some(BOOTSTRAP), None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the bootstrap token is deliberately still valid after the last named credential \
         is revoked: a control plane that can lock its owner out of itself is a worse \
         failure than a shared secret"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_unknown_or_absent_credential_is_not_told_the_surface_exists() {
    let (db, router) = app().await;

    for (label, token) in [
        ("no credential at all", None),
        ("a token that was never issued", Some("op_deadbeef")),
        (
            "a near-miss on the bootstrap secret",
            Some("bootstrap-secret-for-this-tesT"),
        ),
        ("an empty bearer", Some("")),
    ] {
        let (status, _) = call(&router, "GET", "/platform/tenants", token, None).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{label} must get a 404, never a 401 and never data"
        );
    }

    db.drop().await;
}

#[tokio::test]
async fn an_unnamed_credential_is_refused() {
    let (db, router) = app().await;

    // A credential with no name defeats the audit trail it exists to feed: every row it
    // wrote would be attributed to nobody.
    for name in ["", "   "] {
        let (status, body) = call(
            &router,
            "POST",
            "/platform/operators",
            Some(BOOTSTRAP),
            Some(json!({ "name": name })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "name {name:?} is refused");
        assert_eq!(body["type"], "invalid_data");
    }

    db.drop().await;
}
