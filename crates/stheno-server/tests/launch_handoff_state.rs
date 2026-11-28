//! The launch handoff is observable at the HTTP boundary: a tenant exists while work is
//! provisioning, but it remains invisible until the owner handoff is explicitly activated.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use stheno_db::create_pool;
use stheno_platform::{PlatformService, SubmitLaunchRequestInput};
use stheno_server::TestApp;
use tower::ServiceExt;

#[tokio::test]
async fn pending_launch_is_hidden_and_activation_makes_the_store_available() {
    let harness = TestApp::spawn().await;
    let service = PlatformService::new(create_pool(&harness.database_url, 5).expect("pool"));
    service
        .upsert_merchant_account(
            "auth_launch_owner".to_string(),
            "owner@example.com".to_string(),
            Some(Utc::now()),
            serde_json::json!({}),
            "free".to_string(),
        )
        .await
        .expect("verified merchant account");
    let request = service
        .submit_launch_request(SubmitLaunchRequestInput {
            merchant_id: "auth_launch_owner".to_string(),
            store_name: "Pahar Goods".to_string(),
            subdomain: "pahar-goods".to_string(),
            tier: "free".to_string(),
            logo_url: None,
            accent_color: "#1D4ED8".to_string(),
            icon_choice: "rise".to_string(),
            buyer_accounts: false,
            bespoke_brief: None,
            bespoke_assets: Vec::new(),
        })
        .await
        .expect("launch request");
    service
        .transition_launch_request(
            &request.id,
            "begin_provisioning",
            "op_1".to_string(),
            "Operator".to_string(),
            None,
            None,
        )
        .await
        .expect("begin provisioning");
    service
        .provision_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("provision tenant");

    let pending = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/regions")
                .header("host", "pahar-goods.stheno.shop")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("pending request returns a response");
    assert_eq!(pending.status(), StatusCode::NOT_FOUND);

    service
        .activate_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("activate handoff");
    let online = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/regions")
                .header("host", "pahar-goods.stheno.shop")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("online request returns a response");
    assert_eq!(online.status(), StatusCode::OK);
}

/// Provisions and activates one store, returning the host it answers on.
async fn launch_store(service: &PlatformService, slug: &str, buyer_accounts: bool) -> String {
    let merchant_id = format!("auth_{slug}");
    service
        .upsert_merchant_account(
            merchant_id.clone(),
            format!("{slug}@example.com"),
            Some(Utc::now()),
            serde_json::json!({}),
            "free".to_string(),
        )
        .await
        .expect("verified merchant account");
    let request = service
        .submit_launch_request(SubmitLaunchRequestInput {
            merchant_id,
            store_name: slug.to_string(),
            subdomain: slug.to_string(),
            tier: "free".to_string(),
            logo_url: None,
            accent_color: "#1D4ED8".to_string(),
            icon_choice: "rise".to_string(),
            buyer_accounts,
            bespoke_brief: None,
            bespoke_assets: Vec::new(),
        })
        .await
        .expect("launch request");
    service
        .transition_launch_request(
            &request.id,
            "begin_provisioning",
            "op_1".to_string(),
            "Operator".to_string(),
            None,
            None,
        )
        .await
        .expect("begin provisioning");
    service
        .provision_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("provision tenant");
    service
        .activate_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("activate handoff");
    format!("{slug}.stheno.shop")
}

/// End-to-end: the archetype a merchant picks on the store request decides, through
/// tenant resolution and entitlement resolution, which routes their storefront has.
///
/// Both stores are on the same tier and the same process. Nothing but the launch
/// request differs, which is the whole claim being tested.
#[tokio::test]
async fn the_store_request_archetype_decides_which_routes_exist() {
    let harness = TestApp::spawn().await;
    let service = PlatformService::new(create_pool(&harness.database_url, 5).expect("pool"));

    let guest_host = launch_store(&service, "guest-goods", false).await;
    let account_host = launch_store(&service, "account-goods", true).await;

    let get = |host: String, uri: &'static str| {
        let app = harness.app.clone();
        async move {
            app.oneshot(
                Request::builder()
                    .uri(uri)
                    .header("host", host)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response")
        }
    };

    // The buyer surface follows the archetype. 404, not 403: a store without accounts
    // is indistinguishable from one where the route never existed.
    assert_eq!(
        get(guest_host.clone(), "/store/customers/me")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_ne!(
        get(account_host.clone(), "/store/customers/me")
            .await
            .status(),
        StatusCode::NOT_FOUND,
        "the accounts archetype must mount the customer route (401 unauthenticated is fine)"
    );
    assert_eq!(
        get(guest_host.clone(), "/auth/customer/providers")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(account_host.clone(), "/auth/customer/providers")
            .await
            .status(),
        StatusCode::OK
    );

    // Staff auth is not a buyer surface. Getting this wrong locks merchants out of
    // their own dashboard, so it is asserted on the store that has the gate closed.
    assert_eq!(
        get(guest_host.clone(), "/auth/user/providers")
            .await
            .status(),
        StatusCode::OK
    );

    // Guest checkout is the default path under both archetypes and must not move.
    for host in [guest_host.clone(), account_host.clone()] {
        assert_eq!(get(host, "/store/products").await.status(), StatusCode::OK);
    }

    // The one bit the storefront reads to decide whether /account exists.
    let mut tenant_ids = Vec::new();
    for (host, expected) in [(guest_host, false), (account_host, true)] {
        let response = get(host.clone(), "/store/config").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("config body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("config json");
        let tenant_id = json["tenant_id"]
            .as_str()
            .expect("store config returns an explicit tenant id");
        assert!(!tenant_id.is_empty());
        tenant_ids.push(tenant_id.to_string());
        assert_eq!(
            json["buyer_accounts"], expected,
            "{host} reported the wrong archetype"
        );
    }
    assert_ne!(tenant_ids[0], tenant_ids[1]);
}
