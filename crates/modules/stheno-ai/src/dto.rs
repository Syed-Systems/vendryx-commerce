//! DTOs for tenant-owned static merchant knowledge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Merchant-authored policy or FAQ content returned from the knowledge module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allowlisted input for creating a knowledge entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeEntry {
    pub title: String,
    pub category: Option<String>,
    pub content: String,
}

/// Allowlisted mutable fields for updating a knowledge entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKnowledgeEntry {
    pub title: String,
    pub category: Option<String>,
    pub content: String,
}

/// A static knowledge entry and its lexical relevance score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeSearchHit {
    pub entry: KnowledgeEntry,
    pub rank: f32,
}
