//! Public Data Transfer Objects (DTOs) for `stheno-product`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDTO {
    pub id: Id<ProductDTO>,
    pub title: String,
    pub handle: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub thumbnail: Option<String>,
    pub discountable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateProductInput {
    pub title: String,
    pub handle: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub discountable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProductInput {
    pub title: Option<String>,
    pub handle: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub thumbnail: Option<String>,
    pub discountable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariantDTO {
    pub id: Id<ProductVariantDTO>,
    pub product_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub allow_backorder: bool,
    pub manage_inventory: bool,
    pub option_values_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateVariantInput {
    pub product_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub option_value_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategoryDTO {
    pub id: Id<ProductCategoryDTO>,
    pub name: String,
    pub handle: String,
    pub mpath: String,
    pub is_active: bool,
    pub parent_category_id: Option<String>,
    pub rank: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCategoryInput {
    pub name: String,
    pub handle: Option<String>,
    pub parent_category_id: Option<String>,
    pub rank: Option<i32>,
}

/// Partial update for a category. Absent means unchanged.
///
/// `parent_category_id` is deliberately absent: re-parenting rewrites `mpath` for the
/// node and every descendant, which is a tree operation, not a field edit. The dashboard
/// has a separate "organize" screen for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryInput {
    pub name: Option<String>,
    pub handle: Option<String>,
    pub rank: Option<i32>,
    pub is_active: Option<bool>,
    pub is_internal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCollectionDTO {
    pub id: String,
    pub title: String,
    pub handle: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCollectionInput {
    pub title: String,
    pub handle: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCollectionInput {
    pub title: Option<String>,
    pub handle: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    ProductCategoryDTO => "pcat",
    ProductDTO => "prod",
    ProductVariantDTO => "variant",
}
