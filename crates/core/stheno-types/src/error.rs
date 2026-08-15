use crate::currency::CurrencyCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("Invalid data: {code} - {message}")]
    Invalid {
        code: &'static str,
        message: String,
        field: Option<String>,
    },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: missing permission {permission}")]
    Forbidden { permission: &'static str },

    #[error("Conflict: {code} - {message}")]
    Conflict { code: &'static str, message: String },

    #[error("Currency mismatch: expected {expected}, found {found}")]
    CurrencyMismatch {
        expected: CurrencyCode,
        found: CurrencyCode,
    },

    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),

    #[error("Amount overflow")]
    AmountOverflow,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Provider failed: {provider} - {source}")]
    ProviderFailed {
        provider: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}
