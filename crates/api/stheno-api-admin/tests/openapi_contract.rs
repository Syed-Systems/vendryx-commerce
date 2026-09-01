//! The Admin API contract, asserted against `openapi.yaml` rather than a hand-written list.
//!
//! The failure this exists to catch is `405 Method Not Allowed`. Axum matches the path
//! first and the method second, so a route mounted with `get` and `delete` rejects the
//! dashboard's `POST` before any handler runs. `@medusajs/js-sdk` turns that into a thrown
//! `Error: Method Not Allowed`, the dashboard has no boundary for it, and the screen
//! renders an error card instead of the page. Twenty-seven operations were failing this way
//! and nothing in the test suite could see it, because every test named the routes it
//! expected and none of them asked what the dashboard actually calls.
//!
//! `openapi.yaml` is the Medusa Admin API 2.19.0 specification — the contract the forked
//! dashboard is built against. Reading it here means the expected set cannot drift from
//! the client, and new operations arrive as failures instead of as silence.
//!
//! Deliberately distinct from `route_coverage.rs`, which asks a different question ("does a
//! handler run, or is this an empty router 404?") and needs a database to answer it. This
//! one asserts only on method routing, which axum decides before it polls a handler or
//! touches state — so it needs no database, runs in seconds, and is usable as the inner
//! loop while the gap is being closed.
//!
//! A `404` is not a failure here. A route that was never built answering "not found" is a
//! deliberate state — 219 operations are in it today. Conflating that with `405` is what
//! makes a coverage assertion useless.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use stheno_db::create_pool;
use tower::ServiceExt;

use stheno_api_admin::{
    admin_api_key_routes, admin_catalog_routes, admin_config_routes, admin_customer_routes,
    admin_fulfillment_settings_routes, admin_inventory_routes, admin_localisation_routes,
    admin_order_routes, admin_pricing_routes, admin_promotion_routes, admin_rbac_routes,
    admin_reports_routes, admin_store_routes, admin_user_routes, admin_views_routes,
};

/// Tags whose screens this fork has already deleted from `apps/admin`.
///
/// Nothing calls these, so nothing needs to answer them. The list is explicit rather than
/// inferred so that deleting a feature stays a decision somebody wrote down: if a screen
/// comes back, its tag comes off this list and the operations reappear as failures.
///
/// Verified 2026-09-01 — no file under `apps/admin/src` matches `draft-order`,
/// `gift-card`, `store-credit` or `tax-provider`.
const TAGS_WITHOUT_UI: &[&str] = &[
    "Draft Orders",
    "Gift Cards",
    "Store Credit Accounts",
    "Tax Providers",
    "Index",
];

/// Operations the dashboard calls that the Medusa specification does not describe.
///
/// The RBAC surface is Stheno's own (ADR-008). Nothing upstream will ever list it, so it
/// is asserted here by hand or it is not asserted at all.
const STHENO_ONLY_OPERATIONS: &[(&str, &str)] = &[
    ("GET", "/admin/rbac/roles/{name}"),
    ("POST", "/admin/rbac/roles/{name}"),
    ("GET", "/admin/rbac/policies/{id}"),
    ("POST", "/admin/rbac/policies/{id}"),
];

const HTTP_VERBS: &[&str] = &["get", "post", "put", "patch", "delete"];

#[derive(Debug, Clone)]
struct Operation {
    method: String,
    path: String,
    tag: String,
}

