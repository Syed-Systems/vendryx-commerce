//! ProductService domain logic, backed by Postgres (P11-19, P11-20).
//!
//! Variant identity and the category tree are the two parts worth reading:
//!
//! * variant identity is now a stored `option_values_hash` column with a unique
//!   index, not a `variant_hashes` map beside the service that vanished on
//!   restart and could be raced by two concurrent writers;
//! * the category tree keeps its materialized path, and descendant queries
//!   become a prefix `LIKE` -- the one wildcard shape `FilterSpec` allows,
//!   because it is index-backed.

use chrono::Utc;
use sha2::Digest;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    compute_variant_option_values_hash, Product, ProductCategory, ProductCollection, ProductOption,
    ProductOptionValue, ProductProductOptionValue, ProductStatus, ProductTag, ProductType,
    ProductVariant,
};

/// Page size for unpaginated internal listings.
///
/// `FilterSpec` clamps to `MAX_PAGE_LIMIT` anyway; naming it here makes the
/// ceiling visible at the call site rather than a surprise at 101 rows.
const LIST_LIMIT: usize = stheno_db::filter::MAX_PAGE_LIMIT;

#[derive(Clone)]
pub struct ProductService {
    products: PgCrudRepository<Product>,
    variants: PgCrudRepository<ProductVariant>,
    options: PgCrudRepository<ProductOption>,
    option_values: PgCrudRepository<ProductOptionValue>,
    /// variant -> option value. `create_variant` folded these ids into a hash and never
    /// stored them, so nothing could answer "which options is this variant?" — which is
    /// exactly what a storefront needs to match a shopper's selection to a variant.
    variant_option_values: PgCrudRepository<ProductProductOptionValue>,
    categories: PgCrudRepository<ProductCategory>,
    collections: PgCrudRepository<ProductCollection>,
    tags: PgCrudRepository<ProductTag>,
    types: PgCrudRepository<ProductType>,
}

