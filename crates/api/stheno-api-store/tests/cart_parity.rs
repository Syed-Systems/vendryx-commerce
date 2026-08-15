use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::{store_cart_routes, store_payment_routes};
use stheno_cart::CartService;
use stheno_payment::PaymentService;
use stheno_payment_offline::OfflinePaymentProvider;
use tower::ServiceExt;

#[tokio::test]
async fn test_store_cart_and_payment_endpoints_parity() {
    let cart_service = Arc::new(CartService::new());
    let payment_service = Arc::new(PaymentService::new());
    let offline_provider = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    payment_service.register_provider(offline_provider);

    let app = axum::Router::new()
        .merge(store_cart_routes(cart_service.clone()))
        .merge(store_payment_routes(payment_service.clone()));

    // 1. POST /store/carts
    let create_payload = json!({
        "region_id": "reg_us",
        "currency_code": "usd",
        "email": "test@example.com"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/carts")
                .header("Content-Type", "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let cart_id = body["cart"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["cart"]["currency_code"], "usd");

    // 2. GET /store/carts/{id}
    let get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/carts/{}", cart_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);

    // 3. POST /store/carts/{id}/line-items
    let add_item_payload = json!({
        "title": "Merino Wool Beanie",
        "quantity": 2,
        "unit_price": 25.0
    });

    let item_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{}/line-items", cart_id))
                .header("Content-Type", "application/json")
                .body(Body::from(add_item_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(item_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(item_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let item_body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(item_body["cart"]["items"].as_array().unwrap().len(), 1);
    let line_id = item_body["cart"]["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. POST /store/carts/{id}/line-items/{line_id} (update quantity)
    let update_item_payload = json!({
        "quantity": 3
    });

    let upd_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{}/line-items/{}", cart_id, line_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update_item_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(upd_resp.status(), StatusCode::OK);

    // 5. GET /store/payment-providers
    let pp_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/payment-providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(pp_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(pp_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let pp_body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(!pp_body["payment_providers"].as_array().unwrap().is_empty());
}
