//! Public DTOs for `stheno-sales-channel`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesChannelDTO {
    pub id: Id<SalesChannelDTO>,
    pub name: String,
    pub description: Option<String>,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSalesChannelInput {
    pub name: String,
    pub description: Option<String>,
    pub is_disabled: Option<bool>,
}
