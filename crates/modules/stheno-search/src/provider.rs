use async_trait::async_trait;
use stheno_types::Result;

use crate::models::{SearchDocument, SearchQuery, SearchResult};

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn index_documents(&self, index_name: &str, documents: Vec<SearchDocument>)
        -> Result<()>;
    async fn delete_documents(&self, index_name: &str, document_ids: &[String]) -> Result<()>;
    async fn search(&self, index_name: &str, query: SearchQuery) -> Result<SearchResult>;
    async fn clear_index(&self, index_name: &str) -> Result<()>;
}
