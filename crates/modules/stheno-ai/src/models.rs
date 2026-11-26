//! Database rows owned by `stheno-ai`.

use chrono::{DateTime, Utc};
use stheno_db::diesel::QueryableByName;

#[derive(Debug, QueryableByName)]
pub(crate) struct KnowledgeEntryRow {
    #[diesel(sql_type = stheno_db::diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = stheno_db::diesel::sql_types::Text)]
    pub title: String,
    #[diesel(sql_type = stheno_db::diesel::sql_types::Nullable<stheno_db::diesel::sql_types::Text>)]
    pub category: Option<String>,
    #[diesel(sql_type = stheno_db::diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = stheno_db::diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = stheno_db::diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

impl From<KnowledgeEntryRow> for crate::dto::KnowledgeEntry {
    fn from(row: KnowledgeEntryRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            category: row.category,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

stheno_types::prefixed_id! {
    KnowledgeEntryRow => "aiknow",
}

impl KnowledgeEntryRow {
    pub(crate) fn new_id() -> String {
        stheno_types::Id::<Self>::new().to_string()
    }
}
