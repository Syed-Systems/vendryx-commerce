use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AnalyticsEvent {
    pub id: Id<AnalyticsEvent>,
    pub event_name: String,
    pub distinct_id: String,
    pub properties: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for AnalyticsEvent {
    const TABLE: &'static str = "analytics.analytics_event";
    const ENTITY: &'static str = "analytics_event";

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

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    AnalyticsEvent => "anev",
}
