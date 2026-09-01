//! Service layer for `stheno-cart`, backed by PostgreSQL (P13-06, P13-07).

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_db::{CrudEntity, CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Id, Result};
use ulid::Ulid;

use crate::dto::*;
use crate::models::{
    Address, Cart, CreditLine, LineItem, LineItemAdjustment, LineItemTaxLine, ShippingMethod,
    ShippingMethodAdjustment, ShippingMethodTaxLine,
};

#[derive(Clone)]
pub struct CartService {
    carts: PgCrudRepository<Cart>,
    addresses: PgCrudRepository<Address>,
    items: PgCrudRepository<LineItem>,
    item_adjustments: PgCrudRepository<LineItemAdjustment>,
    item_tax_lines: PgCrudRepository<LineItemTaxLine>,
    shipping_methods: PgCrudRepository<ShippingMethod>,
    sm_adjustments: PgCrudRepository<ShippingMethodAdjustment>,
    sm_tax_lines: PgCrudRepository<ShippingMethodTaxLine>,
    credit_lines: PgCrudRepository<CreditLine>,
    creation_timestamps: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl CartService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            carts: PgCrudRepository::new(pool.clone()),
            addresses: PgCrudRepository::new(pool.clone()),
            items: PgCrudRepository::new(pool.clone()),
            item_adjustments: PgCrudRepository::new(pool.clone()),
            item_tax_lines: PgCrudRepository::new(pool.clone()),
            shipping_methods: PgCrudRepository::new(pool.clone()),
            sm_adjustments: PgCrudRepository::new(pool.clone()),
            sm_tax_lines: PgCrudRepository::new(pool.clone()),
            credit_lines: PgCrudRepository::new(pool),
            creation_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ── Cart CRUD ────────────────────────────────────────────────────────────

