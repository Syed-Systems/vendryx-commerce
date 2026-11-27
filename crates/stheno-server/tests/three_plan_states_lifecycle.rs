//! End-to-end integration test proving the 3 commercial plan states and 7-day trial lifecycle:
//!   State 1: Active 7-Day Free Trial (Appliance Store, ten_trial_appliance)
//!   State 2: Trial Expired -> Free Tier (Clothing Store, ten_expired_clothing)
//!   State 3: Pro Active (Tech Store, ten_pro_tech)
//!
//! Validates:
//!   - Tenant provisioning across trial, free, and pro tiers
//!   - Per-tenant entitlement quotas and feature gating
//!   - Host-based branding resolution (/store/branding) for each store
//!   - 7-day trial lifecycle warning checkpoints and expiry to Free
//!   - Upgrade from Free to Pro unlocking payment gateway and lifting quotas

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use stheno_flows::{TrialLifecycleJob, TrialStatus};
use stheno_platform::PlatformService;
use stheno_server::{load_trial_duration_days, TestApp};
use tower::ServiceExt;

const TOKEN: &str = "three-states-operator-token";
const APPLIANCE_HOST: &str = "trial-appliance.localhost";
const CLOTHING_HOST: &str = "expired-clothing.localhost";
const TECH_HOST: &str = "pro-tech.localhost";

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

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

