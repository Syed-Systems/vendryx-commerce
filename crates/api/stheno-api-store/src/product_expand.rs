//! Assembles the nested product shape Medusa storefronts require.
//!
//! `GET /store/products` returned a flat `ProductDTO` — no `variants`, no `options`, no
//! prices. Every storefront behaviour downstream of a price therefore could not work:
//! `getProductPrice` returns null without `variant.calculated_price`, so the PLP showed no
//! price; `ProductActions` matches a variant by its `options` keymap, so with no options
//! it never resolved one and the add-to-cart button never enabled; and social checkout
//! needs a `variant_id` it had no way to obtain.
//!
//! The parts all existed — `ProductService::list_variants`, `list_options`,
//! `list_option_values`, and the whole pricing module. Nothing joined them.
//!
//! Kept in the API layer rather than widened into `ProductDTO`, because this shape is a
//! wire-compatibility concern: the admin, the flows and the 127 other entities have no use
//! for it, and `ProductDTO` is shared with all of them.

use std::collections::HashMap;
use std::sync::Arc;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_inventory::InventoryService;
use stheno_pricing::{CalculatedPriceDTO, PricingContext, PricingService};
use stheno_product::{ProductDTO, ProductService};
use stheno_types::Result;

/// What the storefront reads off `variant.calculated_price`.
///
/// The nesting is Medusa's, not a choice: `get-product-price.ts` reads
/// `calculated_price.calculated_price.price_list_type` to decide whether to render a sale
/// badge, so the inner object has to be there even when nothing is on sale.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreCalculatedPrice {
    pub id: String,
    pub calculated_amount: Decimal,
    pub original_amount: Decimal,
    pub currency_code: String,
    pub is_calculated_price_price_list: bool,
    pub calculated_price: StoreCalculatedPriceDetail,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreCalculatedPriceDetail {
    pub price_list_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreVariantOption {
    pub option_id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreVariant {
    pub id: String,
    pub product_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub allow_backorder: bool,
    pub manage_inventory: bool,
    /// The storefront disables add-to-cart when a stock-tracked variant has no quantity,
    /// so omitting this made every product read "Out of stock".
    pub inventory_quantity: i32,
    pub options: Vec<StoreVariantOption>,
    /// Absent when the variant has no price set. Deliberately not zero — an unpriced
    /// variant must not read as free.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculated_price: Option<StoreCalculatedPrice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreProductOptionValue {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreProductOption {
    pub id: String,
    pub title: String,
    pub product_id: String,
    pub values: Vec<StoreProductOptionValue>,
}

/// `ProductDTO` plus everything a storefront needs, flattened so the JSON is one object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreProduct {
    #[serde(flatten)]
    pub product: ProductDTO,
    pub variants: Vec<StoreVariant>,
    pub options: Vec<StoreProductOption>,
    /// Always present, even empty: clients call `.map` on it without a guard.
    pub images: Vec<serde_json::Value>,
}

/// Expands products with their variants, options and prices.
///
/// Batched where it counts: one price evaluation covers every variant on the page. The
/// per-product option queries are the remaining N+1 and are bounded by the page limit.
// ponytail: options are fetched per product; fold into a batch query if a large page
// shows up in a flamegraph. Prices — the expensive half — are already batched.
pub async fn expand_products(
    products: Vec<ProductDTO>,
    product_service: &Arc<ProductService>,
    pricing_service: &Arc<PricingService>,
    inventory_service: &Arc<InventoryService>,
    currency_code: &str,
    region_id: Option<String>,
) -> Result<Vec<StoreProduct>> {
    let mut expanded: Vec<(ProductDTO, Vec<StoreVariant>, Vec<StoreProductOption>)> = Vec::new();
    let mut all_variant_ids: Vec<String> = Vec::new();

    for product in products {
        let product_id = product.id.to_string();

        // option_id -> (value_id -> value), used to turn a variant's option-value ids into
        // the {option_id, value} pairs the storefront matches on.
        let mut options = Vec::new();
        let mut value_lookup: HashMap<String, (String, String)> = HashMap::new();
        for (option_id, title) in product_service.list_options(&product_id).await? {
            let values = product_service.list_option_values(&option_id).await?;
            for (value_id, value) in &values {
                value_lookup.insert(value_id.clone(), (option_id.clone(), value.clone()));
            }
            options.push(StoreProductOption {
                id: option_id.clone(),
                title,
                product_id: product_id.clone(),
                values: values
                    .into_iter()
                    .map(|(id, value)| StoreProductOptionValue { id, value })
                    .collect(),
            });
        }

        let mut variants = Vec::new();
        for variant in product_service.list_variants(&product_id).await? {
            let variant_id = variant.id.to_string();
            all_variant_ids.push(variant_id.clone());

            let variant_options = product_service
                .list_variant_option_values(&variant_id)
                .await?
                .into_iter()
                .filter_map(|value_id| {
                    value_lookup
                        .get(&value_id)
                        .map(|(option_id, value)| StoreVariantOption {
                            option_id: option_id.clone(),
                            value: value.clone(),
                        })
                })
                .collect();

            variants.push(StoreVariant {
                id: variant_id,
                product_id: variant.product_id,
                title: variant.title,
                sku: variant.sku,
                barcode: variant.barcode,
                allow_backorder: variant.allow_backorder,
                manage_inventory: variant.manage_inventory,
                options: variant_options,
                inventory_quantity: 0,
                calculated_price: None,
            });
        }

        expanded.push((product, variants, options));
    }

    let prices = pricing_service
        .calculate_for_variants(
            &all_variant_ids,
            &PricingContext {
                currency_code: currency_code.to_string(),
                region_id,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(1),
                as_of: None,
            },
        )
        .await?;

    // A variant with no inventory item is not stock-tracked, so it stays available rather
    // than reading as sold out.
    let availability = inventory_service
        .availability_for_variants(&all_variant_ids)
        .await?;

    Ok(expanded
        .into_iter()
        .map(|(product, mut variants, options)| {
            for variant in &mut variants {
                variant.calculated_price = prices.get(&variant.id).map(to_store_price);
                variant.inventory_quantity = match availability.get(&variant.id) {
                    Some(quantity) => *quantity,
                    None => {
                        variant.manage_inventory = false;
                        0
                    }
                };
            }
            StoreProduct {
                product,
                variants,
                options,
                images: Vec::new(),
            }
        })
        .collect())
}

fn to_store_price(price: &CalculatedPriceDTO) -> StoreCalculatedPrice {
    StoreCalculatedPrice {
        id: price.price_set_id.clone(),
        calculated_amount: price.calculated_amount,
        original_amount: price.original_amount,
        currency_code: price.currency_code.clone(),
        is_calculated_price_price_list: price.price_list_type.is_some(),
        calculated_price: StoreCalculatedPriceDetail {
            price_list_type: price.price_list_type.clone(),
        },
    }
}
