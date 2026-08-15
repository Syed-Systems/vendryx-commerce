use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use stheno_types::{Error, Id, Result};

use crate::dto::{
    IndexDataDTO, IndexFieldDTO, IndexRelationDTO, IndexSyncStateDTO, QueryIndexFilterInput,
    UpsertIndexDataInput,
};
use crate::models::{IndexData, IndexField, IndexRelation, IndexSyncState};

pub struct IndexService {
    index_data: RwLock<HashMap<String, IndexData>>,
    relations: RwLock<Vec<IndexRelation>>,
    fields: RwLock<HashMap<String, IndexField>>,
    sync_states: RwLock<HashMap<String, IndexSyncState>>,
}

impl Default for IndexService {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexService {
    pub fn new() -> Self {
        Self {
            index_data: RwLock::new(HashMap::new()),
            relations: RwLock::new(Vec::new()),
            fields: RwLock::new(HashMap::new()),
            sync_states: RwLock::new(HashMap::new()),
        }
    }

    fn make_data_key(entity_name: &str, entity_id: &str) -> String {
        format!("{}:{}", entity_name, entity_id)
    }

    // ==========================================
    // 1. Incremental Update & Projection (P8-13)
    // ==========================================

    pub async fn upsert_index_data(&self, input: UpsertIndexDataInput) -> Result<IndexDataDTO> {
        let key = Self::make_data_key(&input.entity_name, &input.entity_id);
        let now = Utc::now();
        let version = input.version.unwrap_or(1);

        let data = IndexData {
            id: Id::new(),
            entity_name: input.entity_name.clone(),
            entity_id: input.entity_id.clone(),
            data: input.data,
            version,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let dto = IndexDataDTO {
            id: data.id.to_string(),
            entity_name: data.entity_name.clone(),
            entity_id: data.entity_id.clone(),
            data: data.data.clone(),
            version: data.version,
            created_at: data.created_at,
            updated_at: data.updated_at,
        };

        {
            let mut map = self.index_data.write().unwrap();
            map.insert(key, data);
        }

        // Update sync state & lag
        {
            let mut sync_map = self.sync_states.write().unwrap();
            let state = sync_map
                .entry(input.entity_name.clone())
                .or_insert_with(|| IndexSyncState {
                    id: Id::new(),
                    entity_name: input.entity_name.clone(),
                    last_synced_version: 0,
                    last_synced_at: now,
                    lag_seconds: 0,
                    status: "synced".to_string(),
                });

            state.last_synced_version = version;
            state.last_synced_at = now;
            state.lag_seconds = 0; // Freshly updated
            state.status = "synced".to_string();
        }

        Ok(dto)
    }

    pub async fn delete_index_data(&self, entity_name: &str, entity_id: &str) -> Result<()> {
        let key = Self::make_data_key(entity_name, entity_id);
        let mut map = self.index_data.write().unwrap();
        if let Some(entry) = map.get_mut(&key) {
            entry.deleted_at = Some(Utc::now());
        }
        Ok(())
    }

    // ==========================================
    // 2. Full Rebuild & Lag Metric (P8-13)
    // ==========================================

    pub async fn rebuild_index(
        &self,
        entity_name: &str,
        documents: Vec<(String, serde_json::Value)>,
    ) -> Result<IndexSyncStateDTO> {
        let now = Utc::now();
        {
            let mut map = self.index_data.write().unwrap();
            // Remove previous entries for entity_name
            map.retain(|_, v| v.entity_name != entity_name);

            for (id_str, val) in documents {
                let key = Self::make_data_key(entity_name, &id_str);
                map.insert(
                    key,
                    IndexData {
                        id: Id::new(),
                        entity_name: entity_name.to_string(),
                        entity_id: id_str,
                        data: val,
                        version: 1,
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    },
                );
            }
        }

        let sync_state = IndexSyncState {
            id: Id::new(),
            entity_name: entity_name.to_string(),
            last_synced_version: 1,
            last_synced_at: now,
            lag_seconds: 0,
            status: "ready".to_string(),
        };

        let dto = IndexSyncStateDTO {
            id: sync_state.id.to_string(),
            entity_name: sync_state.entity_name.clone(),
            last_synced_version: sync_state.last_synced_version,
            last_synced_at: sync_state.last_synced_at,
            lag_seconds: sync_state.lag_seconds,
            status: sync_state.status.clone(),
        };

        {
            let mut sync_map = self.sync_states.write().unwrap();
            sync_map.insert(entity_name.to_string(), sync_state);
        }

        Ok(dto)
    }

    pub async fn get_sync_state(&self, entity_name: &str) -> Result<IndexSyncStateDTO> {
        let map = self.sync_states.read().unwrap();
        let state = map.get(entity_name).ok_or_else(|| Error::NotFound {
            entity: "index_sync_state",
            id: entity_name.to_string(),
        })?;

        let now = Utc::now();
        let lag = (now - state.last_synced_at).num_seconds();

        Ok(IndexSyncStateDTO {
            id: state.id.to_string(),
            entity_name: state.entity_name.clone(),
            last_synced_version: state.last_synced_version,
            last_synced_at: state.last_synced_at,
            lag_seconds: lag.max(0),
            status: state.status.clone(),
        })
    }

    // ==========================================
    // 3. Cross-Module Filtering & Sorting (P8-14)
    // ==========================================

    pub async fn query_index(&self, input: QueryIndexFilterInput) -> Result<Vec<IndexDataDTO>> {
        let map = self.index_data.read().unwrap();
        let mut results: Vec<IndexData> = map
            .values()
            .filter(|d| d.entity_name == input.entity_name && d.deleted_at.is_none())
            .filter(|d| {
                // Filter matching against JSON projection fields
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
            .cloned()
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
        };

        let dto = IndexRelationDTO {
            id: id.to_string(),
            parent_entity: rel.parent_entity.clone(),
            parent_id: rel.parent_id.clone(),
            child_entity: rel.child_entity.clone(),
            child_id: rel.child_id.clone(),
            relation_type: rel.relation_type.clone(),
            created_at: rel.created_at,
        };

        {
            let mut list = self.relations.write().unwrap();
            list.push(rel);
        }

        Ok(dto)
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
        };

        let dto = IndexFieldDTO {
            id: id.to_string(),
            entity_name: field.entity_name.clone(),
            field_name: field.field_name.clone(),
            field_type: field.field_type.clone(),
            is_searchable,
            is_filterable,
            is_sortable,
        };

        let key = format!("{}:{}", entity_name, field_name);
        {
            let mut map = self.fields.write().unwrap();
            map.insert(key, field);
        }

        Ok(dto)
    }
}
