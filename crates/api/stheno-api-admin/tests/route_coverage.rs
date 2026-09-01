//! Asserts the admin router actually mounts what the dashboard calls.
//!
//! The previous version of this file built the router and ended with `let _ = router;`.
//! It was named for coverage and checked none: every route could have been deleted and it
//! would still have passed. That is how `/admin/product-tags`, `/admin/rbac/roles` and
//! `/admin/shipping-profiles` went on answering with hardcoded literals — nothing
//! compared the mounted set against what was supposed to exist.
//!
//! This walks the real router with real requests. A route that is absent returns 404 from
//! axum, which is distinguishable from a route that exists and rejects the input.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use stheno_api_admin::{
    admin_catalog_routes, admin_customer_routes, admin_fulfillment_settings_routes,
    admin_localisation_routes, admin_rbac_routes,
};
use stheno_customer::CustomerService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::FulfillmentService;
use stheno_product::ProductService;
use stheno_rbac::RbacService;
use stheno_settings::SettingsService;
use stheno_translation::TranslationService;
use tower::ServiceExt;

/// Every route below must exist.
///
/// "Not 404" is not a usable test for this. A retrieve route probed with an id that does
/// not exist answers 404 *correctly*, so the naive check confuses a working handler with a
/// missing one — and it did: these assertions passed only while the handlers were stubs
/// returning 200 for anything, and started failing the moment the stubs became real.
///
/// The two 404s are distinguishable by their body. Axum's router answers an unmounted path
/// with an empty body; every handler here answers with the API error envelope. So a 404
/// carrying a body is a handler that ran, which is exactly what "mounted" means.
async fn assert_mounted(router: axum::Router, method: &str, uri: &str) {
    let response = router
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");

    assert!(
        status != StatusCode::NOT_FOUND || !body.is_empty(),
        "{method} {uri} is not mounted (404 with an empty body is the router's, not a handler's)"
    );
}

#[tokio::test]
async fn catalog_taxonomy_routes_are_mounted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(ProductService::new(pool.clone()));
    let pricing = Arc::new(stheno_pricing::PricingService::new(pool.clone()));
    let inventory = Arc::new(stheno_inventory::InventoryService::new(pool));

    // Tags and types answered with hardcoded empty arrays and had no write routes at all.
    for (method, uri) in [
        ("GET", "/admin/product-tags"),
        ("POST", "/admin/product-tags"),
        ("GET", "/admin/product-tags/ptag_x"),
        ("POST", "/admin/product-tags/ptag_x"),
        ("DELETE", "/admin/product-tags/ptag_x"),
        ("GET", "/admin/product-types"),
        ("POST", "/admin/product-types"),
        ("GET", "/admin/product-types/ptyp_x"),
        ("DELETE", "/admin/product-types/ptyp_x"),
    ] {
        assert_mounted(
            admin_catalog_routes(service.clone(), pricing.clone(), inventory.clone()),
            method,
            uri,
        )
        .await;
    }

    db.drop().await;
}

#[tokio::test]
async fn rbac_routes_are_mounted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(RbacService::new(pool));

    for (method, uri) in [
        ("GET", "/admin/rbac/roles"),
        ("POST", "/admin/rbac/roles"),
        ("GET", "/admin/rbac/roles/admin"),
        ("DELETE", "/admin/rbac/roles/admin"),
        ("GET", "/admin/rbac/policies"),
        ("POST", "/admin/rbac/policies"),
        ("DELETE", "/admin/rbac/policies/pol_x"),
        ("POST", "/admin/rbac/evaluate"),
        ("GET", "/admin/rbac/audit"),
    ] {
        assert_mounted(admin_rbac_routes(service.clone()), method, uri).await;
    }

    db.drop().await;
}

#[tokio::test]
async fn fulfillment_settings_routes_are_mounted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(FulfillmentService::new(pool));

    for (method, uri) in [
        ("GET", "/admin/shipping-profiles"),
        ("POST", "/admin/shipping-profiles"),
        ("GET", "/admin/shipping-profiles/sp_x"),
        ("DELETE", "/admin/shipping-profiles/sp_x"),
        ("GET", "/admin/fulfillment-sets"),
        ("POST", "/admin/fulfillment-sets"),
        ("GET", "/admin/fulfillment-sets/fset_x/service-zones"),
        ("GET", "/admin/shipping-option-types"),
        ("GET", "/admin/fulfillment-providers"),
    ] {
        assert_mounted(
            admin_fulfillment_settings_routes(service.clone()),
            method,
            uri,
        )
        .await;
    }

    db.drop().await;
}

#[tokio::test]
async fn localisation_routes_are_mounted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let translation = Arc::new(TranslationService::new(pool.clone()));
    let settings = Arc::new(SettingsService::new(pool));

    for (method, uri) in [
        ("GET", "/admin/locales"),
        ("POST", "/admin/locales"),
        ("GET", "/admin/locales/en"),
        ("POST", "/admin/translations"),
        (
            "GET",
            "/admin/property-labels?resource=product&property=title",
        ),
        ("POST", "/admin/property-labels"),
        ("GET", "/admin/settings"),
        ("POST", "/admin/settings"),
    ] {
        assert_mounted(
            admin_localisation_routes(translation.clone(), settings.clone()),
            method,
            uri,
        )
        .await;
    }

    db.drop().await;
}

/// The bug this file previously could not have caught.
///
/// `/admin/customer-groups/{id}` was bound to `get(update_customer_group)` — a handler
/// that extracts `Json<UpdateCustomerGroupInput>`. A GET cannot carry that body, so
/// updating a customer group was impossible and no test noticed.
#[tokio::test]
async fn customer_group_mutations_are_not_bound_to_get() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(CustomerService::new(pool));

    for (method, uri) in [
        ("GET", "/admin/customer-groups"),
        ("POST", "/admin/customer-groups"),
        ("GET", "/admin/customer-groups/cgrp_x"),
        ("POST", "/admin/customer-groups/cgrp_x"),
        ("DELETE", "/admin/customer-groups/cgrp_x"),
    ] {
        assert_mounted(admin_customer_routes(service.clone()), method, uri).await;
    }

    db.drop().await;
}

/// A route that genuinely does not exist must still 404, or `assert_mounted` proves
/// nothing. This is the control for every assertion above.
#[tokio::test]
async fn an_unmounted_route_is_reported_as_missing() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(ProductService::new(pool.clone()));
    let pricing = Arc::new(stheno_pricing::PricingService::new(pool.clone()));
    let inventory = Arc::new(stheno_inventory::InventoryService::new(pool));
    let response = admin_catalog_routes(service, pricing, inventory)
        .oneshot(
            Request::builder()
                .uri("/admin/definitely-not-a-route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    db.drop().await;
}