    pub async fn create_cart(&self, input: CreateCartInput) -> Result<CartDTO> {
        let id: Id<Cart> = Id::new();
        let cart_secret = format!("csec_{}", Ulid::new().to_string().to_lowercase());

        let mut shipping_address_id = None;
        if let Some(addr_input) = input.shipping_address {
            let addr = self.create_address_internal(addr_input).await?;
            shipping_address_id = Some(addr.id.to_string());
        }

        let mut billing_address_id = None;
        if let Some(addr_input) = input.billing_address {
            let addr = self.create_address_internal(addr_input).await?;
            billing_address_id = Some(addr.id.to_string());
        }

        let now = Utc::now();
        let cart = Cart {
            id,
            region_id: input.region_id,
            customer_id: input.customer_id,
            sales_channel_id: input.sales_channel_id,
            email: input.email,
            currency_code: input.currency_code.to_lowercase(),
            shipping_address_id,
            billing_address_id,
            cart_secret,
            metadata: input.metadata,
            completed_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created_cart = self.carts.create(None, cart).await?;
        let cart_id_str = created_cart.id.to_string();

        if let Some(items) = input.items {
            for item_input in items {
                self.add_line_item(&cart_id_str, item_input).await?;
            }
        }

        self.get_cart(&cart_id_str).await
    }

    pub async fn get_cart(&self, id: &str) -> Result<CartDTO> {
        let cart = self.carts.retrieve(None, id).await.map_err(|e| match e {
            Error::NotFound { .. } => Error::NotFound {
                entity: "cart",
                id: id.to_string(),
            },
            other => other,
        })?;

        self.hydrate_cart_dto(&cart).await
    }

    pub async fn get_cart_with_secret(&self, id: &str, secret: &str) -> Result<CartDTO> {
        let cart = self.get_cart(id).await?;
        if cart.cart_secret != secret {
            return Err(Error::Unauthorized("Invalid cart secret".to_string()));
        }
        Ok(cart)
    }

    pub async fn update_cart(&self, id: &str, input: UpdateCartInput) -> Result<CartDTO> {
        let mut cart = self.carts.retrieve(None, id).await.map_err(|e| match e {
            Error::NotFound { .. } => Error::NotFound {
                entity: "cart",
                id: id.to_string(),
            },
            other => other,
        })?;

        if let Some(region_id) = input.region_id {
            cart.region_id = Some(region_id);
        }
        if let Some(customer_id) = input.customer_id {
            cart.customer_id = Some(customer_id);
        }
        if let Some(sales_channel_id) = input.sales_channel_id {
            cart.sales_channel_id = Some(sales_channel_id);
        }
        if let Some(email) = input.email {
            cart.email = Some(email);
        }
        if let Some(currency_code) = input.currency_code {
            cart.currency_code = currency_code.to_lowercase();
        }
        if let Some(shipping_address_id) = input.shipping_address_id {
            cart.shipping_address_id = Some(shipping_address_id);
        }
        if let Some(billing_address_id) = input.billing_address_id {
            cart.billing_address_id = Some(billing_address_id);
        }
        if let Some(completed_at) = input.completed_at {
            cart.completed_at = Some(completed_at);
        }
        if let Some(metadata) = input.metadata {
            cart.metadata = Some(metadata);
        }

        if let Some(shipping_addr) = input.shipping_address {
            let addr = self.create_address_internal(shipping_addr).await?;
            cart.shipping_address_id = Some(addr.id.to_string());
        }

        if let Some(billing_addr) = input.billing_address {
            let addr = self.create_address_internal(billing_addr).await?;
            cart.billing_address_id = Some(addr.id.to_string());
        }

        cart.updated_at = Utc::now();
        let updated = self.carts.update(None, id, cart).await?;
        self.hydrate_cart_dto(&updated).await
    }

    pub async fn delete_cart(&self, id: &str) -> Result<()> {
        self.carts.soft_delete(None, id).await.map(|_| ())
    }

    pub async fn complete_cart(&self, id: &str) -> Result<CartDTO> {
        let mut cart = self.carts.retrieve(None, id).await.map_err(|e| match e {
            Error::NotFound { .. } => Error::NotFound {
                entity: "cart",
                id: id.to_string(),
            },
            other => other,
        })?;

        cart.completed_at = Some(Utc::now());
        cart.updated_at = Utc::now();
        let updated = self.carts.update(None, id, cart).await?;
        self.hydrate_cart_dto(&updated).await
    }

    // ── Cart Ownership & Claim on Login (P4-06, F6) ──────────────────────────

    pub async fn claim_cart_on_login(
        &self,
        cart_id: &str,
        customer_id: &str,
        cart_secret: &str,
    ) -> Result<CartDTO> {
        let cart = self.get_cart(cart_id).await?;
        if cart.cart_secret != cart_secret {
            return Err(Error::Unauthorized(
                "Cart secret mismatch: cannot claim cart".to_string(),
            ));
        }

        // If cart is already owned by a different customer, fail
        if let Some(existing_customer) = &cart.customer_id {
            if existing_customer != customer_id {
                return Err(Error::Conflict {
                    code: "cart_claimed",
                    message: "Cart already claimed by another customer".to_string(),
                });
            }
        }

        self.update_cart(
            cart_id,
            UpdateCartInput {
                customer_id: Some(customer_id.to_string()),
                ..Default::default()
            },
        )
        .await
    }

    // ── Rate Limiting & TTL (P4-07, F22) ──────────────────────────────────────

    pub fn check_creation_rate_limit(&self, ip: &str, limit_per_minute: usize) -> bool {
        let mut map = self.creation_timestamps.write().unwrap();
        let now = Utc::now();
        let one_minute_ago = now - Duration::minutes(1);

        let timestamps = map.entry(ip.to_string()).or_default();
        timestamps.retain(|t| *t > one_minute_ago);

        if timestamps.len() >= limit_per_minute {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub async fn cleanup_expired_carts(&self, ttl_days: i64) -> Result<usize> {
        let threshold = Utc::now() - Duration::days(ttl_days);
        let spec = FilterSpec::new()
            .with_predicate("completed_at", FilterOp::IsNull)
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_predicate("created_at", FilterOp::Lt(threshold.to_rfc3339()))
            .with_pagination(0, 1000);

        let expired = self.carts.list(None, &spec).await?;
        let count = expired.len();
        for cart in expired {
            let _ = self.carts.soft_delete(None, &cart.id()).await;
        }
        Ok(count)
    }

    // ── Line Items (P4-04) ───────────────────────────────────────────────────

    pub async fn add_line_item(
        &self,
        cart_id: &str,
        input: CreateLineItemInput,
    ) -> Result<LineItemDTO> {
        // Ensure cart exists
        let _ = self
            .carts
            .retrieve(None, cart_id)
            .await
            .map_err(|_| Error::NotFound {
                entity: "cart",
                id: cart_id.to_string(),
            })?;

        let id: Id<LineItem> = Id::new();
        let now = Utc::now();
        let item = LineItem {
            id,
            cart_id: cart_id.to_string(),
            title: input.title,
            subtitle: input.subtitle,
            thumbnail: input.thumbnail,
            quantity: input.quantity,
            variant_id: input.variant_id,
            product_id: input.product_id,
            product_title: input.product_title,
            product_description: input.product_description,
            product_subtitle: input.product_subtitle,
            product_type: input.product_type,
            product_collection: input.product_collection,
            product_handle: input.product_handle,
            variant_sku: input.variant_sku,
            variant_barcode: input.variant_barcode,
            variant_title: input.variant_title,
            variant_option_values: input.variant_option_values,
            requires_shipping: input.requires_shipping.unwrap_or(true),
            is_discountable: input.is_discountable.unwrap_or(true),
            is_tax_inclusive: input.is_tax_inclusive.unwrap_or(false),
            is_custom_price: input.is_custom_price.unwrap_or(false),
            unit_price: input.unit_price,
            compare_at_unit_price: input.compare_at_unit_price,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.items.create(None, item).await?;
        self.get_line_item(&created.id.to_string()).await
    }

    pub async fn get_line_item(&self, item_id: &str) -> Result<LineItemDTO> {
        let item = self
            .items
            .retrieve(None, item_id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "line_item",
                    id: item_id.to_string(),
                },
                other => other,
            })?;

        let adjustments = self.get_line_item_adjustments(&item.id.to_string()).await?;
        let tax_lines = self.get_line_item_tax_lines(&item.id.to_string()).await?;

        let mut dto = LineItemDTO {
            id: item.id.to_string(),
            cart_id: item.cart_id,
            title: item.title,
            subtitle: item.subtitle,
            thumbnail: item.thumbnail,
            quantity: item.quantity,
            variant_id: item.variant_id,
            product_id: item.product_id,
            product_title: item.product_title,
            product_description: item.product_description,
            product_subtitle: item.product_subtitle,
            product_type: item.product_type,
            product_collection: item.product_collection,
            product_handle: item.product_handle,
            variant_sku: item.variant_sku,
            variant_barcode: item.variant_barcode,
            variant_title: item.variant_title,
            variant_option_values: item.variant_option_values,
            requires_shipping: item.requires_shipping,
            is_discountable: item.is_discountable,
            is_tax_inclusive: item.is_tax_inclusive,
            is_custom_price: item.is_custom_price,
            unit_price: item.unit_price,
            compare_at_unit_price: item.compare_at_unit_price,
            adjustments,
            tax_lines,
            totals: None,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        };

        dto.totals = Some(calculate_line_item_totals(&dto, 2));
        Ok(dto)
    }

    pub async fn update_line_item(
        &self,
        _cart_id: &str,
        item_id: &str,
        input: UpdateLineItemInput,
    ) -> Result<LineItemDTO> {
        let mut item = self
            .items
            .retrieve(None, item_id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "line_item",
                    id: item_id.to_string(),
                },
                other => other,
            })?;

        if let Some(title) = input.title {
            item.title = title;
        }
        if let Some(qty) = input.quantity {
            item.quantity = qty;
        }
        if let Some(unit_price) = input.unit_price {
            item.unit_price = unit_price;
        }
        if let Some(compare_at) = input.compare_at_unit_price {
            item.compare_at_unit_price = Some(compare_at);
        }
        if let Some(metadata) = input.metadata {
            item.metadata = Some(metadata);
        }
        item.updated_at = Utc::now();

        let updated = self.items.update(None, item_id, item).await?;
        self.get_line_item(&updated.id.to_string()).await
    }

