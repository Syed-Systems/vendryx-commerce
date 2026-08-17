//! Internal entity models for `stheno-pricing` (TRD §4.1, DATA_MODEL.md).
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PriceSet {
    pub id: Id<PriceSet>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Price {
    pub id: Id<Price>,
    pub price_set_id: String,
    pub price_list_id: Option<String>,
    pub currency_code: String,
    pub amount: Decimal,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub rules_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PriceRule {
    pub id: Id<PriceRule>,
    pub price_id: String,
    pub attribute: String,
    pub value: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PriceList {
    pub id: Id<PriceList>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub r#type: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PriceListRule {
    pub id: Id<PriceListRule>,
    pub price_list_id: String,
    pub attribute: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PricePreference {
    pub id: Id<PricePreference>,
    pub attribute: String,
    pub value: String,
    pub is_tax_inclusive: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Price => "price",
    PriceList => "plist",
    PriceListRule => "plrule",
    PricePreference => "ppref",
    PriceRule => "prule",
    PriceSet => "pset",
}

macro_rules! impl_crud_entity {
    ($t:ident, $table:literal, $entity:literal) => {
        impl CrudEntity for $t {
            const TABLE: &'static str = $table;
            const ENTITY: &'static str = $entity;

            fn id(&self) -> String {
                self.id.to_string()
            }
            fn is_deleted(&self) -> bool {
                self.deleted_at.is_some()
            }
            fn set_deleted(&mut self, deleted: bool) {
                self.deleted_at = if deleted { Some(Utc::now()) } else { None };
            }
        }
    };
}

impl_crud_entity!(PriceSet, "pricing.price_set", "price_set");
impl_crud_entity!(Price, "pricing.price", "price");
impl_crud_entity!(PriceRule, "pricing.price_rule", "price_rule");
impl_crud_entity!(PriceList, "pricing.price_list", "price_list");
impl_crud_entity!(PriceListRule, "pricing.price_list_rule", "price_list_rule");
impl_crud_entity!(
    PricePreference,
    "pricing.price_preference",
    "price_preference"
);
