#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub sales_channel_ids: Vec<String>,
    pub category_ids: Vec<String>,
    pub tags: Vec<String>,
    pub price_cents: Option<i64>,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub document: SearchDocument,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub total_hits: usize,
    pub limit: usize,
    pub offset: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocumentEntity {
    pub id: Id<SearchDocumentEntity>,
    pub index_name: String,
    pub document_id: String,
    pub document: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for SearchDocumentEntity {
    const TABLE: &'static str = "search.search_document";
    const ENTITY: &'static str = "search_document";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

stheno_types::prefixed_id! {
    SearchDocumentEntity => "sdoc",
}