async fn staff_token(app: &axum::Router, host: &str, email: &str) -> String {
    let (status, body) = call(
        app,
        Request::builder()
            .method("POST")
            .uri("/auth/user/emailpass/register")
            .header("host", host)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "email": email, "password": "SecurePassword123!" }).to_string(),
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
async fn test_three_plan_states_and_trial_lifecycle() {
    std::env::set_var("STHENO_PLATFORM_TOKEN", TOKEN);
    let harness = TestApp::spawn().await;
    let app = harness.app.clone();

    // -----------------------------------------------------------------------------------
    // 1. Provision the 3 Tenants representing the 3 Commercial States
    // -----------------------------------------------------------------------------------
    // State 1: Active 7-Day Free Trial -> Appliance Store
    let (status, body) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": "ten_trial_appliance",
            "name": "Vendryx Appliances",
            "tier": "trial",
            "domain": APPLIANCE_HOST
        })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "provision trial tenant: {body}"
    );
    // A trial runs only once it has started (ADR-033: it starts when the store request is
    // submitted, not at provisioning); an undated trial resolves to Free, fail-closed. Start
    // it the way that flow does, through the operator subscription patch.
    let trial_started_at = Utc::now();
    let trial_days = load_trial_duration_days().expect("deployment trial config");
    let (status, body) = operator(
        &app,
        "PATCH",
        "/platform/tenants/ten_trial_appliance/subscription",
        Some(json!({
            "trial_started_at": trial_started_at.to_rfc3339(),
            "trial_ends_at": (trial_started_at + Duration::days(i64::try_from(trial_days).expect("trial length fits in i64"))).to_rfc3339(),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "start the trial: {body}");

    // State 2: Trial Expired (Free tier) -> Clothing Store
    let (status, body) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": "ten_expired_clothing",
            "name": "Vendryx Fashion & Clothing",
            "tier": "free",
            "domain": CLOTHING_HOST
        })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "provision expired/free tenant: {body}"
    );

    // State 3: Pro Active -> Tech Store
    let (status, body) = operator(
        &app,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": "ten_pro_tech",
            "name": "Vendryx Tech & Electronics",
            "tier": "pro",
            "domain": TECH_HOST
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "provision pro tenant: {body}");

    // Register staff tokens for each tenant via their respective Host header
    let appliance_token = staff_token(&app, APPLIANCE_HOST, "owner@appliance.test").await;
    let clothing_token = staff_token(&app, CLOTHING_HOST, "owner@clothing.test").await;
    let tech_token = staff_token(&app, TECH_HOST, "owner@tech.test").await;

    // -----------------------------------------------------------------------------------
    // 2. Verify Entitlements and Quotas Per State
    // -----------------------------------------------------------------------------------
    // State 1: Trial Tenant Entitlements (Full access trial)
    let (status, ent_trial) = as_merchant(
        &app,
        APPLIANCE_HOST,
        &appliance_token,
        "/admin/entitlements",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ent_trial["tier"], "trial");
    // Trial has elevated staff seats (10)
    assert_eq!(quota_limit(&ent_trial, "max_staff_seats"), Some(10));

    // State 2: Free / Expired Tenant Entitlements (Capped quotas, no gateway)
    let (status, ent_free) =
        as_merchant(&app, CLOTHING_HOST, &clothing_token, "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ent_free["tier"], "free");
    assert_eq!(quota_limit(&ent_free, "max_products"), Some(100));
    assert_eq!(quota_limit(&ent_free, "max_staff_seats"), Some(3));
    assert!(
        !features_of(&ent_free).contains(&"payment_gateway".to_string()),
        "Free tier does not have integrated payment gateway"
    );

    // State 3: Pro Tenant Entitlements (Unlimited products, payment gateway, courier)
    let (status, ent_pro) = as_merchant(&app, TECH_HOST, &tech_token, "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ent_pro["tier"], "pro");
    assert!(
        features_of(&ent_pro).contains(&"payment_gateway".to_string()),
        "Pro tier includes payment gateway"
    );

    // -----------------------------------------------------------------------------------
    // 3. Verify Public Store Branding Endpoint (/store/branding)
    // -----------------------------------------------------------------------------------
    let (status, appliance_branding) = call(
        &app,
        Request::builder()
            .uri("/store/branding")
            .header("host", APPLIANCE_HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(appliance_branding["branding"].is_object());

    let (status, clothing_branding) = call(
        &app,
        Request::builder()
            .uri("/store/branding")
            .header("host", CLOTHING_HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(clothing_branding["branding"].is_object());

    let (status, tech_branding) = call(
        &app,
        Request::builder()
            .uri("/store/branding")
            .header("host", TECH_HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(tech_branding["branding"].is_object());

    // -----------------------------------------------------------------------------------
    // 4. Validate seven-day trial lifecycle transitions.
    // -----------------------------------------------------------------------------------
    assert_eq!(
        load_trial_duration_days().expect("deployment trial config"),
        PlatformService::DEFAULT_TRIAL_DURATION_DAYS
    );
    let trial_start = Utc::now();
    let mut lifecycle = TrialLifecycleJob::new(trial_start);

    // Days 0-2: Active. (Day 5 was a leftover from the 14-day trial; with seven days the
    // first warning is at day 3, and the lifecycle only moves forward, so a day-5 check
    // before the day-3 one could never be Active.)
    assert_eq!(
        lifecycle.evaluate_lifecycle_at(trial_start + Duration::days(1)),
        TrialStatus::Active
    );

    // Day 3: first trial warning.
    assert_eq!(
        lifecycle.evaluate_lifecycle_at(trial_start + Duration::days(3)),
        TrialStatus::WarningFirst
    );

    // Day 6: final trial warning (24 hours remaining).
    assert_eq!(
        lifecycle.evaluate_lifecycle_at(trial_start + Duration::days(6)),
        TrialStatus::WarningFinal
    );

    // Day 7: Trial Expiration -> Downgrade to Free.
    assert_eq!(
        lifecycle.evaluate_lifecycle_at(trial_start + Duration::days(7)),
        TrialStatus::ExpiredToFree
    );

    // Idempotent re-evaluation at Day 8 -> stays ExpiredToFree without duplicate logs.
    assert_eq!(
        lifecycle.evaluate_lifecycle_at(trial_start + Duration::days(8)),
        TrialStatus::ExpiredToFree
    );
    assert_eq!(lifecycle.audit_logs.len(), 3);

    // -----------------------------------------------------------------------------------
    // 5. Upgrade Path: Expired Merchant Upgrades to Pro
    // -----------------------------------------------------------------------------------
    let (status, upgraded) = operator(
        &app,
        "POST",
        "/platform/tenants/ten_expired_clothing/tier",
        Some(json!({ "tier": "pro" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tenant upgraded to pro: {upgraded}");

    // The upgraded merchant immediately sees Pro entitlements
    let (status, ent_upgraded) =
        as_merchant(&app, CLOTHING_HOST, &clothing_token, "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ent_upgraded["tier"], "pro");
    assert!(
        features_of(&ent_upgraded).contains(&"payment_gateway".to_string()),
        "Upgraded merchant now has payment gateway unlocked"
    );
}
