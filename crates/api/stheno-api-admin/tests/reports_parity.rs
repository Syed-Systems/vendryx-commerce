use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::NaiveDate;
use rust_decimal_macros::dec;
use serde_json::Value;
use std::sync::Arc;
use stheno_api_admin::admin_reports_routes;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{OrderPlacedRollupItem, RecordOrderPlacedInput, ReportingService};
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_reports_envelope_shape_and_csv() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let reporting_service = Arc::new(ReportingService::new(pool.clone()));

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();
    reporting_service
        .record_order_placed(RecordOrderPlacedInput {
            date,
            currency: "USD".to_string(),
            total_revenue: dec!(250.00),
            units: 3,
            items: vec![OrderPlacedRollupItem {
                product_id: "prod_1".to_string(),
                title: "Leather Jacket".to_string(),
                units: 3,
                total_revenue: dec!(250.00),
            }],
        })
        .await
        .expect("record order");

    let app = admin_reports_routes(reporting_service.clone());

    // 1. GET /admin/reports/revenue-over-time
    let req = Request::builder()
        .uri("/admin/reports/revenue-over-time?currency=USD")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["report"]["report_type"], "revenue_over_time");
    assert_eq!(json["report"]["data"]["currency"], "USD");
    assert_eq!(json["report"]["data"]["total_revenue"], "250.00");

    // 2. GET /admin/reports/orders-over-time
    let req = Request::builder()
        .uri("/admin/reports/orders-over-time")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["report"]["report_type"], "orders_over_time");
    assert_eq!(json["report"]["data"]["total_orders"], 1);
    assert_eq!(json["report"]["data"]["total_units"], 3);

    // 3. GET /admin/reports/top-products
    let req = Request::builder()
        .uri("/admin/reports/top-products?limit=5")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["report"]["report_type"], "top_products");
    assert_eq!(
        json["report"]["data"]["top_products"][0]["product_id"],
        "prod_1"
    );
    assert_eq!(
        json["report"]["data"]["top_products"][0]["title"],
        "Leather Jacket"
    );

    // 4. GET /admin/reports/average-order-value
    let req = Request::builder()
        .uri("/admin/reports/average-order-value?currency=USD")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["report"]["report_type"], "average_order_value");
    assert_eq!(json["report"]["data"]["average_order_value"], "250.00");

    // 5. GET /admin/reports/revenue-over-time?format=csv
    let req = Request::builder()
        .uri("/admin/reports/revenue-over-time?currency=USD&format=csv")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/csv"));
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let csv_str = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(csv_str.starts_with('\u{FEFF}'));
    assert!(csv_str.contains("Date,Currency,Revenue"));
    assert!(csv_str.contains("2026-08-17,USD,250.00"));

    db.drop().await;
}
