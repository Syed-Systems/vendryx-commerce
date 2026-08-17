//! P9-14: workflow-execution observability for the admin dashboard.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_api_admin_platform::{admin_platform_routes, AdminPlatformContext, UsageSnapshot};
use stheno_api_core::{Tier, TierConfig};
use stheno_workflow::{StepState, TransactionState, WorkflowStore};
use tower::ServiceExt;

const TIERS: &str = include_str!("../../../../config/tiers.toml");

fn app(store: Arc<WorkflowStore>) -> axum::Router {
    let entitlements = TierConfig::from_toml(TIERS)
        .unwrap()
        .resolve(Tier::Pro, &BTreeMap::new())
        .unwrap();

    admin_platform_routes(AdminPlatformContext {
        entitlements: Arc::new(entitlements),
        usage: Arc::new(UsageSnapshot::new()),
        workflows: store,
    })
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn empty_store_lists_no_executions() {
    let (status, json) = get_json(
        app(Arc::new(WorkflowStore::new())),
        "/admin/workflows-executions",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
    assert_eq!(json["workflow_executions"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn executions_are_listed_with_state_and_steps() {
    let store = Arc::new(WorkflowStore::new());
    store.get_or_create("txn_alpha", "create-product", json!({ "title": "Hoodie" }));
    store
        .update_state("txn_alpha", TransactionState::Invoking)
        .unwrap();
    store
        .set_step_state("txn_alpha", "CreateProductStep", StepState::Running)
        .unwrap();

    let (status, json) = get_json(app(store), "/admin/workflows-executions").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);

    let exec = &json["workflow_executions"][0];
    assert_eq!(exec["transaction_id"], "txn_alpha");
    assert_eq!(exec["workflow_name"], "create-product");
    // Step-level state is the whole point of the screen: an operator needs to see
    // which step a stuck saga is parked on.
    assert!(exec["step_states"]["CreateProductStep"].is_string());
}

#[tokio::test]
async fn a_single_execution_is_retrievable_by_transaction_id() {
    let store = Arc::new(WorkflowStore::new());
    store.get_or_create("txn_beta", "complete-cart", json!({ "cart_id": "cart_1" }));

    let (status, json) = get_json(app(store), "/admin/workflows-executions/txn_beta").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["workflow_execution"]["transaction_id"], "txn_beta");
    assert_eq!(json["workflow_execution"]["workflow_name"], "complete-cart");
}

#[tokio::test]
async fn an_unknown_transaction_id_is_a_medusa_shaped_404() {
    let (status, json) = get_json(
        app(Arc::new(WorkflowStore::new())),
        "/admin/workflows-executions/txn_missing",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["type"], "not_found");
    assert!(json["message"].as_str().unwrap().contains("txn_missing"));
}

#[tokio::test]
async fn executions_are_listed_newest_first() {
    let store = Arc::new(WorkflowStore::new());
    store.get_or_create("txn_old", "first", json!({}));
    // created_at has sub-second resolution; force a distinct ordering key.
    std::thread::sleep(std::time::Duration::from_millis(5));
    store.get_or_create("txn_new", "second", json!({}));

    let (_, json) = get_json(app(store), "/admin/workflows-executions").await;
    let list = json["workflow_executions"].as_array().unwrap();
    assert_eq!(list[0]["transaction_id"], "txn_new");
    assert_eq!(list[1]["transaction_id"], "txn_old");
}
