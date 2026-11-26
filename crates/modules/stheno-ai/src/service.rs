//! Tenant-scoped persistence and lexical retrieval for merchant-authored knowledge.

use stheno_db::diesel::result::DatabaseErrorKind;
use stheno_db::diesel::sql_types::{BigInt, Float, Nullable, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{Conn, Error, Result};

use crate::dto::{CreateKnowledgeEntry, KnowledgeEntry, KnowledgeSearchHit, UpdateKnowledgeEntry};
use crate::models::KnowledgeEntryRow;

const MAX_TITLE_CHARS: usize = 200;
const MAX_CATEGORY_CHARS: usize = 64;
const MAX_CONTENT_CHARS: usize = 20_000;
const MAX_QUERY_CHARS: usize = 512;
const MAX_SEARCH_RESULTS: i64 = 8;

/// Persists and searches static policy/FAQ content for the active tenant.
///
/// Callers must pass a connection with tenant scope established by
/// `stheno_db::tenant::apply_scope`; PostgreSQL row-level security is the isolation
/// boundary. This service accepts static merchant knowledge, never conversation data.
#[derive(Debug, Default, Clone, Copy)]
pub struct KnowledgeService;

impl KnowledgeService {
    /// Creates a tenant-owned knowledge entry after enforcing field size and content rules.
    pub async fn create(
        &self,
        conn: &mut Conn<'_>,
        input: CreateKnowledgeEntry,
    ) -> Result<KnowledgeEntry> {
        let (title, category, content) =
            validate_fields(input.title, input.category, input.content)?;
        let id = KnowledgeEntryRow::new_id();
        let row = diesel::sql_query(
            "INSERT INTO ai.knowledge_entry (id, title, category, content) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, title, category, content, created_at, updated_at",
        )
        .bind::<Text, _>(&id)
        .bind::<Text, _>(&title)
        .bind::<Nullable<Text>, _>(category)
        .bind::<Text, _>(&content)
        .get_result::<KnowledgeEntryRow>(*conn)
        .await
        .map_err(map_write_error)?;

        Ok(row.into())
    }

    /// Updates one entry in the active tenant. A missing or foreign-tenant ID is not found.
    pub async fn update(
        &self,
        conn: &mut Conn<'_>,
        id: &str,
        input: UpdateKnowledgeEntry,
    ) -> Result<KnowledgeEntry> {
        let (title, category, content) =
            validate_fields(input.title, input.category, input.content)?;
        let row = diesel::sql_query(
            "UPDATE ai.knowledge_entry \
                SET title = $2, category = $3, content = $4, updated_at = now() \
              WHERE id = $1 AND deleted_at IS NULL \
              RETURNING id, title, category, content, created_at, updated_at",
        )
        .bind::<Text, _>(id)
        .bind::<Text, _>(&title)
        .bind::<Nullable<Text>, _>(category)
        .bind::<Text, _>(&content)
        .get_result::<KnowledgeEntryRow>(*conn)
        .await
        .map_err(|error| map_missing_or_write_error(error, id))?;

        Ok(row.into())
    }

    /// Soft-deletes one entry. Repeated deletion and foreign-tenant IDs resolve as not found.
    pub async fn delete(&self, conn: &mut Conn<'_>, id: &str) -> Result<()> {
        let changed = diesel::sql_query(
            "UPDATE ai.knowledge_entry SET deleted_at = now(), updated_at = now() \
              WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind::<Text, _>(id)
        .execute(*conn)
        .await
        .map_err(|error| Error::Database(error.to_string()))?;

        if changed == 0 {
            return Err(Error::NotFound {
                entity: "knowledge_entry",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    /// Lists active entries in stable order with a bounded page size.
    pub async fn list(
        &self,
        conn: &mut Conn<'_>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<KnowledgeEntry>> {
        let limit = i64::from(limit.clamp(1, stheno_db::filter::MAX_PAGE_LIMIT as u32));
        let offset = i64::from(offset);
        let rows = diesel::sql_query(
            "SELECT id, title, category, content, created_at, updated_at \
               FROM ai.knowledge_entry \
              WHERE deleted_at IS NULL \
              ORDER BY lower(title), id \
              LIMIT $1 OFFSET $2",
        )
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<KnowledgeEntryRow>(*conn)
        .await
        .map_err(|error| Error::Database(error.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Retrieves matching static knowledge, never live products, prices, or stock.
    ///
    /// PostgreSQL's `simple` text configuration avoids English stemming and tokenizes
    /// multilingual policy text. Relevance ties are stable by update time and ID, and the
    /// result count is capped to keep prompts bounded.
    pub async fn search(
        &self,
        conn: &mut Conn<'_>,
        query: &str,
        limit: u32,
    ) -> Result<Vec<KnowledgeSearchHit>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        if query.chars().count() > MAX_QUERY_CHARS {
            return Err(invalid(
                "knowledge_query_too_long",
                "query",
                MAX_QUERY_CHARS,
            ));
        }
        if query.contains('\0') {
            return Err(invalid_value(
                "knowledge_query_invalid",
                "query",
                "Search query contains an unsupported null character.",
            ));
        }

        let limit = i64::from(limit.clamp(1, MAX_SEARCH_RESULTS as u32));
        #[derive(QueryableByName)]
        struct SearchRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Nullable<Text>)]
            category: Option<String>,
            #[diesel(sql_type = Text)]
            content: String,
            #[diesel(sql_type = Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Timestamptz)]
            updated_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Float)]
            rank: f32,
        }

        let rows = diesel::sql_query(
            "SELECT id, title, category, content, created_at, updated_at, \
                    ts_rank(search_vector, plainto_tsquery('simple', $1)) AS rank \
               FROM ai.knowledge_entry \
              WHERE deleted_at IS NULL \
                AND search_vector @@ plainto_tsquery('simple', $1) \
              ORDER BY rank DESC, updated_at DESC, id ASC \
              LIMIT $2",
        )
        .bind::<Text, _>(query)
        .bind::<BigInt, _>(limit)
        .load::<SearchRow>(*conn)
        .await
        .map_err(|error| Error::Database(error.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| KnowledgeSearchHit {
                entry: KnowledgeEntry {
                    id: row.id,
                    title: row.title,
                    category: row.category,
                    content: row.content,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                rank: row.rank,
            })
            .collect())
    }
}

fn validate_fields(
    title: String,
    category: Option<String>,
    content: String,
) -> Result<(String, Option<String>, String)> {
    let title = title.trim().to_string();
    let content = content.trim().to_string();
    let category = category.map(|value| value.trim().to_string());
    let category = category.filter(|value| !value.is_empty());

    for (field, value) in [("title", title.as_str()), ("content", content.as_str())] {
        if value.contains('\0') {
            return Err(invalid_value(
                "knowledge_field_invalid",
                field,
                "Knowledge fields must not contain a null character.",
            ));
        }
    }
    if category
        .as_deref()
        .is_some_and(|value| value.contains('\0'))
    {
        return Err(invalid_value(
            "knowledge_field_invalid",
            "category",
            "Knowledge fields must not contain a null character.",
        ));
    }

    if title.is_empty() {
        return Err(Error::Invalid {
            code: "knowledge_title_required",
            message: "Knowledge entry title must not be empty.".to_string(),
            field: Some("title".to_string()),
        });
    }
    if content.is_empty() {
        return Err(Error::Invalid {
            code: "knowledge_content_required",
            message: "Knowledge entry content must not be empty.".to_string(),
            field: Some("content".to_string()),
        });
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(invalid(
            "knowledge_title_too_long",
            "title",
            MAX_TITLE_CHARS,
        ));
    }
    if content.chars().count() > MAX_CONTENT_CHARS {
        return Err(invalid(
            "knowledge_content_too_long",
            "content",
            MAX_CONTENT_CHARS,
        ));
    }
    if category
        .as_ref()
        .is_some_and(|value| value.chars().count() > MAX_CATEGORY_CHARS)
    {
        return Err(invalid(
            "knowledge_category_too_long",
            "category",
            MAX_CATEGORY_CHARS,
        ));
    }

    Ok((title, category, content))
}

fn invalid(code: &'static str, field: &'static str, max_chars: usize) -> Error {
    Error::Invalid {
        code,
        message: format!("Value exceeds the {max_chars}-character limit."),
        field: Some(field.to_string()),
    }
}

fn invalid_value(code: &'static str, field: &'static str, message: &'static str) -> Error {
    Error::Invalid {
        code,
        message: message.to_string(),
        field: Some(field.to_string()),
    }
}

fn map_write_error(error: stheno_db::diesel::result::Error) -> Error {
    match error {
        stheno_db::diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            Error::Conflict {
                code: "duplicate_knowledge_entry",
                message: "An active entry with this title and category already exists.".to_string(),
            }
        }
        other => Error::Database(other.to_string()),
    }
}

fn map_missing_or_write_error(error: stheno_db::diesel::result::Error, id: &str) -> Error {
    match error {
        stheno_db::diesel::result::Error::NotFound => Error::NotFound {
            entity: "knowledge_entry",
            id: id.to_string(),
        },
        other => map_write_error(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalizes_merchant_knowledge() {
        let normalized = validate_fields(
            "  Returns  ".to_string(),
            Some("  Policy  ".to_string()),
            "  Unused items can be returned within 14 days.  ".to_string(),
        )
        .expect("valid input");
        assert_eq!(normalized.0, "Returns");
        assert_eq!(normalized.1.as_deref(), Some("Policy"));
        assert_eq!(normalized.2, "Unused items can be returned within 14 days.");
    }

    #[test]
    fn rejects_empty_content_and_oversized_fields() {
        assert!(validate_fields("FAQ".to_string(), None, "   ".to_string()).is_err());
        assert!(
            validate_fields("x".repeat(MAX_TITLE_CHARS + 1), None, "answer".to_string()).is_err()
        );
        assert!(
            validate_fields("FAQ".to_string(), None, "x".repeat(MAX_CONTENT_CHARS + 1)).is_err()
        );
        assert!(validate_fields(
            "FAQ".to_string(),
            Some("x".repeat(MAX_CATEGORY_CHARS + 1)),
            "answer".to_string()
        )
        .is_err());
        assert!(validate_fields("bad\0title".to_string(), None, "answer".to_string()).is_err());
        assert!(validate_fields("FAQ".to_string(), None, "bad\0content".to_string()).is_err());
        assert!(validate_fields(
            "FAQ".to_string(),
            Some("bad\0category".to_string()),
            "answer".to_string()
        )
        .is_err());
    }
}
