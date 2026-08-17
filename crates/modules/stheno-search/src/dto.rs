use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub category_ids: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub min_price_cents: Option<i64>,
    pub max_price_cents: Option<i64>,
    pub sales_channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub filter: Option<SearchFilter>,
    pub limit: usize,
    pub offset: usize,
    pub sort_by: Option<String>, // "price_asc", "price_desc", "relevance"
}

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
