//! Boundary Data Transfer Objects (DTOs) for `stheno-demo`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoDTO {
    pub id: Id<DemoDTO>,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDemoInput {
    pub name: String,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    DemoDTO => "demo",
}
