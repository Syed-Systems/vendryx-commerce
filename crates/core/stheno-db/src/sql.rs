//! `FilterSpec` -> SQL (P11-09 .. P11-12).
//!
//! Phase 1 specified `FilterSpec` and never executed it: `InMemoryCrudStore`
//! ignores it entirely. This is the translation layer that makes it real.
//!
//! # How values are bound
//!
//! Every predicate value travels inside a single `$1` JSON array -- one element
//! per predicate, at the index the clause references. Values are coerced to the
//! column's own type by `json_populate_record(NULL::<table>, ...)`, which reads
//! the table's row type.
//!
//! That keeps one generic translation for all 127 entities without a per-entity
//! column-type table, and -- crucially -- keeps comparisons in the column's own
//! type rather than in text. Comparing as text would make `quantity > '9'` false
//! for a quantity of 10, and silently wrong range filters on money and stock are
//! the expensive kind of bug.

use crate::filter::{FilterOp, FilterSpec, OrderDirection};
use serde_json::{json, Value};
use stheno_types::{Error, Result};

/// A rendered `WHERE` / `ORDER BY` / `LIMIT` clause and the JSON parameter it binds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlFilter {
    /// Everything after the table reference. References `$1` for values, and
    /// assumes the table is aliased `t`.
    pub clause: String,
    /// The JSON array to bind as `$1`. One element per predicate.
    pub params: Value,
}

impl SqlFilter {
    /// The parameter serialized for binding as a text argument.
    pub fn params_json(&self) -> String {
        self.params.to_string()
    }
}

/// Rejects anything that is not a plain SQL identifier.
///
/// Field names arrive from HTTP query strings and are concatenated into SQL.
/// Postgres cannot parameterise an identifier, so this check -- not a bind -- is
/// what prevents injection. Keep it strict.
pub(crate) fn ident(field: &str) -> Result<&str> {
    let mut chars = field.chars();
    let valid_start = matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_');
    let valid_rest = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    // 63 is Postgres' NAMEDATALEN limit.
    if !valid_start || !valid_rest || field.len() > 63 {
        return Err(Error::Invalid {
            code: "invalid_filter_field",
            message: format!("'{field}' is not a valid column name"),
            field: Some(field.to_string()),
        });
    }
    Ok(field)
}

/// Validates a possibly schema-qualified table name (`product.product`).
pub(crate) fn table_ident(table: &str) -> Result<&str> {
    for part in table.split('.') {
        ident(part)?;
    }
    if table.split('.').count() > 2 {
        return Err(Error::Invalid {
            code: "invalid_table_name",
            message: format!("'{table}' is not a valid table name"),
            field: None,
        });
    }
    Ok(table)
}

/// Validates and quotes each part of a possibly schema-qualified table name (`"product"."product"`).
pub(crate) fn quote_table(table: &str) -> Result<String> {
    table_ident(table)?;
    let quoted = table
        .split('.')
        .map(|p| format!("\"{p}\""))
        .collect::<Vec<_>>()
        .join(".");
    Ok(quoted)
}

