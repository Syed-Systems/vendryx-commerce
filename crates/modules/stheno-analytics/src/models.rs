use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AnalyticsEvent {
    pub id: Id<AnalyticsEvent>,
    pub event_name: String,
    pub distinct_id: String,
    pub properties: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
