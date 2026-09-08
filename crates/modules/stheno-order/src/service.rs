//! Order service, order-change integration, versioning snapshot engine, and transaction ledger (DATA_MODEL §2.4, TASKS P5-01..P5-06, P13-15).
#![allow(dead_code)]

use chrono::Utc;
use diesel::sql_types::BigInt;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::{tenant, CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;

#[derive(QueryableByName)]
struct DisplayIdRow {
    #[diesel(sql_type = BigInt)]
    display_id: i64,
}

use crate::models::{
    Address, Claim, ClaimItem, ClaimItemImage, CreditLine, Exchange, ExchangeItem, LineItem,
    LineItemAdjustment, LineItemTaxLine, Order, OrderChange, OrderChangeAction, OrderItem,
    OrderShippingMethod, OrderSummary, Return, ReturnItem, ReturnReason, ShippingMethod,
    ShippingMethodAdjustment, ShippingMethodTaxLine, Transaction,
};

#[derive(Clone)]
pub struct OrderService {
    orders: Arc<PgCrudRepository<Order>>,
    order_items: Arc<PgCrudRepository<OrderItem>>,
    line_items: Arc<PgCrudRepository<LineItem>>,
    line_item_adjustments: Arc<PgCrudRepository<LineItemAdjustment>>,
    line_item_tax_lines: Arc<PgCrudRepository<LineItemTaxLine>>,
    order_shipping_methods: Arc<PgCrudRepository<OrderShippingMethod>>,
    shipping_methods: Arc<PgCrudRepository<ShippingMethod>>,
    shipping_method_adjustments: Arc<PgCrudRepository<ShippingMethodAdjustment>>,
    shipping_method_tax_lines: Arc<PgCrudRepository<ShippingMethodTaxLine>>,
    order_summaries: Arc<PgCrudRepository<OrderSummary>>,
    order_changes: Arc<PgCrudRepository<OrderChange>>,
    order_change_actions: Arc<PgCrudRepository<OrderChangeAction>>,
    transactions: Arc<PgCrudRepository<Transaction>>,
    credit_lines: Arc<PgCrudRepository<CreditLine>>,
    addresses: Arc<PgCrudRepository<Address>>,
    returns: Arc<PgCrudRepository<Return>>,
    return_items: Arc<PgCrudRepository<ReturnItem>>,
    return_reasons: Arc<PgCrudRepository<ReturnReason>>,
    claims: Arc<PgCrudRepository<Claim>>,
    claim_items: Arc<PgCrudRepository<ClaimItem>>,
    claim_item_images: Arc<PgCrudRepository<ClaimItemImage>>,
    exchanges: Arc<PgCrudRepository<Exchange>>,
    exchange_items: Arc<PgCrudRepository<ExchangeItem>>,
    pool: DbPool,
}

impl OrderService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            orders: Arc::new(PgCrudRepository::new(pool.clone())),
            order_items: Arc::new(PgCrudRepository::new(pool.clone())),
            line_items: Arc::new(PgCrudRepository::new(pool.clone())),
            line_item_adjustments: Arc::new(PgCrudRepository::new(pool.clone())),
            line_item_tax_lines: Arc::new(PgCrudRepository::new(pool.clone())),
            order_shipping_methods: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_methods: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_method_adjustments: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_method_tax_lines: Arc::new(PgCrudRepository::new(pool.clone())),
            order_summaries: Arc::new(PgCrudRepository::new(pool.clone())),
            order_changes: Arc::new(PgCrudRepository::new(pool.clone())),
            order_change_actions: Arc::new(PgCrudRepository::new(pool.clone())),
            transactions: Arc::new(PgCrudRepository::new(pool.clone())),
            credit_lines: Arc::new(PgCrudRepository::new(pool.clone())),
            addresses: Arc::new(PgCrudRepository::new(pool.clone())),
            returns: Arc::new(PgCrudRepository::new(pool.clone())),
            return_items: Arc::new(PgCrudRepository::new(pool.clone())),
            return_reasons: Arc::new(PgCrudRepository::new(pool.clone())),
            claims: Arc::new(PgCrudRepository::new(pool.clone())),
            claim_items: Arc::new(PgCrudRepository::new(pool.clone())),
            claim_item_images: Arc::new(PgCrudRepository::new(pool.clone())),
            exchanges: Arc::new(PgCrudRepository::new(pool.clone())),
            exchange_items: Arc::new(PgCrudRepository::new(pool.clone())),
            pool,
        }
    }

    // ── Gap-Free Sequential display_id Generation (P5-06) ───────────────────

    /// The next order number for the current tenant.
    ///
    /// This used to be `Arc<AtomicI64>` starting at 1000 — process memory, shared by every
    /// tenant, reset by every restart. The first order created after a restart therefore
    /// reused a number already stored and failed on the unique index, which surfaced as a
    /// duplicate-key error on checkout rather than as anything resembling its cause.
    ///
    /// `UPDATE ... RETURNING` holds a row lock for the length of the statement, so two
    /// concurrent orders in one tenant serialise here instead of both reading the same
    /// value. RLS scopes the row, so "the current tenant's counter" needs no argument.
    pub async fn next_display_id(&self) -> Result<i64> {
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                // The upsert covers a tenant's first order, where no counter row exists yet.
                let rows: Vec<DisplayIdRow> = diesel::sql_query(
                    "INSERT INTO \"order\".display_id_counter (tenant_id, next_value)                      VALUES (current_setting('app.tenant_id', true), 2)                      ON CONFLICT (tenant_id) DO UPDATE                        SET next_value = \"order\".display_id_counter.next_value + 1                      RETURNING next_value - 1 AS display_id",
                )
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                rows.into_iter()
                    .next()
                    .map(|r| r.display_id)
                    .ok_or_else(|| Error::Database("display_id counter returned no row".into()))
            })
        })
        .await
    }

    // ── Order Creation & Retrieval (P5-01) ───────────────────────────────────

    pub async fn create_order(&self, input: CreateOrderInput) -> Result<OrderDTO> {
        let id: Id<Order> = Id::new();
        let display_id = self.next_display_id().await?;
        let now = Utc::now();

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

        let order = Order {
            id,
            display_id,
            region_id: input.region_id,
            customer_id: input.customer_id,
            sales_channel_id: input.sales_channel_id,
            email: input.email,
            currency_code: input.currency_code.to_lowercase(),
            shipping_address_id,
            billing_address_id,
            status: if input.is_draft_order.unwrap_or(false) {
                "draft".to_string()
            } else {
                "pending".to_string()
            },
            payment_status: "not_paid".to_string(),
            fulfillment_status: "not_fulfilled".to_string(),
            version: 1,
            is_draft_order: input.is_draft_order.unwrap_or(false),
            metadata: input.metadata,
            canceled_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let order_id_str = order.id.to_string();
        self.orders.create(None, order).await?;

        // Add line items and initial order_item snapshot at version 1
        if let Some(items) = input.items {
            for item_input in items {
                self.add_line_item_internal(&order_id_str, item_input, 1)
                    .await?;
            }
        }

        // Add shipping methods at version 1
        if let Some(sms) = input.shipping_methods {
            for sm_input in sms {
                self.add_shipping_method_internal(&order_id_str, sm_input, 1)
                    .await?;
            }
        }

        // Compute initial summary at version 1
        self.recompute_order_summary(&order_id_str, 1).await?;

        self.get_order(&order_id_str).await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<OrderDTO> {
        let order = self.orders.retrieve(None, order_id).await?;
        self.assemble_order_dto(&order, order.version).await
    }

    pub async fn list_orders(&self, filter: OrderFilter) -> Result<(Vec<OrderDTO>, usize)> {
        let mut spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        if let Some(cid) = filter.customer_id {
            spec = spec.with_predicate("customer_id", FilterOp::Eq(cid));
        }
        if let Some(scid) = filter.sales_channel_id {
            spec = spec.with_predicate("sales_channel_id", FilterOp::Eq(scid));
        }
        if let Some(rid) = filter.region_id {
            spec = spec.with_predicate("region_id", FilterOp::Eq(rid));
        }
        if let Some(st) = filter.status {
            spec = spec.with_predicate("status", FilterOp::Eq(st));
        }
        if let Some(pst) = filter.payment_status {
            spec = spec.with_predicate("payment_status", FilterOp::Eq(pst));
        }
        if let Some(fst) = filter.fulfillment_status {
            spec = spec.with_predicate("fulfillment_status", FilterOp::Eq(fst));
        }
        if let Some(did) = filter.display_id {
            spec = spec.with_predicate("display_id", FilterOp::Eq(did.to_string()));
        }

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(20);
        spec = spec.with_pagination(offset, limit);

        let (orders, total) = self.orders.list_and_count(None, &spec).await?;
        let mut dtos = Vec::with_capacity(orders.len());
        for ord in orders {
            dtos.push(self.assemble_order_dto(&ord, ord.version).await?);
        }

        Ok((dtos, total as usize))
    }

    pub async fn get_order_at_version(&self, order_id: &str, version: i32) -> Result<OrderDTO> {
        let order = self.orders.retrieve(None, order_id).await?;
        self.assemble_order_dto(&order, version).await
    }

    pub async fn update_order(&self, order_id: &str, input: UpdateOrderInput) -> Result<OrderDTO> {
        let mut order = self.orders.retrieve(None, order_id).await?;

        if let Some(r_id) = input.region_id {
            order.region_id = Some(r_id);
        }
        if let Some(c_id) = input.customer_id {
            order.customer_id = Some(c_id);
        }
        if let Some(sc_id) = input.sales_channel_id {
            order.sales_channel_id = Some(sc_id);
        }
        if let Some(em) = input.email {
            order.email = Some(em);
        }
        if let Some(st) = input.status {
            order.status = st;
        }
        if let Some(ps) = input.payment_status {
            order.payment_status = ps;
        }
        if let Some(fs) = input.fulfillment_status {
            order.fulfillment_status = fs;
        }
        if let Some(meta) = input.metadata {
            order.metadata = Some(meta);
        }
        order.updated_at = Utc::now();

        let updated = self.orders.update(None, order_id, order).await?;
        self.assemble_order_dto(&updated, updated.version).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<OrderDTO> {
        let mut order = self.orders.retrieve(None, order_id).await?;

        if order.status == "canceled" {
            return Err(Error::Conflict {
                code: "order_already_canceled",
                message: "Order is already canceled".to_string(),
            });
        }

        order.status = "canceled".to_string();
        order.canceled_at = Some(Utc::now());
        order.updated_at = Utc::now();

        let updated = self.orders.update(None, order_id, order).await?;
        self.assemble_order_dto(&updated, updated.version).await
    }

    pub async fn archive_order(&self, order_id: &str) -> Result<OrderDTO> {
        self.update_order(
            order_id,
            UpdateOrderInput {
                status: Some("archived".to_string()),
                ..Default::default()
            },
        )
        .await
    }

    pub async fn complete_order(&self, order_id: &str) -> Result<OrderDTO> {
        self.update_order(
            order_id,
            UpdateOrderInput {
                status: Some("completed".to_string()),
                ..Default::default()
            },
        )
        .await
    }

    /// Updates the three distinct COD accounting amounts on an order (P24-02, BP8):
    /// order_total, collected_amount, remitted_amount (net of courier_fee).
    pub async fn update_cod_amounts(
        &self,
        order_id: &str,
        order_total: Decimal,
        collected: Decimal,
        remitted: Decimal,
        courier_fee: Decimal,
    ) -> Result<CodAmounts> {
        let cod = CodAmounts::new(order_total, collected, remitted, courier_fee)?;
        let mut order = self.orders.retrieve(None, order_id).await?;
        order.set_cod_amounts(cod.clone());
        self.orders.update(None, order_id, order).await?;
        Ok(cod)
    }

    /// Fetches the distinct COD accounting amounts for an order.
    pub async fn get_cod_amounts(&self, order_id: &str) -> Result<Option<CodAmounts>> {
        let order = self.orders.retrieve(None, order_id).await?;
        Ok(order.get_cod_amounts())
    }

    // ── Versioned Snapshot Loading (P5-03) ──────────────────────────────────

    async fn assemble_order_dto(&self, order: &Order, version: i32) -> Result<OrderDTO> {
        let order_id = order.id.to_string();

        let shipping_address = if let Some(addr_id) = &order.shipping_address_id {
            self.addresses
                .retrieve(None, addr_id)
                .await
                .ok()
                .map(|a| Self::to_address_dto(&a))
        } else {
            None
        };

        let billing_address = if let Some(addr_id) = &order.billing_address_id {
            self.addresses
                .retrieve(None, addr_id)
                .await
                .ok()
                .map(|a| Self::to_address_dto(&a))
        } else {
            None
        };

        // Load items at specified version (P5-03)
        let ois = self
            .order_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut items = Vec::new();
        for oi in ois.into_iter().filter(|o| o.version == version) {
            if let Ok(li) = self.line_items.retrieve(None, &oi.item_id).await {
                let adjs = self
                    .line_item_adjustments
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("item_id", FilterOp::Eq(li.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                let taxes = self
                    .line_item_tax_lines
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("item_id", FilterOp::Eq(li.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                items.push(LineItemDTO {
                    id: li.id.to_string(),
                    title: li.title.clone(),
                    subtitle: li.subtitle.clone(),
                    thumbnail: li.thumbnail.clone(),
                    quantity: oi.quantity,
                    unit_price: li.unit_price,
                    is_tax_inclusive: li.is_tax_inclusive,
                    compare_at_unit_price: li.compare_at_unit_price,
                    requires_shipping: li.requires_shipping,
                    is_discountable: li.is_discountable,
                    is_custom_price: li.is_custom_price,
                    variant_id: li.variant_id.clone(),
                    product_id: li.product_id.clone(),
                    product_title: li.product_title.clone(),
                    product_description: li.product_description.clone(),
                    product_subtitle: li.product_subtitle.clone(),
                    product_type: li.product_type.clone(),
                    product_collection: li.product_collection.clone(),
                    product_handle: li.product_handle.clone(),
                    variant_sku: li.variant_sku.clone(),
                    variant_barcode: li.variant_barcode.clone(),
                    variant_title: li.variant_title.clone(),
                    variant_option_values: li.variant_option_values.clone(),
                    adjustments: adjs.into_iter().map(Self::to_adj_dto).collect(),
                    tax_lines: taxes.into_iter().map(Self::to_tax_dto).collect(),
                    metadata: li.metadata.clone(),
                    created_at: li.created_at,
                    updated_at: li.updated_at,
                });
            }
        }

        // Load shipping methods at specified version
        let osms = self
            .order_shipping_methods
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut shipping_methods = Vec::new();
        for osm in osms.into_iter().filter(|o| o.version == version) {
            if let Ok(sm) = self
                .shipping_methods
                .retrieve(None, &osm.shipping_method_id)
                .await
            {
                let adjs = self
                    .shipping_method_adjustments
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("shipping_method_id", FilterOp::Eq(sm.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                let taxes = self
                    .shipping_method_tax_lines
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("shipping_method_id", FilterOp::Eq(sm.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                shipping_methods.push(ShippingMethodDTO {
                    id: sm.id.to_string(),
                    name: sm.name.clone(),
                    description: sm.description.clone(),
                    amount: sm.amount,
                    is_tax_inclusive: sm.is_tax_inclusive,
                    shipping_option_id: sm.shipping_option_id.clone(),
                    data: sm.data.clone(),
                    adjustments: adjs.into_iter().map(Self::to_sm_adj_dto).collect(),
                    tax_lines: taxes.into_iter().map(Self::to_sm_tax_dto).collect(),
                    metadata: sm.metadata.clone(),
                    created_at: sm.created_at,
                    updated_at: sm.updated_at,
                });
            }
        }

        // Load summary at specified version
        let sums = self
            .order_summaries
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;
        let summary = sums
            .into_iter()
            .find(|s| s.version == version)
            .map(|s| Self::to_summary_dto(&s));

        // Transactions & Credit Lines
        let txs = self
            .transactions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;
        let transactions = txs.into_iter().map(Self::to_tx_dto).collect();

        let cls = self
            .credit_lines
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;
        let credit_lines = cls.into_iter().map(Self::to_cl_dto).collect();

        Ok(OrderDTO {
            id: order.id.to_string(),
            display_id: order.display_id,
            region_id: order.region_id.clone(),
            customer_id: order.customer_id.clone(),
            sales_channel_id: order.sales_channel_id.clone(),
            email: order.email.clone(),
            currency_code: order.currency_code.clone(),
            shipping_address,
            billing_address,
            items,
            shipping_methods,
            summary,
            cod_amounts: order.get_cod_amounts(),
            transactions,
            credit_lines,
            status: order.status.clone(),
            payment_status: order.payment_status.clone(),
            fulfillment_status: order.fulfillment_status.clone(),
            version,
            is_draft_order: order.is_draft_order,
            metadata: order.metadata.clone(),
            canceled_at: order.canceled_at,
            created_at: order.created_at,
            updated_at: order.updated_at,
        })
    }

    // ── Order Summary Calculation (P5-04) ───────────────────────────────────

    pub async fn recompute_order_summary(
        &self,
        order_id: &str,
        version: i32,
    ) -> Result<OrderSummaryDTO> {
        let mut item_subtotal = Decimal::ZERO;
        let mut shipping_subtotal = Decimal::ZERO;
        let mut total_discount = Decimal::ZERO;
        let mut total_tax = Decimal::ZERO;

        // Sum items at version N
        let ois = self
            .order_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for oi in ois.into_iter().filter(|o| o.version == version) {
            if let Ok(li) = self.line_items.retrieve(None, &oi.item_id).await {
                let item_gross = li.unit_price * Decimal::from(oi.quantity);
                item_subtotal += item_gross;

                let adjs = self
                    .line_item_adjustments
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("item_id", FilterOp::Eq(li.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                for adj in adjs {
                    total_discount += adj.amount;
                }

                let taxes = self
                    .line_item_tax_lines
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("item_id", FilterOp::Eq(li.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                for t in taxes {
                    total_tax += (item_gross * t.rate) / dec!(100);
                }
            }
        }

        // Sum shipping at version N
        let osms = self
            .order_shipping_methods
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for osm in osms.into_iter().filter(|o| o.version == version) {
            if let Ok(sm) = self
                .shipping_methods
                .retrieve(None, &osm.shipping_method_id)
                .await
            {
                shipping_subtotal += sm.amount;

                let adjs = self
                    .shipping_method_adjustments
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("shipping_method_id", FilterOp::Eq(sm.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                for adj in adjs {
                    total_discount += adj.amount;
                }

                let taxes = self
                    .shipping_method_tax_lines
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("shipping_method_id", FilterOp::Eq(sm.id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull)
                            .with_pagination(0, 100),
                    )
                    .await?;
                for t in taxes {
                    total_tax += (sm.amount * t.rate) / dec!(100);
                }
            }
        }

        let current_order_total =
            (item_subtotal + shipping_subtotal + total_tax - total_discount).max(Decimal::ZERO);

        // Calculate paid, refunded, credit line totals from Transactions
        let mut paid_total = Decimal::ZERO;
        let mut refunded_total = Decimal::ZERO;
        let txs = self
            .transactions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for tx in txs {
            if tx.reference == "payment" || tx.reference == "capture" {
                paid_total += tx.amount;
            } else if tx.reference == "refund" {
                refunded_total += tx.amount;
            }
        }

        let mut credit_line_total = Decimal::ZERO;
        let cls = self
            .credit_lines
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;
        for cl in cls {
            credit_line_total += cl.amount;
        }

        let accounting_total = paid_total - refunded_total + credit_line_total;
        let raw_difference = current_order_total - accounting_total;
        let pending_difference = raw_difference;

        // Upsert / Create summary for version
        let existing_summaries = self
            .order_summaries
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let existing = existing_summaries
            .into_iter()
            .find(|s| s.version == version);

        let summary = if let Some(mut existing_sum) = existing {
            existing_sum.paid_total = paid_total;
            existing_sum.refunded_total = refunded_total;
            existing_sum.credit_line_total = credit_line_total;
            existing_sum.raw_difference = raw_difference;
            existing_sum.pending_difference = pending_difference;
            existing_sum.current_order_total = current_order_total;
            existing_sum.accounting_total = accounting_total;
            existing_sum.updated_at = Utc::now();
            self.order_summaries
                .update(None, &existing_sum.id.to_string(), existing_sum)
                .await?
        } else {
            let new_sum = OrderSummary {
                id: Id::new(),
                order_id: order_id.to_string(),
                version,
                paid_total,
                refunded_total,
                credit_line_total,
                raw_difference,
                pending_difference,
                current_order_total,
                original_order_total: current_order_total,
                accounting_total,
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            self.order_summaries.create(None, new_sum).await?
        };

        Ok(Self::to_summary_dto(&summary))
    }

    // ── Order Change Engine Integration (P5-02, P5-03, P5-04) ───────────────

    pub async fn create_order_change(
        &self,
        input: CreateOrderChangeInput,
    ) -> Result<OrderChangeDTO> {
        let order = self.get_order(&input.order_id).await?;

        let id: Id<OrderChange> = Id::new();
        let change = OrderChange {
            id,
            order_id: input.order_id.clone(),
            version: order.version,
            change_type: input.change_type,
            status: "pending".to_string(),
            requested_by: input.requested_by,
            requested_at: Some(Utc::now()),
            confirmed_by: None,
            confirmed_at: None,
            declined_by: None,
            declined_at: None,
            canceled_by: None,
            canceled_at: None,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.order_changes.create(None, change).await?;
        Ok(Self::to_order_change_dto(&created, vec![]))
    }

    pub async fn add_order_change_action(
        &self,
        change_id: &str,
        input: CreateOrderChangeActionInput,
    ) -> Result<OrderChangeActionDTO> {
        let change = self.order_changes.retrieve(None, change_id).await?;

        if change.status != "pending" && change.status != "requested" {
            return Err(Error::Conflict {
                code: "order_change_not_open",
                message: format!("Order change is already {}", change.status),
            });
        }

        let action = OrderChangeAction {
            id: Id::new(),
            order_change_id: change_id.to_string(),
            order_id: change.order_id.clone(),
            action: input.action,
            reference: input.reference,
            reference_id: input.reference_id,
            amount: input.amount,
            applied: false,
            details: input.details,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.order_change_actions.create(None, action).await?;
        Ok(Self::to_action_dto(&created))
    }

    /// Preview Order Change: evaluates pending actions against version-N snapshot in-memory (P5-02, DATA_MODEL §2.4 step 3).
    pub async fn preview_order_change(&self, change_id: &str) -> Result<OrderChangePreview> {
        let change = self.order_changes.retrieve(None, change_id).await?;

        let order = self
            .get_order_at_version(&change.order_id, change.version)
            .await?;
        let current_total = order
            .summary
            .as_ref()
            .map(|s| s.current_order_total)
            .unwrap_or(Decimal::ZERO);

        let currency = order
            .currency_code
            .parse()
            .unwrap_or(stheno_types::CurrencyCode::Usd);
        let current_money = stheno_types::Money::new(current_total, currency);

        let actions = self
            .order_change_actions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_change_id", FilterOp::Eq(change_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut delta = stheno_types::Money::zero(currency);

        for act in &actions {
            match act.action.as_str() {
                "ITEM_ADD" => {
                    if let Some(amt) = act.amount {
                        let money_amt = stheno_types::Money::new(amt, currency);
                        delta = (delta + money_amt)?;
                    } else if let Some(details) = &act.details {
                        let qty = details
                            .get("quantity")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1) as i32;
                        let price = details
                            .get("unit_price")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                            .unwrap_or(Decimal::ZERO);
                        let amt = price * Decimal::from(qty);
                        let money_amt = stheno_types::Money::new(amt, currency);
                        delta = (delta + money_amt)?;
                    }
                }
                "ITEM_REMOVE" => {
                    if let Some(ref_id) = &act.reference_id {
                        if let Some(item) = order.items.iter().find(|i| &i.id == ref_id) {
                            let item_total = item.unit_price * Decimal::from(item.quantity);
                            let money_amt = stheno_types::Money::new(item_total, currency);
                            delta = (delta - money_amt)?;
                        }
                    }
                }
                "ITEM_UPDATE" => {
                    if let Some(ref_id) = &act.reference_id {
                        if let Some(item) = order.items.iter().find(|i| &i.id == ref_id) {
                            let old_total = item.unit_price * Decimal::from(item.quantity);
                            let new_qty = act
                                .details
                                .as_ref()
                                .and_then(|d| d.get("quantity"))
                                .and_then(|v| v.as_i64())
                                .map(|q| q as i32)
                                .unwrap_or(item.quantity);
                            let new_price = act
                                .details
                                .as_ref()
                                .and_then(|d| d.get("unit_price"))
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<Decimal>().ok())
                                .unwrap_or(item.unit_price);
                            let new_total = new_price * Decimal::from(new_qty);
                            let diff = new_total - old_total;
                            let money_diff = stheno_types::Money::new(diff, currency);
                            delta = (delta + money_diff)?;
                        }
                    }
                }
                "SHIPPING_ADD" => {
                    if let Some(amt) = act.amount {
                        let money_amt = stheno_types::Money::new(amt, currency);
                        delta = (delta + money_amt)?;
                    }
                }
                "SHIPPING_REMOVE" => {
                    if let Some(ref_id) = &act.reference_id {
                        if let Some(sm) = order.shipping_methods.iter().find(|s| &s.id == ref_id) {
                            let money_amt = stheno_types::Money::new(sm.amount, currency);
                            delta = (delta - money_amt)?;
                        }
                    }
                }
                "SHIPPING_UPDATE" => {
                    if let Some(ref_id) = &act.reference_id {
                        if let Some(sm) = order.shipping_methods.iter().find(|s| &s.id == ref_id) {
                            if let Some(new_amt) = act.amount {
                                if new_amt >= sm.amount {
                                    let diff = new_amt - sm.amount;
                                    let money_diff = stheno_types::Money::new(diff, currency);
                                    delta = (delta + money_diff)?;
                                } else {
                                    let diff = sm.amount - new_amt;
                                    let money_diff = stheno_types::Money::new(diff, currency);
                                    delta = (delta - money_diff)?;
                                }
                            }
                        }
                    }
                }
                _ => {
                    if let Some(amt) = act.amount {
                        let money_amt = stheno_types::Money::new(amt, currency);
                        delta = (delta + money_amt)?;
                    }
                }
            }
        }

        let new_total = (current_money + delta)?;

        Ok(OrderChangePreview {
            current_total: current_money,
            difference_total: delta,
            new_total,
            actions_count: actions.len(),
        })
    }

    /// Confirm Order Change: applies actions, increments order.version to N+1, writes new order_items, recomputes summary (P5-02, DATA_MODEL §2.4 step 4).
    pub async fn confirm_order_change(
        &self,
        change_id: &str,
        confirmed_by: Option<&str>,
    ) -> Result<OrderDTO> {
        let mut change = self.order_changes.retrieve(None, change_id).await?;

        if change.status != "pending" && change.status != "requested" {
            return Err(Error::Conflict {
                code: "order_change_not_open",
                message: format!("Order change is already {}", change.status),
            });
        }

        let order_id = change.order_id.clone();
        let old_version = change.version;
        let new_version = old_version + 1;

        // Clone version N items to version N+1
        let ois = self
            .order_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let version_n_items: Vec<OrderItem> = ois
            .into_iter()
            .filter(|o| o.version == old_version)
            .collect();

        for mut item in version_n_items {
            item.id = Id::new();
            item.version = new_version;
            item.created_at = Utc::now();
            item.updated_at = Utc::now();
            self.order_items.create(None, item).await?;
        }

        // Clone version N shipping methods to version N+1
        let osms = self
            .order_shipping_methods
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let version_n_sms: Vec<OrderShippingMethod> = osms
            .into_iter()
            .filter(|o| o.version == old_version)
            .collect();

        for mut sm in version_n_sms {
            sm.id = Id::new();
            sm.version = new_version;
            sm.created_at = Utc::now();
            sm.updated_at = Utc::now();
            self.order_shipping_methods.create(None, sm).await?;
        }

        // Apply actions to version N+1
        let actions = self
            .order_change_actions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_change_id", FilterOp::Eq(change_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for act in actions {
            match act.action.as_str() {
                "ITEM_ADD" => {
                    if let Some(details) = &act.details {
                        let title = details
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Item");
                        let qty = details
                            .get("quantity")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1) as i32;
                        let unit_price = details
                            .get("unit_price")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                            .or_else(|| {
                                details.get("unit_price").and_then(|v| v.as_f64()).map(|f| {
                                    Decimal::from_str_exact(&f.to_string()).unwrap_or(Decimal::ZERO)
                                })
                            })
                            .unwrap_or(Decimal::ZERO);
                        let variant_id = details
                            .get("variant_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let line_item = LineItem {
                            id: Id::new(),
                            title: title.to_string(),
                            subtitle: None,
                            thumbnail: None,
                            variant_id,
                            product_id: None,
                            product_title: None,
                            product_description: None,
                            product_subtitle: None,
                            product_type: None,
                            product_collection: None,
                            product_handle: None,
                            variant_sku: None,
                            variant_barcode: None,
                            variant_title: None,
                            variant_option_values: None,
                            requires_shipping: true,
                            is_discountable: true,
                            is_tax_inclusive: false,
                            compare_at_unit_price: None,
                            unit_price,
                            quantity: qty,
                            is_custom_price: false,
                            metadata: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            deleted_at: None,
                        };
                        let line_item_id = line_item.id.to_string();
                        self.line_items.create(None, line_item).await?;

                        self.order_items
                            .create(
                                None,
                                OrderItem {
                                    id: Id::new(),
                                    order_id: order_id.clone(),
                                    item_id: line_item_id,
                                    version: new_version,
                                    quantity: qty,
                                    fulfilled_quantity: 0,
                                    shipped_quantity: 0,
                                    return_requested_quantity: 0,
                                    return_received_quantity: 0,
                                    return_dismissed_quantity: 0,
                                    written_off_quantity: 0,
                                    metadata: None,
                                    created_at: Utc::now(),
                                    updated_at: Utc::now(),
                                    deleted_at: None,
                                },
                            )
                            .await?;
                    }
                }
                "ITEM_REMOVE" => {
                    if let Some(ref_id) = &act.reference_id {
                        let current_ois = self
                            .order_items
                            .list(
                                None,
                                &FilterSpec::new()
                                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                                    .with_predicate("deleted_at", FilterOp::IsNull)
                                    .with_pagination(0, 100),
                            )
                            .await?;
                        for oi in current_ois {
                            if oi.version == new_version && &oi.item_id == ref_id {
                                self.order_items.delete(None, &oi.id.to_string()).await?;
                            }
                        }
                    }
                }
                "ITEM_UPDATE" => {
                    if let (Some(ref_id), Some(details)) = (&act.reference_id, &act.details) {
                        if let Some(new_qty) = details.get("quantity").and_then(|v| v.as_i64()) {
                            let current_ois = self
                                .order_items
                                .list(
                                    None,
                                    &FilterSpec::new()
                                        .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                                        .with_predicate("deleted_at", FilterOp::IsNull)
                                        .with_pagination(0, 100),
                                )
                                .await?;
                            for mut oi in current_ois {
                                if oi.version == new_version && &oi.item_id == ref_id {
                                    oi.quantity = new_qty as i32;
                                    oi.updated_at = Utc::now();
                                    self.order_items
                                        .update(None, &oi.id.to_string(), oi)
                                        .await?;
                                }
                            }
                        }
                        if let Some(new_price) = details
                            .get("unit_price")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                        {
                            if let Ok(mut li) = self.line_items.retrieve(None, ref_id).await {
                                li.unit_price = new_price;
                                li.updated_at = Utc::now();
                                self.line_items.update(None, ref_id, li).await?;
                            }
                        }
                    }
                }
                "SHIPPING_ADD" => {
                    if let Some(details) = &act.details {
                        let name = details
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Shipping");
                        let amount = details
                            .get("amount")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                            .unwrap_or(Decimal::ZERO);
                        let shipping_option_id = details
                            .get("shipping_option_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let sm = ShippingMethod {
                            id: Id::new(),
                            name: name.to_string(),
                            description: None,
                            shipping_option_id,
                            amount,
                            is_tax_inclusive: false,
                            data: None,
                            metadata: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            deleted_at: None,
                        };
                        let sm_id = sm.id.to_string();
                        self.shipping_methods.create(None, sm).await?;

                        self.order_shipping_methods
                            .create(
                                None,
                                OrderShippingMethod {
                                    id: Id::new(),
                                    order_id: order_id.clone(),
                                    shipping_method_id: sm_id,
                                    version: new_version,
                                    created_at: Utc::now(),
                                    updated_at: Utc::now(),
                                    deleted_at: None,
                                },
                            )
                            .await?;
                    }
                }
                "SHIPPING_UPDATE" => {
                    if let (Some(ref_id), Some(details)) = (&act.reference_id, &act.details) {
                        if let Some(amt) = details
                            .get("amount")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                        {
                            if let Ok(mut sm) = self.shipping_methods.retrieve(None, ref_id).await {
                                sm.amount = amt;
                                sm.updated_at = Utc::now();
                                self.shipping_methods.update(None, ref_id, sm).await?;
                            }
                        }
                    }
                }
                "SHIPPING_REMOVE" => {
                    if let Some(ref_id) = &act.reference_id {
                        let current_osms = self
                            .order_shipping_methods
                            .list(
                                None,
                                &FilterSpec::new()
                                    .with_predicate("order_id", FilterOp::Eq(order_id.clone()))
                                    .with_predicate("deleted_at", FilterOp::IsNull)
                                    .with_pagination(0, 100),
                            )
                            .await?;
                        for osm in current_osms {
                            if osm.version == new_version && &osm.shipping_method_id == ref_id {
                                self.order_shipping_methods
                                    .delete(None, &osm.id.to_string())
                                    .await?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Increment order.version
        let mut order = self.orders.retrieve(None, &order_id).await?;
        order.version = new_version;
        order.updated_at = Utc::now();
        self.orders.update(None, &order_id, order).await?;

        // Recompute order_summary at N+1
        self.recompute_order_summary(&order_id, new_version).await?;

        // Update change status
        change.status = "confirmed".to_string();
        change.confirmed_by = confirmed_by.map(|s| s.to_string());
        change.confirmed_at = Some(Utc::now());
        change.updated_at = Utc::now();

        self.order_changes.update(None, change_id, change).await?;

        self.get_order(&order_id).await
    }

    pub async fn cancel_order_change(&self, change_id: &str) -> Result<OrderChangeDTO> {
        let mut change = self.order_changes.retrieve(None, change_id).await?;

        if change.status == "confirmed" || change.status == "canceled" {
            return Err(Error::Conflict {
                code: "order_change_immutable",
                message: format!("Cannot cancel order change in status {}", change.status),
            });
        }

        change.status = "canceled".to_string();
        change.canceled_at = Some(Utc::now());
        change.updated_at = Utc::now();

        let updated = self.order_changes.update(None, change_id, change).await?;

        let actions = self
            .order_change_actions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_change_id", FilterOp::Eq(change_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        Ok(Self::to_order_change_dto(
            &updated,
            actions.iter().map(Self::to_action_dto).collect(),
        ))
    }

    pub async fn decline_order_change(
        &self,
        change_id: &str,
        declined_by: Option<&str>,
        _reason: Option<&str>,
    ) -> Result<OrderChangeDTO> {
        let mut change = self.order_changes.retrieve(None, change_id).await?;

        if change.status != "pending" && change.status != "requested" {
            return Err(Error::Conflict {
                code: "order_change_not_open",
                message: format!("Cannot decline order change in status {}", change.status),
            });
        }

        change.status = "declined".to_string();
        change.declined_by = declined_by.map(|s| s.to_string());
        change.declined_at = Some(Utc::now());
        change.updated_at = Utc::now();

        let updated = self.order_changes.update(None, change_id, change).await?;

        let actions = self
            .order_change_actions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_change_id", FilterOp::Eq(change_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        Ok(Self::to_order_change_dto(
            &updated,
            actions.iter().map(Self::to_action_dto).collect(),
        ))
    }

    // ── Returns Engine (P6-07 to P6-10) ─────────────────────────────────────

    pub async fn create_return(&self, input: CreateReturnInput) -> Result<ReturnDTO> {
        let order = self.get_order(&input.order_id).await?;

        let ret = Return {
            id: Id::new(),
            order_id: order.id.clone(),
            claim_id: None,
            exchange_id: None,
            status: "requested".to_string(),
            refund_amount: input.refund_amount,
            location_id: input.location_id,
            metadata: input.metadata,
            requested_at: Some(Utc::now()),
            received_at: None,
            canceled_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let ret_id = ret.id.to_string();
        self.returns.create(None, ret).await?;
        self.get_return_dto(&ret_id).await
    }

    pub async fn add_return_item(&self, input: AddReturnItemInput) -> Result<ReturnItemDTO> {
        let ret = self.returns.retrieve(None, &input.return_id).await?;

        if ret.status != "requested" && ret.status != "open" {
            return Err(Error::Conflict {
                code: "return_not_open",
                message: format!("Cannot add items to return in status {}", ret.status),
            });
        }

        let item = ReturnItem {
            id: Id::new(),
            return_id: input.return_id.clone(),
            item_id: input.item_id,
            quantity: input.quantity,
            received_quantity: 0,
            damaged_quantity: 0,
            reason_id: input.reason_id,
            note: input.note,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.return_items.create(None, item).await?;
        Ok(Self::to_return_item_dto(&created))
    }

    pub async fn receive_return_items(
        &self,
        return_id: &str,
        items: Vec<ReceiveReturnItemsInput>,
    ) -> Result<ReturnDTO> {
        let mut ret = self.returns.retrieve(None, return_id).await?;

        if ret.status == "canceled" || ret.status == "completed" {
            return Err(Error::Conflict {
                code: "return_immutable",
                message: format!("Cannot receive items on return in status {}", ret.status),
            });
        }

        let r_items = self
            .return_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("return_id", FilterOp::Eq(return_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for in_item in items {
            if let Some(mut ri) = r_items
                .iter()
                .find(|r| r.item_id == in_item.item_id)
                .cloned()
            {
                ri.received_quantity += in_item.received_quantity;
                if let Some(dmg) = in_item.damaged_quantity {
                    ri.damaged_quantity += dmg;
                }
                ri.updated_at = Utc::now();
                self.return_items
                    .update(None, &ri.id.to_string(), ri)
                    .await?;
            }
        }

        ret.status = "received".to_string();
        ret.received_at = Some(Utc::now());
        ret.updated_at = Utc::now();

        self.returns.update(None, return_id, ret).await?;
        self.get_return_dto(return_id).await
    }

    /// Reverses a receival, putting the return back to `requested`.
    ///
    /// The dashboard offers this as "cancel receive" while a return is received but not
    /// yet confirmed — a warehouse miscount, most often. Received and damaged quantities
    /// go back to zero along with the status, because leaving them behind would let a
    /// second receive double-count against the same return.
    ///
    /// Refuses once the return is `completed` or `canceled`, matching every other
    /// transition on this entity: after confirmation a refund may already have been
    /// issued, and unwinding that is a credit note, not a status change.
    pub async fn cancel_return_receipt(&self, return_id: &str) -> Result<ReturnDTO> {
        let mut ret = self.returns.retrieve(None, return_id).await?;

        if ret.status != "received" {
            return Err(Error::Conflict {
                code: "return_not_received",
                message: format!(
                    "Cannot cancel receival on return in status {} — only a received return can be reversed",
                    ret.status
                ),
            });
        }

        let filter = FilterSpec::new()
            .with_predicate("return_id", FilterOp::Eq(return_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        for mut item in self.return_items.list(None, &filter).await? {
            item.received_quantity = 0;
            item.damaged_quantity = 0;
            item.updated_at = Utc::now();
            self.return_items
                .update(None, &item.id.to_string(), item)
                .await?;
        }

        ret.status = "requested".to_string();
        ret.received_at = None;
        ret.updated_at = Utc::now();

        self.returns.update(None, return_id, ret).await?;
        self.get_return_dto(return_id).await
    }

    pub async fn confirm_return(
        &self,
        return_id: &str,
        refund_amount: Option<Decimal>,
    ) -> Result<ReturnDTO> {
        let mut ret = self.returns.retrieve(None, return_id).await?;

        if ret.status == "completed" || ret.status == "canceled" {
            return Err(Error::Conflict {
                code: "return_immutable",
                message: format!("Cannot confirm return in status {}", ret.status),
            });
        }

        ret.status = "completed".to_string();
        if let Some(amt) = refund_amount {
            ret.refund_amount = Some(amt);
        }
        ret.updated_at = Utc::now();

        self.returns.update(None, return_id, ret.clone()).await?;

        // If refund amount is set, record refund transaction on order
        if let Some(amt) = ret.refund_amount {
            if amt > Decimal::ZERO {
                let _ = self
                    .add_transaction(
                        &ret.order_id,
                        CreateTransactionInput {
                            amount: amt,
                            currency_code: "usd".to_string(),
                            reference: "refund".to_string(),
                            reference_id: return_id.to_string(),
                            metadata: None,
                        },
                    )
                    .await;
            }
        }

        self.get_return_dto(return_id).await
    }

    pub async fn cancel_return(&self, return_id: &str) -> Result<ReturnDTO> {
        let mut ret = self.returns.retrieve(None, return_id).await?;

        if ret.status == "completed" {
            return Err(Error::Conflict {
                code: "return_already_completed",
                message: "Cannot cancel a completed return".to_string(),
            });
        }

        ret.status = "canceled".to_string();
        ret.canceled_at = Some(Utc::now());
        ret.updated_at = Utc::now();

        self.returns.update(None, return_id, ret).await?;
        self.get_return_dto(return_id).await
    }

    pub async fn dismiss_return(&self, return_id: &str, reason: Option<&str>) -> Result<ReturnDTO> {
        let mut ret = self.returns.retrieve(None, return_id).await?;

        ret.status = "dismissed".to_string();
        if let Some(r) = reason {
            let mut meta = ret.metadata.unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = meta.as_object_mut() {
                obj.insert("dismiss_reason".to_string(), serde_json::json!(r));
            }
            ret.metadata = Some(meta);
        }
        ret.updated_at = Utc::now();

        self.returns.update(None, return_id, ret).await?;
        self.get_return_dto(return_id).await
    }

    pub async fn list_returns(&self, order_id: Option<&str>) -> Result<Vec<ReturnDTO>> {
        let mut spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        if let Some(oid) = order_id {
            spec = spec.with_predicate("order_id", FilterOp::Eq(oid.to_string()));
        }
        spec = spec.with_pagination(0, 100);

        let returns = self.returns.list(None, &spec).await?;
        let mut list = Vec::new();
        for r in returns {
            list.push(self.get_return_dto(&r.id.to_string()).await?);
        }
        list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(list)
    }

    pub async fn get_return_dto(&self, return_id: &str) -> Result<ReturnDTO> {
        let ret = self.returns.retrieve(None, return_id).await?;

        let items = self
            .return_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("return_id", FilterOp::Eq(return_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        Ok(ReturnDTO {
            id: ret.id.to_string(),
            order_id: ret.order_id,
            claim_id: ret.claim_id,
            exchange_id: ret.exchange_id,
            status: ret.status,
            refund_amount: ret.refund_amount,
            location_id: ret.location_id,
            items: items.iter().map(Self::to_return_item_dto).collect(),
            requested_at: ret.requested_at,
            received_at: ret.received_at,
            canceled_at: ret.canceled_at,
            metadata: ret.metadata,
            created_at: ret.created_at,
            updated_at: ret.updated_at,
        })
    }

    fn to_return_item_dto(i: &ReturnItem) -> ReturnItemDTO {
        ReturnItemDTO {
            id: i.id.to_string(),
            return_id: i.return_id.clone(),
            item_id: i.item_id.clone(),
            quantity: i.quantity,
            received_quantity: i.received_quantity,
            damaged_quantity: i.damaged_quantity,
            reason_id: i.reason_id.clone(),
            note: i.note.clone(),
            metadata: i.metadata.clone(),
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }

    // ── Return Reasons ──

    /// Partial update for a return reason.
    pub async fn update_return_reason(
        &self,
        id: &str,
        input: UpdateReturnReasonInput,
    ) -> Result<ReturnReasonDTO> {
        let mut reason = self.return_reasons.retrieve(None, id).await?;
        if let Some(value) = input.value {
            reason.value = value;
        }
        if let Some(label) = input.label {
            reason.label = label;
        }
        if let Some(description) = input.description {
            reason.description = Some(description);
        }
        reason.updated_at = Utc::now();

        let updated = self.return_reasons.update(None, id, reason).await?;
        Ok(ReturnReasonDTO {
            id: updated.id.to_string(),
            value: updated.value,
            label: updated.label,
            description: updated.description,
            parent_return_reason_id: updated.parent_return_reason_id,
            metadata: updated.metadata,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        })
    }

    pub async fn create_return_reason(
        &self,
        input: CreateReturnReasonInput,
    ) -> Result<ReturnReasonDTO> {
        let id: Id<ReturnReason> = Id::new();
        let reason = ReturnReason {
            id,
            value: input.value,
            label: input.label,
            description: input.description,
            parent_return_reason_id: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.return_reasons.create(None, reason).await?;
        Ok(ReturnReasonDTO {
            id: created.id.to_string(),
            value: created.value,
            label: created.label,
            description: created.description,
            parent_return_reason_id: None,
            metadata: None,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    /// Soft-deletes a return reason.
    ///
    /// The admin route reported `deleted: true` and did nothing, so a reason removed from
    /// the settings screen came back on the next load.
    pub async fn delete_return_reason(&self, id: &str) -> Result<()> {
        self.return_reasons.soft_delete(None, id).await?;
        Ok(())
    }

    /// Every order change, newest first, optionally narrowed to one order.
    ///
    /// `/admin/order-edits` answered with a hardcoded empty array, so an edit created
    /// through `create_order_change` was never listed anywhere.
    pub async fn list_order_changes(&self, order_id: Option<&str>) -> Result<Vec<OrderChangeDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        if let Some(order_id) = order_id {
            filter = filter.with_predicate("order_id", FilterOp::Eq(order_id.to_string()));
        }

        let rows = self.order_changes.list(None, &filter).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(Self::to_order_change_dto(&row, vec![]));
        }
        Ok(out)
    }

    /// Every claim, newest first, optionally narrowed to one order.
    pub async fn list_claims(&self, order_id: Option<&str>) -> Result<Vec<ClaimDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        if let Some(order_id) = order_id {
            filter = filter.with_predicate("order_id", FilterOp::Eq(order_id.to_string()));
        }

        let rows = self.claims.list(None, &filter).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.get_claim_dto(&row.id.to_string()).await?);
        }
        Ok(out)
    }

    pub async fn get_claim(&self, id: &str) -> Result<ClaimDTO> {
        self.get_claim_dto(id).await
    }

    /// Every exchange, newest first, optionally narrowed to one order.
    pub async fn list_exchanges(&self, order_id: Option<&str>) -> Result<Vec<ExchangeDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        if let Some(order_id) = order_id {
            filter = filter.with_predicate("order_id", FilterOp::Eq(order_id.to_string()));
        }

        let rows = self.exchanges.list(None, &filter).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.get_exchange_dto(&row.id.to_string()).await?);
        }
        Ok(out)
    }

    pub async fn get_exchange(&self, id: &str) -> Result<ExchangeDTO> {
        self.get_exchange_dto(id).await
    }

    pub async fn transfer_order(&self, order_id: &str, customer_id: &str) -> Result<OrderDTO> {
        let mut order = self.orders.retrieve(None, order_id).await?;
        order.customer_id = Some(customer_id.to_string());
        order.updated_at = Utc::now();
        self.orders.update(None, order_id, order).await?;
        self.get_order(order_id).await
    }

    pub async fn preview_order(&self, id: &str) -> Result<OrderDTO> {
        self.get_order(id).await
    }

    pub async fn list_draft_orders(&self) -> Result<Vec<OrderDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let rows = self.orders.list(None, &filter).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let order = self.get_order(&row.id.to_string()).await?;
            if order.is_draft_order {
                out.push(order);
            }
        }
        Ok(out)
    }

    pub async fn create_draft_order(&self, mut input: CreateOrderInput) -> Result<OrderDTO> {
        input.is_draft_order = Some(true);
        self.create_order(input).await
    }

    pub async fn get_draft_order(&self, id: &str) -> Result<OrderDTO> {
        self.get_order(id).await
    }

    pub async fn list_return_reasons(&self) -> Result<Vec<ReturnReasonDTO>> {
        let reasons = self
            .return_reasons
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        Ok(reasons
            .into_iter()
            .map(|r| ReturnReasonDTO {
                id: r.id.to_string(),
                value: r.value,
                label: r.label,
                description: r.description,
                parent_return_reason_id: r.parent_return_reason_id,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    // ── Claims Engine (P6-11 to P6-12) ──

    pub async fn create_claim(&self, input: CreateClaimInput) -> Result<ClaimDTO> {
        let order = self.get_order(&input.order_id).await?;

        let claim = Claim {
            id: Id::new(),
            order_id: order.id.clone(),
            r#type: input.r#type,
            order_change_id: None,
            status: "open".to_string(),
            refund_amount: input.refund_amount,
            requested_at: Some(Utc::now()),
            canceled_at: None,
            created_by: input.created_by,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let claim_id = claim.id.to_string();
        self.claims.create(None, claim).await?;
        self.get_claim_dto(&claim_id).await
    }

    pub async fn add_claim_item(&self, input: AddClaimItemInput) -> Result<ClaimItemDTO> {
        let claim = self.claims.retrieve(None, &input.claim_id).await?;

        if claim.status != "open" {
            return Err(Error::Conflict {
                code: "claim_not_open",
                message: format!("Cannot add items to claim in status {}", claim.status),
            });
        }

        let item = ClaimItem {
            id: Id::new(),
            claim_id: input.claim_id.clone(),
            item_id: input.item_id,
            quantity: input.quantity,
            reason: input.reason,
            raw_quantity: None,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.claim_items.create(None, item).await?;
        let item_id_str = created.id.to_string();

        if let Some(imgs) = input.images {
            for img_url in imgs {
                self.claim_item_images
                    .create(
                        None,
                        ClaimItemImage {
                            id: Id::new(),
                            claim_item_id: item_id_str.clone(),
                            url: img_url,
                            metadata: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            deleted_at: None,
                        },
                    )
                    .await?;
            }
        }

        Ok(self.to_claim_item_dto(&created).await)
    }

    pub async fn confirm_claim(&self, claim_id: &str) -> Result<ClaimDTO> {
        let mut claim = self.claims.retrieve(None, claim_id).await?;

        claim.status = "confirmed".to_string();
        claim.updated_at = Utc::now();

        self.claims.update(None, claim_id, claim.clone()).await?;

        if claim.r#type == "refund" {
            if let Some(amt) = claim.refund_amount {
                if amt > Decimal::ZERO {
                    let _ = self
                        .add_transaction(
                            &claim.order_id,
                            CreateTransactionInput {
                                amount: amt,
                                currency_code: "usd".to_string(),
                                reference: "refund".to_string(),
                                reference_id: claim_id.to_string(),
                                metadata: None,
                            },
                        )
                        .await;
                }
            }
        }

        self.get_claim_dto(claim_id).await
    }

    pub async fn cancel_claim(&self, claim_id: &str) -> Result<ClaimDTO> {
        let mut claim = self.claims.retrieve(None, claim_id).await?;

        claim.status = "canceled".to_string();
        claim.canceled_at = Some(Utc::now());
        claim.updated_at = Utc::now();

        self.claims.update(None, claim_id, claim).await?;
        self.get_claim_dto(claim_id).await
    }

    pub async fn get_claim_dto(&self, claim_id: &str) -> Result<ClaimDTO> {
        let claim = self.claims.retrieve(None, claim_id).await?;

        let raw_items = self
            .claim_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("claim_id", FilterOp::Eq(claim_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut items = Vec::new();
        for item in raw_items {
            items.push(self.to_claim_item_dto(&item).await);
        }

        Ok(ClaimDTO {
            id: claim.id.to_string(),
            order_id: claim.order_id,
            r#type: claim.r#type,
            order_change_id: claim.order_change_id,
            status: claim.status,
            refund_amount: claim.refund_amount,
            items,
            requested_at: claim.requested_at,
            canceled_at: claim.canceled_at,
            created_by: claim.created_by,
            metadata: claim.metadata,
            created_at: claim.created_at,
            updated_at: claim.updated_at,
        })
    }

    async fn to_claim_item_dto(&self, item: &ClaimItem) -> ClaimItemDTO {
        let item_id_str = item.id.to_string();
        let images = self
            .claim_item_images
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("claim_item_id", FilterOp::Eq(item_id_str.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await
            .unwrap_or_default();

        ClaimItemDTO {
            id: item_id_str,
            claim_id: item.claim_id.clone(),
            item_id: item.item_id.clone(),
            quantity: item.quantity,
            reason: item.reason.clone(),
            raw_quantity: item.raw_quantity.clone(),
            images: images
                .into_iter()
                .map(|img| ClaimItemImageDTO {
                    id: img.id.to_string(),
                    claim_item_id: img.claim_item_id,
                    url: img.url,
                    metadata: img.metadata,
                    created_at: img.created_at,
                    updated_at: img.updated_at,
                })
                .collect(),
            metadata: item.metadata.clone(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    // ── Exchanges Engine (P6-13 to P6-14) ───────────────────────────────────

    pub async fn create_exchange(&self, input: CreateExchangeInput) -> Result<ExchangeDTO> {
        let order = self.get_order(&input.order_id).await?;

        let exchange = Exchange {
            id: Id::new(),
            order_id: order.id.clone(),
            order_change_id: None,
            status: "open".to_string(),
            allow_backorder: input.allow_backorder.unwrap_or(false),
            difference_due: None,
            created_by: input.created_by,
            canceled_at: None,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let exchange_id = exchange.id.to_string();
        self.exchanges.create(None, exchange).await?;
        self.get_exchange_dto(&exchange_id).await
    }

    pub async fn add_exchange_item(&self, input: AddExchangeItemInput) -> Result<ExchangeItemDTO> {
        let exchange = self.exchanges.retrieve(None, &input.exchange_id).await?;

        if exchange.status != "open" {
            return Err(Error::Conflict {
                code: "exchange_not_open",
                message: format!("Cannot add items to exchange in status {}", exchange.status),
            });
        }

        let item = ExchangeItem {
            id: Id::new(),
            exchange_id: input.exchange_id.clone(),
            item_id: input.item_id,
            quantity: input.quantity,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.exchange_items.create(None, item).await?;
        Ok(Self::to_exchange_item_dto(&created))
    }

    pub async fn confirm_exchange(
        &self,
        exchange_id: &str,
        difference_due: Option<Decimal>,
    ) -> Result<ExchangeDTO> {
        let mut exchange = self.exchanges.retrieve(None, exchange_id).await?;

        exchange.status = "confirmed".to_string();
        exchange.difference_due = difference_due;
        exchange.updated_at = Utc::now();

        self.exchanges.update(None, exchange_id, exchange).await?;
        self.get_exchange_dto(exchange_id).await
    }

    pub async fn cancel_exchange(&self, exchange_id: &str) -> Result<ExchangeDTO> {
        let mut exchange = self.exchanges.retrieve(None, exchange_id).await?;

        exchange.status = "canceled".to_string();
        exchange.canceled_at = Some(Utc::now());
        exchange.updated_at = Utc::now();

        self.exchanges.update(None, exchange_id, exchange).await?;
        self.get_exchange_dto(exchange_id).await
    }

    pub async fn get_exchange_dto(&self, exchange_id: &str) -> Result<ExchangeDTO> {
        let exchange = self.exchanges.retrieve(None, exchange_id).await?;

        let items = self
            .exchange_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("exchange_id", FilterOp::Eq(exchange_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        Ok(ExchangeDTO {
            id: exchange.id.to_string(),
            order_id: exchange.order_id,
            order_change_id: exchange.order_change_id,
            status: exchange.status,
            allow_backorder: exchange.allow_backorder,
            difference_due: exchange.difference_due,
            items: items.iter().map(Self::to_exchange_item_dto).collect(),
            created_by: exchange.created_by,
            canceled_at: exchange.canceled_at,
            metadata: exchange.metadata,
            created_at: exchange.created_at,
            updated_at: exchange.updated_at,
        })
    }

    fn to_exchange_item_dto(i: &ExchangeItem) -> ExchangeItemDTO {
        ExchangeItemDTO {
            id: i.id.to_string(),
            exchange_id: i.exchange_id.clone(),
            item_id: i.item_id.clone(),
            quantity: i.quantity,
            metadata: i.metadata.clone(),
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }

    // ── Draft Order Conversion (P6-15 to P6-16) ─────────────────────────────

    pub async fn convert_draft_order(&self, order_id: &str) -> Result<OrderDTO> {
        let mut order = self.orders.retrieve(None, order_id).await?;

        if order.status != "draft" {
            return Err(Error::Conflict {
                code: "not_a_draft_order",
                message: format!(
                    "Order {} is not a draft order (status: {})",
                    order_id, order.status
                ),
            });
        }

        order.status = "pending".to_string();
        order.payment_status = "not_paid".to_string();
        order.fulfillment_status = "not_fulfilled".to_string();
        order.updated_at = Utc::now();

        let updated = self.orders.update(None, order_id, order).await?;
        self.assemble_order_dto(&updated, updated.version).await
    }

    // ── Timeline Reconstruction (P6-18) ─────────────────────────────────────

    pub async fn get_order_timeline(&self, order_id: &str) -> Result<Vec<OrderTimelineEventDTO>> {
        let order = self.get_order(order_id).await?;
        let mut events = Vec::new();

        // 1. Order created event
        events.push(OrderTimelineEventDTO {
            id: format!("evt_created_{}", order.id),
            order_id: order.id.clone(),
            event_type: "order_created".to_string(),
            description: format!(
                "Order #{} created with status {}",
                order.display_id, order.status
            ),
            actor_id: order.customer_id.clone(),
            created_at: order.created_at,
            metadata: None,
        });

        // 2. Order changes and actions
        let chgs = self
            .order_changes
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order.id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for chg in chgs {
            let chg_id = chg.id.to_string();
            let actions = self
                .order_change_actions
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("order_change_id", FilterOp::Eq(chg_id.clone()))
                        .with_predicate("deleted_at", FilterOp::IsNull)
                        .with_pagination(0, 100),
                )
                .await?;

            for act in actions {
                events.push(OrderTimelineEventDTO {
                    id: format!("evt_act_{}", act.id),
                    order_id: order.id.clone(),
                    event_type: format!("change_action_{}", act.action.to_lowercase()),
                    description: format!("Order change action: {}", act.action),
                    actor_id: chg.requested_by.clone(),
                    created_at: act.created_at,
                    metadata: act.details.clone(),
                });
            }

            if chg.status == "confirmed" {
                events.push(OrderTimelineEventDTO {
                    id: format!("evt_chg_conf_{}", chg.id),
                    order_id: order.id.clone(),
                    event_type: "order_change_confirmed".to_string(),
                    description: format!("Order change confirmed (v{})", chg.version),
                    actor_id: chg.confirmed_by.clone(),
                    created_at: chg.confirmed_at.unwrap_or(chg.updated_at),
                    metadata: None,
                });
            }
        }

        // 3. Transactions (Payment captured / refund issued)
        let txs = self
            .transactions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order.id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for tx in txs {
            let ev_type = match tx.reference.as_str() {
                "refund" => "refund_issued",
                _ => "payment_captured",
            };
            events.push(OrderTimelineEventDTO {
                id: format!("evt_tx_{}", tx.id),
                order_id: order.id.clone(),
                event_type: ev_type.to_string(),
                description: format!(
                    "Transaction {} of {} {}",
                    tx.reference, tx.amount, tx.currency_code
                ),
                actor_id: None,
                created_at: tx.created_at,
                metadata: tx.metadata.clone(),
            });
        }

        // 4. Returns
        let rets = self
            .returns
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order.id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for ret in rets {
            events.push(OrderTimelineEventDTO {
                id: format!("evt_ret_{}", ret.id),
                order_id: order.id.clone(),
                event_type: "return_requested".to_string(),
                description: format!("Return requested in status {}", ret.status),
                actor_id: None,
                created_at: ret.created_at,
                metadata: ret.metadata.clone(),
            });
        }

        // 5. Claims
        let clms = self
            .claims
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order.id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for clm in clms {
            events.push(OrderTimelineEventDTO {
                id: format!("evt_clm_{}", clm.id),
                order_id: order.id.clone(),
                event_type: "claim_created".to_string(),
                description: format!("Claim created of type {}", clm.r#type),
                actor_id: clm.created_by.clone(),
                created_at: clm.created_at,
                metadata: clm.metadata.clone(),
            });
        }

        // 6. Exchanges
        let excs = self
            .exchanges
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("order_id", FilterOp::Eq(order.id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for exc in excs {
            events.push(OrderTimelineEventDTO {
                id: format!("evt_exc_{}", exc.id),
                order_id: order.id.clone(),
                event_type: "exchange_created".to_string(),
                description: format!("Exchange created in status {}", exc.status),
                actor_id: exc.created_by.clone(),
                created_at: exc.created_at,
                metadata: exc.metadata.clone(),
            });
        }

        // Chronological sort
        events.sort_by_key(|e| e.created_at);

        Ok(events)
    }

    // ── Transaction Ledger (P5-05) ──────────────────────────────────────────

    pub async fn add_transaction(
        &self,
        order_id: &str,
        input: CreateTransactionInput,
    ) -> Result<TransactionDTO> {
        let order = self.get_order(order_id).await?;

        let tx = Transaction {
            id: Id::new(),
            order_id: order_id.to_string(),
            amount: input.amount,
            currency_code: input.currency_code.to_lowercase(),
            reference: input.reference,
            reference_id: input.reference_id,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.transactions.create(None, tx).await?;
        let dto = Self::to_tx_dto(created);

        // Recompute summary with updated paid/refunded totals
        self.recompute_order_summary(order_id, order.version)
            .await?;

        // Update payment_status independently from fulfillment_status (P5-05)
        let updated_summary = self
            .recompute_order_summary(order_id, order.version)
            .await?;
        let payment_status = if updated_summary.paid_total >= updated_summary.current_order_total
            && updated_summary.current_order_total > Decimal::ZERO
        {
            "captured".to_string()
        } else if updated_summary.paid_total > Decimal::ZERO {
            "partially_captured".to_string()
        } else {
            "not_paid".to_string()
        };

        self.update_order(
            order_id,
            UpdateOrderInput {
                payment_status: Some(payment_status),
                ..Default::default()
            },
        )
        .await?;

        Ok(dto)
    }

    pub async fn create_credit_line(
        &self,
        order_id: &str,
        amount: Decimal,
        reference: Option<String>,
        reference_id: Option<String>,
    ) -> Result<CreditLineDTO> {
        let order = self.get_order(order_id).await?;

        let cl = CreditLine {
            id: Id::new(),
            order_id: order_id.to_string(),
            reference,
            reference_id,
            amount,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.credit_lines.create(None, cl).await?;
        self.recompute_order_summary(order_id, order.version)
            .await?;
        Ok(Self::to_cl_dto(created))
    }

    // ── Internal Helpers ────────────────────────────────────────────────────

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

    async fn add_line_item_internal(
        &self,
        order_id: &str,
        input: CreateLineItemInput,
        version: i32,
    ) -> Result<LineItem> {
        let line_item = LineItem {
            id: Id::new(),
            title: input.title,
            subtitle: input.subtitle,
            thumbnail: input.thumbnail,
            quantity: input.quantity,
            unit_price: input.unit_price,
            is_tax_inclusive: input.is_tax_inclusive.unwrap_or(false),
            compare_at_unit_price: input.compare_at_unit_price,
            requires_shipping: input.requires_shipping.unwrap_or(true),
            is_discountable: input.is_discountable.unwrap_or(true),
            is_custom_price: input.is_custom_price.unwrap_or(false),
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
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let item_id_str = line_item.id.to_string();
        let created_li = self.line_items.create(None, line_item).await?;

        let order_item = OrderItem {
            id: Id::new(),
            order_id: order_id.to_string(),
            item_id: item_id_str,
            version,
            quantity: input.quantity,
            fulfilled_quantity: 0,
            shipped_quantity: 0,
            return_requested_quantity: 0,
            return_received_quantity: 0,
            return_dismissed_quantity: 0,
            written_off_quantity: 0,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        self.order_items.create(None, order_item).await?;
        Ok(created_li)
    }

    async fn add_shipping_method_internal(
        &self,
        order_id: &str,
        input: CreateShippingMethodInput,
        version: i32,
    ) -> Result<ShippingMethod> {
        let sm = ShippingMethod {
            id: Id::new(),
            name: input.name,
            description: input.description,
            amount: input.amount,
            is_tax_inclusive: input.is_tax_inclusive.unwrap_or(false),
            shipping_option_id: input.shipping_option_id,
            data: input.data,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let sm_id_str = sm.id.to_string();
        let created_sm = self.shipping_methods.create(None, sm).await?;

        let osm = OrderShippingMethod {
            id: Id::new(),
            order_id: order_id.to_string(),
            shipping_method_id: sm_id_str,
            version,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        self.order_shipping_methods.create(None, osm).await?;
        Ok(created_sm)
    }

    fn to_address_dto(a: &Address) -> AddressDTO {
        AddressDTO {
            id: a.id.to_string(),
            customer_id: a.customer_id.clone(),
            company: a.company.clone(),
            first_name: a.first_name.clone(),
            last_name: a.last_name.clone(),
            address_1: a.address_1.clone(),
            address_2: a.address_2.clone(),
            city: a.city.clone(),
            country_code: a.country_code.clone(),
            province: a.province.clone(),
            postal_code: a.postal_code.clone(),
            phone: a.phone.clone(),
            metadata: a.metadata.clone(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }

    fn to_adj_dto(a: LineItemAdjustment) -> LineItemAdjustmentDTO {
        LineItemAdjustmentDTO {
            id: a.id.to_string(),
            item_id: a.item_id,
            code: a.code,
            amount: a.amount,
            description: a.description,
            promotion_id: a.promotion_id,
            metadata: a.metadata,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }

    fn to_tax_dto(t: LineItemTaxLine) -> LineItemTaxLineDTO {
        LineItemTaxLineDTO {
            id: t.id.to_string(),
            item_id: t.item_id,
            code: t.code,
            rate: t.rate,
            provider_id: t.provider_id,
            tax_rate_id: t.tax_rate_id,
            description: t.description,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn to_sm_adj_dto(a: ShippingMethodAdjustment) -> ShippingMethodAdjustmentDTO {
        ShippingMethodAdjustmentDTO {
            id: a.id.to_string(),
            shipping_method_id: a.shipping_method_id,
            code: a.code,
            amount: a.amount,
            description: a.description,
            promotion_id: a.promotion_id,
            metadata: a.metadata,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }

    fn to_sm_tax_dto(t: ShippingMethodTaxLine) -> ShippingMethodTaxLineDTO {
        ShippingMethodTaxLineDTO {
            id: t.id.to_string(),
            shipping_method_id: t.shipping_method_id,
            code: t.code,
            rate: t.rate,
            provider_id: t.provider_id,
            tax_rate_id: t.tax_rate_id,
            description: t.description,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn to_summary_dto(s: &OrderSummary) -> OrderSummaryDTO {
        OrderSummaryDTO {
            id: s.id.to_string(),
            order_id: s.order_id.clone(),
            version: s.version,
            paid_total: s.paid_total,
            refunded_total: s.refunded_total,
            credit_line_total: s.credit_line_total,
            raw_difference: s.raw_difference,
            pending_difference: s.pending_difference,
            current_order_total: s.current_order_total,
            original_order_total: s.original_order_total,
            accounting_total: s.accounting_total,
            metadata: s.metadata.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }

    fn to_tx_dto(t: Transaction) -> TransactionDTO {
        TransactionDTO {
            id: t.id.to_string(),
            order_id: t.order_id,
            amount: t.amount,
            currency_code: t.currency_code,
            reference: t.reference,
            reference_id: t.reference_id,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn to_cl_dto(c: CreditLine) -> CreditLineDTO {
        CreditLineDTO {
            id: c.id.to_string(),
            order_id: c.order_id,
            reference: c.reference,
            reference_id: c.reference_id,
            amount: c.amount,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }

    fn to_order_change_dto(c: &OrderChange, actions: Vec<OrderChangeActionDTO>) -> OrderChangeDTO {
        OrderChangeDTO {
            id: c.id.to_string(),
            order_id: c.order_id.clone(),
            version: c.version,
            change_type: c.change_type.clone(),
            status: c.status.clone(),
            actions,
            requested_by: c.requested_by.clone(),
            requested_at: c.requested_at,
            confirmed_by: c.confirmed_by.clone(),
            confirmed_at: c.confirmed_at,
            declined_by: c.declined_by.clone(),
            declined_at: c.declined_at,
            canceled_by: c.canceled_by.clone(),
            canceled_at: c.canceled_at,
            metadata: c.metadata.clone(),
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }

    fn to_action_dto(a: &OrderChangeAction) -> OrderChangeActionDTO {
        OrderChangeActionDTO {
            id: a.id.to_string(),
            order_change_id: a.order_change_id.clone(),
            order_id: a.order_id.clone(),
            action: a.action.clone(),
            reference: a.reference.clone(),
            reference_id: a.reference_id.clone(),
            amount: a.amount,
            applied: a.applied,
            details: a.details.clone(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }

    pub async fn pseudonymize_customer_orders(&self, customer_id: &str) -> Result<()> {
        let orders = self
            .orders
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("customer_id", FilterOp::Eq(customer_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        for mut order in orders {
            order.email = Some("erased@pseudonym.local".to_string());
            if let Some(s_id) = &order.shipping_address_id {
                if let Ok(mut addr) = self.addresses.retrieve(None, s_id).await {
                    addr.first_name = Some("[ERASED]".to_string());
                    addr.last_name = Some("[ERASED]".to_string());
                    addr.address_1 = Some("[ERASED]".to_string());
                    addr.address_2 = None;
                    addr.phone = Some("[ERASED]".to_string());
                    self.addresses.update(None, s_id, addr).await?;
                }
            }
            if let Some(b_id) = &order.billing_address_id {
                if let Ok(mut addr) = self.addresses.retrieve(None, b_id).await {
                    addr.first_name = Some("[ERASED]".to_string());
                    addr.last_name = Some("[ERASED]".to_string());
                    addr.address_1 = Some("[ERASED]".to_string());
                    addr.address_2 = None;
                    addr.phone = Some("[ERASED]".to_string());
                    self.addresses.update(None, b_id, addr).await?;
                }
            }
            self.orders
                .update(None, &order.id.to_string(), order)
                .await?;
        }
        Ok(())
    }
}
