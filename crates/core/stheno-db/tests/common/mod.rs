// Shared by several test binaries; each uses a different subset of the helpers,
// so unused ones here are expected rather than dead.
#![allow(dead_code)]

//! A single reference entity used by the `PgCrudRepository` and FilterSpec
//! tests, with the column shapes that actually matter: text, numeric, integer,
//! timestamp, boolean, and a nullable soft-delete marker.
//!
//! Numeric and integer columns are here deliberately -- they are what proves the
//! filter translation compares in the column's type rather than as text.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::testing::TestDb;
use stheno_db::CrudEntity;

pub const CREATE_TABLE: &str = r#"
CREATE SCHEMA IF NOT EXISTS catalog;
CREATE TABLE catalog.widget (
    id           text PRIMARY KEY,
    title        text NOT NULL,
    handle       text NULL,
    status       text NOT NULL DEFAULT 'draft',
    quantity     integer NOT NULL DEFAULT 0,
    price        numeric(20,6) NOT NULL DEFAULT 0,
    discountable boolean NOT NULL DEFAULT true,
    created_at   timestamptz NOT NULL DEFAULT now(),
    deleted_at   timestamptz NULL
);
"#;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub title: String,
    pub handle: Option<String>,
    pub status: String,
    pub quantity: i32,
    pub price: rust_decimal::Decimal,
    pub discountable: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Widget {
    const TABLE: &'static str = "catalog.widget";
    const ENTITY: &'static str = "widget";

    fn id(&self) -> String {
        self.id.clone()
    }

    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

impl Widget {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            handle: Some(title.to_lowercase().replace(' ', "-")),
            status: "draft".to_string(),
            quantity: 0,
            price: rust_decimal::Decimal::ZERO,
            discountable: true,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = status.to_string();
        self
    }

    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn with_price(mut self, price: i64) -> Self {
        self.price = rust_decimal::Decimal::from(price);
        self
    }

    pub fn with_handle(mut self, handle: Option<&str>) -> Self {
        self.handle = handle.map(str::to_string);
        self
    }
}

/// A database with the widget table created, but no migrations -- these tests
/// exercise the repository, not the module schemas.
pub async fn widget_db() -> TestDb {
    let db = TestDb::create().await;
    db.execute(CREATE_TABLE).await;
    db
}
