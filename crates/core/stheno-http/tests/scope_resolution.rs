//! Test P7-17: Publishable Key -> Sales Channel Scope resolution and 404 enforcement on out-of-scope access.

use axum::http::HeaderMap;
use std::collections::HashMap;
use stheno_http::{extract_publishable_key, resolve_sales_channel_scope, PUBLISHABLE_KEY_HEADER};

#[tokio::test]
async fn test_publishable_key_scope_resolution_and_404_enforcement() {
    let mut key_store = HashMap::new();
    key_store.insert(
        "pk_us_storefront_123".to_string(),
        vec!["sc_us_web".to_string(), "sc_us_mobile".to_string()],
    );

    // 1. Header extraction
    let mut headers = HeaderMap::new();
    headers.insert(
        PUBLISHABLE_KEY_HEADER,
        "pk_us_storefront_123".parse().unwrap(),
    );

    let extracted = extract_publishable_key(&headers);
    assert_eq!(extracted, Some("pk_us_storefront_123".to_string()));

    // 2. Resolve scope
    let scope = resolve_sales_channel_scope(&headers, &key_store).unwrap();
    assert_eq!(scope.allowed_sales_channel_ids.len(), 2);
    assert!(scope.is_channel_allowed("sc_us_web"));
    assert!(scope.is_channel_allowed("sc_us_mobile"));
    assert!(!scope.is_channel_allowed("sc_eu_web"));

    // 3. Enforce allowed channel -> Ok
    assert!(scope.enforce("sc_us_web").is_ok());

    // 4. Enforce out-of-scope channel -> returns 404 NotFound (F20, AGENTS.md §2.10)
    let out_of_scope_res = scope.enforce("sc_eu_web");
    assert!(out_of_scope_res.is_err());
    let err_str = out_of_scope_res.unwrap_err().to_string();
    assert!(err_str.contains("Not found: sales_channel with id sc_eu_web"));
}
