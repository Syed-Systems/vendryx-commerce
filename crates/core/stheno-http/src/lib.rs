//! `stheno-http`: Axum extractors, middleware, CSRF, rate-limiting, egress-guard, and standard error formatting.

pub mod csrf;
pub mod egress_guard;
pub mod owned;
pub mod rate_limit;
pub mod redaction;
pub mod scope_resolution;

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use stheno_types::Error;

pub use csrf::{validate_csrf, CSRF_COOKIE_NAME, CSRF_HEADER_NAME};
pub use egress_guard::EgressGuard;
pub use owned::{Owned, ScopeChecker};
pub use rate_limit::{resolve_client_ip, RateLimiter};
pub use redaction::redact_value;
pub use scope_resolution::{
    extract_publishable_key, resolve_sales_channel_scope, SalesChannelScope, PUBLISHABLE_KEY_HEADER,
};

pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug)]
pub struct HttpError(pub Error);

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, error_type, message, code) = match self.0 {
            Error::NotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{} with id {} was not found", entity, id),
                Some("not_found".to_string()),
            ),
            Error::Invalid { code, message, .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_data",
                message,
                Some(code.to_string()),
            ),
            Error::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                msg,
                Some("unauthorized".to_string()),
            ),
            Error::Forbidden { permission } => (
                StatusCode::NOT_FOUND, // AGENTS.md §2.10: Return 404, not 403, to prevent existence oracle
                "not_found",
                format!(
                    "Resource not found or insufficient permissions for {}",
                    permission
                ),
                Some("not_found".to_string()),
            ),
            Error::Conflict { code, message } => (
                StatusCode::CONFLICT,
                "conflict",
                message,
                Some(code.to_string()),
            ),
            Error::CurrencyMismatch { expected, found } => (
                StatusCode::BAD_REQUEST,
                "invalid_data",
                format!("Currency mismatch: expected {}, found {}", expected, found),
                Some("currency_mismatch".to_string()),
            ),
            Error::InvalidCurrency(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_data",
                msg,
                Some("invalid_currency".to_string()),
            ),
            Error::AmountOverflow => (
                StatusCode::BAD_REQUEST,
                "invalid_data",
                "Amount overflow".to_string(),
                Some("amount_overflow".to_string()),
            ),
            Error::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                err,
                Some("database_error".to_string()),
            ),
            Error::ProviderFailed { provider, source } => (
                StatusCode::BAD_GATEWAY,
                "provider_error",
                format!("Provider {} failed: {}", provider, source),
                Some("provider_failed".to_string()),
            ),
            Error::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                err.to_string(),
                Some("internal_error".to_string()),
            ),
        };

        let body = Json(ErrorResponse {
            message,
            error_type: error_type.to_string(),
            code,
        });

        (status, body).into_response()
    }
}

/// Helper function to extract Idempotency-Key from incoming headers.
pub fn extract_idempotency_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|val| val.to_str().ok())
        .map(|s| s.trim().to_string())
}