/// Every operation in the specification's `paths:` block.
///
/// `serde_yaml` rather than a hand-rolled scan: the parser is already in the lock file, and
/// a line reader that depends on the generated document's exact indentation is a second
/// thing to maintain for no gain. The document is 3.8 MB, so this is the slow part of the
/// test — it still costs less than the database this test does not need.
fn load_operations() -> Vec<Operation> {
    #[derive(serde::Deserialize)]
    struct Spec {
        paths: BTreeMap<String, BTreeMap<String, serde_yaml::Value>>,
    }

    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../openapi.yaml"
    ))
    .expect("openapi.yaml must be readable from the repository root");

    let spec: Spec = serde_yaml::from_str(&raw).expect("openapi.yaml must parse");

    let mut operations = Vec::new();
    for (path, item) in spec.paths {
        for verb in HTTP_VERBS {
            let Some(op) = item.get(*verb) else { continue };
            let tag = op
                .get("tags")
                .and_then(|t| t.as_sequence())
                .and_then(|t| t.first())
                .and_then(|t| t.as_str())
                .unwrap_or("(untagged)")
                .to_string();
            operations.push(Operation {
                method: verb.to_uppercase(),
                path: path.clone(),
                tag,
            });
        }
    }

    assert!(
        operations.len() > 300,
        "parsed only {} operations from openapi.yaml — the paths: block layout changed",
        operations.len()
    );
    operations
}

/// The admin surface exactly as `stheno-server` composes it, minus the auth layer.
///
/// The credential check wraps the whole admin router, so a production-shaped router
/// answers every unauthenticated probe with 401 and hides the method routing underneath.
/// This composes the same mount functions without it, which is the layer under test.
fn admin_router() -> Router {
    // Never connected. `create_pool` builds a deadpool handle without dialling, and 405 is
    // decided before a handler is polled — so no probe in this test needs a database. Port
    // 1 on loopback refuses instantly, so the handlers that *do* run fail fast instead of
    // holding the test open for a connect timeout.
    let pool = create_pool("postgres://stheno:stheno@127.0.0.1:1/stheno_contract_probe", 1)
        .expect("pool handle");

    let product = Arc::new(stheno_product::ProductService::new(pool.clone()));
    let pricing = Arc::new(stheno_pricing::PricingService::new(pool.clone()));
    let inventory = Arc::new(stheno_inventory::InventoryService::new(pool.clone()));
    let stock_location = Arc::new(stheno_stock_location::StockLocationService::new(pool.clone()));
    let order = Arc::new(stheno_order::OrderService::new(pool.clone()));
    let fulfillment = Arc::new(stheno_fulfillment::FulfillmentService::new(pool.clone()));
    let payment = Arc::new(stheno_payment::PaymentService::new(pool.clone()));
    let customer = Arc::new(stheno_customer::CustomerService::new(pool.clone()));
    let promotion = Arc::new(stheno_promotion::PromotionService::new(pool.clone()));
    let region = Arc::new(stheno_region::RegionService::new(pool.clone()));
    let currency = Arc::new(stheno_currency::CurrencyService::new(pool.clone()));
    let tax = Arc::new(stheno_tax::TaxService::new(pool.clone()));
    let sales_channel = Arc::new(stheno_sales_channel::SalesChannelService::new(pool.clone()));
    let user = Arc::new(stheno_user::UserService::new(pool.clone()));
    let store = Arc::new(stheno_store::StoreService::new(pool.clone()));
    let translation = Arc::new(stheno_translation::TranslationService::new(pool.clone()));
    let settings = Arc::new(stheno_settings::SettingsService::new(pool.clone()));
    let rbac = Arc::new(stheno_rbac::RbacService::new(pool.clone()));
    let api_keys = Arc::new(stheno_api_key::ApiKeyService::new(pool.clone()));
    let reporting = Arc::new(stheno_reporting::ReportingService::new(pool.clone()));

    Router::new()
        .merge(admin_catalog_routes(
            product.clone(),
            pricing.clone(),
            inventory.clone(),
        ))
        .merge(admin_order_routes(
            order,
            fulfillment.clone(),
            payment.clone(),
        ))
        .merge(admin_customer_routes(customer))
        .merge(admin_inventory_routes(inventory, stock_location))
        .merge(admin_pricing_routes(pricing))
        .merge(admin_promotion_routes(promotion))
        .merge(admin_user_routes(user))
        .merge(admin_store_routes(store, translation.clone()))
        .merge(admin_config_routes(
            region,
            currency,
            tax,
            sales_channel,
            payment,
            Arc::new(stheno_links::PgLinkStore::new(pool)),
        ))
        .merge(admin_api_key_routes(api_keys))
        .merge(admin_views_routes())
        .merge(admin_rbac_routes(rbac))
        .merge(admin_fulfillment_settings_routes(fulfillment))
        .merge(admin_localisation_routes(translation, settings))
        .merge(admin_reports_routes(reporting))
}

