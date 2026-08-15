//! Store Returns API Parity Tests (TASKS P6-20, F6, F20).
use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::store_return_routes;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_returns_endpoints_parity() {
    let order_service = Arc::new(OrderService::new());
    let return_flows = Arc::new(ReturnWorkflowEngine::new(order_service.clone(), None));

    // 1. Create base order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_ret_parity".to_string()),
            sales_channel_id: Some("sc_default".to_string()),
            region_id: Some("reg_us".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Parity".to_string()),
                last_name: Some("Tester".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("parity@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Leather Wallet".to_string(),
                quantity: 1,
                unit_price: dec!(45.00),
                variant_id: Some("var_wallet".to_string()),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }]),
            shipping_methods: Some(vec![]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    let item_id = order.items[0].id.clone();

    let app = store_return_routes(order_service.clone(), return_flows.clone());

    // ── Test 1: POST /store/returns ─────────────────────────────────────────
    let create_payload = serde_json::json!({
        "order_id": order.id,
        "location_id": "loc_default",
        "items": [{
            "id": item_id,
            "quantity": 1,
            "reason_id": "reason_defective",
            "note": "Broken zipper"
        }]
    });

    let req = Request::builder()
        .uri("/store/returns")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("return").is_some());
    let ret_id = json["return"]["id"].as_str().unwrap().to_string();
    assert_eq!(json["return"]["status"], "requested");

    // ── Test 2: GET /store/returns ──────────────────────────────────────────
    let req = Request::builder()
        .uri(format!("/store/returns?order_id={}", order.id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["count"], 1);
    assert_eq!(json["returns"][0]["id"], ret_id);

    // ── Test 3: GET /store/returns/{id} ─────────────────────────────────────
    let req = Request::builder()
        .uri(format!("/store/returns/{ret_id}"))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["return"]["id"], ret_id);
    assert_eq!(json["return"]["items"].as_array().unwrap().len(), 1);
}
