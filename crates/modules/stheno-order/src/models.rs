//! Order entity models (DATA_MODEL §2.4, TASKS P5-01, P13-15).
//! 23 entities in schema `order`.
//! Strictly `pub(crate)` (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

// 1. Order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Order {
    pub id: Id<Order>,
    pub display_id: i64,
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address_id: Option<String>,
    pub billing_address_id: Option<String>,
    pub status: String, // "pending", "completed", "draft", "archived", "canceled", "requires_action"
    pub payment_status: String, // "not_paid", "awaiting", "authorized", "partially_authorized", "captured", "partially_captured", "partially_refunded", "refunded", "canceled", "requires_action"
    pub fulfillment_status: String, // "not_fulfilled", "partially_fulfilled", "fulfilled", "partially_shipped", "shipped", "partially_delivered", "delivered", "canceled"
    pub version: i32,
    pub is_draft_order: bool,
    pub metadata: Option<serde_json::Value>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn get_cod_amounts(&self) -> Option<crate::dto::CodAmounts> {
        let meta = self.metadata.as_ref()?;
        let cod = meta.get("cod_amounts")?;
        serde_json::from_value(cod.clone()).ok()
    }

    pub fn set_cod_amounts(&mut self, amounts: crate::dto::CodAmounts) {
        let mut map = match self.metadata.take() {
            Some(serde_json::Value::Object(m)) => m,
            _ => serde_json::Map::new(),
        };
        map.insert(
            "cod_amounts".to_string(),
            serde_json::to_value(amounts).unwrap_or_default(),
        );
        self.metadata = Some(serde_json::Value::Object(map));
    }
}

