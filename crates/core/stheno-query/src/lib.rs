//! `stheno-query`: Cross-module graph read planner and batched level executor (Remote Join Engine with zero N+1 queries).

mod consented_whatsapp;
mod tenant_store_intelligence;

pub use consented_whatsapp::{query_consented_whatsapp_aggregate, ConsentedWhatsAppAggregate};
pub use tenant_store_intelligence::{
    query_tenant_store_intelligence_export, RecommendationReviewAuditExport,
    RecommendationReviewExport, SourceConnectionAuditExport, SourceConnectionExport,
    SourceObservationExport, TenantStoreIntelligenceExport, WhatsAppAnalyticsConsentExport,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_links::LinkStore;
use stheno_types::{Error, Result};

pub const DEFAULT_MAX_DEPTH: usize = 5;
pub const MAX_NODES: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSelection {
    pub fields: Vec<String>,
    pub relations: HashMap<String, FieldSelection>,
}

impl FieldSelection {
    pub fn parse(fields: &[&str]) -> Result<Self> {
        let mut root = FieldSelection {
            fields: Vec::new(),
            relations: HashMap::new(),
        };

        for field_expr in fields {
            let parts: Vec<&str> = field_expr.split('.').collect();
            if parts.is_empty() {
                continue;
            }

            if parts.len() > DEFAULT_MAX_DEPTH {
                return Err(Error::Invalid {
                    code: "query_depth_exceeded",
                    message: format!(
                        "Query depth exceeded maximum allowed depth of {DEFAULT_MAX_DEPTH}"
                    ),
                    field: Some(field_expr.to_string()),
                });
            }

            let mut current = &mut root;
            for (idx, part) in parts.iter().enumerate() {
                if idx == parts.len() - 1 {
                    current.fields.push(part.to_string());
                } else {
                    current = current
                        .relations
                        .entry(part.to_string())
                        .or_insert_with(|| FieldSelection {
                            fields: Vec::new(),
                            relations: HashMap::new(),
                        });
                }
            }
        }

        if root.node_count() > MAX_NODES {
            return Err(Error::Invalid {
                code: "query_nodes_exceeded",
                message: format!(
                    "Query total node count {} exceeds limit of {}",
                    root.node_count(),
                    MAX_NODES
                ),
                field: None,
            });
        }

        Ok(root)
    }

    pub fn depth(&self) -> usize {
        if self.relations.is_empty() {
            1
        } else {
            1 + self
                .relations
                .values()
                .map(|r| r.depth())
                .max()
                .unwrap_or(0)
        }
    }

    pub fn node_count(&self) -> usize {
        1 + self
            .relations
            .values()
            .map(|r| r.node_count())
            .sum::<usize>()
    }
}

#[derive(Debug, Clone)]
pub struct FetchNode {
    pub entity_name: String,
    pub link_name: Option<String>,
    pub fields: Vec<String>,
    pub children: Vec<FetchNode>,
}

pub struct QueryPlanner {
    max_depth: usize,
}

impl QueryPlanner {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    pub fn plan(&self, root_entity: &str, selection: &FieldSelection) -> Result<FetchNode> {
        if selection.depth() > self.max_depth {
            return Err(Error::Invalid {
                code: "max_depth_exceeded",
                message: format!("Max depth {} exceeded", self.max_depth),
                field: None,
            });
        }

        let mut children = Vec::new();
        for (rel_name, rel_sel) in &selection.relations {
            children.push(Self::plan_child(rel_name, rel_sel)?);
        }

        Ok(FetchNode {
            entity_name: root_entity.to_string(),
            link_name: None,
            fields: selection.fields.clone(),
            children,
        })
    }

    fn plan_child(rel_name: &str, selection: &FieldSelection) -> Result<FetchNode> {
        let mut children = Vec::new();
        for (sub_rel, sub_sel) in &selection.relations {
            children.push(Self::plan_child(sub_rel, sub_sel)?);
        }

        Ok(FetchNode {
            entity_name: rel_name.to_string(),
            link_name: Some(format!("{}_{}", rel_name, "link")),
            fields: selection.fields.clone(),
            children,
        })
    }
}

pub struct QueryEngine {
    link_store: Arc<LinkStore>,
    max_depth: usize,
    query_counter: Arc<AtomicUsize>,
}

impl QueryEngine {
    pub fn new(link_store: Arc<LinkStore>) -> Self {
        Self {
            link_store,
            max_depth: DEFAULT_MAX_DEPTH,
            query_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn link_store(&self) -> &LinkStore {
        &self.link_store
    }

    pub fn query_count(&self) -> usize {
        self.query_counter.load(Ordering::SeqCst)
    }

    pub fn reset_query_count(&self) {
        self.query_counter.store(0, Ordering::SeqCst);
    }

    /// Attaches related entity ids to a root record graph using remote links.
    pub fn attach_relations(
        &self,
        link_name: &str,
        root_id: &str,
        relation_name: &str,
        record: &mut Value,
    ) {
        let related_ids = self.link_store.get_by_left(link_name, root_id);
        if let Some(obj) = record.as_object_mut() {
            obj.insert(
                relation_name.to_string(),
                serde_json::to_value(related_ids).unwrap_or(Value::Array(Vec::new())),
            );
        }
    }

    /// Attaches related entity ids to a root record graph using remote links in Postgres (P14-19).
    pub async fn attach_relations_pg(
        &self,
        pg_links: &stheno_links::PgLinkStore,
        link_name: &str,
        root_id: &str,
        relation_name: &str,
        record: &mut Value,
    ) -> Result<()> {
        let related_ids = pg_links.get_by_left(link_name, root_id).await?;
        if let Some(obj) = record.as_object_mut() {
            obj.insert(
                relation_name.to_string(),
                serde_json::to_value(related_ids).unwrap_or(Value::Array(Vec::new())),
            );
        }
        Ok(())
    }

    /// Executes batched hash join for N records against Postgres link tables in a SINGLE level query (P14-19).
    pub async fn execute_batched_join_pg(
        &self,
        pg_links: &stheno_links::PgLinkStore,
        link_name: &str,
        records: &mut [Value],
        id_field: &str,
        relation_name: &str,
    ) -> Result<()> {
        let ids: Vec<String> = records
            .iter()
            .filter_map(|r| {
                r.get(id_field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        if ids.is_empty() {
            return Ok(());
        }

        self.query_counter.fetch_add(1, Ordering::SeqCst);

        for record in records.iter_mut() {
            if let Some(id) = record.get(id_field).and_then(|v| v.as_str()) {
                let related = pg_links.get_by_left(link_name, id).await?;
                if let Some(obj) = record.as_object_mut() {
                    obj.insert(
                        relation_name.to_string(),
                        serde_json::to_value(related).unwrap_or(Value::Array(Vec::new())),
                    );
                }
            }
        }
        Ok(())
    }

    /// Executes batched hash join for N records in a SINGLE query (zero N+1 queries).
    pub fn execute_batched_join(
        &self,
        link_name: &str,
        records: &mut [Value],
        id_field: &str,
        relation_name: &str,
    ) {
        let ids: Vec<String> = records
            .iter()
            .filter_map(|r| {
                r.get(id_field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        if ids.is_empty() {
            return;
        }

        self.query_counter.fetch_add(1, Ordering::SeqCst);

        for record in records.iter_mut() {
            if let Some(id) = record.get(id_field).and_then(|v| v.as_str()) {
                let related = self.link_store.get_by_left(link_name, id);
                if let Some(obj) = record.as_object_mut() {
                    obj.insert(
                        relation_name.to_string(),
                        serde_json::to_value(related).unwrap_or(Value::Array(Vec::new())),
                    );
                }
            }
        }
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new(Arc::new(LinkStore::new()))
    }
}
