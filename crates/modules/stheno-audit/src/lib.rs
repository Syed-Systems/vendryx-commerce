//! `stheno-audit`: Append-only audit log module.

pub mod models;

use chrono::Utc;
use models::AuditLog;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use stheno_types::{Id, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateAuditEntry {
    pub actor_id: String,
    pub actor_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct AuditService {
    // In-memory append-only storage for testing and local operation
    entries: Arc<RwLock<Vec<AuditLog>>>,
}

impl AuditService {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Append a new audit log entry (strictly append-only, no update/delete methods exist)
    pub async fn append(&self, entry: CreateAuditEntry) -> Result<AuditLog> {
        let log = AuditLog {
            id: Id::new(),
            actor_id: entry.actor_id,
            actor_type: entry.actor_type,
            action: entry.action,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            diff: entry.diff,
            created_at: Utc::now(),
        };

        let mut entries = self.entries.write().unwrap();
        entries.push(log.clone());
        Ok(log)
    }

    /// Query audit logs for an entity
    pub async fn list_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<AuditLog>> {
        let entries = self.entries.read().unwrap();
        let matched = entries
            .iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .cloned()
            .collect();
        Ok(matched)
    }

    /// Total count of logs
    pub async fn count(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}
