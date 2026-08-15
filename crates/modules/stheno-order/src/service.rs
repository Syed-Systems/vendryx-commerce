//! Order service, order-change integration, versioning snapshot engine, and transaction ledger (DATA_MODEL §2.4, TASKS P5-01..P5-06).
#![allow(dead_code)]

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    Address, Claim, ClaimItem, ClaimItemImage, CreditLine, Exchange, ExchangeItem, LineItem,
    LineItemAdjustment, LineItemTaxLine, Order, OrderChange, OrderChangeAction, OrderItem,
    OrderShippingMethod, OrderSummary, Return, ReturnItem, ReturnReason, ShippingMethod,
    ShippingMethodAdjustment, ShippingMethodTaxLine, Transaction,
};

#[derive(Clone, Default)]
pub struct OrderService {
    orders: Arc<RwLock<HashMap<String, Order>>>,
    order_items: Arc<RwLock<HashMap<String, Vec<OrderItem>>>>, // key: order_id
    line_items: Arc<RwLock<HashMap<String, LineItem>>>,
    line_item_adjustments: Arc<RwLock<HashMap<String, Vec<LineItemAdjustment>>>>,
    line_item_tax_lines: Arc<RwLock<HashMap<String, Vec<LineItemTaxLine>>>>,
    order_shipping_methods: Arc<RwLock<HashMap<String, Vec<OrderShippingMethod>>>>,
    shipping_methods: Arc<RwLock<HashMap<String, ShippingMethod>>>,
    shipping_method_adjustments: Arc<RwLock<HashMap<String, Vec<ShippingMethodAdjustment>>>>,
    shipping_method_tax_lines: Arc<RwLock<HashMap<String, Vec<ShippingMethodTaxLine>>>>,
    order_summaries: Arc<RwLock<HashMap<String, Vec<OrderSummary>>>>,
    order_changes: Arc<RwLock<HashMap<String, OrderChange>>>,
    order_change_actions: Arc<RwLock<HashMap<String, Vec<OrderChangeAction>>>>, // key: order_change_id
    transactions: Arc<RwLock<HashMap<String, Vec<Transaction>>>>,
    credit_lines: Arc<RwLock<HashMap<String, Vec<CreditLine>>>>,
    addresses: Arc<RwLock<HashMap<String, Address>>>,
    returns: Arc<RwLock<HashMap<String, Return>>>,
    return_items: Arc<RwLock<HashMap<String, Vec<ReturnItem>>>>,
    return_reasons: Arc<RwLock<HashMap<String, ReturnReason>>>,
    claims: Arc<RwLock<HashMap<String, Claim>>>,
    claim_items: Arc<RwLock<HashMap<String, Vec<ClaimItem>>>>,
    claim_item_images: Arc<RwLock<HashMap<String, Vec<ClaimItemImage>>>>,
    exchanges: Arc<RwLock<HashMap<String, Exchange>>>,
    exchange_items: Arc<RwLock<HashMap<String, Vec<ExchangeItem>>>>,
    display_id_counter: Arc<AtomicI64>,
}

impl OrderService {
    pub fn new() -> Self {
        Self {
            display_id_counter: Arc::new(AtomicI64::new(1000)),
            ..Default::default()
        }
    }

    // ── Gap-Free Sequential display_id Generation (P5-06) ───────────────────

    pub fn next_display_id(&self) -> i64 {
        self.display_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    // ── Order Creation & Retrieval (P5-01) ───────────────────────────────────

    pub async fn create_order(&self, input: CreateOrderInput) -> Result<OrderDTO> {
        let id: Id<Order> = Id::new();
        let display_id = self.next_display_id();
        let now = Utc::now();

        let mut shipping_address_id = None;
        if let Some(addr_input) = input.shipping_address {
            let addr = self.create_address_internal(addr_input);
            shipping_address_id = Some(addr.id.to_string());
        }

        let mut billing_address_id = None;
        if let Some(addr_input) = input.billing_address {
            let addr = self.create_address_internal(addr_input);
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
        {
            let mut map = self.orders.write().unwrap();
            map.insert(order_id_str.clone(), order);
        }

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
        let order = {
            let map = self.orders.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order",
                id: order_id.to_string(),
            })?
        };

        self.assemble_order_dto(&order, order.version).await
    }

