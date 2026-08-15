//! `stheno-registry`: Compile-time dependency injection and AppContext.

use std::sync::Arc;
use stheno_db::DbPool;
use stheno_events::{EventBus, InMemoryEventBus};
use stheno_links::LinkStore;
use stheno_query::QueryEngine;

#[derive(Clone)]
pub struct AppContext {
    pub pool: Option<DbPool>,
    pub events: Arc<dyn EventBus>,
    pub links: Arc<LinkStore>,
    pub query: Arc<QueryEngine>,
}

impl AppContext {
    pub fn new(pool: Option<DbPool>, events: Arc<dyn EventBus>, links: Arc<LinkStore>) -> Self {
        let query = Arc::new(QueryEngine::new(links.clone()));
        Self {
            pool,
            events,
            links,
            query,
        }
    }

    /// Creates an in-memory AppContext for unit tests and local mock harnesses.
    pub fn create_test_context() -> Self {
        let events = Arc::new(InMemoryEventBus::new());
        let links = Arc::new(LinkStore::new());
        let query = Arc::new(QueryEngine::new(links.clone()));
        Self {
            pool: None,
            events,
            links,
            query,
        }
    }
}
