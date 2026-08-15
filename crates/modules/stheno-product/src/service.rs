//! ProductService domain logic with variant identity enforcement and tree queries.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    compute_variant_option_values_hash, Product, ProductCategory, ProductStatus, ProductVariant,
};

#[derive(Clone, Default)]
pub struct ProductService {
    products: Arc<RwLock<HashMap<String, Product>>>,
    variants: Arc<RwLock<HashMap<String, ProductVariant>>>,
    variant_hashes: Arc<RwLock<HashMap<String, String>>>, // product_id::hash -> variant_id (for fast unique index)
    categories: Arc<RwLock<HashMap<String, ProductCategory>>>,
}

impl ProductService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_product(&self, input: CreateProductInput) -> Result<ProductDTO> {
        let id = format!("prod_{}", ulid::Ulid::new());
        let handle = input
            .handle
            .unwrap_or_else(|| input.title.to_lowercase().replace(' ', "-"));

        let product = Product {
            id: id.clone(),
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            metadata: None,
        };

        let mut map = self.products.write().unwrap();
        map.insert(product.id.clone(), product.clone());

        Ok(ProductDTO {
            id: Id::from_string(&product.id).unwrap_or_default(),
            title: product.title,
            handle: product.handle,
            subtitle: product.subtitle,
            description: product.description,
            status: "draft".to_string(),
            thumbnail: product.thumbnail,
            discountable: product.discountable,
            created_at: product.created_at,
            updated_at: product.updated_at,
        })
    }

    pub async fn get_product(&self, id: &str) -> Result<ProductDTO> {
        let map = self.products.read().unwrap();
        let p = map
            .values()
            .find(|p| (p.id == id || p.id.ends_with(id)) && p.deleted_at.is_none())
            .or_else(|| map.get(id).filter(|p| p.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "product",
                id: id.to_string(),
            })?;

        Ok(ProductDTO {
            id: Id::from_string(&p.id).unwrap_or_default(),
            title: p.title.clone(),
            handle: p.handle.clone(),
            subtitle: p.subtitle.clone(),
            description: p.description.clone(),
            status: "draft".to_string(),
            thumbnail: p.thumbnail.clone(),
            discountable: p.discountable,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
    }

    pub async fn update_product(&self, id: &str, input: UpdateProductInput) -> Result<ProductDTO> {
        let mut map = self.products.write().unwrap();
        let p = map
            .values_mut()
            .find(|p| (p.id == id || p.id.ends_with(id)) && p.deleted_at.is_none())
            .ok_or_else(|| Error::NotFound {
                entity: "product",
                id: id.to_string(),
            })?;

        if let Some(t) = input.title {
            p.title = t;
        }
        if let Some(h) = input.handle {
            p.handle = h;
        }
        if let Some(s) = input.subtitle {
            p.subtitle = Some(s);
        }
        if let Some(d) = input.description {
            p.description = Some(d);
        }
        if let Some(st) = input.status {
            p.status = st;
        }
        if let Some(thumb) = input.thumbnail {
            p.thumbnail = Some(thumb);
        }
        if let Some(disc) = input.discountable {
            p.discountable = disc;
        }
        p.updated_at = Utc::now();

        let updated = p.clone();
        Ok(ProductDTO {
            id: Id::from_string(&updated.id).unwrap_or_default(),
            title: updated.title,
            handle: updated.handle,
            subtitle: updated.subtitle,
            description: updated.description,
            status: updated.status,
            thumbnail: updated.thumbnail,
            discountable: updated.discountable,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        })
    }

    pub async fn delete_product(&self, id: &str) -> Result<()> {
        let mut map = self.products.write().unwrap();
        let p = map
            .values_mut()
            .find(|p| (p.id == id || p.id.ends_with(id)) && p.deleted_at.is_none())
            .ok_or_else(|| Error::NotFound {
                entity: "product",
                id: id.to_string(),
            })?;
        p.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn list_products(&self) -> Result<Vec<ProductDTO>> {
        let map = self.products.read().unwrap();
        let list = map
            .values()
            .filter(|p| p.deleted_at.is_none())
            .map(|p| ProductDTO {
                id: Id::from_string(&p.id).unwrap_or_default(),
                title: p.title.clone(),
                handle: p.handle.clone(),
                subtitle: p.subtitle.clone(),
                description: p.description.clone(),
                status: p.status.clone(),
                thumbnail: p.thumbnail.clone(),
                discountable: p.discountable,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect();
        Ok(list)
    }

    pub async fn get_category(&self, id: &str) -> Result<ProductCategoryDTO> {
        let cats = self.categories.read().unwrap();
        let c = cats
            .values()
            .find(|c| (c.id == id || c.id.ends_with(id)) && c.deleted_at.is_none())
            .or_else(|| cats.get(id).filter(|c| c.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "product_category",
                id: id.to_string(),
            })?;

        Ok(ProductCategoryDTO {
            id: Id::from_string(&c.id).unwrap_or_default(),
            name: c.name.clone(),
            handle: c.handle.clone(),
            mpath: c.mpath.clone(),
            is_active: c.is_active,
            parent_category_id: c.parent_category_id.clone(),
            rank: c.rank,
        })
    }

    pub async fn list_categories(&self) -> Result<Vec<ProductCategoryDTO>> {
        let cats = self.categories.read().unwrap();
        let list = cats
            .values()
            .filter(|c| c.deleted_at.is_none())
            .map(|c| ProductCategoryDTO {
                id: Id::from_string(&c.id).unwrap_or_default(),
                name: c.name.clone(),
                handle: c.handle.clone(),
                mpath: c.mpath.clone(),
                is_active: c.is_active,
                parent_category_id: c.parent_category_id.clone(),
                rank: c.rank,
            })
            .collect();
        Ok(list)
    }

    /// Creates variant with unique index on sorted option-value hash (P2-04, P2-05)
    pub async fn create_variant(&self, input: CreateVariantInput) -> Result<ProductVariantDTO> {
        let hash = compute_variant_option_values_hash(input.option_value_ids);
        let hash_key = format!("{}::{}", input.product_id, hash);

        let mut hashes = self.variant_hashes.write().unwrap();
        if hashes.contains_key(&hash_key) {
            return Err(Error::Conflict {
                code: "variant_option_combination_exists",
                message: "A variant with this exact combination of options already exists on this product".to_string(),
            });
        }

        let variant_id = format!("variant_{}", ulid::Ulid::new());
        let variant = ProductVariant {
            id: variant_id.clone(),
            product_id: input.product_id.clone(),
            title: input.title,
            sku: input.sku,
            barcode: input.barcode,
            ean: None,
            upc: None,
            allow_backorder: false,
            manage_inventory: true,
            variant_rank: 0,
            weight: None,
            length: None,
            height: None,
            width: None,
            hs_code: None,
            origin_country: None,
            mid_code: None,
            material: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            metadata: None,
        };

        hashes.insert(hash_key, variant.id.clone());
        let mut vars = self.variants.write().unwrap();
        vars.insert(variant.id.clone(), variant.clone());

        Ok(ProductVariantDTO {
            id: Id::from_string(&variant.id).unwrap_or_default(),
            product_id: variant.product_id,
            title: variant.title,
            sku: variant.sku,
            barcode: variant.barcode,
            allow_backorder: variant.allow_backorder,
            manage_inventory: variant.manage_inventory,
            option_values_hash: hash,
            created_at: variant.created_at,
        })
    }

    /// Creates category with materialized path `mpath` (P2-06)
    pub async fn create_category(&self, input: CreateCategoryInput) -> Result<ProductCategoryDTO> {
        let cat_id = format!("pcat_{}", ulid::Ulid::new());
        let handle = input
            .handle
            .unwrap_or_else(|| input.name.to_lowercase().replace(' ', "-"));

        let mpath = if let Some(parent_id) = &input.parent_category_id {
            let cats = self.categories.read().unwrap();
            let parent = cats
                .values()
                .find(|c| c.id == *parent_id || c.id.ends_with(parent_id))
                .or_else(|| cats.get(parent_id))
                .ok_or_else(|| Error::NotFound {
                    entity: "product_category",
                    id: parent_id.clone(),
                })?;
            format!("{}{}.", parent.mpath, cat_id)
        } else {
            format!("{}.", cat_id)
        };

        let cat = ProductCategory {
            id: cat_id.clone(),
            name: input.name,
            handle,
            parent_category_id: input.parent_category_id.clone(),
            rank: input.rank.unwrap_or(0),
            is_active: true,
            is_internal: false,
            mpath: mpath.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            metadata: None,
        };

        let mut cats = self.categories.write().unwrap();
        cats.insert(cat.id.clone(), cat.clone());

        Ok(ProductCategoryDTO {
            id: Id::from_string(&cat.id).unwrap_or_default(),
            name: cat.name,
            handle: cat.handle,
            mpath: cat.mpath,
            is_active: cat.is_active,
            parent_category_id: cat.parent_category_id,
            rank: cat.rank,
        })
    }

    /// Queries all descendants of a category via materialized path prefix match
    pub async fn list_descendants(&self, category_id: &str) -> Result<Vec<ProductCategoryDTO>> {
        let cats = self.categories.read().unwrap();
        let target = cats
            .values()
            .find(|c| c.id == category_id || c.id.ends_with(category_id))
            .or_else(|| cats.get(category_id))
            .ok_or_else(|| Error::NotFound {
                entity: "product_category",
                id: category_id.to_string(),
            })?;

        let prefix = &target.mpath;
        let target_id = &target.id;
        let descendants = cats
            .values()
            .filter(|c| c.mpath.starts_with(prefix) && c.id != *target_id)
            .map(|c| ProductCategoryDTO {
                id: Id::from_string(&c.id).unwrap_or_default(),
                name: c.name.clone(),
                handle: c.handle.clone(),
                mpath: c.mpath.clone(),
                is_active: c.is_active,
                parent_category_id: c.parent_category_id.clone(),
                rank: c.rank,
            })
            .collect();

        Ok(descendants)
    }
}
