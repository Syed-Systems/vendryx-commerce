//! `stheno-http::csrf`: CSRF double-submit token and Origin validation (SECURITY §6).

use axum::http::{HeaderMap, Method};
use stheno_types::{Error, Result};

pub const CSRF_HEADER_NAME: &str = "X-CSRF-Token";
pub const CSRF_COOKIE_NAME: &str = "stheno_csrf";

pub fn validate_csrf(method: &Method, headers: &HeaderMap, allowed_origins: &[&str]) -> Result<()> {
    // Safe idempotent methods do not require CSRF validation
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
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
