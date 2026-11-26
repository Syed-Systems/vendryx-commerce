//! `stheno-db::filter`: Type-safe FilterSpec query builder with strict limits and index protection.

use serde::{Deserialize, Serialize};
use stheno_types::{Error, Result};

pub const MAX_IN_ELEMENTS: usize = 100;
pub const MAX_FILTER_DEPTH: usize = 3;
pub const MAX_PREDICATES: usize = 20;
pub const DEFAULT_PAGE_LIMIT: usize = 20;
pub const MAX_PAGE_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOp {
    Eq(String),
    Ne(String),
    In(Vec<String>),
    Gt(String),
    Gte(String),
    Lt(String),
    Lte(String),
    Like(String), // Trailing wildcard only (e.g. "prefix%")
    IsNull,
    NotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterSpec {
    /// Predicates in the order they were added, `AND`-ed together.
    ///
    /// A list and not a map keyed by field: a range needs two operators on one
    /// column (`created_at >= x AND created_at <= y`), which every order and
    /// product list filter in the admin API uses. Keying by field silently
    /// dropped the first of the pair.
    pub predicates: Vec<(String, FilterOp)>,
    pub order_by: Vec<OrderBy>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub depth: usize,
}

impl FilterSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_predicate(mut self, field: impl Into<String>, op: FilterOp) -> Self {
        self.predicates.push((field.into(), op));
        self
    }

    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.offset = Some(offset);
        self.limit = Some(limit);
        self
    }

    pub fn with_order(mut self, field: impl Into<String>, direction: OrderDirection) -> Self {
        self.order_by.push(OrderBy {
            field: field.into(),
            direction,
        });
        self
    }

    pub fn from_json(val: &serde_json::Value) -> Result<Self> {
        let mut spec = Self::new();
        if let Some(obj) = val.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    spec.predicates
                        .push((k.clone(), FilterOp::Eq(s.to_string())));
                } else if let Some(sub_obj) = v.as_object() {
                    for (op_k, op_v) in sub_obj {
                        match op_k.as_str() {
                            "$eq" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Eq(s.to_string())));
                                }
                            }
                            "$ne" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Ne(s.to_string())));
                                }
                            }
                            "$in" => {
                                if let Some(arr) = op_v.as_array() {
                                    let items = arr
                                        .iter()
                                        .filter_map(|item| item.as_str().map(ToString::to_string))
                                        .collect();
                                    spec.predicates.push((k.clone(), FilterOp::In(items)));
                                }
                            }
                            "$gt" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Gt(s.to_string())));
                                }
                            }
                            "$gte" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Gte(s.to_string())));
                                }
                            }
                            "$lt" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Lt(s.to_string())));
                                }
                            }
                            "$lte" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Lte(s.to_string())));
                                }
                            }
                            "$like" | "$ilike" => {
                                if let Some(s) = op_v.as_str() {
                                    spec.predicates
                                        .push((k.clone(), FilterOp::Like(s.to_string())));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        spec.validate()?;
        Ok(spec)
    }

    /// Validates filter limits per TRD §5.3 / AGENTS.md:
    /// - predicates <= 20
    /// - depth <= 3
    /// - $in <= 100 elements
    /// - Like operators MUST NOT have leading wildcards (to guarantee index usage)
    pub fn validate(&self) -> Result<()> {
        if self.depth > MAX_FILTER_DEPTH {
            return Err(Error::Invalid {
                code: "filter_depth_exceeded",
                message: format!(
                    "Filter depth {} exceeds maximum of {}",
                    self.depth, MAX_FILTER_DEPTH
                ),
                field: None,
            });
        }

        if self.predicates.len() > MAX_PREDICATES {
            return Err(Error::Invalid {
                code: "too_many_predicates",
                message: format!(
                    "Predicate count {} exceeds maximum of {}",
                    self.predicates.len(),
                    MAX_PREDICATES
                ),
                field: None,
            });
        }

        for (field, op) in &self.predicates {
            match op {
                FilterOp::In(elements) if elements.len() > MAX_IN_ELEMENTS => {
                    return Err(Error::Invalid {
                        code: "in_clause_limit_exceeded",
                        message: format!(
                            "$in on field '{}' has {} items (max {})",
                            field,
                            elements.len(),
                            MAX_IN_ELEMENTS
                        ),
                        field: Some(field.clone()),
                    });
                }
                FilterOp::Like(pattern) if pattern.starts_with('%') || pattern.starts_with('*') => {
                    return Err(Error::Invalid {
                        code: "leading_wildcard_forbidden",
                        message: format!(
                            "Leading wildcard in LIKE filter on field '{field}' is forbidden to prevent unindexed sequential scans"
                        ),
                        field: Some(field.clone()),
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Computes sanitized SQL pagination limits (limit <= 100, default 20)
    pub fn page_limit(&self) -> usize {
        self.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT)
    }

    pub fn page_offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }
}
