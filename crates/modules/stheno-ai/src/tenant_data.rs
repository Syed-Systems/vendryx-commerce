//! Tenant erasure for Store Intelligence source data (ADR-035).

use stheno_db::diesel::sql_query;
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{Conn, Error, Result};

/// Owns removal of source credentials, observations, review state, audit, and OAuth state.
#[derive(Debug, Clone, Copy, Default)]
pub struct StoreIntelligenceTenantDataService;

impl StoreIntelligenceTenantDataService {
    /// Erases the current tenant's Store Intelligence source records.
    pub async fn erase(&self, conn: &mut Conn<'_>) -> Result<()> {
        for query in [
            "DELETE FROM ai.search_console_oauth_state",
            "DELETE FROM ai.store_intelligence_review_audit",
            "DELETE FROM ai.store_intelligence_review",
            "DELETE FROM ai.store_intelligence_source_observation",
            "DELETE FROM ai.source_connection_audit",
            "DELETE FROM ai.configuration WHERE kind IN \
             ('search_console', 'meta_insights_facebook', 'meta_insights_instagram')",
        ] {
            sql_query(query)
                .execute(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
        }
        Ok(())
    }
}
