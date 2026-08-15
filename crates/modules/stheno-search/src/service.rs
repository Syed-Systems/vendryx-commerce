use std::sync::Arc;
use stheno_types::Result;

use crate::models::{SearchDocument, SearchQuery, SearchResult};
use crate::provider::SearchProvider;

pub struct SearchService {
    provider: Arc<dyn SearchProvider>,
}

impl SearchService {
    pub fn new(provider: Arc<dyn SearchProvider>) -> Self {
        Self { provider }
    }

    pub async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<SearchDocument>,
    ) -> Result<()> {
        self.provider.index_documents(index_name, documents).await
    }

    pub async fn delete_documents(&self, index_name: &str, document_ids: &[String]) -> Result<()> {
        self.provider
            .delete_documents(index_name, document_ids)
            .await
    }

    pub async fn search(&self, index_name: &str, query: SearchQuery) -> Result<SearchResult> {
        self.provider.search(index_name, query).await
    }

    pub async fn clear_index(&self, index_name: &str) -> Result<()> {
        self.provider.clear_index(index_name).await
    }

    /// Event-driven sync handler (Task P8-10)
    pub async fn handle_product_event(
        &self,
        event_name: &str,
        payload: &serde_json::Value,
    ) -> Result<()> {
        match event_name {
            "product.created" | "product.updated" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let title = payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let description = payload
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let handle = payload
                    .get("handle")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let price_cents = payload.get("price_cents").and_then(|v| v.as_i64());

                let doc = SearchDocument {
                    id,
                    title,
                    description,
                    handle,
                    sales_channel_ids: vec![],
                    category_ids: vec![],
                    tags: vec![],
                    price_cents,
                    attributes: std::collections::HashMap::new(),
                };
                self.index_documents("products", vec![doc]).await?;
            }
            "product.deleted" => {
                if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                    self.delete_documents("products", &[id.to_string()]).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
