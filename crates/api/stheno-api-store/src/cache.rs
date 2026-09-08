//! Public HTTP cache headers helper according to ADR-024.
//!
//! Emits:
//! - `Cache-Control: public, s-maxage=3600, stale-while-revalidate=86400`
//! - `Vary: Accept-Encoding` (never `Vary: Cookie`)
//! - `Surrogate-Key` and `Cache-Tag` for CDN edge purging
//! - Guaranteed zero `Set-Cookie` on public cacheable paths

use axum::http::{header, HeaderMap, HeaderValue};

/// Returns standard ADR-024 HTTP caching headers for public store endpoints.
pub fn public_cache_headers(tags: &[&str]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, s-maxage=3600, stale-while-revalidate=86400"),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    let tag_str = tags.join(" ");
    if let Ok(val) = HeaderValue::from_str(&tag_str) {
        headers.insert(
            header::HeaderName::from_static("surrogate-key"),
            val.clone(),
        );
        headers.insert(header::HeaderName::from_static("cache-tag"), val);
    }
    headers
}
