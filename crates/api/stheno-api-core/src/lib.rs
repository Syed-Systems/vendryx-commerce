//! `stheno-api-core`: Shared response envelopes and DTO wrappers for Medusa v2 wire-compatibility.

use serde::{Deserialize, Serialize};

/// Standard envelope for list endpoints returning offset paginated data.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    /// The page of resources.
    /// This field must match the exact plural name of the entity in Medusa (e.g. `products`, `orders`).
    /// We will allow custom field mapping or generic serialization.
    pub data: T,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Standard envelope for retrieve/get endpoints returning a single resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse<T> {
    /// The single resource.
    /// This field must match the exact singular name of the entity in Medusa.
    pub data: T,
}

/// Standard envelope for deletion responses.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

/// Standard flat error response matching Medusa v2 exact shape (Rule 2.2 / TRD §6).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub r#type: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(r#type: &str, message: &str) -> Self {
        Self {
            r#type: r#type.to_string(),
            message: message.to_string(),
        }
    }
}
