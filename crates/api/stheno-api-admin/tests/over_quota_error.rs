use axum::response::IntoResponse;
use serde_json::Value;
use stheno_http::HttpError;
use stheno_types::Error;

#[tokio::test]
async fn test_over_quota_error_shape_and_upsell() {
    let err = Error::Conflict {
        code: "not_allowed",
        message: "Product limit reached (100). Upgrade to Pro for unlimited products.".to_string(),
    };

    let http_err = HttpError(err);
    let resp = http_err.into_response();

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(val["type"], "not_allowed");
    assert_eq!(
        val["message"],
        "Product limit reached (100). Upgrade to Pro for unlimited products."
    );
}