    pub async fn list_orders(&self, filter: OrderFilter) -> Result<(Vec<OrderDTO>, usize)> {
        let orders = {
            let map = self.orders.read().unwrap();
            let mut list: Vec<_> = map
                .values()
                .filter(|o| {
                    if o.deleted_at.is_some() {
                        return false;
                    }
                    if let Some(ref cid) = filter.customer_id {
                        if o.customer_id.as_ref() != Some(cid) {
                            return false;
                        }
                    }
                    if let Some(ref scid) = filter.sales_channel_id {
                        if o.sales_channel_id.as_ref() != Some(scid) {
                            return false;
                        }
                    }
                    if let Some(ref rid) = filter.region_id {
                        if o.region_id.as_ref() != Some(rid) {
                            return false;
                        }
                    }
                    if let Some(ref st) = filter.status {
                        if &o.status != st {
                            return false;
                        }
                    }
                    if let Some(ref pst) = filter.payment_status {
                        if &o.payment_status != pst {
                            return false;
                        }
                    }
                    if let Some(ref fst) = filter.fulfillment_status {
                        if &o.fulfillment_status != fst {
                            return false;
                        }
                    }
                    if let Some(did) = filter.display_id {
                        if o.display_id != did {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
            list
        };

        let total = orders.len();
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(20);

        let paged: Vec<_> = orders.into_iter().skip(offset).take(limit).collect();
        let mut dtos = Vec::new();
        for ord in paged {
            dtos.push(self.assemble_order_dto(&ord, ord.version).await?);
        }

        Ok((dtos, total))
    }

    pub async fn get_order_at_version(&self, order_id: &str, version: i32) -> Result<OrderDTO> {
        let order = {
            let map = self.orders.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order",
                id: order_id.to_string(),
            })?
        };

        self.assemble_order_dto(&order, version).await
    }

    pub async fn update_order(&self, order_id: &str, input: UpdateOrderInput) -> Result<OrderDTO> {
        let mut order = {
            let map = self.orders.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order",
                id: order_id.to_string(),
            })?
        };

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

        {
            let mut map = self.orders.write().unwrap();
            map.insert(order_id.to_string(), order.clone());
        }

        self.assemble_order_dto(&order, order.version).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<OrderDTO> {
        let mut order = {
            let map = self.orders.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order",
                id: order_id.to_string(),
            })?
        };

        if order.status == "canceled" {
            return Err(Error::Conflict {
                code: "order_already_canceled",
                message: "Order is already canceled".to_string(),
            });
        }

        order.status = "canceled".to_string();
        order.canceled_at = Some(Utc::now());
        order.updated_at = Utc::now();

        {
            let mut map = self.orders.write().unwrap();
            map.insert(order_id.to_string(), order.clone());
        }

        self.assemble_order_dto(&order, order.version).await
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

    // ── Versioned Snapshot Loading (P5-03) ──────────────────────────────────

