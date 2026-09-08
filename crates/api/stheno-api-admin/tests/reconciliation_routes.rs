//! P24-10: Admin reconciliation routes: per-cycle owed, matched, and unmatched.
//! Verified by: cargo test -p stheno-api-admin --test reconciliation_routes

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_admin::admin_reconciliation_routes;
use stheno_payment::{
    AuthorizePaymentSessionInput, CodDeliveryConfirmationInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService,
};
use stheno_payment_offline::OfflinePaymentProvider;
use tower::ServiceExt;

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    let val = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, val)
}

#[tokio::test]
async fn test_admin_reconciliation_routes_lifecycle() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");

    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_provider(Arc::new(OfflinePaymentProvider::new(Some(
        "pp_system_default",
    ))));

    // 1. Create and capture an order
    let col = payment_service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "BDT".to_string(),
            amount: dec!(2500.00),
            metadata: None,
        })
        .await
        .expect("col");
    let ses = payment_service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(2500.00),
                currency_code: "BDT".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("ses");
    let pay = payment_service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: ses.id.clone(),
            context: None,
        })
        .await
        .expect("pay");

    payment_service
        .confirm_cod_delivery(CodDeliveryConfirmationInput {
            payment_id: Some(pay.id),
            payment_session_id: None,
            delivery_event_id: "del_route_001".to_string(),
            collected_amount: Some(dec!(2500.00)),
            courier_id: Some("pathao".to_string()),
            consignment_id: Some("cons_admin_route_01".to_string()),
            tracking_code: Some("trk_admin_route_01".to_string()),
            metadata: None,
        })
        .await
        .expect("confirm delivery");

    let router = admin_reconciliation_routes(payment_service.clone());

    // 2. POST /admin/reconciliation/import — import courier settlement report
    let import_body = json!({
        "provider_id": "pathao",
        "courier_cycle_id": "pth_invoice_2026_w36",
        "currency": "BDT",
        "lines": [
            {
                "consignment_id": "cons_admin_route_01",
                "tracking_code": "trk_admin_route_01",
                "order_id": "ord_route_01",
                "collected_amount": 2500.00,
                "courier_fee": 80.00,
                "cod_fee": 25.00,
                "remitted_amount": 2395.00
            },
            {
                "consignment_id": "cons_unmatched_route_02",
                "tracking_code": "trk_unmatched_route_02",
                "order_id": "ord_route_02",
                "collected_amount": 1000.00,
                "courier_fee": 60.00,
                "cod_fee": 10.00,
                "remitted_amount": 930.00
            }
        ]
    });

    let (status, import_res) = call(
        &router,
        Request::builder()
            .method("POST")
            .uri("/admin/reconciliation/import")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&import_body).unwrap()))
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(import_res["import"]["imported_lines"], 2);
    let cycle_id = import_res["import"]["cycle_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. GET /admin/reconciliation/cycles — list settlement cycles
    let (status, cycles_res) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri("/admin/reconciliation/cycles?provider_id=pathao")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let cycles = cycles_res["cycles"].as_array().expect("cycles array");
    assert!(!cycles.is_empty());
    assert_eq!(cycles[0]["courier_cycle_id"], "pth_invoice_2026_w36");

    // 4. GET /admin/reconciliation/cycles/{id} — get cycle detail
    let (status, cycle_detail) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri(format!("/admin/reconciliation/cycles/{cycle_id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(cycle_detail["cycle"]["id"], cycle_id);
    assert_eq!(cycle_detail["cycle"]["provider_id"], "pathao");

    // 5. GET /admin/reconciliation/cycles/{id}/lines — list settlement lines
    let (status, lines_res) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri(format!("/admin/reconciliation/cycles/{cycle_id}/lines"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let lines = lines_res["lines"].as_array().expect("lines array");
    assert_eq!(lines.len(), 2);

    // 6. POST /admin/reconciliation/cycles/{id}/reconcile — run reconciliation
    let (status, rec_res) = call(
        &router,
        Request::builder()
            .method("POST")
            .uri(format!("/admin/reconciliation/cycles/{cycle_id}/reconcile"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let rep = &rec_res["report"];
    assert_eq!(rep["cycle_id"], cycle_id);
    assert_eq!(rep["matched_count"], 1, "cons_admin_route_01 is matched");
    assert_eq!(
        rep["remitted_not_matched_count"], 1,
        "cons_unmatched_route_02 has no order"
    );
    assert_eq!(rep["state"], "drift_found");

    // 7. GET /admin/reconciliation/cycles/{id}/alerts — check alerts
    let (status, alerts_res) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri(format!(
                "/admin/reconciliation/cycles/{cycle_id}/alerts?cycle_days=0"
            ))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let alerts = alerts_res["alerts"].as_array().expect("alerts array");
    // With threshold 0 days, the unmatched line raises an alert
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0]["consignment_id"], "cons_unmatched_route_02");
}
