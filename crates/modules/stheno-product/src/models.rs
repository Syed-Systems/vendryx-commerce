//! `pub(crate)` Diesel entity models for `stheno-product` (AGENTS.md §2.9, TRD §4.1).

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::schema::{
    product, product_category, product_collection, product_image, product_option,
    product_option_value, product_product_option, product_product_option_value, product_tag,
    product_type, product_variant, product_variant_product_image,
};

/// Computes a deterministic SHA-256 hash of sorted option value IDs to define variant identity (P2-04, P2-05).
pub fn compute_variant_option_values_hash(mut option_value_ids: Vec<String>) -> String {
    option_value_ids.sort();
    let mut hasher = Sha256::new();
    for id in option_value_ids {
        hasher.update(id.as_bytes());
        hasher.update(b",");
    }
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Draft,
    Proposed,
    Published,
    Rejected,
}

impl ProductStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Proposed => "proposed",
            Self::Published => "published",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "proposed" => Some(Self::Proposed),
            "published" => Some(Self::Published),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub handle: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub thumbnail: Option<String>,
    pub weight: Option<Decimal>,
    pub length: Option<Decimal>,
    pub height: Option<Decimal>,
    pub width: Option<Decimal>,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub discountable: bool,
    pub external_id: Option<String>,
    pub type_id: Option<String>,
    pub collection_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_variant)]
pub struct ProductVariant {
    pub id: String,
    pub product_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
    pub allow_backorder: bool,
    pub manage_inventory: bool,
    pub variant_rank: i32,
    pub weight: Option<Decimal>,
    pub length: Option<Decimal>,
    pub height: Option<Decimal>,
    pub width: Option<Decimal>,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_option)]
pub struct ProductOption {
    pub id: String,
    pub product_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_option_value)]
pub struct ProductOptionValue {
    pub id: String,
    pub option_id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_product_option)]
pub struct ProductProductOption {
    pub id: String,
    pub product_id: String,
    pub option_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_product_option_value)]
pub struct ProductProductOptionValue {
    pub id: String,
    pub variant_id: String,
    pub option_value_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_image)]
pub struct ProductImage {
    pub id: String,
    pub url: String,
    pub rank: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_variant_product_image)]
pub struct ProductVariantProductImage {
    pub id: String,
    pub variant_id: String,
    pub image_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_category)]
pub struct ProductCategory {
    pub id: String,
    pub name: String,
    pub handle: String,
    pub parent_category_id: Option<String>,
    pub rank: i32,
    pub is_active: bool,
    pub is_internal: bool,
    pub mpath: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_collection)]
pub struct ProductCollection {
    pub id: String,
    pub title: String,
    pub handle: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_tag)]
pub struct ProductTag {
    pub id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = product_type)]
pub struct ProductType {
    pub id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}
