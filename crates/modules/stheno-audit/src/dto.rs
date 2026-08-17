use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::AuditLog;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateAuditEntry {
    pub actor_id: String,
    pub actor_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogDTO {
    pub id: String,
    pub actor_id: String,
    pub actor_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLog> for AuditLogDTO {
    fn from(log: AuditLog) -> Self {
        Self {
            id: log.id.to_string(),
            actor_id: log.actor_id,
            actor_type: log.actor_type,
            action: log.action,
            entity_type: log.entity_type,
            entity_id: log.entity_id,
            diff: log.diff,
            created_at: log.created_at,
        }
    }
}