    async fn assemble_order_dto(&self, order: &Order, version: i32) -> Result<OrderDTO> {
        let order_id = order.id.to_string();

        let shipping_address = if let Some(addr_id) = &order.shipping_address_id {
            let addrs = self.addresses.read().unwrap();
            addrs.get(addr_id).map(Self::to_address_dto)
        } else {
            None
        };

        let billing_address = if let Some(addr_id) = &order.billing_address_id {
            let addrs = self.addresses.read().unwrap();
            addrs.get(addr_id).map(Self::to_address_dto)
        } else {
            None
        };

        // Load items at specified version (P5-03)
        let mut items = Vec::new();
        {
            let oi_map = self.order_items.read().unwrap();
            if let Some(ois) = oi_map.get(&order_id) {
                let li_map = self.line_items.read().unwrap();
                let adj_map = self.line_item_adjustments.read().unwrap();
                let tax_map = self.line_item_tax_lines.read().unwrap();

                for oi in ois.iter().filter(|o| o.version == version) {
                    if let Some(li) = li_map.get(&oi.item_id) {
                        let adjs = adj_map.get(&li.id.to_string()).cloned().unwrap_or_default();
                        let taxes = tax_map.get(&li.id.to_string()).cloned().unwrap_or_default();
                        items.push(LineItemDTO {
                            id: li.id.to_string(),
                            title: li.title.clone(),
                            subtitle: li.subtitle.clone(),
                            thumbnail: li.thumbnail.clone(),
                            quantity: oi.quantity, // Snapshot quantity at version N
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
            }
        }

        // Load shipping methods at specified version
        let mut shipping_methods = Vec::new();
        {
            let osm_map = self.order_shipping_methods.read().unwrap();
            if let Some(osms) = osm_map.get(&order_id) {
                let sm_map = self.shipping_methods.read().unwrap();
                let adj_map = self.shipping_method_adjustments.read().unwrap();
                let tax_map = self.shipping_method_tax_lines.read().unwrap();

                for osm in osms.iter().filter(|o| o.version == version) {
                    if let Some(sm) = sm_map.get(&osm.shipping_method_id) {
                        let adjs = adj_map.get(&sm.id.to_string()).cloned().unwrap_or_default();
                        let taxes = tax_map.get(&sm.id.to_string()).cloned().unwrap_or_default();
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
            }
        }

        // Load summary at specified version
        let summary = {
            let s_map = self.order_summaries.read().unwrap();
            s_map
                .get(&order_id)
                .and_then(|sums| sums.iter().find(|s| s.version == version))
                .map(Self::to_summary_dto)
        };

        // Transactions & Credit Lines
        let transactions = {
            let tx_map = self.transactions.read().unwrap();
            tx_map
                .get(&order_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(Self::to_tx_dto)
                .collect()
        };

        let credit_lines = {
            let cl_map = self.credit_lines.read().unwrap();
            cl_map
                .get(&order_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(Self::to_cl_dto)
                .collect()
        };

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
        {
            let oi_map = self.order_items.read().unwrap();
            if let Some(ois) = oi_map.get(order_id) {
                let li_map = self.line_items.read().unwrap();
                let adj_map = self.line_item_adjustments.read().unwrap();
                let tax_map = self.line_item_tax_lines.read().unwrap();

                for oi in ois.iter().filter(|o| o.version == version) {
                    if let Some(li) = li_map.get(&oi.item_id) {
                        let item_gross = li.unit_price * Decimal::from(oi.quantity);
                        item_subtotal += item_gross;

                        if let Some(adjs) = adj_map.get(&li.id.to_string()) {
                            for adj in adjs {
                                total_discount += adj.amount;
                            }
                        }

                        if let Some(taxes) = tax_map.get(&li.id.to_string()) {
                            for t in taxes {
                                total_tax += (item_gross * t.rate) / dec!(100);
                            }
                        }
                    }
                }
            }
        }

        // Sum shipping at version N
        {
            let osm_map = self.order_shipping_methods.read().unwrap();
            if let Some(osms) = osm_map.get(order_id) {
                let sm_map = self.shipping_methods.read().unwrap();
                let adj_map = self.shipping_method_adjustments.read().unwrap();
                let tax_map = self.shipping_method_tax_lines.read().unwrap();

                for osm in osms.iter().filter(|o| o.version == version) {
                    if let Some(sm) = sm_map.get(&osm.shipping_method_id) {
                        shipping_subtotal += sm.amount;

                        if let Some(adjs) = adj_map.get(&sm.id.to_string()) {
                            for adj in adjs {
                                total_discount += adj.amount;
                            }
                        }

                        if let Some(taxes) = tax_map.get(&sm.id.to_string()) {
                            for t in taxes {
                                total_tax += (sm.amount * t.rate) / dec!(100);
                            }
                        }
                    }
                }
            }
        }

        let current_order_total =
            (item_subtotal + shipping_subtotal + total_tax - total_discount).max(Decimal::ZERO);

        // Calculate paid, refunded, credit line totals from Transactions
        let mut paid_total = Decimal::ZERO;
        let mut refunded_total = Decimal::ZERO;
        {
            let tx_map = self.transactions.read().unwrap();
            if let Some(txs) = tx_map.get(order_id) {
                for tx in txs {
                    if tx.reference == "payment" || tx.reference == "capture" {
                        paid_total += tx.amount;
                    } else if tx.reference == "refund" {
                        refunded_total += tx.amount;
                    }
                }
            }
        }

        let mut credit_line_total = Decimal::ZERO;
        {
            let cl_map = self.credit_lines.read().unwrap();
            if let Some(cls) = cl_map.get(order_id) {
                for cl in cls {
                    credit_line_total += cl.amount;
                }
            }
        }

        let accounting_total = paid_total - refunded_total + credit_line_total;
        let raw_difference = current_order_total - accounting_total;
        let pending_difference = raw_difference;

        let summary = OrderSummary {
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
        };

        let dto = Self::to_summary_dto(&summary);

        {
            let mut s_map = self.order_summaries.write().unwrap();
            let list = s_map.entry(order_id.to_string()).or_default();
            list.retain(|s| s.version != version);
            list.push(summary);
        }

        Ok(dto)
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
        };

        let change_id_str = change.id.to_string();
        {
            let mut map = self.order_changes.write().unwrap();
            map.insert(change_id_str.clone(), change.clone());
        }

        Ok(Self::to_order_change_dto(&change, vec![]))
    }

    pub async fn add_order_change_action(
        &self,
        change_id: &str,
        input: CreateOrderChangeActionInput,
    ) -> Result<OrderChangeActionDTO> {
        let change = {
            let map = self.order_changes.read().unwrap();
            map.get(change_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_change",
                id: change_id.to_string(),
            })?
        };

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
        };

        let dto = Self::to_action_dto(&action);

        {
            let mut act_map = self.order_change_actions.write().unwrap();
            act_map
                .entry(change_id.to_string())
                .or_default()
                .push(action);
        }

        Ok(dto)
    }

    /// Preview Order Change: evaluates pending actions against version-N snapshot in-memory (P5-02, DATA_MODEL §2.4 step 3).
    pub async fn preview_order_change(&self, change_id: &str) -> Result<OrderChangePreview> {
        let change = {
            let map = self.order_changes.read().unwrap();
            map.get(change_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_change",
                id: change_id.to_string(),
            })?
        };

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

        let actions = {
            let act_map = self.order_change_actions.read().unwrap();
            act_map.get(change_id).cloned().unwrap_or_default()
        };

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
                                let diff = new_amt - sm.amount;
                                let money_diff = stheno_types::Money::new(diff, currency);
                                delta = (delta + money_diff)?;
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
        let mut change = {
            let map = self.order_changes.read().unwrap();
            map.get(change_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_change",
                id: change_id.to_string(),
            })?
        };

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
        {
            let mut oi_map = self.order_items.write().unwrap();
            let ois = oi_map.entry(order_id.clone()).or_default();
            let version_n_items: Vec<OrderItem> = ois
                .iter()
                .filter(|o| o.version == old_version)
                .cloned()
                .collect();

            for mut item in version_n_items {
                item.id = Id::new();
                item.version = new_version;
                item.created_at = Utc::now();
                item.updated_at = Utc::now();
                ois.push(item);
            }
        }

        // Clone version N shipping methods to version N+1
        {
            let mut osm_map = self.order_shipping_methods.write().unwrap();
            let osms = osm_map.entry(order_id.clone()).or_default();
            let version_n_sms: Vec<OrderShippingMethod> = osms
                .iter()
                .filter(|o| o.version == old_version)
                .cloned()
                .collect();

            for mut sm in version_n_sms {
                sm.id = Id::new();
                sm.version = new_version;
                sm.created_at = Utc::now();
                sm.updated_at = Utc::now();
                osms.push(sm);
            }
        }

        // Apply actions to version N+1
        {
            let act_map = self.order_change_actions.read().unwrap();
            if let Some(actions) = act_map.get(change_id) {
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
                                        details.get("unit_price").and_then(|v| v.as_f64()).map(
                                            |f| {
                                                Decimal::from_str_exact(&f.to_string())
                                                    .unwrap_or(Decimal::ZERO)
                                            },
                                        )
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
                                {
                                    let mut li_map = self.line_items.write().unwrap();
                                    li_map.insert(line_item_id.clone(), line_item);
                                }

                                let mut oi_map = self.order_items.write().unwrap();
                                let ois = oi_map.entry(order_id.clone()).or_default();
                                ois.push(OrderItem {
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
                                });
                            }
                        }
                        "ITEM_REMOVE" => {
                            if let Some(ref_id) = &act.reference_id {
                                let mut oi_map = self.order_items.write().unwrap();
                                let ois = oi_map.entry(order_id.clone()).or_default();
                                ois.retain(|o| !(o.version == new_version && &o.item_id == ref_id));
                            }
                        }
                        "ITEM_UPDATE" => {
                            if let (Some(ref_id), Some(details)) = (&act.reference_id, &act.details)
                            {
                                if let Some(new_qty) =
                                    details.get("quantity").and_then(|v| v.as_i64())
                                {
                                    let mut oi_map = self.order_items.write().unwrap();
                                    let ois = oi_map.entry(order_id.clone()).or_default();
                                    if let Some(oi) = ois
                                        .iter_mut()
                                        .find(|o| o.version == new_version && &o.item_id == ref_id)
                                    {
                                        oi.quantity = new_qty as i32;
                                        oi.updated_at = Utc::now();
                                    }
                                }
                                if let Some(new_price) = details
                                    .get("unit_price")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<Decimal>().ok())
                                {
                                    let mut li_map = self.line_items.write().unwrap();
                                    if let Some(li) = li_map.get_mut(ref_id) {
                                        li.unit_price = new_price;
                                        li.updated_at = Utc::now();
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
                                {
                                    let mut sm_map = self.shipping_methods.write().unwrap();
                                    sm_map.insert(sm_id.clone(), sm);
                                }

                                let mut osm_map = self.order_shipping_methods.write().unwrap();
                                let osms = osm_map.entry(order_id.clone()).or_default();
                                osms.push(OrderShippingMethod {
                                    id: Id::new(),
                                    order_id: order_id.clone(),
                                    shipping_method_id: sm_id,
                                    version: new_version,
                                    created_at: Utc::now(),
                                    updated_at: Utc::now(),
                                });
                            }
                        }
                        "SHIPPING_UPDATE" => {
                            if let (Some(ref_id), Some(details)) = (&act.reference_id, &act.details)
                            {
                                if let Some(amt) = details
                                    .get("amount")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<Decimal>().ok())
                                {
                                    let mut sm_map = self.shipping_methods.write().unwrap();
                                    if let Some(sm) = sm_map.get_mut(ref_id) {
                                        sm.amount = amt;
                                        sm.updated_at = Utc::now();
                                    }
                                }
                            }
                        }
                        "SHIPPING_REMOVE" => {
                            if let Some(ref_id) = &act.reference_id {
                                let mut osm_map = self.order_shipping_methods.write().unwrap();
                                if let Some(osms) = osm_map.get_mut(&order_id) {
                                    osms.retain(|o| {
                                        !(o.version == new_version
                                            && &o.shipping_method_id == ref_id)
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Increment order.version
        {
            let mut map = self.orders.write().unwrap();
            if let Some(order) = map.get_mut(&order_id) {
                order.version = new_version;
                order.updated_at = Utc::now();
            }
        }

        // Recompute order_summary at N+1
        self.recompute_order_summary(&order_id, new_version).await?;

        // Update change status
        change.status = "confirmed".to_string();
        change.confirmed_by = confirmed_by.map(|s| s.to_string());
        change.confirmed_at = Some(Utc::now());
        change.updated_at = Utc::now();

        {
            let mut map = self.order_changes.write().unwrap();
            map.insert(change_id.to_string(), change);
        }

        self.get_order(&order_id).await
    }

    pub async fn cancel_order_change(&self, change_id: &str) -> Result<OrderChangeDTO> {
        let mut change = {
            let map = self.order_changes.read().unwrap();
            map.get(change_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_change",
                id: change_id.to_string(),
            })?
        };

        if change.status == "confirmed" || change.status == "canceled" {
            return Err(Error::Conflict {
                code: "order_change_immutable",
                message: format!("Cannot cancel order change in status {}", change.status),
            });
        }

        change.status = "canceled".to_string();
        change.canceled_at = Some(Utc::now());
        change.updated_at = Utc::now();

        {
            let mut map = self.order_changes.write().unwrap();
            map.insert(change_id.to_string(), change.clone());
        }

        let actions = {
            let act_map = self.order_change_actions.read().unwrap();
            act_map
                .get(change_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|a| Self::to_action_dto(&a))
                .collect()
        };

        Ok(Self::to_order_change_dto(&change, actions))
    }

    pub async fn decline_order_change(
        &self,
        change_id: &str,
        declined_by: Option<&str>,
        _reason: Option<&str>,
    ) -> Result<OrderChangeDTO> {
        let mut change = {
            let map = self.order_changes.read().unwrap();
            map.get(change_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_change",
                id: change_id.to_string(),
            })?
        };

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

        {
            let mut map = self.order_changes.write().unwrap();
            map.insert(change_id.to_string(), change.clone());
        }

        let actions = {
            let act_map = self.order_change_actions.read().unwrap();
            act_map
                .get(change_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|a| Self::to_action_dto(&a))
                .collect()
        };

        Ok(Self::to_order_change_dto(&change, actions))
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
        };

        let ret_id = ret.id.to_string();
        {
            let mut map = self.returns.write().unwrap();
            map.insert(ret_id.clone(), ret.clone());
        }

        self.get_return_dto(&ret_id).await
    }

    pub async fn add_return_item(&self, input: AddReturnItemInput) -> Result<ReturnItemDTO> {
        let ret = {
            let map = self.returns.read().unwrap();
            map.get(&input.return_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "return",
                    id: input.return_id.clone(),
                })?
        };

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
        };

        let dto = Self::to_return_item_dto(&item);
        {
            let mut map = self.return_items.write().unwrap();
            map.entry(input.return_id).or_default().push(item);
        }

        Ok(dto)
    }

    pub async fn receive_return_items(
        &self,
        return_id: &str,
        items: Vec<ReceiveReturnItemsInput>,
    ) -> Result<ReturnDTO> {
        let mut ret = {
            let map = self.returns.read().unwrap();
            map.get(return_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "return",
                id: return_id.to_string(),
            })?
        };

        if ret.status == "canceled" || ret.status == "completed" {
            return Err(Error::Conflict {
                code: "return_immutable",
                message: format!("Cannot receive items on return in status {}", ret.status),
            });
        }

        {
            let mut map = self.return_items.write().unwrap();
            if let Some(r_items) = map.get_mut(return_id) {
                for in_item in items {
                    if let Some(ri) = r_items.iter_mut().find(|r| r.item_id == in_item.item_id) {
                        ri.received_quantity += in_item.received_quantity;
                        if let Some(dmg) = in_item.damaged_quantity {
                            ri.damaged_quantity += dmg;
                        }
                        ri.updated_at = Utc::now();
                    }
                }
            }
        }

        ret.status = "received".to_string();
        ret.received_at = Some(Utc::now());
        ret.updated_at = Utc::now();

        {
            let mut map = self.returns.write().unwrap();
            map.insert(return_id.to_string(), ret);
        }

        self.get_return_dto(return_id).await
    }

    pub async fn confirm_return(
        &self,
        return_id: &str,
        refund_amount: Option<Decimal>,
    ) -> Result<ReturnDTO> {
        let mut ret = {
            let map = self.returns.read().unwrap();
            map.get(return_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "return",
                id: return_id.to_string(),
            })?
        };

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

        {
            let mut map = self.returns.write().unwrap();
            map.insert(return_id.to_string(), ret.clone());
        }

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
        let mut ret = {
            let map = self.returns.read().unwrap();
            map.get(return_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "return",
                id: return_id.to_string(),
            })?
        };

        if ret.status == "completed" {
            return Err(Error::Conflict {
                code: "return_already_completed",
                message: "Cannot cancel a completed return".to_string(),
            });
        }

        ret.status = "canceled".to_string();
        ret.canceled_at = Some(Utc::now());
        ret.updated_at = Utc::now();

        {
            let mut map = self.returns.write().unwrap();
            map.insert(return_id.to_string(), ret);
        }

        self.get_return_dto(return_id).await
    }

