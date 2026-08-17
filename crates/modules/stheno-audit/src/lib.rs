//! `stheno-audit`: Append-only audit log module with write-time PII redaction (P14-11).

pub mod dto;
pub(crate) mod models;

use chrono::Utc;
pub use dto::{AuditLogDTO, CreateAuditEntry};
use models::AuditLog;
use serde_json::Value;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Id, Result};

#[derive(Clone)]
pub struct AuditService {
    entries: PgCrudRepository<AuditLog>,
}

impl AuditService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            entries: PgCrudRepository::new(pool),
        }
    }

    /// Redact sensitive keys from JSON Value
    pub fn redact_pii(val: &mut Value) {
        match val {
            Value::Object(map) => {
                for (k, v) in map.iter_mut() {
                    let k_lower = k.to_lowercase();
                    if k_lower.contains("password")
                        || k_lower.contains("token")
                        || k_lower.contains("secret")
                        || k_lower.contains("card")
                        || k_lower.contains("cvv")
                        || k_lower.contains("api_key")
                    {
                        *v = Value::String("[REDACTED]".to_string());
                    } else {
                        Self::redact_pii(v);
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::redact_pii(item);
                }
            }
            _ => {}
        }
    }

    /// Append a new audit log entry (strictly append-only, no update/delete methods exist)
    pub async fn append(&self, mut entry: CreateAuditEntry) -> Result<AuditLogDTO> {
        if let Some(ref mut diff) = entry.diff {
            Self::redact_pii(diff);
        }

        let log = AuditLog {
            id: Id::new(),
            actor_id: entry.actor_id,
            actor_type: entry.actor_type,
            action: entry.action,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            diff: entry.diff,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.entries.create(None, log).await?;
        Ok(AuditLogDTO::from(created))
    }

    /// Query audit logs for an entity
    pub async fn list_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<AuditLogDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("entity_type", FilterOp::Eq(entity_type.to_string()))
            .with_predicate("entity_id", FilterOp::Eq(entity_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.entries.list(None, &filter).await?;
        Ok(list.into_iter().map(AuditLogDTO::from).collect())
    }

    /// Total count of logs
    pub async fn count(&self) -> usize {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        self.entries
            .list(None, &filter)
            .await
            .map(|l| l.len())
            .unwrap_or(0)
    }
}
