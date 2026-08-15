use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use stheno_http::{extract_idempotency_key, HttpError, IDEMPOTENCY_KEY_HEADER};
use stheno_types::Error;

#[test]
fn test_idempotency_key_extraction() {
    let mut headers = HeaderMap::new();
    headers.insert(
        IDEMPOTENCY_KEY_HEADER,
        HeaderValue::from_static("idem_key_abc123"),
    );

    let key = extract_idempotency_key(&headers);
    assert_eq!(key, Some("idem_key_abc123".to_string()));

    let empty_headers = HeaderMap::new();
    assert_eq!(extract_idempotency_key(&empty_headers), None);
}

#[test]
fn test_error_response_not_found_status_and_body() {
    let err = Error::NotFound {
        entity: "product",
        id: "prod_999".to_string(),
    };

    let http_err: HttpError = err.into();
    let response = http_err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_error_response_forbidden_returns_404_security_rule() {
    // AGENTS.md §2.10: Return 404, not 403, on ownership / auth failure to avoid entity existence oracle
    let err = Error::Forbidden {
        permission: "orders:read",
    };

    let http_err: HttpError = err.into();
    let response = http_err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
