//! Publishable Key Sales-Channel Scope Resolution Middleware & Extractor (P7-17, F20).
//!
//! Rule 2.10: Out-of-scope access returns 404, not 403, preventing existence oracle attacks.

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use stheno_types::{Error, Result};

pub const PUBLISHABLE_KEY_HEADER: &str = "x-publishable-api-key";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SalesChannelScope {
    pub publishable_key: Option<String>,
    pub allowed_sales_channel_ids: Vec<String>,
}

impl SalesChannelScope {
    pub fn new(publishable_key: Option<String>, allowed_sales_channel_ids: Vec<String>) -> Self {
        Self {
            publishable_key,
            allowed_sales_channel_ids,
        }
    }

    pub fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if self.allowed_sales_channel_ids.is_empty() {
            // Unrestricted or no scope limits applied
            true
        } else {
            self.allowed_sales_channel_ids
                .iter()
                .any(|id| id == channel_id)
        }
    }

    /// Enforce that requested channel is allowed within this scope.
    /// Returns 404 NotFound if out of scope (AGENTS.md §2.10).
    pub fn enforce(&self, channel_id: &str) -> Result<()> {
        if self.is_channel_allowed(channel_id) {
            Ok(())
        } else {
            Err(Error::NotFound {
                entity: "sales_channel",
                id: channel_id.to_string(),
            })
        }
    }
}

/// Extract publishable API key string from request headers.
pub fn extract_publishable_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(PUBLISHABLE_KEY_HEADER)
        .and_then(|val| val.to_str().ok())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|val| val.to_str().ok())
                .and_then(|auth| {
                    if auth.starts_with("Bearer pk_") {
                        Some(auth[7..].trim().to_string())
                    } else {
                        None
                    }
                })
        })
}

/// Helper function to resolve scope from key and map of valid keys to channel IDs.
#[allow(clippy::result_large_err)]
pub fn resolve_sales_channel_scope(
    headers: &HeaderMap,
    key_store: &std::collections::HashMap<String, Vec<String>>,
) -> std::result::Result<SalesChannelScope, Response> {
    if let Some(key) = extract_publishable_key(headers) {
        if let Some(allowed_channels) = key_store.get(&key) {
            Ok(SalesChannelScope::new(Some(key), allowed_channels.clone()))
        } else {
            // Invalid key -> 401
            Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "type": "unauthorized",
                    "message": "Invalid publishable API key"
                })),
            )
                .into_response())
        }
    } else {
        // No key provided -> default empty scope
        Ok(SalesChannelScope::default())
    }
}