impl ProductService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            products: PgCrudRepository::new(pool.clone()),
            variants: PgCrudRepository::new(pool.clone()),
            options: PgCrudRepository::new(pool.clone()),
            option_values: PgCrudRepository::new(pool.clone()),
            variant_option_values: PgCrudRepository::new(pool.clone()),
            categories: PgCrudRepository::new(pool.clone()),
            tags: PgCrudRepository::new(pool.clone()),
            types: PgCrudRepository::new(pool.clone()),
            collections: PgCrudRepository::new(pool),
        }
    }

    fn live(limit: usize) -> FilterSpec {
        FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, limit)
    }

    // ---- products -------------------------------------------------------

    pub async fn create_product(&self, input: CreateProductInput) -> Result<ProductDTO> {
        let handle = input
            .handle
            .unwrap_or_else(|| input.title.to_lowercase().replace(' ', "-"));

        let now = Utc::now();
        let product = Product {
            id: format!("prod_{}", ulid::Ulid::new()),
            title: input.title,
            handle,
            subtitle: input.subtitle,
            description: input.description,
            status: ProductStatus::Draft.as_str().to_string(),
            thumbnail: None,
            weight: None,
            length: None,
            height: None,
            width: None,
            hs_code: None,
            origin_country: None,
            mid_code: None,
            material: None,
            discountable: input.discountable.unwrap_or(true),
            external_id: None,
            type_id: None,
            collection_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: None,
        };

        self.products
            .create(None, product)
            .await
            .map(|p| product_dto(&p))
    }

    pub async fn get_product(&self, id: &str) -> Result<ProductDTO> {
        self.products
            .retrieve(None, id)
            .await
            .map(|p| product_dto(&p))
    }

    pub async fn update_product(&self, id: &str, input: UpdateProductInput) -> Result<ProductDTO> {
        let mut product = self.products.retrieve(None, id).await?;

        if let Some(title) = input.title {
            product.title = title;
        }
        if let Some(handle) = input.handle {
            product.handle = handle;
        }
        if let Some(subtitle) = input.subtitle {
            product.subtitle = Some(subtitle);
        }
        if let Some(description) = input.description {
            product.description = Some(description);
        }
        if let Some(status) = input.status {
            // An unknown status is rejected rather than stored: the value
            // reaches the wire, and a status nothing can render is worse than
            // a 400.
            ProductStatus::from_str_opt(&status).ok_or_else(|| Error::Invalid {
                code: "invalid_product_status",
                message: format!("'{status}' is not a product status"),
                field: Some("status".to_string()),
            })?;
            product.status = status;
        }
        if let Some(discountable) = input.discountable {
            product.discountable = discountable;
        }
        product.updated_at = Utc::now();

        let product_id = product.id.clone();
        self.products
            .update(None, &product_id, product)
            .await
            .map(|p| product_dto(&p))
    }

    pub async fn delete_product(&self, id: &str) -> Result<()> {
        let product = self.products.retrieve(None, id).await?;
        self.products.soft_delete(None, &product.id).await
    }

    pub async fn list_products(&self) -> Result<Vec<ProductDTO>> {
        let filter = Self::live(LIST_LIMIT).with_order("created_at", OrderDirection::Asc);
        Ok(self
            .products
            .list(None, &filter)
            .await?
            .iter()
            .map(product_dto)
            .collect())
    }

    // ---- options and option values --------------------------------------

    pub async fn create_option(&self, product_id: &str, title: &str) -> Result<String> {
        // The product must exist: an option on a missing product is orphaned
        // data the foreign key would reject anyway, but this gives a 404.
        self.products.retrieve(None, product_id).await?;

        let now = Utc::now();
        let option = ProductOption {
            id: format!("opt_{}", ulid::Ulid::new()),
            product_id: product_id.to_string(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: None,
        };
        self.options.create(None, option).await.map(|o| o.id)
    }

    pub async fn create_option_value(&self, option_id: &str, value: &str) -> Result<String> {
        self.options.retrieve(None, option_id).await?;

        let now = Utc::now();
        let option_value = ProductOptionValue {
            id: format!("optval_{}", ulid::Ulid::new()),
            option_id: option_id.to_string(),
            value: value.to_string(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: None,
        };
        self.option_values
            .create(None, option_value)
            .await
            .map(|v| v.id)
    }

    pub async fn list_options(&self, product_id: &str) -> Result<Vec<(String, String)>> {
        let filter = Self::live(LIST_LIMIT)
            .with_predicate("product_id", FilterOp::Eq(product_id.to_string()))
            .with_order("created_at", OrderDirection::Asc);
        Ok(self
            .options
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|o| (o.id, o.title))
            .collect())
    }

    pub async fn list_option_values(&self, option_id: &str) -> Result<Vec<(String, String)>> {
        let filter = Self::live(LIST_LIMIT)
            .with_predicate("option_id", FilterOp::Eq(option_id.to_string()))
            .with_order("value", OrderDirection::Asc);
        Ok(self
            .option_values
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|v| (v.id, v.value))
            .collect())
    }

    // ---- variants -------------------------------------------------------

    pub async fn create_variant(&self, input: CreateVariantInput) -> Result<ProductVariantDTO> {
        let option_value_ids = input.option_value_ids.clone();

        // Checked before the variant row is written, not after.
        //
        // The option-value join rows are inserted once the variant exists, and they carry a
        // foreign key. An id that does not resolve therefore failed *after* the variant had
        // already been created, leaving a variant with no options behind — one no shopper
        // could ever select, and no error path removed it.
        if !option_value_ids.is_empty() {
            let found = self
                .option_values
                .list(
                    None,
                    &Self::live(LIST_LIMIT)
                        .with_predicate("id", FilterOp::In(option_value_ids.clone())),
                )
                .await?;
            if found.len() != option_value_ids.len() {
                let known: std::collections::HashSet<String> =
                    found.into_iter().map(|v| v.id).collect();
                let missing: Vec<&String> = option_value_ids
                    .iter()
                    .filter(|id| !known.contains(*id))
                    .collect();
                return Err(Error::Invalid {
                    code: "unknown_option_value",
                    message: format!("No such product option value(s): {missing:?}"),
                    field: Some("option_value_ids".to_string()),
                });
            }
        }
        let hash = if input.option_value_ids.is_empty() {
            let mut hasher = sha2::Sha256::new();
            hasher.update(input.title.as_bytes());
            hex::encode(hasher.finalize())
        } else {
            compute_variant_option_values_hash(input.option_value_ids)
        };

        let now = Utc::now();
        let variant = ProductVariant {
            id: format!("variant_{}", ulid::Ulid::new()),
            product_id: input.product_id.clone(),
            title: input.title,
            sku: input.sku,
            barcode: input.barcode,
            ean: None,
            upc: None,
            allow_backorder: false,
            manage_inventory: true,
            variant_rank: 0,
            option_values_hash: hash,
            weight: None,
            length: None,
            height: None,
            width: None,
            hs_code: None,
            origin_country: None,
            mid_code: None,
            material: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: None,
        };

        // Uniqueness is the database's job. A read-then-write check here would
        // let two concurrent writers both observe "absent" and both insert.
        match self.variants.create(None, variant).await {
            Ok(v) => {
                // The hash proves two variants differ; it cannot be read back into the
                // options that produced it. These rows are what makes the association
                // retrievable, and without them a storefront can never resolve a
                // shopper's selection to a variant.
                for option_value_id in &option_value_ids {
                    self.variant_option_values
                        .create(
                            None,
                            ProductProductOptionValue {
                                id: format!("provalue_{}", ulid::Ulid::new()),
                                variant_id: v.id.to_string(),
                                option_value_id: option_value_id.clone(),
                                created_at: now,
                                updated_at: now,
                                deleted_at: None,
                            },
                        )
                        .await?;
                }
                Ok(variant_dto(&v))
            }
            Err(Error::Conflict { .. }) => Err(Error::Conflict {
                code: "variant_option_combination_exists",
                message:
                    "A variant with this exact combination of options already exists on this product"
                        .to_string(),
            }),
            Err(other) => Err(other),
        }
    }

    pub async fn get_variant(&self, id: &str) -> Result<ProductVariantDTO> {
        self.variants
            .retrieve(None, id)
            .await
            .map(|v| variant_dto(&v))
    }

    pub async fn list_all_options(&self) -> Result<Vec<(String, String, String)>> {
        let filter = Self::live(LIST_LIMIT).with_order("title", OrderDirection::Asc);
        Ok(self
            .options
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|o| (o.id, o.title, o.product_id))
            .collect())
    }

    pub async fn update_variant(
        &self,
        id: &str,
        title: Option<String>,
        sku: Option<String>,
        barcode: Option<String>,
    ) -> Result<ProductVariantDTO> {
        let mut v = self.variants.retrieve(None, id).await?;
        if let Some(t) = title {
            v.title = t;
        }
        if let Some(s) = sku {
            v.sku = Some(s);
        }
        if let Some(b) = barcode {
            v.barcode = Some(b);
        }
        v.updated_at = Utc::now();
        self.variants
            .update(None, id, v)
            .await
            .map(|v| variant_dto(&v))
    }

    pub async fn delete_variant(&self, id: &str) -> Result<()> {
        self.variants.soft_delete(None, id).await
    }

    /// The option-value ids a variant is defined by.
    pub async fn list_variant_option_values(&self, variant_id: &str) -> Result<Vec<String>> {
        let filter = Self::live(LIST_LIMIT)
            .with_predicate("variant_id", FilterOp::Eq(variant_id.to_string()));
        Ok(self
            .variant_option_values
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|v| v.option_value_id)
            .collect())
    }

    pub async fn list_variants(&self, product_id: &str) -> Result<Vec<ProductVariantDTO>> {
        let filter = Self::live(LIST_LIMIT)
            .with_predicate("product_id", FilterOp::Eq(product_id.to_string()))
            .with_order("variant_rank", OrderDirection::Asc);
        Ok(self
            .variants
            .list(None, &filter)
            .await?
            .iter()
            .map(variant_dto)
            .collect())
    }

    // ---- categories -----------------------------------------------------

    /// Creates a category, extending its parent's materialized path (P2-06).
    pub async fn create_category(&self, input: CreateCategoryInput) -> Result<ProductCategoryDTO> {
        let cat_id = format!("pcat_{}", ulid::Ulid::new());
        let handle = input
            .handle
            .unwrap_or_else(|| input.name.to_lowercase().replace(' ', "-"));

        let mpath = match &input.parent_category_id {
            Some(parent_id) => {
                let parent = self.categories.retrieve(None, parent_id).await?;
                format!("{}{}.", parent.mpath, cat_id)
            }
            None => format!("{cat_id}."),
        };

        let now = Utc::now();
        let cat = ProductCategory {
            id: cat_id,
            name: input.name,
            handle,
            parent_category_id: input.parent_category_id.clone(),
            rank: input.rank.unwrap_or(0),
            is_active: true,
            is_internal: false,
            mpath,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: None,
        };

        self.categories
            .create(None, cat)
            .await
            .map(|c| category_dto(&c))
    }

    pub async fn get_category(&self, id: &str) -> Result<ProductCategoryDTO> {
        self.categories
            .retrieve(None, id)
            .await
            .map(|c| category_dto(&c))
    }

    pub async fn list_categories(&self) -> Result<Vec<ProductCategoryDTO>> {
        let filter = Self::live(LIST_LIMIT).with_order("mpath", OrderDirection::Asc);
        Ok(self
            .categories
            .list(None, &filter)
            .await?
            .iter()
            .map(category_dto)
            .collect())
    }

    /// All descendants of a category, via materialized-path prefix match.
    ///
    /// A trailing-wildcard `LIKE` is the one pattern `FilterSpec` permits,
    /// precisely because it can use an index -- which is the entire reason the
    /// tree is stored as a path rather than walked parent by parent.
    pub async fn list_descendants(&self, category_id: &str) -> Result<Vec<ProductCategoryDTO>> {
        let target = self.categories.retrieve(None, category_id).await?;

        let filter = Self::live(LIST_LIMIT)
            .with_predicate("mpath", FilterOp::Like(format!("{}%", target.mpath)))
            .with_order("mpath", OrderDirection::Asc);

        Ok(self
            .categories
            .list(None, &filter)
            .await?
            .iter()
            // The prefix match includes the target itself.
            .filter(|c| c.id != target.id)
            .map(category_dto)
            .collect())
    }

    // ---- collections ----------------------------------------------------

    pub async fn create_collection(
        &self,
        input: CreateCollectionInput,
    ) -> Result<ProductCollectionDTO> {
        let handle = input
            .handle
            .unwrap_or_else(|| input.title.to_lowercase().replace(' ', "-"));

        let now = Utc::now();
        let collection = ProductCollection {
            id: format!("pcol_{}", ulid::Ulid::new()),
            title: input.title,
            handle,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            metadata: input.metadata,
        };

        self.collections
            .create(None, collection)
            .await
            .map(|c| collection_dto(&c))
    }

    pub async fn get_collection(&self, id: &str) -> Result<ProductCollectionDTO> {
        self.collections
            .retrieve(None, id)
            .await
            .map(|c| collection_dto(&c))
    }

    pub async fn update_collection(
        &self,
        id: &str,
        input: UpdateCollectionInput,
    ) -> Result<ProductCollectionDTO> {
        let mut collection = self.collections.retrieve(None, id).await?;
        if let Some(t) = input.title {
            collection.title = t;
        }
        if let Some(h) = input.handle {
            collection.handle = h;
        }
        if let Some(m) = input.metadata {
            collection.metadata = Some(m);
        }
        collection.updated_at = Utc::now();
        self.collections
            .update(None, id, collection)
            .await
            .map(|c| collection_dto(&c))
    }

    pub async fn delete_collection(&self, id: &str) -> Result<()> {
        self.collections.soft_delete(None, id).await
    }

    pub async fn list_collections(&self) -> Result<Vec<ProductCollectionDTO>> {
        let filter = Self::live(LIST_LIMIT).with_order("title", OrderDirection::Asc);
        Ok(self
            .collections
            .list(None, &filter)
            .await?
            .iter()
            .map(collection_dto)
            .collect())
    }
    // ── Product tags and types ──────────────────────────────────────────────
    //
    // `product.product_tag` and `product.product_type` have existed since the first
    // product migration, with models and `CrudEntity` bindings. There were no service
    // methods, so `GET /admin/product-tags` and `/product-types` returned hardcoded
    // empty arrays and the admin's four tag/type screens had nothing behind them.

    pub async fn list_tags(&self) -> Result<Vec<ProductTag>> {
        self.tags.list(None, &Self::live(LIST_LIMIT)).await
    }

    pub async fn get_tag(&self, id: &str) -> Result<ProductTag> {
        self.tags.retrieve(None, id).await
    }

    pub async fn create_tag(&self, value: String) -> Result<ProductTag> {
        let now = Utc::now();
        self.tags
            .create(
                None,
                ProductTag {
                    id: format!("ptag_{}", ulid::Ulid::new()),
                    value,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                    metadata: None,
                },
            )
            .await
    }

    pub async fn update_tag(&self, id: &str, value: String) -> Result<ProductTag> {
        let mut tag = self.tags.retrieve(None, id).await?;
        tag.value = value;
        tag.updated_at = Utc::now();
        self.tags.update(None, id, tag).await
    }

    pub async fn delete_tag(&self, id: &str) -> Result<()> {
        self.tags.soft_delete(None, id).await?;
        Ok(())
    }

    pub async fn list_types(&self) -> Result<Vec<ProductType>> {
        self.types.list(None, &Self::live(LIST_LIMIT)).await
    }

    pub async fn get_type(&self, id: &str) -> Result<ProductType> {
        self.types.retrieve(None, id).await
    }

    pub async fn create_type(&self, value: String) -> Result<ProductType> {
        let now = Utc::now();
        self.types
            .create(
                None,
                ProductType {
                    id: format!("ptyp_{}", ulid::Ulid::new()),
                    value,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                    metadata: None,
                },
            )
            .await
    }

    /// Partial update for a category.
    ///
    /// Does not touch `mpath` or `parent_category_id` — see `UpdateCategoryInput`.
    /// Soft-deletes a category.
    ///
    /// Refuses while it still has children. `mpath` encodes ancestry as a string, so
    /// removing a parent silently orphans every descendant's path — they would keep a
    /// prefix pointing at a category that no longer exists, and the tree would render with
    /// a hole in it. Medusa's dashboard reparents before deleting; this makes the
    /// requirement explicit instead of corrupting the tree.
    pub async fn delete_category(&self, id: &str) -> Result<()> {
        let children = self
            .categories
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("parent_category_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1),
            )
            .await?;

        if !children.is_empty() {
            return Err(Error::Conflict {
                code: "category_has_children",
                message: format!(
                    "Category {id} still has child categories; move or delete them first"
                ),
            });
        }

        self.categories.soft_delete(None, id).await?;
        Ok(())
    }

    /// Soft-deletes a product option and the values under it.
    ///
    /// The values go in the same call: an option value whose option is gone is a row no
    /// query filters out, and `list_option_values` keys on the option id rather than on
    /// the parent's state.
    pub async fn delete_option(&self, id: &str) -> Result<()> {
        let values = self
            .option_values
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("option_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT),
            )
            .await?;

        for value in values {
            self.option_values
                .soft_delete(None, &value.id.to_string())
                .await?;
        }

        self.options.soft_delete(None, id).await?;
        Ok(())
    }

    pub async fn update_category(
        &self,
        id: &str,
        input: UpdateCategoryInput,
    ) -> Result<ProductCategoryDTO> {
        let mut cat = self.categories.retrieve(None, id).await?;

        if let Some(name) = input.name {
            cat.name = name;
        }
        if let Some(handle) = input.handle {
            cat.handle = handle;
        }
        if let Some(rank) = input.rank {
            cat.rank = rank;
        }
        if let Some(is_active) = input.is_active {
            cat.is_active = is_active;
        }
        if let Some(is_internal) = input.is_internal {
            cat.is_internal = is_internal;
        }
        cat.updated_at = Utc::now();

        self.categories
            .update(None, id, cat)
            .await
            .map(|c| category_dto(&c))
    }

    pub async fn update_type(&self, id: &str, value: String) -> Result<ProductType> {
        let mut ty = self.types.retrieve(None, id).await?;
        ty.value = value;
        ty.updated_at = Utc::now();
        self.types.update(None, id, ty).await
    }

    pub async fn delete_type(&self, id: &str) -> Result<()> {
        self.types.soft_delete(None, id).await?;
        Ok(())
    }
}

