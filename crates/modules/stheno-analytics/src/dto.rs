use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEventDTO {
    pub id: String,
    pub event_name: String,
    pub distinct_id: String,
    pub properties: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackEventInput {
    pub event_name: String,
    pub distinct_id: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentifyUserInput {
    pub distinct_id: String,
    pub traits: serde_json::Value,
}