    pub async fn delete_line_item(&self, _cart_id: &str, item_id: &str) -> Result<()> {
        self.items.soft_delete(None, item_id).await.map(|_| ())
    }

    // ── Line Item Adjustments & Tax Lines ─────────────────────────────────────

    pub async fn add_line_item_adjustment(
        &self,
        item_id: &str,
        input: CreateLineItemAdjustmentInput,
    ) -> Result<LineItemAdjustmentDTO> {
        let adj = LineItemAdjustment {
            id: Id::new(),
            item_id: item_id.to_string(),
            amount: input.amount,
            promotion_id: input.promotion_id,
            code: input.code,
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.item_adjustments.create(None, adj).await?;

        Ok(LineItemAdjustmentDTO {
            id: created.id.to_string(),
            item_id: created.item_id,
            amount: created.amount,
            promotion_id: created.promotion_id,
            code: created.code,
            description: created.description,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn delete_line_item_adjustments(&self, item_id: &str) -> Result<()> {
        let spec = FilterSpec::new()
            .with_predicate("item_id", FilterOp::Eq(item_id.to_string()))
            .with_pagination(0, 100);
        let adjs = self.item_adjustments.list(None, &spec).await?;
        for a in adjs {
            let _ = self.item_adjustments.delete(None, &a.id()).await;
        }
        Ok(())
    }

    pub async fn add_line_item_tax_line(
        &self,
        item_id: &str,
        input: CreateLineItemTaxLineInput,
    ) -> Result<LineItemTaxLineDTO> {
        let tax = LineItemTaxLine {
            id: Id::new(),
            item_id: item_id.to_string(),
            rate: input.rate,
            code: input.code,
            tax_rate_id: input.tax_rate_id,
            provider_id: input.provider_id,
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.item_tax_lines.create(None, tax).await?;

        Ok(LineItemTaxLineDTO {
            id: created.id.to_string(),
            item_id: created.item_id,
            rate: created.rate,
            code: created.code,
            tax_rate_id: created.tax_rate_id,
            provider_id: created.provider_id,
            description: created.description,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    // ── Shipping Methods ─────────────────────────────────────────────────────

    pub async fn add_shipping_method(
        &self,
        cart_id: &str,
        input: CreateShippingMethodInput,
    ) -> Result<ShippingMethodDTO> {
        let sm = ShippingMethod {
            id: Id::new(),
            cart_id: cart_id.to_string(),
            name: input.name,
            amount: input.amount,
            is_tax_inclusive: input.is_tax_inclusive.unwrap_or(false),
            shipping_option_id: input.shipping_option_id,
            data: input.data,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.shipping_methods.create(None, sm).await?;
        self.get_shipping_method(&created.id.to_string()).await
    }

    pub async fn get_shipping_method(&self, sm_id: &str) -> Result<ShippingMethodDTO> {
        let sm = self
            .shipping_methods
            .retrieve(None, sm_id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "shipping_method",
                    id: sm_id.to_string(),
                },
                other => other,
            })?;

        let adjustments = self
            .get_shipping_method_adjustments(&sm.id.to_string())
            .await?;
        let tax_lines = self
            .get_shipping_method_tax_lines(&sm.id.to_string())
            .await?;

        let mut dto = ShippingMethodDTO {
            id: sm.id.to_string(),
            cart_id: sm.cart_id,
            name: sm.name,
            amount: sm.amount,
            is_tax_inclusive: sm.is_tax_inclusive,
            shipping_option_id: sm.shipping_option_id,
            data: sm.data,
            adjustments,
            tax_lines,
            totals: None,
            metadata: sm.metadata,
            created_at: sm.created_at,
            updated_at: sm.updated_at,
        };

        dto.totals = Some(calculate_shipping_method_totals(&dto, 2));
        Ok(dto)
    }

    pub async fn add_shipping_method_adjustment(
        &self,
        shipping_method_id: &str,
        input: CreateShippingMethodAdjustmentInput,
    ) -> Result<ShippingMethodAdjustmentDTO> {
        let adj = ShippingMethodAdjustment {
            id: Id::new(),
            shipping_method_id: shipping_method_id.to_string(),
            amount: input.amount,
            promotion_id: input.promotion_id,
            code: input.code,
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.sm_adjustments.create(None, adj).await?;

        Ok(ShippingMethodAdjustmentDTO {
            id: created.id.to_string(),
            shipping_method_id: created.shipping_method_id,
            amount: created.amount,
            promotion_id: created.promotion_id,
            code: created.code,
            description: created.description,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn delete_shipping_method_adjustments(&self, shipping_method_id: &str) -> Result<()> {
        let spec = FilterSpec::new()
            .with_predicate(
                "shipping_method_id",
                FilterOp::Eq(shipping_method_id.to_string()),
            )
            .with_pagination(0, 100);
        let adjs = self.sm_adjustments.list(None, &spec).await?;
        for a in adjs {
            let _ = self.sm_adjustments.delete(None, &a.id()).await;
        }
        Ok(())
    }

    pub async fn add_shipping_method_tax_line(
        &self,
        shipping_method_id: &str,
        input: CreateShippingMethodTaxLineInput,
    ) -> Result<ShippingMethodTaxLineDTO> {
        let tax = ShippingMethodTaxLine {
            id: Id::new(),
            shipping_method_id: shipping_method_id.to_string(),
            rate: input.rate,
            code: input.code,
            tax_rate_id: input.tax_rate_id,
            provider_id: input.provider_id,
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.sm_tax_lines.create(None, tax).await?;

        Ok(ShippingMethodTaxLineDTO {
            id: created.id.to_string(),
            shipping_method_id: created.shipping_method_id,
            rate: created.rate,
            code: created.code,
            tax_rate_id: created.tax_rate_id,
            provider_id: created.provider_id,
            description: created.description,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    // ── Credit Lines (P4-05) ─────────────────────────────────────────────────

    pub async fn add_credit_line(
        &self,
        cart_id: &str,
        input: CreateCreditLineInput,
    ) -> Result<CreditLineDTO> {
        // Ensure cart exists
        let _ = self
            .carts
            .retrieve(None, cart_id)
            .await
            .map_err(|_| Error::NotFound {
                entity: "cart",
                id: cart_id.to_string(),
            })?;

        let cl = CreditLine {
            id: Id::new(),
            cart_id: cart_id.to_string(),
            amount: input.amount,
            reference: input.reference,
            reference_id: input.reference_id,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.credit_lines.create(None, cl).await?;

        Ok(CreditLineDTO {
            id: created.id.to_string(),
            cart_id: created.cart_id,
            amount: created.amount,
            reference: created.reference,
            reference_id: created.reference_id,
            metadata: created.metadata,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn delete_credit_line(&self, _cart_id: &str, id: &str) -> Result<()> {
        self.credit_lines.delete(None, id).await.map(|_| ())
    }

    // ── Internal Helpers ─────────────────────────────────────────────────────

    async fn create_address_internal(&self, input: CreateAddressInput) -> Result<Address> {
        let addr = Address {
            id: Id::new(),
            customer_id: input.customer_id,
            company: input.company,
            first_name: input.first_name,
            last_name: input.last_name,
            address_1: input.address_1,
            address_2: input.address_2,
            city: input.city,
            country_code: input.country_code.map(|c| c.to_lowercase()),
            province: input.province,
            postal_code: input.postal_code,
            phone: input.phone,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        self.addresses.create(None, addr).await
    }

    async fn get_address_dto(&self, id: &str) -> Option<AddressDTO> {
        self.addresses
            .retrieve(None, id)
            .await
            .ok()
            .map(|a| AddressDTO {
                id: a.id.to_string(),
                customer_id: a.customer_id,
                company: a.company,
                first_name: a.first_name,
                last_name: a.last_name,
                address_1: a.address_1,
                address_2: a.address_2,
                city: a.city,
                country_code: a.country_code,
                province: a.province,
                postal_code: a.postal_code,
                phone: a.phone,
                metadata: a.metadata,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
    }

    async fn get_line_item_adjustments(&self, item_id: &str) -> Result<Vec<LineItemAdjustmentDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("item_id", FilterOp::Eq(item_id.to_string()))
            .with_pagination(0, 100);
        let list = self.item_adjustments.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|a| LineItemAdjustmentDTO {
                id: a.id.to_string(),
                item_id: a.item_id,
                amount: a.amount,
                promotion_id: a.promotion_id,
                code: a.code,
                description: a.description,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
            .collect())
    }

    async fn get_line_item_tax_lines(&self, item_id: &str) -> Result<Vec<LineItemTaxLineDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("item_id", FilterOp::Eq(item_id.to_string()))
            .with_pagination(0, 100);
        let list = self.item_tax_lines.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|t| LineItemTaxLineDTO {
                id: t.id.to_string(),
                item_id: t.item_id,
                rate: t.rate,
                code: t.code,
                tax_rate_id: t.tax_rate_id,
                provider_id: t.provider_id,
                description: t.description,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    async fn get_shipping_method_adjustments(
        &self,
        sm_id: &str,
    ) -> Result<Vec<ShippingMethodAdjustmentDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("shipping_method_id", FilterOp::Eq(sm_id.to_string()))
            .with_pagination(0, 100);
        let list = self.sm_adjustments.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|a| ShippingMethodAdjustmentDTO {
                id: a.id.to_string(),
                shipping_method_id: a.shipping_method_id,
                amount: a.amount,
                promotion_id: a.promotion_id,
                code: a.code,
                description: a.description,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
            .collect())
    }

    async fn get_shipping_method_tax_lines(
        &self,
        sm_id: &str,
    ) -> Result<Vec<ShippingMethodTaxLineDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("shipping_method_id", FilterOp::Eq(sm_id.to_string()))
            .with_pagination(0, 100);
        let list = self.sm_tax_lines.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|t| ShippingMethodTaxLineDTO {
                id: t.id.to_string(),
                shipping_method_id: t.shipping_method_id,
                rate: t.rate,
                code: t.code,
                tax_rate_id: t.tax_rate_id,
                provider_id: t.provider_id,
                description: t.description,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    async fn hydrate_cart_dto(&self, cart: &Cart) -> Result<CartDTO> {
        let cart_id_str = cart.id.to_string();

        let shipping_address = match &cart.shipping_address_id {
            Some(id) => self.get_address_dto(id).await,
            None => None,
        };
        let billing_address = match &cart.billing_address_id {
            Some(id) => self.get_address_dto(id).await,
            None => None,
        };

        let items_spec = FilterSpec::new()
            .with_predicate("cart_id", FilterOp::Eq(cart_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 500);
        let items_entities = self.items.list(None, &items_spec).await?;

        let mut items = Vec::new();
        for item in items_entities {
            let adjustments = self.get_line_item_adjustments(&item.id.to_string()).await?;
            let tax_lines = self.get_line_item_tax_lines(&item.id.to_string()).await?;
            let mut dto = LineItemDTO {
                id: item.id.to_string(),
                cart_id: item.cart_id.clone(),
                title: item.title.clone(),
                subtitle: item.subtitle.clone(),
                thumbnail: item.thumbnail.clone(),
                quantity: item.quantity,
                variant_id: item.variant_id.clone(),
                product_id: item.product_id.clone(),
                product_title: item.product_title.clone(),
                product_description: item.product_description.clone(),
                product_subtitle: item.product_subtitle.clone(),
                product_type: item.product_type.clone(),
                product_collection: item.product_collection.clone(),
                product_handle: item.product_handle.clone(),
                variant_sku: item.variant_sku.clone(),
                variant_barcode: item.variant_barcode.clone(),
                variant_title: item.variant_title.clone(),
                variant_option_values: item.variant_option_values.clone(),
                requires_shipping: item.requires_shipping,
                is_discountable: item.is_discountable,
                is_tax_inclusive: item.is_tax_inclusive,
                is_custom_price: item.is_custom_price,
                unit_price: item.unit_price,
                compare_at_unit_price: item.compare_at_unit_price,
                adjustments,
                tax_lines,
                totals: None,
                metadata: item.metadata.clone(),
                created_at: item.created_at,
                updated_at: item.updated_at,
            };
            dto.totals = Some(calculate_line_item_totals(&dto, 2));
            items.push(dto);
        }

        let sm_spec = FilterSpec::new()
            .with_predicate("cart_id", FilterOp::Eq(cart_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let sm_entities = self.shipping_methods.list(None, &sm_spec).await?;

        let mut shipping_methods = Vec::new();
        for sm in sm_entities {
            let adjustments = self
                .get_shipping_method_adjustments(&sm.id.to_string())
                .await?;
            let tax_lines = self
                .get_shipping_method_tax_lines(&sm.id.to_string())
                .await?;
            let mut dto = ShippingMethodDTO {
                id: sm.id.to_string(),
                cart_id: sm.cart_id.clone(),
                name: sm.name.clone(),
                amount: sm.amount,
                is_tax_inclusive: sm.is_tax_inclusive,
                shipping_option_id: sm.shipping_option_id.clone(),
                data: sm.data.clone(),
                adjustments,
                tax_lines,
                totals: None,
                metadata: sm.metadata.clone(),
                created_at: sm.created_at,
                updated_at: sm.updated_at,
            };
            dto.totals = Some(calculate_shipping_method_totals(&dto, 2));
            shipping_methods.push(dto);
        }

        let cl_spec = FilterSpec::new()
            .with_predicate("cart_id", FilterOp::Eq(cart_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let cl_entities = self.credit_lines.list(None, &cl_spec).await?;

        let credit_lines: Vec<CreditLineDTO> = cl_entities
            .into_iter()
            .map(|c| CreditLineDTO {
                id: c.id.to_string(),
                cart_id: c.cart_id,
                amount: c.amount,
                reference: c.reference,
                reference_id: c.reference_id,
                metadata: c.metadata,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();

        let totals = calculate_cart_totals(&items, &shipping_methods, &credit_lines, 2);

        Ok(CartDTO {
            id: cart.id.to_string(),
            region_id: cart.region_id.clone(),
            customer_id: cart.customer_id.clone(),
            sales_channel_id: cart.sales_channel_id.clone(),
            email: cart.email.clone(),
            currency_code: cart.currency_code.clone(),
            shipping_address,
            billing_address,
            items,
            shipping_methods,
            credit_lines,
            totals,
            cart_secret: cart.cart_secret.clone(),
            completed_at: cart.completed_at,
            created_at: cart.created_at,
            updated_at: cart.updated_at,
            metadata: cart.metadata.clone(),
        })
    }

    pub async fn purge_expired_guest_carts(&self, cutoff: DateTime<Utc>) -> Result<usize> {
        let spec = FilterSpec::new()
            .with_predicate("customer_id", FilterOp::IsNull)
            .with_predicate("completed_at", FilterOp::IsNull)
            .with_predicate("updated_at", FilterOp::Lt(cutoff.to_rfc3339()))
            .with_pagination(0, 1000);

        let expired = self.carts.list(None, &spec).await?;
        let count = expired.len();
        for cart in expired {
            let _ = self.carts.delete(None, &cart.id()).await;
        }
        Ok(count)
    }
}
