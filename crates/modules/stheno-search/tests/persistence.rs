//! Indexed documents survive the service instance (and provider) that wrote them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_search::{SearchDocument, SearchProvider, SearchQuery, SearchResult, SearchService};
use stheno_types::Result;

/// Provider whose only state is a list of documents, so anything a second
/// service instance finds must have come back out of Postgres.
#[derive(Default)]
struct RecordingProvider {
    docs: Mutex<Vec<SearchDocument>>,
}

#[async_trait]
impl SearchProvider for RecordingProvider {
    fn provider_id(&self) -> &'static str {
        "recording"
    }
    async fn index_documents(&self, _index: &str, documents: Vec<SearchDocument>) -> Result<()> {
        self.docs.lock().unwrap().extend(documents);
        Ok(())
    }
    async fn delete_documents(&self, _index: &str, _ids: &[String]) -> Result<()> {
        Ok(())
    }
    async fn search(&self, _index: &str, query: SearchQuery) -> Result<SearchResult> {
        Ok(SearchResult {
            hits: vec![],
            total_hits: 0,
            limit: query.limit,
            offset: query.offset,
            processing_time_ms: 0,
        })
    }
    async fn clear_index(&self, _index: &str) -> Result<()> {
        self.docs.lock().unwrap().clear();
        Ok(())
    }
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = SearchService::with_pool(Arc::new(RecordingProvider::default()), pool.clone());
    writer
        .index_documents(
            "products",
            vec![SearchDocument {
                id: "prod_outlive".to_string(),
                title: "Kept Sneakers".to_string(),
                description: Some("persisted".to_string()),
                handle: Some("kept-sneakers".to_string()),
                sales_channel_ids: vec!["sc_main".to_string()],
                category_ids: vec![],
                tags: vec!["shoes".to_string()],
                price_cents: Some(14000),
                attributes: HashMap::new(),
            }],
        )
        .await
        .expect("index");
    drop(writer);

    // A fresh provider knows nothing; the rebuild can only read Postgres.
    let fresh = Arc::new(RecordingProvider::default());
    let reader = SearchService::with_pool(fresh.clone(), pool);
    let restored = reader
        .rebuild_from_storage("products")
        .await
        .expect("rebuild");
    assert_eq!(restored, 1);
    {
        let docs = fresh.docs.lock().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, "prod_outlive");
        assert_eq!(docs[0].title, "Kept Sneakers");
        assert_eq!(docs[0].price_cents, Some(14000));
    }

    db.drop().await;
}
