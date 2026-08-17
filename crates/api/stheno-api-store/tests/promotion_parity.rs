use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::store_cart_routes_with_promotions;
use stheno_cart::CartService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{CreateApplicationMethodInput, CreatePromotionInput, PromotionService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_cart_promotion_application_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let cart_service = Arc::new(CartService::new(pool.clone()));
    let promo_service = Arc::new(PromotionService::new(pool));

    // 1. Create a 20% discount promotion
    promo_service
        .create_promotion(CreatePromotionInput {
            code: "PROMO20".to_string(),
            promotion_type: Some("standard".to_string()),
            is_automatic: Some(false),
            campaign_id: None,
            status: Some("active".to_string()),
            application_method: Some(CreateApplicationMethodInput {
                method_type: "percentage".to_string(),
                target_type: "items".to_string(),
                allocation: Some("across".to_string()),
                value: dec!(20),
                currency_code: None,
                max_quantity: None,
                apply_to_quantity: None,
                buy_rules_min_quantity: None,
                target_rules: None,
                buy_rules: None,
                metadata: None,
            }),
            rules: None,
            metadata: None,
        })
        .await
        .expect("promotion created");

    let app = store_cart_routes_with_promotions(cart_service.clone(), Some(promo_service));

    // 2. Create Cart: POST /store/carts
    let create_cart_payload = json!({
        "currency_code": "usd",
        "email": "customer@example.com"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/carts")
                .header("Content-Type", "application/json")
                .body(Body::from(create_cart_payload.to_string()))
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

    // 3. Add Line Item: POST /store/carts/{id}/line-items ($100 x 2 = $200)
    let add_item_payload = json!({
        "title": "Winter Jacket",
        "quantity": 2,
        "unit_price": 100.0
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

    // 4. Apply Promotion: POST /store/carts/{id}/promotions
    let promo_payload = json!({
        "promo_codes": ["PROMO20"]
    });

    let promo_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{}/promotions", cart_id))
                .header("Content-Type", "application/json")
                .body(Body::from(promo_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(promo_resp.status(), StatusCode::OK);
    let promo_bytes = axum::body::to_bytes(promo_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: stheno_api_store::carts::CartResponse =
        serde_json::from_slice(&promo_bytes).expect("deserialise CartResponse");

    assert_eq!(res.cart.items.len(), 1);
    assert_eq!(res.cart.items[0].adjustments.len(), 1);
    assert_eq!(
        res.cart.items[0].adjustments[0].code.as_deref(),
        Some("PROMO20")
    );
    assert_eq!(res.cart.items[0].adjustments[0].amount, dec!(40));

    // Totals verification
    assert_eq!(res.cart.totals.discount_total, dec!(40));
    assert_eq!(res.cart.totals.total, dec!(160));
}
