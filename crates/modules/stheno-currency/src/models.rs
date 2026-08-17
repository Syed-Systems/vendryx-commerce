//! Internal entity models for `stheno-currency` (DATA_MODEL §1, AGENTS.md §2.9).
//! Strictly `pub(crate)`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Currency {
    pub code: String,
    pub symbol: String,
    pub symbol_native: String,
    pub name: String,
    pub decimal_digits: i16,
    pub rounding: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Currency {
    const TABLE: &'static str = "currency.currency";
    const ENTITY: &'static str = "currency";
    const ID_COLUMN: &'static str = "code";

    fn id(&self) -> String {
        self.code.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}
