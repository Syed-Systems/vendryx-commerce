//! `stheno-links`: Module links registry (36 cross-module link definitions + link CRUD operations).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub module: &'static str,
    pub entity: &'static str,
    pub field: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CascadeBehaviour {
    Delete,
    Nullify,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDefinition {
    pub name: &'static str,
    pub left: EntityRef,
    pub right: EntityRef,
    /// `Some(table)` for a writable link; `None` for a read-only, traversal-only link.
    pub table: Option<&'static str>,
    pub cardinality: Cardinality,
    pub cascade: CascadeBehaviour,
}

/// The complete 36 cross-module link definitions: 22 writable + 14 read-only
/// (TRD §4.3 / DATA_MODEL.md §3).
pub static ALL_LINKS: &[LinkDefinition] = &[
    // --- Writable links (22) — real tables in the `links` schema. DATA_MODEL.md §3.1 ---
    // 1. product.product_variant -> pricing.price_set
    LinkDefinition {
        name: "product_variant_price_set",
        left: EntityRef {
            module: "product",
            entity: "product_variant",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: Some("link_product_variant_price_set"),
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 2. product.product_variant -> inventory.inventory_item
    LinkDefinition {
        name: "product_variant_inventory_item",
        left: EntityRef {
            module: "product",
            entity: "product_variant",
            field: "id",
        },
        right: EntityRef {
            module: "inventory",
            entity: "inventory_item",
            field: "id",
        },
        table: Some("link_product_variant_inventory_item"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 3. product.product -> sales_channel.sales_channel
    LinkDefinition {
        name: "product_sales_channel",
        left: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: Some("link_product_sales_channel"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 4. product.product -> fulfillment.shipping_profile
    LinkDefinition {
        name: "product_shipping_profile",
        left: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "shipping_profile",
            field: "id",
        },
        table: Some("link_product_shipping_profile"),
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Nullify,
    },
    // 5. fulfillment.shipping_option -> pricing.price_set
    LinkDefinition {
        name: "shipping_option_price_set",
        left: EntityRef {
            module: "fulfillment",
            entity: "shipping_option",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: Some("link_shipping_option_price_set"),
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 6. cart.cart -> payment.payment_collection
    LinkDefinition {
        name: "cart_payment_collection",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: Some("link_cart_payment_collection"),
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 7. cart.cart -> promotion.promotion
    LinkDefinition {
        name: "cart_promotion",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        table: Some("link_cart_promotion"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 8. order.order -> cart.cart
    LinkDefinition {
        name: "order_cart",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        table: Some("link_order_cart"),
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::None,
    },
    // 9. order.order -> payment.payment_collection
    LinkDefinition {
        name: "order_payment_collection",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: Some("link_order_payment_collection"),
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 10. order.claim -> payment.payment_collection
    LinkDefinition {
        name: "order_claim_payment_collection",
        left: EntityRef {
            module: "order",
            entity: "claim",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: Some("link_order_claim_payment_collection"),
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 11. order.exchange -> payment.payment_collection
    LinkDefinition {
        name: "order_exchange_payment_collection",
        left: EntityRef {
            module: "order",
            entity: "exchange",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: Some("link_order_exchange_payment_collection"),
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 12. order.order -> promotion.promotion
    LinkDefinition {
        name: "order_promotion",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        table: Some("link_order_promotion"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::None,
    },
    // 13. order.order -> fulfillment.fulfillment
    LinkDefinition {
        name: "order_fulfillment",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "fulfillment",
            field: "id",
        },
        table: Some("link_order_fulfillment"),
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 14. order.return -> fulfillment.fulfillment
    LinkDefinition {
        name: "order_return_fulfillment",
        left: EntityRef {
            module: "order",
            entity: "return",
            field: "id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "fulfillment",
            field: "id",
        },
        table: Some("link_order_return_fulfillment"),
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 15. customer.customer -> payment.account_holder
    LinkDefinition {
        name: "customer_account_holder",
        left: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "account_holder",
            field: "id",
        },
        table: Some("link_customer_account_holder"),
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 16. region.region -> payment.payment_provider
    LinkDefinition {
        name: "region_payment_provider",
        left: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_provider",
            field: "id",
        },
        table: Some("link_region_payment_provider"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 17. sales_channel.sales_channel -> stock_location.stock_location
    LinkDefinition {
        name: "sales_channel_location",
        left: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: Some("link_sales_channel_location"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 18. fulfillment.fulfillment_set -> stock_location.stock_location
    LinkDefinition {
        name: "fulfillment_set_location",
        left: EntityRef {
            module: "fulfillment",
            entity: "fulfillment_set",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: Some("link_fulfillment_set_location"),
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 19. fulfillment.fulfillment_provider -> stock_location.stock_location
    LinkDefinition {
        name: "fulfillment_provider_location",
        left: EntityRef {
            module: "fulfillment",
            entity: "fulfillment_provider",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: Some("link_fulfillment_provider_location"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 20. api_key.api_key -> sales_channel.sales_channel
    LinkDefinition {
        name: "publishable_api_key_sales_channel",
        left: EntityRef {
            module: "api_key",
            entity: "api_key",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: Some("link_publishable_api_key_sales_channel"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 21. user.user -> rbac.role
    LinkDefinition {
        name: "user_rbac_role",
        left: EntityRef {
            module: "user",
            entity: "user",
            field: "id",
        },
        right: EntityRef {
            module: "rbac",
            entity: "role",
            field: "id",
        },
        table: Some("link_user_rbac_role"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 22. user.invite -> rbac.role
    LinkDefinition {
        name: "invite_rbac_role",
        left: EntityRef {
            module: "user",
            entity: "invite",
            field: "id",
        },
        right: EntityRef {
            module: "rbac",
            entity: "role",
            field: "id",
        },
        table: Some("link_invite_rbac_role"),
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // --- Read-only links (14) — no join table, traversed via ids already on the entity.
    //     DATA_MODEL.md §3.2 ---
    // 1. cart.cart.customer_id -> customer.customer
    LinkDefinition {
        name: "cart_customer",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "customer_id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 2. cart.line_item.product_id -> product.product
    LinkDefinition {
        name: "cart_product",
        left: EntityRef {
            module: "cart",
            entity: "line_item",
            field: "product_id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 3. cart.cart.region_id -> region.region
    LinkDefinition {
        name: "cart_region",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "region_id",
        },
        right: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 4. cart.cart.sales_channel_id -> sales_channel.sales_channel
    LinkDefinition {
        name: "cart_sales_channel",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "sales_channel_id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 5. cart.shipping_method.shipping_option_id -> fulfillment.shipping_option
    LinkDefinition {
        name: "cart_shipping_option",
        left: EntityRef {
            module: "cart",
            entity: "shipping_method",
            field: "shipping_option_id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "shipping_option",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 6. order.order.customer_id -> customer.customer
    LinkDefinition {
        name: "order_customer",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "customer_id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 7. order.line_item.product_id -> product.product
    LinkDefinition {
        name: "order_product",
        left: EntityRef {
            module: "order",
            entity: "line_item",
            field: "product_id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 8. order.order.region_id -> region.region
    LinkDefinition {
        name: "order_region",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "region_id",
        },
        right: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 9. order.order.sales_channel_id -> sales_channel.sales_channel
    LinkDefinition {
        name: "order_sales_channel",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "sales_channel_id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 10. inventory.inventory_level.location_id -> stock_location.stock_location
    LinkDefinition {
        name: "inventory_level_stock_location",
        left: EntityRef {
            module: "inventory",
            entity: "inventory_level",
            field: "location_id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 11. cart.line_item_adjustment.promotion_id -> promotion.promotion
    LinkDefinition {
        name: "line_item_adjustment_promotion",
        left: EntityRef {
            module: "cart",
            entity: "line_item_adjustment",
            field: "promotion_id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 12. product.product.id -> translation.translation
    LinkDefinition {
        name: "product_translation",
        left: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        right: EntityRef {
            module: "translation",
            entity: "translation",
            field: "entity_id",
        },
        table: None,
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::None,
    },
    // 13. store.store_currency.currency_code -> currency.currency
    LinkDefinition {
        name: "store_currency",
        left: EntityRef {
            module: "store",
            entity: "store_currency",
            field: "currency_code",
        },
        right: EntityRef {
            module: "currency",
            entity: "currency",
            field: "code",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 14. store.store_locale.locale_code -> translation.locale
    LinkDefinition {
        name: "store_locale",
        left: EntityRef {
            module: "store",
            entity: "store_locale",
            field: "locale_code",
        },
        right: EntityRef {
            module: "translation",
            entity: "locale",
            field: "code",
        },
        table: None,
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
];

pub static STANDARD_LINKS: &[LinkDefinition] = ALL_LINKS;

type LinkIndex = HashMap<String, Vec<String>>;
type LinkTableMap = Arc<RwLock<HashMap<String, LinkIndex>>>;

/// In-memory link store for fast bidirectional resolution and testing.
#[derive(Clone, Default)]
pub struct LinkStore {
    // link_name -> (left_id -> Vec<right_id>)
    forward: LinkTableMap,
    // link_name -> (right_id -> Vec<left_id>)
    reverse: LinkTableMap,
}

impl LinkStore {
    pub fn new() -> Self {
        Self {
            forward: Arc::new(RwLock::new(HashMap::new())),
            reverse: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn link(&self, link_name: &str, left_id: impl Into<String>, right_id: impl Into<String>) {
        let left = left_id.into();
        let right = right_id.into();

        let mut fwd = self.forward.write().unwrap_or_else(|e| e.into_inner());
        fwd.entry(link_name.to_string())
            .or_default()
            .entry(left.clone())
            .or_default()
            .push(right.clone());

        let mut rev = self.reverse.write().unwrap_or_else(|e| e.into_inner());
        rev.entry(link_name.to_string())
            .or_default()
            .entry(right)
            .or_default()
            .push(left);
    }

    pub fn get_by_left(&self, link_name: &str, left_id: &str) -> Vec<String> {
        let fwd = self.forward.read().unwrap_or_else(|e| e.into_inner());
        fwd.get(link_name)
            .and_then(|m| m.get(left_id))
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_by_right(&self, link_name: &str, right_id: &str) -> Vec<String> {
        let rev = self.reverse.read().unwrap_or_else(|e| e.into_inner());
        rev.get(link_name)
            .and_then(|m| m.get(right_id))
            .cloned()
            .unwrap_or_default()
    }

    pub fn cascade_delete_left(&self, link_name: &str, left_id: &str) -> Vec<String> {
        let mut fwd = self.forward.write().unwrap_or_else(|e| e.into_inner());
        let mut rev = self.reverse.write().unwrap_or_else(|e| e.into_inner());

        let rights = fwd
            .get_mut(link_name)
            .and_then(|m| m.remove(left_id))
            .unwrap_or_default();

        if let Some(rev_map) = rev.get_mut(link_name) {
            for r in &rights {
                if let Some(lefts) = rev_map.get_mut(r) {
                    lefts.retain(|l| l != left_id);
                }
            }
        }

        rights
    }

    pub fn unlink(&self, link_name: &str, left_id: &str, right_id: &str) {
        let mut fwd = self.forward.write().unwrap_or_else(|e| e.into_inner());
        if let Some(map) = fwd.get_mut(link_name) {
            if let Some(list) = map.get_mut(left_id) {
                list.retain(|r| r != right_id);
            }
        }

        let mut rev = self.reverse.write().unwrap_or_else(|e| e.into_inner());
        if let Some(map) = rev.get_mut(link_name) {
            if let Some(list) = map.get_mut(right_id) {
                list.retain(|l| l != left_id);
            }
        }
    }
}

use diesel::sql_types::Text;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use stheno_db::tenant;
use stheno_db::DbPool;
use stheno_types::{Error, Result};

#[derive(QueryableByName)]
struct StringIdRow {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(QueryableByName)]
struct LinkPairRow {
    #[diesel(sql_type = Text)]
    left_id: String,
    #[diesel(sql_type = Text)]
    right_id: String,
}

pub fn find_link(name: &str) -> Option<&'static LinkDefinition> {
    ALL_LINKS.iter().find(|l| l.name == name)
}

/// Postgres-backed link store for production link table operations (P14-18, P14-20, P14-21).
#[derive(Clone)]
pub struct PgLinkStore {
    pool: DbPool,
}

impl PgLinkStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn link(&self, link_name: &str, left_id: &str, right_id: &str) -> Result<()> {
        let link_name = link_name.to_string();
        let left_id = left_id.to_string();
        let right_id = right_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!(
                        "Link '{link_name}' is read-only and has no writable join table"
                    ),
                    field: Some(link_name.to_string()),
                })?;

                let id = ulid::Ulid::new().to_string();
                let sql = format!(
                    r#"
                    INSERT INTO links.{table} (id, left_id, right_id, created_at, updated_at, deleted_at)
                    VALUES ($1, $2, $3, now(), now(), NULL)
                    ON CONFLICT (left_id, right_id) WHERE deleted_at IS NULL
                    DO UPDATE SET updated_at = now()
                    "#
                );

                diesel::sql_query(sql)
                    .bind::<Text, _>(&id)
                    .bind::<Text, _>(&left_id)
                    .bind::<Text, _>(&right_id)
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())
            })
        })
        .await
    }

    pub async fn get_by_left(&self, link_name: &str, left_id: &str) -> Result<Vec<String>> {
        let link_name = link_name.to_string();
        let left_id = left_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "SELECT right_id as value FROM links.{table} WHERE left_id = $1 AND deleted_at IS NULL"
                );


                let rows: Vec<StringIdRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&left_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(rows.into_iter().map(|r| r.value).collect())

            })
        })
        .await
    }

    /// `get_by_left` for many left ids at once, returning left -> right ids.
    ///
    /// Exists because the catalog resolves a price for every variant on the page. Looping
    /// `get_by_left` there is one round trip per variant — twenty products with three
    /// variants each is sixty queries to render one list — which the catalog's p95 budget
    /// does not have room for.
    pub async fn get_by_left_many(
        &self,
        link_name: &str,
        left_ids: &[String],
    ) -> Result<std::collections::HashMap<String, Vec<String>>> {
        if left_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let link_name = link_name.to_string();
        let left_ids = left_ids.to_vec();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "SELECT left_id, right_id FROM links.{table}                      WHERE left_id = ANY($1) AND deleted_at IS NULL"
                );

                let rows: Vec<LinkPairRow> = diesel::sql_query(sql)
                    .bind::<diesel::sql_types::Array<Text>, _>(&left_ids)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                let mut out: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                for row in rows {
                    out.entry(row.left_id).or_default().push(row.right_id);
                }
                Ok(out)
            })
        })
        .await
    }

    pub async fn get_by_right(&self, link_name: &str, right_id: &str) -> Result<Vec<String>> {
        let link_name = link_name.to_string();
        let right_id = right_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "SELECT left_id as value FROM links.{table} WHERE right_id = $1 AND deleted_at IS NULL"
                );


                let rows: Vec<StringIdRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&right_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(rows.into_iter().map(|r| r.value).collect())

            })
        })
        .await
    }

    pub async fn unlink(&self, link_name: &str, left_id: &str, right_id: &str) -> Result<()> {
        let link_name = link_name.to_string();
        let left_id = left_id.to_string();
        let right_id = right_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "UPDATE links.{table} SET deleted_at = now() WHERE left_id = $1 AND right_id = $2 AND deleted_at IS NULL"
                );


                diesel::sql_query(sql)
                    .bind::<Text, _>(&left_id)
                    .bind::<Text, _>(&right_id)
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())

            })
        })
        .await
    }

    pub async fn cascade_delete_left(&self, link_name: &str, left_id: &str) -> Result<Vec<String>> {
        let link_name = link_name.to_string();
        let left_id = left_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "UPDATE links.{table} SET deleted_at = now() WHERE left_id = $1 AND deleted_at IS NULL RETURNING right_id as value"
                );


                let rows: Vec<StringIdRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&left_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(rows.into_iter().map(|r| r.value).collect())

            })
        })
        .await
    }

    pub async fn cascade_delete_right(
        &self,
        link_name: &str,
        right_id: &str,
    ) -> Result<Vec<String>> {
        let link_name = link_name.to_string();
        let right_id = right_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let def = find_link(&link_name).ok_or_else(|| Error::NotFound {
                    entity: "link_definition",
                    id: link_name.to_string(),
                })?;

                let table = def.table.ok_or_else(|| Error::Invalid {
                    code: "read_only_link",
                    message: format!("Link '{link_name}' is read-only and has no join table"),
                    field: Some(link_name.to_string()),
                })?;

                let sql = format!(
                    "UPDATE links.{table} SET deleted_at = now() WHERE right_id = $1 AND deleted_at IS NULL RETURNING left_id as value"
                );


                let rows: Vec<StringIdRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&right_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(rows.into_iter().map(|r| r.value).collect())

            })
        })
        .await
    }

    /// Hard delete / GDPR erasure cascade (P14-21).
    pub async fn gdpr_erase_entity(&self, entity_id: &str) -> Result<usize> {
        let entity_id = entity_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let mut total_deleted = 0;

                for def in ALL_LINKS {
                    if let Some(table) = def.table {
                        let sql = format!(
                            "DELETE FROM links.{table} WHERE left_id = $1 OR right_id = $1"
                        );
                        let count = diesel::sql_query(sql)
                            .bind::<Text, _>(&entity_id)
                            .execute(conn)
                            .await
                            .map_err(|e| Error::Database(e.to_string()))?;
                        total_deleted += count;
                    }
                }

                Ok(total_deleted)
            })
        })
        .await
    }
}
