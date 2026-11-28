//! Public HTTP cache headers helper according to ADR-024.
//!
//! Emits:
//! - `Cache-Control: public, s-maxage=3600, stale-while-revalidate=86400`
//! - `Vary: Accept-Encoding` (never `Vary: Cookie`)
//! - `Surrogate-Key` and `Cache-Tag` for CDN edge purging
//! - Guaranteed zero `Set-Cookie` on public cacheable paths

use axum::http::{header, HeaderMap, HeaderValue};

/// Returns standard ADR-024 HTTP caching headers for public store endpoints.
pub fn public_cache_headers(
    tenant_id: &str,
    entities: &[(&str, &str)],
    include_listing: bool,
) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let mut tags = Vec::with_capacity(entities.len() + usize::from(include_listing));
    for (entity, id) in entities {
        let Some(tag) = entity_tag(tenant_id, entity, id) else {
            return no_store(headers);
        };
        tags.push(tag);
    }
    if include_listing {
        let Some(tag) = listing_tag(tenant_id) else {
            return no_store(headers);
        };
        tags.push(tag);
    }
    if tags.is_empty() || tags.len() > 1000 {
        return no_store(headers);
    }

    let cache_tags = tags.join(",");
    let surrogate_keys = tags.join(" ");
    if cache_tags.len() > 16 * 1024 {
        return no_store(headers);
    }

    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, s-maxage=3600, stale-while-revalidate=86400"),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    let (Ok(cache_tags), Ok(surrogate_keys)) = (
        HeaderValue::from_str(&cache_tags),
        HeaderValue::from_str(&surrogate_keys),
    ) else {
        return no_store(headers);
    };
    headers.insert(header::HeaderName::from_static("cache-tag"), cache_tags);
    headers.insert(
        header::HeaderName::from_static("surrogate-key"),
        surrogate_keys,
    );
    headers
}

/// Formats one public cache tag for the supported catalog entity types.
pub fn entity_tag(tenant_id: &str, entity: &str, id: &str) -> Option<String> {
    if !valid_atom(tenant_id)
        || !matches!(entity, "product" | "collection" | "category" | "region")
        || id.is_empty()
        || id.len() > 900
    {
        return None;
    }

    let tag = format!("tenant:{tenant_id}:{entity}:{}", encode_tag_atom(id));
    (tag.len() <= 1024).then_some(tag)
}

/// Formats the one tenant-wide tag attached to catalog listings.
pub fn listing_tag(tenant_id: &str) -> Option<String> {
    if !valid_atom(tenant_id) {
        return None;
    }
    Some(format!("tenant:{tenant_id}:listing"))
}

fn valid_atom(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 900
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn encode_tag_atom(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn no_store(mut headers: HeaderMap) -> HeaderMap {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    headers
}

#[cfg(test)]
mod tests {
    use super::{entity_tag, listing_tag, public_cache_headers};
    use axum::http::header;

    #[test]
    fn public_tags_are_tenant_scoped_and_use_header_specific_separators() {
        let entities = [("product", "prod_01"), ("region", "reg_02")];
        let headers = public_cache_headers("ten_abc", &entities, true);

        assert_eq!(
            headers.get("cache-tag").and_then(|value| value.to_str().ok()),
            Some("tenant:ten_abc:product:prod_01,tenant:ten_abc:region:reg_02,tenant:ten_abc:listing")
        );
        assert_eq!(
            headers
                .get("surrogate-key")
                .and_then(|value| value.to_str().ok()),
            Some("tenant:ten_abc:product:prod_01 tenant:ten_abc:region:reg_02 tenant:ten_abc:listing")
        );
        assert!(headers.get(header::SET_COOKIE).is_none());
        assert_eq!(
            headers
                .get(header::VARY)
                .and_then(|value| value.to_str().ok()),
            Some("Accept-Encoding")
        );
    }

    #[test]
    fn unscoped_or_invalid_tags_disable_public_caching() {
        let headers = public_cache_headers("", &[("product", "prod_01")], true);
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert!(headers.get("cache-tag").is_none());
        assert!(headers.get("surrogate-key").is_none());

        assert_eq!(
            entity_tag("ten_abc", "product", "bad id").as_deref(),
            Some("tenant:ten_abc:product:bad%20id")
        );
        assert_eq!(entity_tag("ten_abc", "product", ""), None);
        assert_eq!(listing_tag("bad tenant"), None);
    }

    #[test]
    fn bangla_product_handle_has_a_deterministic_ascii_tag_atom() {
        assert_eq!(
            entity_tag("ten_abc", "product", "শার্ট").as_deref(),
            Some("tenant:ten_abc:product:%E0%A6%B6%E0%A6%BE%E0%A6%B0%E0%A7%8D%E0%A6%9F")
        );
    }
}