    pub async fn dismiss_return(&self, return_id: &str, reason: Option<&str>) -> Result<ReturnDTO> {
        let mut ret = {
            let map = self.returns.read().unwrap();
            map.get(return_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "return",
                id: return_id.to_string(),
            })?
        };

        ret.status = "dismissed".to_string();
        if let Some(r) = reason {
            let mut meta = ret.metadata.unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = meta.as_object_mut() {
                obj.insert("dismiss_reason".to_string(), serde_json::json!(r));
            }
            ret.metadata = Some(meta);
        }
        ret.updated_at = Utc::now();

        {
            let mut map = self.returns.write().unwrap();
            map.insert(return_id.to_string(), ret);
        }

        self.get_return_dto(return_id).await
    }

    pub async fn list_returns(&self, order_id: Option<&str>) -> Result<Vec<ReturnDTO>> {
        let ids: Vec<String> = {
            let ret_map = self.returns.read().unwrap();
            ret_map
                .values()
                .filter(|r| {
                    if let Some(oid) = order_id {
                        r.order_id == oid
                    } else {
                        true
                    }
                })
                .map(|r| r.id.to_string())
                .collect()
        };

        let mut list = Vec::new();
        for id in ids {
            list.push(self.get_return_dto(&id).await?);
        }
        list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(list)
    }

    pub async fn get_return_dto(&self, return_id: &str) -> Result<ReturnDTO> {
        let ret = {
            let map = self.returns.read().unwrap();
            map.get(return_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "return",
                id: return_id.to_string(),
            })?
        };

        let items = {
            let item_map = self.return_items.read().unwrap();
            item_map
                .get(return_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|i| Self::to_return_item_dto(&i))
                .collect()
        };

        Ok(ReturnDTO {
            id: ret.id.to_string(),
            order_id: ret.order_id,
            claim_id: ret.claim_id,
            exchange_id: ret.exchange_id,
            status: ret.status,
            refund_amount: ret.refund_amount,
            location_id: ret.location_id,
            items,
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

        let dto = ReturnReasonDTO {
            id: reason.id.to_string(),
            value: reason.value.clone(),
            label: reason.label.clone(),
            description: reason.description.clone(),
            parent_return_reason_id: None,
            metadata: None,
            created_at: reason.created_at,
            updated_at: reason.updated_at,
        };

        let mut map = self.return_reasons.write().unwrap();
        map.insert(reason.value.clone(), reason);

        Ok(dto)
    }

    pub async fn list_return_reasons(&self) -> Result<Vec<ReturnReasonDTO>> {
        let map = self.return_reasons.read().unwrap();
        let list = map
            .values()
            .filter(|r| r.deleted_at.is_none())
            .map(|r| ReturnReasonDTO {
                id: r.id.to_string(),
                value: r.value.clone(),
                label: r.label.clone(),
                description: r.description.clone(),
                parent_return_reason_id: r.parent_return_reason_id.clone(),
                metadata: r.metadata.clone(),
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();
        Ok(list)
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
        };

        let claim_id = claim.id.to_string();
        {
            let mut map = self.claims.write().unwrap();
            map.insert(claim_id.clone(), claim);
        }

        self.get_claim_dto(&claim_id).await
    }

    pub async fn add_claim_item(&self, input: AddClaimItemInput) -> Result<ClaimItemDTO> {
        let claim = {
            let map = self.claims.read().unwrap();
            map.get(&input.claim_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "claim",
                    id: input.claim_id.clone(),
                })?
        };

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
        };

        let item_id_str = item.id.to_string();

        if let Some(imgs) = input.images {
            let mut img_map = self.claim_item_images.write().unwrap();
            let list = img_map.entry(item_id_str.clone()).or_default();
            for img_url in imgs {
                list.push(ClaimItemImage {
                    id: Id::new(),
                    claim_item_id: item_id_str.clone(),
                    url: img_url,
                    metadata: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
        }

        let dto = self.to_claim_item_dto(&item).await;
        {
            let mut map = self.claim_items.write().unwrap();
            map.entry(input.claim_id).or_default().push(item);
        }

        Ok(dto)
    }

    pub async fn confirm_claim(&self, claim_id: &str) -> Result<ClaimDTO> {
        let mut claim = {
            let map = self.claims.read().unwrap();
            map.get(claim_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "claim",
                id: claim_id.to_string(),
            })?
        };

        claim.status = "confirmed".to_string();
        claim.updated_at = Utc::now();

        {
            let mut map = self.claims.write().unwrap();
            map.insert(claim_id.to_string(), claim.clone());
        }

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
        let mut claim = {
            let map = self.claims.read().unwrap();
            map.get(claim_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "claim",
                id: claim_id.to_string(),
            })?
        };

        claim.status = "canceled".to_string();
        claim.canceled_at = Some(Utc::now());
        claim.updated_at = Utc::now();

        {
            let mut map = self.claims.write().unwrap();
            map.insert(claim_id.to_string(), claim);
        }

        self.get_claim_dto(claim_id).await
    }

    pub async fn get_claim_dto(&self, claim_id: &str) -> Result<ClaimDTO> {
        let claim = {
            let map = self.claims.read().unwrap();
            map.get(claim_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "claim",
                id: claim_id.to_string(),
            })?
        };

        let raw_items = {
            let map = self.claim_items.read().unwrap();
            map.get(claim_id).cloned().unwrap_or_default()
        };

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
        let images = {
            let map = self.claim_item_images.read().unwrap();
            map.get(&item_id_str)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|img| ClaimItemImageDTO {
                    id: img.id.to_string(),
                    claim_item_id: img.claim_item_id,
                    url: img.url,
                    metadata: img.metadata,
                    created_at: img.created_at,
                    updated_at: img.updated_at,
                })
                .collect()
        };

        ClaimItemDTO {
            id: item_id_str,
            claim_id: item.claim_id.clone(),
            item_id: item.item_id.clone(),
            quantity: item.quantity,
            reason: item.reason.clone(),
            raw_quantity: item.raw_quantity.clone(),
            images,
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
        };

        let exchange_id = exchange.id.to_string();
        {
            let mut map = self.exchanges.write().unwrap();
            map.insert(exchange_id.clone(), exchange);
        }

        self.get_exchange_dto(&exchange_id).await
    }

    pub async fn add_exchange_item(&self, input: AddExchangeItemInput) -> Result<ExchangeItemDTO> {
        let exchange = {
            let map = self.exchanges.read().unwrap();
            map.get(&input.exchange_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "exchange",
                    id: input.exchange_id.clone(),
                })?
        };

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
        };

        let dto = Self::to_exchange_item_dto(&item);
        {
            let mut map = self.exchange_items.write().unwrap();
            map.entry(input.exchange_id).or_default().push(item);
        }

        Ok(dto)
    }

    pub async fn confirm_exchange(
        &self,
        exchange_id: &str,
        difference_due: Option<Decimal>,
    ) -> Result<ExchangeDTO> {
        let mut exchange = {
            let map = self.exchanges.read().unwrap();
            map.get(exchange_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "exchange",
                    id: exchange_id.to_string(),
                })?
        };

        exchange.status = "confirmed".to_string();
        exchange.difference_due = difference_due;
        exchange.updated_at = Utc::now();

        {
            let mut map = self.exchanges.write().unwrap();
            map.insert(exchange_id.to_string(), exchange);
        }

        self.get_exchange_dto(exchange_id).await
    }

    pub async fn cancel_exchange(&self, exchange_id: &str) -> Result<ExchangeDTO> {
        let mut exchange = {
            let map = self.exchanges.read().unwrap();
            map.get(exchange_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "exchange",
                    id: exchange_id.to_string(),
                })?
        };

        exchange.status = "canceled".to_string();
        exchange.canceled_at = Some(Utc::now());
        exchange.updated_at = Utc::now();

        {
            let mut map = self.exchanges.write().unwrap();
            map.insert(exchange_id.to_string(), exchange);
        }

        self.get_exchange_dto(exchange_id).await
    }

    pub async fn get_exchange_dto(&self, exchange_id: &str) -> Result<ExchangeDTO> {
        let exchange = {
            let map = self.exchanges.read().unwrap();
            map.get(exchange_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "exchange",
                    id: exchange_id.to_string(),
                })?
        };

        let items = {
            let map = self.exchange_items.read().unwrap();
            map.get(exchange_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|i| Self::to_exchange_item_dto(&i))
                .collect()
        };

        Ok(ExchangeDTO {
            id: exchange.id.to_string(),
            order_id: exchange.order_id,
            order_change_id: exchange.order_change_id,
            status: exchange.status,
            allow_backorder: exchange.allow_backorder,
            difference_due: exchange.difference_due,
            items,
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
        let mut order = {
            let map = self.orders.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order",
                id: order_id.to_string(),
            })?
        };

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

        {
            let mut map = self.orders.write().unwrap();
            map.insert(order_id.to_string(), order.clone());
        }

        self.assemble_order_dto(&order, order.version).await
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
        {
            let chg_map = self.order_changes.read().unwrap();
            let act_map = self.order_change_actions.read().unwrap();

            for chg in chg_map.values().filter(|c| c.order_id == order.id) {
                let chg_id = chg.id.to_string();
                if let Some(actions) = act_map.get(&chg_id) {
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
        }

        // 3. Transactions (Payment captured / refund issued)
        {
            let tx_map = self.transactions.read().unwrap();
            if let Some(txs) = tx_map.get(&order.id) {
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
            }
        }

        // 4. Returns
        {
            let ret_map = self.returns.read().unwrap();
            for ret in ret_map.values().filter(|r| r.order_id == order.id) {
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
        }

        // 5. Claims
        {
            let clm_map = self.claims.read().unwrap();
            for clm in clm_map.values().filter(|c| c.order_id == order.id) {
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
        }

        // 6. Exchanges
        {
            let exc_map = self.exchanges.read().unwrap();
            for exc in exc_map.values().filter(|e| e.order_id == order.id) {
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
        };

        let dto = Self::to_tx_dto(tx.clone());

        {
            let mut map = self.transactions.write().unwrap();
            map.entry(order_id.to_string()).or_default().push(tx);
        }

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

    // ── Internal Helpers ────────────────────────────────────────────────────

    fn create_address_internal(&self, input: CreateAddressInput) -> Address {
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

        let mut map = self.addresses.write().unwrap();
        map.insert(addr.id.to_string(), addr.clone());
        addr
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
        {
            let mut map = self.line_items.write().unwrap();
            map.insert(item_id_str.clone(), line_item.clone());
        }

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
        };

        {
            let mut map = self.order_items.write().unwrap();
            map.entry(order_id.to_string())
                .or_default()
                .push(order_item);
        }

        Ok(line_item)
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
        {
            let mut map = self.shipping_methods.write().unwrap();
            map.insert(sm_id_str.clone(), sm.clone());
        }

        let osm = OrderShippingMethod {
            id: Id::new(),
            order_id: order_id.to_string(),
            shipping_method_id: sm_id_str,
            version,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        {
            let mut map = self.order_shipping_methods.write().unwrap();
            map.entry(order_id.to_string()).or_default().push(osm);
        }

        Ok(sm)
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
        let mut orders_map = self.orders.write().unwrap();
        let mut addresses_map = self.addresses.write().unwrap();

        for order in orders_map.values_mut() {
            if order.customer_id.as_deref() == Some(customer_id) {
                order.email = Some("erased@pseudonym.local".to_string());
                if let Some(ref s_id) = order.shipping_address_id {
                    if let Some(addr) = addresses_map.get_mut(s_id) {
                        addr.first_name = Some("[ERASED]".to_string());
                        addr.last_name = Some("[ERASED]".to_string());
                        addr.address_1 = Some("[ERASED]".to_string());
                        addr.address_2 = None;
                        addr.phone = Some("[ERASED]".to_string());
                    }
                }
                if let Some(ref b_id) = order.billing_address_id {
                    if let Some(addr) = addresses_map.get_mut(b_id) {
                        addr.first_name = Some("[ERASED]".to_string());
                        addr.last_name = Some("[ERASED]".to_string());
                        addr.address_1 = Some("[ERASED]".to_string());
                        addr.address_2 = None;
                        addr.phone = Some("[ERASED]".to_string());
                    }
                }
            }
        }
        Ok(())
    }
}