/// Substitutes a placeholder for every `{param}` segment.
///
/// The value is deliberately not a valid `Id<T>` — this asserts on method routing, and a
/// handler that runs and rejects the id has already proved the verb is mounted.
fn probe_uri(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut in_param = false;
    for ch in path.chars() {
        match ch {
            '{' => {
                in_param = true;
                out.push('x');
            }
            '}' => in_param = false,
            _ if in_param => {}
            _ => out.push(ch),
        }
    }
    out
}

/// The set of verbs mounted at `uri`, or `None` if no route matches the path at all.
///
/// Probing with `TRACE` — a verb nothing mounts — makes every matched path answer 405, and
/// axum puts the permitted verbs in the `Allow` header of that response. So one request per
/// path reads the whole mounted method set *without ever polling a handler*.
///
/// This is what makes the test usable as an inner loop. Sending each operation's real verb
/// works too, and was the first version, but it executes ~150 handlers that reach for a
/// database this test deliberately does not have — three and a half minutes of connection
/// failures to answer a question that is pure routing.
async fn mounted_methods(router: Router, uri: &str) -> Option<BTreeSet<String>> {
    let response = router
        .oneshot(
            Request::builder()
                .method("TRACE")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router responds");

    if response.status() != StatusCode::METHOD_NOT_ALLOWED {
        // No route matched the path. Every operation on it is 404-class, which is a
        // deliberate state, not this test's concern.
        return None;
    }

    let allow = response
        .headers()
        .get(axum::http::header::ALLOW)
        .expect("axum sets Allow on a 405")
        .to_str()
        .expect("Allow is ascii")
        .to_string();

    Some(
        allow
            .split(',')
            .map(|m| m.trim().to_uppercase())
            .filter(|m| !m.is_empty())
            .collect(),
    )
}

#[tokio::test]
async fn no_admin_operation_answers_method_not_allowed() {
    let skipped: BTreeSet<&str> = TAGS_WITHOUT_UI.iter().copied().collect();

    // `/auth` and the MFA surface are mounted by `stheno-api-auth`, outside this router.
    // Probing them here would assert nothing and report a pass for it.
    let mut expected: Vec<Operation> = load_operations()
        .into_iter()
        .filter(|op| op.path.starts_with("/admin/"))
        .filter(|op| !skipped.contains(op.tag.as_str()))
        .collect();

    for (method, path) in STHENO_ONLY_OPERATIONS {
        expected.push(Operation {
            method: (*method).to_string(),
            path: (*path).to_string(),
            tag: "RBAC (Stheno)".to_string(),
        });
    }

    // Built once and cloned per probe; `oneshot` consumes the router.
    let router = admin_router();

    // One probe per path rather than per operation — the Allow header answers for all of
    // that path's verbs at once.
    let mut by_path: BTreeMap<String, Vec<&Operation>> = BTreeMap::new();
    for op in &expected {
        by_path.entry(op.path.clone()).or_default().push(op);
    }

    let mut failures: Vec<String> = Vec::new();
    for (path, ops) in &by_path {
        let Some(allowed) = mounted_methods(router.clone(), &probe_uri(path)).await else {
            continue; // path not mounted at all — 404-class
        };
        for op in ops {
            if !allowed.contains(&op.method) {
                failures.push(format!(
                    "  405  {:6} {}   mounted: {}   [{}]",
                    op.method,
                    op.path,
                    allowed.iter().cloned().collect::<Vec<_>>().join(","),
                    op.tag
                ));
            }
        }
    }

    failures.sort();
    assert!(
        failures.is_empty(),
        "{} of {} admin operations answer 405 Method Not Allowed.\n\
         The path is mounted; the verb the dashboard sends is not. Every one of these \
         renders an error card in the panel.\n\n{}\n",
        failures.len(),
        expected.len(),
        failures.join("\n")
    );
}