impl FilterSpec {
    /// Renders this spec against `table`, which must be aliased `t` by the caller.
    pub fn to_sql(&self, table: &str) -> Result<SqlFilter> {
        self.validate()?;
        let quoted_tbl = quote_table(table)?;

        let (conditions, params) = self.render_predicates(&quoted_tbl)?;

        let mut clause = String::new();
        if !conditions.is_empty() {
            clause.push_str("WHERE ");
            clause.push_str(&conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            let mut orders = Vec::with_capacity(self.order_by.len());
            for o in &self.order_by {
                let col = ident(&o.field)?;
                let dir = match o.direction {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                };
                orders.push(format!("t.\"{col}\" {dir}"));
            }
            if !clause.is_empty() {
                clause.push(' ');
            }
            clause.push_str("ORDER BY ");
            clause.push_str(&orders.join(", "));
        }

        // `page_limit` clamps to MAX_PAGE_LIMIT, so an unbounded scan cannot be
        // requested from the wire. Rendered as literals because they are
        // integers this code produced, never caller text.
        if !clause.is_empty() {
            clause.push(' ');
        }
        clause.push_str(&format!(
            "LIMIT {} OFFSET {}",
            self.page_limit(),
            self.page_offset()
        ));

        Ok(SqlFilter {
            clause,
            params: Value::Array(params),
        })
    }

    /// The same predicates without ordering or pagination, for `COUNT(*)`.
    pub fn to_count_sql(&self, table: &str) -> Result<SqlFilter> {
        self.validate()?;
        let quoted_tbl = quote_table(table)?;
        let (conditions, params) = self.render_predicates(&quoted_tbl)?;

        let clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        Ok(SqlFilter {
            clause,
            params: Value::Array(params),
        })
    }

    fn render_predicates(&self, table: &str) -> Result<(Vec<String>, Vec<Value>)> {
        // Insertion order is preserved, so the clause and its parameter
        // positions are stable between calls. An unstable clause is untestable
        // and defeats Postgres' prepared-statement cache.
        let mut conditions = Vec::with_capacity(self.predicates.len() + self.or_groups.len());
        let mut params: Vec<Value> = Vec::with_capacity(
            self.predicates.len()
                + self
                    .or_groups
                    .iter()
                    .map(|g| g.predicates.len())
                    .sum::<usize>(),
        );

        // 1️⃣ Flat predicates — AND-ed together, each gets its own param slot.
        for (field, op) in &self.predicates {
            let col = ident(field)?;
            let i = params.len();
            // The value, coerced to the column's type via the table's row type.
            let bound = format!("(json_populate_record(NULL::{table}, $1::json -> {i})).\"{col}\"");

            let condition = match op {
                FilterOp::Eq(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" = {bound}")
                }
                FilterOp::Ne(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" IS DISTINCT FROM {bound}")
                }
                FilterOp::Gt(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" > {bound}")
                }
                FilterOp::Gte(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" >= {bound}")
                }
                FilterOp::Lt(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" < {bound}")
                }
                FilterOp::Lte(v) => {
                    params.push(json!({ col: v }));
                    format!("t.\"{col}\" <= {bound}")
                }
                FilterOp::Like(pattern) => {
                    // A LIKE pattern is text by definition, so it is read out as
                    // text rather than coerced through the row type.
                    params.push(json!({ col: pattern }));
                    format!("t.\"{col}\"::text LIKE ($1::json -> {i} ->> '{col}')")
                }
                FilterOp::In(values) => {
                    // Each element is coerced individually through the row type.
                    let elements: Vec<Value> = values.iter().map(|v| json!({ col: v })).collect();
                    params.push(Value::Array(elements));
                    format!(
                        "t.\"{col}\" = ANY(ARRAY(SELECT (json_populate_record(NULL::{table}, e)).\"{col}\" \
                         FROM json_array_elements($1::json -> {i}) e))"
                    )
                }
                FilterOp::IsNull => {
                    // Carries no value but still occupies a slot, so every
                    // predicate's index equals its position in the array.
                    params.push(Value::Null);
                    format!("t.\"{col}\" IS NULL")
                }
                FilterOp::NotNull => {
                    params.push(Value::Null);
                    format!("t.\"{col}\" IS NOT NULL")
                }
            };
            conditions.push(condition);
        }

        // 2️⃣ OR-group predicates — each group's predicates are OR-ed together,
        //    and the whole group is AND-ed with the flat predicates above.
        //    Each predicate in a group gets the next param slot after the flat ones.
        for group in &self.or_groups {
            let mut group_conditions = Vec::with_capacity(group.predicates.len());
            for (field, op) in &group.predicates {
                let col = ident(field)?;
                let i = params.len(); // param index among all params (flat + groups so far)
                let bound =
                    format!("(json_populate_record(NULL::{table}, $1::json -> {i})).\"{col}\"");

                let condition = match op {
                    FilterOp::Eq(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" = {bound}")
                    }
                    FilterOp::Ne(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" IS DISTINCT FROM {bound}")
                    }
                    FilterOp::Gt(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" > {bound}")
                    }
                    FilterOp::Gte(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" >= {bound}")
                    }
                    FilterOp::Lt(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" < {bound}")
                    }
                    FilterOp::Lte(v) => {
                        params.push(json!({ col: v }));
                        format!("t.\"{col}\" <= {bound}")
                    }
                    FilterOp::Like(pattern) => {
                        params.push(json!({ col: pattern }));
                        format!("t.\"{col}\"::text LIKE ($1::json -> {i} ->> '{col}')")
                    }
                    FilterOp::In(values) => {
                        let elements: Vec<Value> =
                            values.iter().map(|v| json!({ col: v })).collect();
                        params.push(Value::Array(elements));
                        format!(
                            "t.\"{col}\" = ANY(ARRAY(SELECT (json_populate_record(NULL::{table}, e)).\"{col}\" \
                             FROM json_array_elements($1::json -> {i}) e))"
                        )
                    }
                    FilterOp::IsNull => {
                        params.push(Value::Null);
                        format!("t.\"{col}\" IS NULL")
                    }
                    FilterOp::NotNull => {
                        params.push(Value::Null);
                        format!("t.\"{col}\" IS NOT NULL")
                    }
                };
                group_conditions.push(condition);
            }
            // Wrap the group's conditions in parentheses joined by ` OR `
            let group_condition = if group_conditions.len() == 1 {
                group_conditions.into_iter().next().unwrap()
            } else {
                format!(
                    "({})",
                    group_conditions
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(" OR ")
                )
            };
            conditions.push(group_condition);
        }

        Ok((conditions, params))
    }
}
