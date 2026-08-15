//! `stheno-server`: Server application, router mounting, and universal test harness.

use axum::Router;
use stheno_registry::AppContext;

pub struct AppServer {
    pub context: AppContext,
    pub router: Router,
}

impl AppServer {
    pub fn new(context: AppContext) -> Self {
        let router = Router::new();
        Self { context, router }
    }
}

/// Universal Test Harness providing isolated, ephemeral test environment per test execution.
pub struct TestApp {
    pub context: AppContext,
    pub schema_name: String,
    pub app: Router,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let schema_name = format!("test_schema_{}", ulid::Ulid::new());
        let context = AppContext::create_test_context();
        let server = AppServer::new(context.clone());

        Self {
            context,
            schema_name,
            app: server.router,
        }
    }
}
