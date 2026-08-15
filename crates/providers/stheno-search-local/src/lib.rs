//! Provider `stheno-search-local` for `stheno-search` (Full-Text & In-Memory Index)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use async_trait::async_trait;
use stheno_search::{SearchDocument, SearchHit, SearchProvider, SearchQuery, SearchResult};
use stheno_types::Result;

pub struct LocalSearchProvider {
    indexes: Arc<RwLock<HashMap<String, HashMap<String, SearchDocument>>>>,
}

impl Default for LocalSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSearchProvider {
    pub fn new() -> Self {
        Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SearchProvider for LocalSearchProvider {
    fn provider_id(&self) -> &'static str {
        "local_search"
    }

    async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<SearchDocument>,
    ) -> Result<()> {
        let mut map = self.indexes.write().unwrap();
        let index = map.entry(index_name.to_string()).or_default();
        for doc in documents {
            index.insert(doc.id.clone(), doc);
        }
        Ok(())
    }

    async fn delete_documents(&self, index_name: &str, document_ids: &[String]) -> Result<()> {
        let mut map = self.indexes.write().unwrap();
        if let Some(index) = map.get_mut(index_name) {
            for id in document_ids {
                index.remove(id);
            }
        }
        Ok(())
    }

    async fn clear_index(&self, index_name: &str) -> Result<()> {
        let mut map = self.indexes.write().unwrap();
        if let Some(index) = map.get_mut(index_name) {
            index.clear();
        }
        Ok(())
    }

    async fn search(&self, index_name: &str, query: SearchQuery) -> Result<SearchResult> {
        let start = Instant::now();
        let map = self.indexes.read().unwrap();
        let empty_map = HashMap::new();
        let index = map.get(index_name).unwrap_or(&empty_map);

        let query_tokens: Vec<String> = query
            .q
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut hits: Vec<SearchHit> = Vec::new();

        for doc in index.values() {
            // 1. Filtering
            if let Some(ref filter) = query.filter {
                if let Some(ref cat_ids) = filter.category_ids {
                    if !cat_ids.iter().any(|c| doc.category_ids.contains(c)) {
                        continue;
                    }
                }
                if let Some(ref tags) = filter.tags {
                    if !tags.iter().any(|t| doc.tags.contains(t)) {
                        continue;
                    }
                }
                if let Some(min) = filter.min_price_cents {
                    if doc.price_cents.unwrap_or(0) < min {
                        continue;
                    }
                }
                if let Some(max) = filter.max_price_cents {
                    if doc.price_cents.unwrap_or(0) > max {
                        continue;
                    }
                }
                if let Some(ref channel) = filter.sales_channel_id {
                    if !doc.sales_channel_ids.is_empty() && !doc.sales_channel_ids.contains(channel)
                    {
                        continue;
                    }
                }
            }

            // 2. Query relevance scoring
            let mut score = 0.0f32;
            let title_lower = doc.title.to_lowercase();
            let desc_lower = doc
                .description
                .as_deref()
                .unwrap_or_default()
                .to_lowercase();
            let handle_lower = doc.handle.as_deref().unwrap_or_default().to_lowercase();

            if query_tokens.is_empty() {
                score = 1.0;
            } else {
                for token in &query_tokens {
                    if title_lower.contains(token) {
                        score += 10.0;
                    }
                    if handle_lower.contains(token) {
                        score += 5.0;
                    }
                    if desc_lower.contains(token) {
                        score += 2.0;
                    }
                    if doc.tags.iter().any(|t| t.to_lowercase().contains(token)) {
                        score += 3.0;
                    }
                }
            }

            if score > 0.0 {
                hits.push(SearchHit {
                    document: doc.clone(),
                    score,
                });
            }
        }

        // 3. Sorting
        match query.sort_by.as_deref() {
            Some("price_asc") => {
                hits.sort_by(|a, b| a.document.price_cents.cmp(&b.document.price_cents));
            }
            Some("price_desc") => {
                hits.sort_by(|a, b| b.document.price_cents.cmp(&a.document.price_cents));
            }
            _ => {
                // Default by relevance score descending
                hits.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        let total_hits = hits.len();
        let paginated: Vec<SearchHit> = hits
            .into_iter()
            .skip(query.offset)
            .take(if query.limit > 0 { query.limit } else { 20 })
            .collect();

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(SearchResult {
            hits: paginated,
            total_hits,
            limit: query.limit,
            offset: query.offset,
            processing_time_ms,
        })
    }
}
