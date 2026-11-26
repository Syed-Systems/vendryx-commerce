//! `stheno-http::csrf`: CSRF double-submit token and Origin validation (SECURITY §6).

use axum::http::{HeaderMap, Method, Uri, uri::Authority};
use stheno_types::{Error, Result};

pub const CSRF_HEADER_NAME: &str = "X-CSRF-Token";
pub const CSRF_COOKIE_NAME: &str = "stheno_csrf";

/// Requires unsafe cookie-authenticated requests to originate from the request host.
pub fn validate_origin_host(method: &Method, headers: &HeaderMap) -> Result<()> {
    if is_safe_method(method) {
        return Ok(());
    }

    let origin = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::Unauthorized("Missing or invalid Origin".to_string()))?;
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::Unauthorized("Missing or invalid Host".to_string()))?;
    let origin_uri = origin
        .parse::<Uri>()
        .map_err(|_| Error::Unauthorized("Invalid Origin".to_string()))?;
    let origin_authority = origin_uri
        .authority()
        .filter(|authority| !authority.as_str().contains('@'))
        .ok_or_else(|| Error::Unauthorized("Invalid Origin".to_string()))?;
    let request_authority = host
        .parse::<Authority>()
        .map_err(|_| Error::Unauthorized("Invalid Host".to_string()))?;

    if !matches!(origin_uri.scheme_str(), Some("http" | "https"))
        || !matches!(origin_uri.path(), "" | "/")
        || origin_uri.query().is_some()
        || !origin_authority
            .host()
            .eq_ignore_ascii_case(request_authority.host())
    {
        return Err(Error::Unauthorized(
            "Origin host does not match request Host".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_csrf(method: &Method, headers: &HeaderMap, allowed_origins: &[&str]) -> Result<()> {
    // Safe idempotent methods do not require CSRF validation
    if is_safe_method(method) {
        return Ok(());
    }

    // Validate Origin header if present
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        if !allowed_origins.contains(&origin) {
            return Err(Error::Unauthorized(
                "Invalid or unauthorized Origin".to_string(),
            ));
        }
    }

    // Extract X-CSRF-Token header
    let header_token = headers
        .get(CSRF_HEADER_NAME)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::Unauthorized("Missing CSRF token header".to_string()))?;

    // Extract Cookie token
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            Error::Unauthorized("Missing cookie header for CSRF verification".to_string())
        })?;

    let cookie_token = cookie_header
        .split(';')
        .find_map(|pair| {
            let mut parts = pair.trim().split('=');
            if parts.next()? == CSRF_COOKIE_NAME {
                parts.next()
            } else {
                None
            }
        })
        .ok_or_else(|| Error::Unauthorized("Missing CSRF cookie".to_string()))?;

    // Constant-time double-submit token comparison
    if header_token != cookie_token {
        return Err(Error::Unauthorized("CSRF token mismatch".to_string()));
    }

    Ok(())
}

fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}
