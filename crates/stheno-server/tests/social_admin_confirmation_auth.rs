//! The staff-confirmed social order conversion route is covered by the server-wide
//! admin authentication layer before its order or courier side effects can run.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use stheno_platform::{PlatformService, SubscriptionPatch};
use stheno_server::TestApp;
use tower::ServiceExt;

#[tokio::test]
async fn social_confirmation_rejects_missing_and_invalid_staff_credentials() {
    let previous_tenant = std::env::var("STHENO_TENANT").ok();
    std::env::set_var("STHENO_TENANT", stheno_db::testing::TestDb::TEST_TENANT);
    let harness = TestApp::spawn().await;
    match previous_tenant {
        Some(tenant) => std::env::set_var("STHENO_TENANT", tenant),
        None => std::env::remove_var("STHENO_TENANT"),
    }
    let platform = PlatformService::new(
        stheno_db::create_pool(&harness.database_url, 5).expect("test database pool"),
    );
    platform
        .update_subscription(
            stheno_db::testing::TestDb::TEST_TENANT,
            SubscriptionPatch {
                feature_overrides: Some(serde_json::json!({"buyer_accounts": true})),
                ..Default::default()
            },
        )
        .await
        .expect("enable buyer-account auth for the customer-role assertion");
    let path = "/admin/draft-orders/social-draft-id/convert-to-order";

    let missing = harness
        .app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .expect("router responds to unauthenticated request");
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let invalid = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("authorization", "Bearer invalid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router responds to invalid credential");
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);

    let customer = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/customer/emailpass/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"buyer@social-confirm.test","password":"Sup3rSecret!"}"#,
                ))
                .unwrap(),
        )
        .await
        .expect("router responds to customer registration");
    assert_eq!(customer.status(), StatusCode::OK);
    let customer_body = axum::body::to_bytes(customer.into_body(), usize::MAX)
        .await
        .expect("customer registration body reads");
    let customer_json: serde_json::Value =
        serde_json::from_slice(&customer_body).expect("customer registration returns JSON");
    let customer_token = customer_json["token"]
        .as_str()
        .expect("customer registration issues a token");

    let non_staff = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header("authorization", format!("Bearer {customer_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router responds to customer credential");
    assert_eq!(non_staff.status(), StatusCode::FORBIDDEN);

    let staff = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user/emailpass/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"staff@social-confirm.test","password":"Sup3rSecret!"}"#,
                ))
                .unwrap(),
        )
        .await
        .expect("router responds to staff registration");
    assert_eq!(staff.status(), StatusCode::OK);
    let staff_body = axum::body::to_bytes(staff.into_body(), usize::MAX)
        .await
        .expect("staff registration body reads");
    let staff_json: serde_json::Value =
        serde_json::from_slice(&staff_body).expect("staff registration returns JSON");
    let staff_token = staff_json["token"]
        .as_str()
        .expect("staff registration issues a token");

    let authenticated_staff = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("authorization", format!("Bearer {staff_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router responds to staff credential");
    assert_eq!(authenticated_staff.status(), StatusCode::NOT_FOUND);

    let reconciliation_path = "/admin/orders/social-order-id/courier-dispatch/reconcile";
    let reconciliation_payload =
        r#"{"outcome":"not_created","evidence_reference":"courier-ticket-7"}"#;
    let reconciliation_missing = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(reconciliation_path)
                .header("content-type", "application/json")
                .body(Body::from(reconciliation_payload))
                .unwrap(),
        )
        .await
        .expect("router responds to unauthenticated reconciliation");
    assert_eq!(reconciliation_missing.status(), StatusCode::UNAUTHORIZED);

    let reconciliation_customer = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(reconciliation_path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {customer_token}"))
                .body(Body::from(reconciliation_payload))
                .unwrap(),
        )
        .await
        .expect("router rejects customer reconciliation");
    assert_eq!(reconciliation_customer.status(), StatusCode::FORBIDDEN);

    let reconciliation_staff = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(reconciliation_path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {staff_token}"))
                .body(Body::from(reconciliation_payload))
                .unwrap(),
        )
        .await
        .expect("router admits staff reconciliation to the order lookup");
    assert_eq!(reconciliation_staff.status(), StatusCode::NOT_FOUND);

    harness.teardown().await;
}
