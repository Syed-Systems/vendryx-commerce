use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use stheno_registry::AppContext;
use stheno_server::{load_entitlements, AppServer};
use tower::ServiceExt;

#[tokio::test]
async fn payment_hooks_are_mounted_on_the_server_router() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let server = AppServer::new(
        AppContext::create_test_context(),
        load_entitlements().expect("entitlements"),
        pool,
    );

    let response = server
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hooks/payment/pp_missing")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .expect("router responds");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    assert_eq!(
        body["type"], "not_found",
        "an unmounted hook route returns Axum's empty 404, not the hook handler's JSON error"
    );

    db.drop().await;
}