// 2. OrderItem (Versioned Snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OrderItem {
    pub id: Id<OrderItem>,
    pub order_id: String,
    pub item_id: String,
    pub version: i32,
    pub quantity: i32,
    pub fulfilled_quantity: i32,
    pub shipped_quantity: i32,
    pub return_requested_quantity: i32,
    pub return_received_quantity: i32,
    pub return_dismissed_quantity: i32,
    pub written_off_quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 3. LineItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItem {
    pub id: Id<LineItem>,
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnail: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_tax_inclusive: bool,
    pub compare_at_unit_price: Option<Decimal>,
    pub requires_shipping: bool,
    pub is_discountable: bool,
    pub is_custom_price: bool,
    pub variant_id: Option<String>,
    pub product_id: Option<String>,
    pub product_title: Option<String>,
    pub product_description: Option<String>,
    pub product_subtitle: Option<String>,
    pub product_type: Option<String>,
    pub product_collection: Option<String>,
    pub product_handle: Option<String>,
    pub variant_sku: Option<String>,
    pub variant_barcode: Option<String>,
    pub variant_title: Option<String>,
    pub variant_option_values: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 4. LineItemAdjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItemAdjustment {
    pub id: Id<LineItemAdjustment>,
    pub item_id: String,
    pub code: Option<String>,
    pub amount: Decimal,
    pub description: Option<String>,
    pub promotion_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 5. LineItemTaxLine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItemTaxLine {
    pub id: Id<LineItemTaxLine>,
    pub item_id: String,
    pub code: String,
    pub rate: Decimal,
    pub provider_id: Option<String>,
    pub tax_rate_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 6. OrderShippingMethod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OrderShippingMethod {
    pub id: Id<OrderShippingMethod>,
    pub order_id: String,
    pub shipping_method_id: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 7. ShippingMethod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethod {
    pub id: Id<ShippingMethod>,
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub is_tax_inclusive: bool,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 8. ShippingMethodAdjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethodAdjustment {
    pub id: Id<ShippingMethodAdjustment>,
    pub shipping_method_id: String,
    pub code: Option<String>,
    pub amount: Decimal,
    pub description: Option<String>,
    pub promotion_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 9. ShippingMethodTaxLine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethodTaxLine {
    pub id: Id<ShippingMethodTaxLine>,
    pub shipping_method_id: String,
    pub code: String,
    pub rate: Decimal,
    pub provider_id: Option<String>,
    pub tax_rate_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 10. OrderSummary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OrderSummary {
    pub id: Id<OrderSummary>,
    pub order_id: String,
    pub version: i32,
    pub paid_total: Decimal,
    pub refunded_total: Decimal,
    pub credit_line_total: Decimal,
    pub raw_difference: Decimal,
    pub pending_difference: Decimal,
    pub current_order_total: Decimal,
    pub original_order_total: Decimal,
    pub accounting_total: Decimal,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 11. OrderChange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OrderChange {
    pub id: Id<OrderChange>,
    pub order_id: String,
    pub version: i32,
    pub change_type: String, // "edit", "claim", "exchange", "return", "transfer"
    pub status: String,      // "pending", "requested", "confirmed", "declined", "canceled"
    pub requested_by: Option<String>,
    pub requested_at: Option<DateTime<Utc>>,
    pub confirmed_by: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub declined_by: Option<String>,
    pub declined_at: Option<DateTime<Utc>>,
    pub canceled_by: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 12. OrderChangeAction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OrderChangeAction {
    pub id: Id<OrderChangeAction>,
    pub order_change_id: String,
    pub order_id: String,
    pub action: String, // "ITEM_ADD", "ITEM_UPDATE", "ITEM_REMOVE", "SHIPPING_ADD", "SHIPPING_REMOVE", "RETURN_ITEM", "RECEIVE_RETURN_ITEM", "WRITE_OFF_ITEM", "SHIP_ITEM", "FULFILL_ITEM", "CANCEL_ITEM_FULFILLMENT"
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Option<Decimal>,
    pub applied: bool,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 13. Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Transaction {
    pub id: Id<Transaction>,
    pub order_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub reference: String, // "payment", "refund", "credit_line"
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 14. CreditLine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CreditLine {
    pub id: Id<CreditLine>,
    pub order_id: String,
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Decimal,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 15. Address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Address {
    pub id: Id<Address>,
    pub customer_id: Option<String>,
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 16. Return
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Return {
    pub id: Id<Return>,
    pub order_id: String,
    pub claim_id: Option<String>,
    pub exchange_id: Option<String>,
    pub status: String, // "requested", "received", "partially_received", "canceled"
    pub refund_amount: Option<Decimal>,
    pub location_id: Option<String>,
    pub requested_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 17. ReturnItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReturnItem {
    pub id: Id<ReturnItem>,
    pub return_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub received_quantity: i32,
    pub damaged_quantity: i32,
    pub reason_id: Option<String>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 18. ReturnReason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReturnReason {
    pub id: Id<ReturnReason>,
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub parent_return_reason_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 19. Claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claim {
    pub id: Id<Claim>,
    pub order_id: String,
    pub r#type: String, // "refund", "replace"
    pub order_change_id: Option<String>,
    pub status: String, // "not_paid", "refunded", "canceled"
    pub refund_amount: Option<Decimal>,
    pub requested_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 20. ClaimItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClaimItem {
    pub id: Id<ClaimItem>,
    pub claim_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub reason: String, // "missing_item", "wrong_item", "production_failure", "other"
    pub raw_quantity: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 21. ClaimItemImage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClaimItemImage {
    pub id: Id<ClaimItemImage>,
    pub claim_item_id: String,
    pub url: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 22. Exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Exchange {
    pub id: Id<Exchange>,
    pub order_id: String,
    pub order_change_id: Option<String>,
    pub status: String, // "not_paid", "difference_refunded", "canceled"
    pub difference_due: Option<Decimal>,
    pub allow_backorder: bool,
    pub created_by: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 23. ExchangeItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExchangeItem {
    pub id: Id<ExchangeItem>,
    pub exchange_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Address => "addr",
    Claim => "claim",
    ClaimItem => "claim_item",
    ClaimItemImage => "claim_img",
    CreditLine => "cl",
    Exchange => "exch",
    ExchangeItem => "exch_item",
    LineItem => "item",
    LineItemAdjustment => "lia",
    LineItemTaxLine => "lit",
    Order => "order",
    OrderChange => "ordch",
    OrderChangeAction => "ordact",
    OrderItem => "orditem",
    OrderShippingMethod => "ord_sm",
    OrderSummary => "ordsum",
    Return => "ret",
    ReturnItem => "ret_item",
    ReturnReason => "ret_reason",
    ShippingMethod => "sm",
    ShippingMethodAdjustment => "sma",
    ShippingMethodTaxLine => "smt",
    Transaction => "tx",
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

impl_crud_entity!(Address, "order.address", "address");
impl_crud_entity!(Order, "order.order", "order");
impl_crud_entity!(LineItem, "order.line_item", "line_item");
impl_crud_entity!(OrderItem, "order.order_item", "order_item");
impl_crud_entity!(
    LineItemAdjustment,
    "order.line_item_adjustment",
    "line_item_adjustment"
);
impl_crud_entity!(
    LineItemTaxLine,
    "order.line_item_tax_line",
    "line_item_tax_line"
);
impl_crud_entity!(ShippingMethod, "order.shipping_method", "shipping_method");
impl_crud_entity!(
    OrderShippingMethod,
    "order.order_shipping_method",
    "order_shipping_method"
);
impl_crud_entity!(
    ShippingMethodAdjustment,
    "order.shipping_method_adjustment",
    "shipping_method_adjustment"
);
impl_crud_entity!(
    ShippingMethodTaxLine,
    "order.shipping_method_tax_line",
    "shipping_method_tax_line"
);
impl_crud_entity!(OrderSummary, "order.order_summary", "order_summary");
impl_crud_entity!(OrderChange, "order.order_change", "order_change");
impl_crud_entity!(
    OrderChangeAction,
    "order.order_change_action",
    "order_change_action"
);
impl_crud_entity!(Transaction, "order.transaction", "transaction");
impl_crud_entity!(CreditLine, "order.credit_line", "credit_line");
impl_crud_entity!(ReturnReason, "order.return_reason", "return_reason");
impl_crud_entity!(Return, "order.return", "return");
impl_crud_entity!(ReturnItem, "order.return_item", "return_item");
impl_crud_entity!(Claim, "order.claim", "claim");
impl_crud_entity!(ClaimItem, "order.claim_item", "claim_item");
impl_crud_entity!(ClaimItemImage, "order.claim_item_image", "claim_item_image");
impl_crud_entity!(Exchange, "order.exchange", "exchange");
impl_crud_entity!(ExchangeItem, "order.exchange_item", "exchange_item");
