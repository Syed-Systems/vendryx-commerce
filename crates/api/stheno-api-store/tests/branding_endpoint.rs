//! P26-03: Tests for GET /store/branding.
//!
//! Asserts that:
//! 1. An absent branding row yields documented defaults (200 OK, null fields).
//! 2. An upserted branding row is returned accurately.
//! 3. Two tenants have isolated branding rows (RLS multi-tenancy).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::store_branding_routes;
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant};
use stheno_store::dto::UpsertBrandingInput;
use stheno_store::StoreService;
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("parse json")
}

#[tokio::test]
async fn absent_row_yields_documented_defaults() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let store_service = Arc::new(StoreService::new(pool));
    let app = store_branding_routes(store_service);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/branding")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let branding = &json["branding"];
    assert!(
        branding.is_object(),
        "response must have a branding envelope"
    );
    assert!(
        branding["store_name"].is_null(),
        "absent store_name should be null"
    );
    assert!(
        branding["font_key"].is_null(),
        "absent font_key should be null"
    );
    assert!(
        branding["color_primary"].is_null(),
        "absent color_primary should be null"
    );

    db.drop().await;
}

#[tokio::test]
async fn upserted_branding_is_exposed_accurately() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let store_service = Arc::new(StoreService::new(pool));
    let app = store_branding_routes(store_service.clone());

    // Upsert tenant branding
    store_service
        .upsert_branding(UpsertBrandingInput {
            store_name: Some("Artisan Watches".into()),
            font_key: Some("inter".into()),
            color_primary: Some("#1A202C".into()),
            color_secondary: Some("#2D3748".into()),
            color_accent: Some("#E53E3E".into()),
            color_background: Some("#FFFFFF".into()),
            color_surface: Some("#F7FAFC".into()),
            color_text: Some("#1A202C".into()),
            logo_file_id: None,
            favicon_file_id: None,
        })
        .await
        .expect("upsert branding");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/branding")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let branding = &json["branding"];
    assert_eq!(branding["store_name"], "Artisan Watches");
    assert_eq!(branding["font_key"], "inter");
    assert_eq!(branding["color_primary"], "#1A202C");
    assert_eq!(branding["color_secondary"], "#2D3748");
    assert_eq!(branding["color_accent"], "#E53E3E");
    assert_eq!(branding["color_background"], "#FFFFFF");
    assert_eq!(branding["color_surface"], "#F7FAFC");
    assert_eq!(branding["color_text"], "#1A202C");

    db.drop().await;
}

#[tokio::test]
async fn multi_tenant_branding_isolation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let store_service = Arc::new(StoreService::new(pool));
    let app = store_branding_routes(store_service.clone());

    let tenant_a = "tenant_brand_a".to_string();
    let tenant_b = "tenant_brand_b".to_string();

    // Tenant A sets branding
    tenant::scope(tenant_a.clone(), async {
        store_service
            .upsert_branding(UpsertBrandingInput {
                store_name: Some("Tenant A Store".into()),
                font_key: Some("inter".into()),
                color_primary: Some("#AA0000".into()),
                ..Default::default()
            })
            .await
            .expect("upsert A");
    })
    .await;

    // Tenant B queries branding before setting any -> gets defaults
    tenant::scope(tenant_b.clone(), async {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/store/branding")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response B");

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert!(
            json["branding"]["store_name"].is_null(),
            "tenant B should see null, not tenant A"
        );
    })
    .await;

    // Tenant A queries branding -> gets Tenant A's branding
    tenant::scope(tenant_a.clone(), async {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/store/branding")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response A");

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["branding"]["store_name"], "Tenant A Store");
    })
    .await;

    db.drop().await;
}
