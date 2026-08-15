use axum::http::{HeaderMap, HeaderValue, Method};
use stheno_http::{validate_csrf, CSRF_HEADER_NAME};

#[test]
fn test_csrf_safe_methods_bypass() {
    let headers = HeaderMap::new();
    let allowed_origins = vec!["https://store.example.com"];

    assert!(validate_csrf(&Method::GET, &headers, &allowed_origins).is_ok());
    assert!(validate_csrf(&Method::HEAD, &headers, &allowed_origins).is_ok());
    assert!(validate_csrf(&Method::OPTIONS, &headers, &allowed_origins).is_ok());
}

#[test]
fn test_csrf_double_submit_success() {
    let mut headers = HeaderMap::new();
    headers.insert(
        CSRF_HEADER_NAME,
        HeaderValue::from_static("token_secret_12345"),
    );
    headers.insert(
        "cookie",
        HeaderValue::from_static("stheno_csrf=token_secret_12345; session=xyz"),
    );
    headers.insert(
        "origin",
        HeaderValue::from_static("https://store.example.com"),
    );

    let allowed_origins = vec!["https://store.example.com"];
    assert!(validate_csrf(&Method::POST, &headers, &allowed_origins).is_ok());
}

#[test]
fn test_csrf_token_mismatch_fails() {
    let mut headers = HeaderMap::new();
    headers.insert(CSRF_HEADER_NAME, HeaderValue::from_static("attacker_token"));
    headers.insert(
        "cookie",
        HeaderValue::from_static("stheno_csrf=legit_token"),
    );

    let allowed_origins = vec!["https://store.example.com"];
    assert!(validate_csrf(&Method::POST, &headers, &allowed_origins).is_err());
}
