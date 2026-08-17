use chrono::Utc;
use std::sync::Arc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Id, Result};

use crate::dto::{SearchDocument, SearchQuery, SearchResult};
use crate::models::SearchDocumentEntity;
use crate::provider::SearchProvider;

pub struct SearchService {
    provider: Arc<dyn SearchProvider>,
    storage: Option<PgCrudRepository<SearchDocumentEntity>>,
}

impl SearchService {
    pub fn new(provider: Arc<dyn SearchProvider>) -> Self {
        Self {
            provider,
            storage: None,
        }
    }

    pub fn with_pool(provider: Arc<dyn SearchProvider>, pool: DbPool) -> Self {
        Self {
            provider,
            storage: Some(PgCrudRepository::new(pool)),
        }
    }

    pub async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<SearchDocument>,
    ) -> Result<()> {
        if let Some(ref storage) = self.storage {
            for doc in &documents {
                let id = Id::new();
                let now = Utc::now();
                let entity = SearchDocumentEntity {
                    id,
                    index_name: index_name.to_string(),
                    document_id: doc.id.clone(),
                    document: serde_json::to_value(doc).unwrap_or_default(),
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };
                let _ = storage.create(None, entity).await;
            }
        }
        self.provider.index_documents(index_name, documents).await
    }

    pub async fn rebuild_from_storage(&self, index_name: &str) -> Result<usize> {
        if let Some(ref storage) = self.storage {
            let filter = FilterSpec::new()
                .with_predicate("index_name", FilterOp::Eq(index_name.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

            let list = storage.list(None, &filter).await?;
            let mut docs = Vec::new();
            for item in list {
                if let Ok(doc) = serde_json::from_value::<SearchDocument>(item.document) {
                    docs.push(doc);
                }
            }

            let count = docs.len();
            self.provider.clear_index(index_name).await?;
            self.provider.index_documents(index_name, docs).await?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    pub async fn delete_documents(&self, index_name: &str, document_ids: &[String]) -> Result<()> {
        if let Some(ref storage) = self.storage {
            for doc_id in document_ids {
                let filter = FilterSpec::new()
                    .with_predicate("index_name", FilterOp::Eq(index_name.to_string()))
                    .with_predicate("document_id", FilterOp::Eq(doc_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1);

                if let Ok(list) = storage.list(None, &filter).await {
                    for item in list {
                        let _ = storage.soft_delete(None, &item.id.to_string()).await;
                    }
                }
            }
        }
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
