use axum::http::{HeaderMap, HeaderValue};
use stheno_http::{resolve_client_ip, RateLimiter};

#[test]
fn test_trusted_proxy_ip_resolution() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-forwarded-for",
        HeaderValue::from_static("203.0.113.195, 10.0.0.1"),
    );

    let trusted_proxies = vec!["10.0.0.1"];
    let client_ip = resolve_client_ip(&headers, Some("10.0.0.1"), &trusted_proxies);
    assert_eq!(client_ip, "203.0.113.195");

    // Untrusted proxy directly returns peer addr
    let untrusted_client = resolve_client_ip(&headers, Some("198.51.100.2"), &trusted_proxies);
    assert_eq!(untrusted_client, "198.51.100.2");
}

#[test]
fn test_rate_limiter_threshold() {
    let limiter = RateLimiter::new(3, 60);

    assert!(limiter.check("ip_1.2.3.4").is_ok());
    assert!(limiter.check("ip_1.2.3.4").is_ok());
    assert!(limiter.check("ip_1.2.3.4").is_ok());

    // 4th request in same window fails
    assert!(limiter.check("ip_1.2.3.4").is_err());

    // Different IP is untouched
    assert!(limiter.check("ip_5.6.7.8").is_ok());
}
