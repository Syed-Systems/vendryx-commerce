use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    IndexDataDTO, IndexFieldDTO, IndexRelationDTO, IndexSyncStateDTO, QueryIndexFilterInput,
    UpsertIndexDataInput,
};
use crate::models::{IndexData, IndexField, IndexRelation, IndexSyncState};

#[derive(Clone)]
pub struct IndexService {
    index_data: PgCrudRepository<IndexData>,
    relations: PgCrudRepository<IndexRelation>,
    fields: PgCrudRepository<IndexField>,
    sync_states: PgCrudRepository<IndexSyncState>,
}

impl IndexService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            index_data: PgCrudRepository::new(pool.clone()),
            relations: PgCrudRepository::new(pool.clone()),
            fields: PgCrudRepository::new(pool.clone()),
            sync_states: PgCrudRepository::new(pool),
        }
    }

    // ==========================================
    // 1. Incremental Update & Projection (P8-13, P14-14)
    // ==========================================

    pub async fn upsert_index_data(&self, input: UpsertIndexDataInput) -> Result<IndexDataDTO> {
        let filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(input.entity_name.clone()))
            .with_predicate("entity_id", FilterOp::Eq(input.entity_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.index_data.list(None, &filter).await?;
        let now = Utc::now();
        let version = input.version.unwrap_or(1);

        let data = if let Some(mut existing_data) = existing.into_iter().next() {
            existing_data.data = input.data;
            existing_data.version = version;
            existing_data.updated_at = now;
            self.index_data
                .update(None, &existing_data.id.to_string(), existing_data)
                .await?
        } else {
            let id = Id::new();
            let data = IndexData {
                id,
                entity_name: input.entity_name.clone(),
                entity_id: input.entity_id.clone(),
                data: input.data,
                version,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.index_data.create(None, data).await?
        };

        // Update sync state
        let sync_filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(input.entity_name.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let sync_existing = self.sync_states.list(None, &sync_filter).await?;
        if let Some(mut state) = sync_existing.into_iter().next() {
            state.last_synced_version = version;
            state.last_synced_at = now;
            state.lag_seconds = 0;
            state.status = "synced".to_string();
            let _ = self
                .sync_states
                .update(None, &state.id.to_string(), state)
                .await;
        } else {
            let state = IndexSyncState {
                id: Id::new(),
                entity_name: input.entity_name.clone(),
                last_synced_version: version,
                last_synced_at: now,
                lag_seconds: 0,
                status: "synced".to_string(),
                deleted_at: None,
            };
            let _ = self.sync_states.create(None, state).await;
        }

        Ok(IndexDataDTO {
            id: data.id.to_string(),
            entity_name: data.entity_name,
            entity_id: data.entity_id,
            data: data.data,
            version: data.version,
            created_at: data.created_at,
            updated_at: data.updated_at,
        })
    }

    pub async fn delete_index_data(&self, entity_name: &str, entity_id: &str) -> Result<()> {
        let filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(entity_name.to_string()))
            .with_predicate("entity_id", FilterOp::Eq(entity_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.index_data.list(None, &filter).await?;
        if let Some(data) = list.into_iter().next() {
            self.index_data
                .soft_delete(None, &data.id.to_string())
                .await?;
        }
        Ok(())
    }

    // ==========================================
    // 2. Full Rebuild & Lag Metric (P8-13, P14-14)
    // ==========================================

    pub async fn rebuild_index(
        &self,
        entity_name: &str,
        documents: Vec<(String, serde_json::Value)>,
    ) -> Result<IndexSyncStateDTO> {
        let now = Utc::now();

        // Soft delete old entries
        let filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(entity_name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let old = self.index_data.list(None, &filter).await?;
        for entry in old {
            let _ = self
                .index_data
                .soft_delete(None, &entry.id.to_string())
                .await;
        }

        // Insert new documents
        for (id_str, val) in documents {
            let data = IndexData {
                id: Id::new(),
                entity_name: entity_name.to_string(),
                entity_id: id_str,
                data: val,
                version: 1,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            let _ = self.index_data.create(None, data).await;
        }

        let sync_filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(entity_name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let sync_existing = self.sync_states.list(None, &sync_filter).await?;
        let state = if let Some(mut existing_state) = sync_existing.into_iter().next() {
            existing_state.last_synced_version = 1;
            existing_state.last_synced_at = now;
            existing_state.lag_seconds = 0;
            existing_state.status = "ready".to_string();
            self.sync_states
                .update(None, &existing_state.id.to_string(), existing_state)
                .await?
        } else {
            let state = IndexSyncState {
                id: Id::new(),
                entity_name: entity_name.to_string(),
                last_synced_version: 1,
                last_synced_at: now,
                lag_seconds: 0,
                status: "ready".to_string(),
                deleted_at: None,
            };
            self.sync_states.create(None, state).await?
        };

        Ok(IndexSyncStateDTO {
            id: state.id.to_string(),
            entity_name: state.entity_name,
            last_synced_version: state.last_synced_version,
            last_synced_at: state.last_synced_at,
            lag_seconds: state.lag_seconds,
            status: state.status,
        })
    }

    pub async fn get_sync_state(&self, entity_name: &str) -> Result<IndexSyncStateDTO> {
        let filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(entity_name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.sync_states.list(None, &filter).await?;
        let state = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "index_sync_state",
            id: entity_name.to_string(),
        })?;

        let now = Utc::now();
        let lag = (now - state.last_synced_at).num_seconds();

        Ok(IndexSyncStateDTO {
            id: state.id.to_string(),
            entity_name: state.entity_name,
            last_synced_version: state.last_synced_version,
            last_synced_at: state.last_synced_at,
            lag_seconds: lag.max(0),
            status: state.status,
        })
    }

    // ==========================================
    // 3. Cross-Module Filtering & Sorting (P8-14)
    // ==========================================

    pub async fn query_index(&self, input: QueryIndexFilterInput) -> Result<Vec<IndexDataDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("entity_name", FilterOp::Eq(input.entity_name.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.index_data.list(None, &filter).await?;

        let mut results: Vec<IndexData> = list
            .into_iter()
            .filter(|d| {
                for (k, expected_val) in &input.filters {
                    match d.data.get(k) {
                        Some(actual_val) => {
                            if actual_val != expected_val {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            })
            .collect();

        // Sort
        if let Some(ref sort_field) = input.sort_by {
            let is_desc = input.sort_order.as_deref() == Some("desc");
            results.sort_by(|a, b| {
                let val_a = a.data.get(sort_field);
                let val_b = b.data.get(sort_field);
                let cmp = match (val_a, val_b) {
                    (Some(serde_json::Value::Number(na)), Some(serde_json::Value::Number(nb))) => {
                        na.as_f64()
                            .unwrap_or(0.0)
                            .partial_cmp(&nb.as_f64().unwrap_or(0.0))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Some(serde_json::Value::String(sa)), Some(serde_json::Value::String(sb))) => {
                        sa.cmp(sb)
                    }
                    _ => std::cmp::Ordering::Equal,
                };
                if is_desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }

        let offset = input.offset.unwrap_or(0);
        let limit = input.limit.unwrap_or(50);

        let dtos: Vec<IndexDataDTO> = results
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|d| IndexDataDTO {
                id: d.id.to_string(),
                entity_name: d.entity_name,
                entity_id: d.entity_id,
                data: d.data,
                version: d.version,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect();

        Ok(dtos)
    }

    // ==========================================
    // 4. Relations & Fields
    // ==========================================

    pub async fn add_relation(
        &self,
        parent_entity: &str,
        parent_id: &str,
        child_entity: &str,
        child_id: &str,
        relation_type: &str,
    ) -> Result<IndexRelationDTO> {
        let id = Id::new();
        let now = Utc::now();

        let rel = IndexRelation {
            id,
            parent_entity: parent_entity.to_string(),
            parent_id: parent_id.to_string(),
            child_entity: child_entity.to_string(),
            child_id: child_id.to_string(),
            relation_type: relation_type.to_string(),
            created_at: now,
            deleted_at: None,
        };

        let created = self.relations.create(None, rel).await?;

        Ok(IndexRelationDTO {
            id: created.id.to_string(),
            parent_entity: created.parent_entity,
            parent_id: created.parent_id,
            child_entity: created.child_entity,
            child_id: created.child_id,
            relation_type: created.relation_type,
            created_at: created.created_at,
        })
    }

    pub async fn define_field(
        &self,
        entity_name: &str,
        field_name: &str,
        field_type: &str,
        is_searchable: bool,
        is_filterable: bool,
        is_sortable: bool,
    ) -> Result<IndexFieldDTO> {
        let id = Id::new();
        let field = IndexField {
            id,
            entity_name: entity_name.to_string(),
            field_name: field_name.to_string(),
            field_type: field_type.to_string(),
            is_searchable,
            is_filterable,
            is_sortable,
            deleted_at: None,
        };

        let created = self.fields.create(None, field).await?;

        Ok(IndexFieldDTO {
            id: created.id.to_string(),
            entity_name: created.entity_name,
            field_name: created.field_name,
            field_type: created.field_type,
            is_searchable: created.is_searchable,
            is_filterable: created.is_filterable,
            is_sortable: created.is_sortable,
        })
    }
}