fn product_dto(p: &Product) -> ProductDTO {
    ProductDTO {
        id: Id::from_string(&p.id).unwrap_or_default(),
        title: p.title.clone(),
        handle: p.handle.clone(),
        subtitle: p.subtitle.clone(),
        description: p.description.clone(),
        // The stored status, not a hardcoded "draft": a product published by
        // `update_product` must not read back as a draft.
        status: p.status.clone(),
        thumbnail: p.thumbnail.clone(),
        discountable: p.discountable,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

fn variant_dto(v: &ProductVariant) -> ProductVariantDTO {
    ProductVariantDTO {
        id: Id::from_string(&v.id).unwrap_or_default(),
        product_id: v.product_id.clone(),
        title: v.title.clone(),
        sku: v.sku.clone(),
        barcode: v.barcode.clone(),
        allow_backorder: v.allow_backorder,
        manage_inventory: v.manage_inventory,
        option_values_hash: v.option_values_hash.clone(),
        created_at: v.created_at,
    }
}

fn category_dto(c: &ProductCategory) -> ProductCategoryDTO {
    ProductCategoryDTO {
        id: Id::from_string(&c.id).unwrap_or_default(),
        name: c.name.clone(),
        handle: c.handle.clone(),
        mpath: c.mpath.clone(),
        is_active: c.is_active,
        parent_category_id: c.parent_category_id.clone(),
        rank: c.rank,
    }
}

fn collection_dto(c: &ProductCollection) -> ProductCollectionDTO {
    ProductCollectionDTO {
        id: c.id.clone(),
        title: c.title.clone(),
        handle: c.handle.clone(),
        created_at: c.created_at,
        updated_at: c.updated_at,
        metadata: c.metadata.clone(),
    }
}
