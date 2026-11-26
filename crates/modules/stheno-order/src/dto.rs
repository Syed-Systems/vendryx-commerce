//! DTOs and input types for `stheno-order` (DATA_MODEL §2.4, TASKS P5-01).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ── 1. Order DTOs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDTO {
    pub id: String,
    pub display_id: i64,
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address: Option<AddressDTO>,
    pub billing_address: Option<AddressDTO>,
    pub items: Vec<LineItemDTO>,
    pub shipping_methods: Vec<ShippingMethodDTO>,
    pub summary: Option<OrderSummaryDTO>,
    pub cod_amounts: Option<CodAmounts>,
    pub transactions: Vec<TransactionDTO>,
    pub credit_lines: Vec<CreditLineDTO>,
    pub status: String,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub version: i32,
    pub is_draft_order: bool,
    pub metadata: Option<serde_json::Value>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shopper-submitted reference for a manual order payment. Submission is never evidence
/// that funds were received; only a staff review can change this state to verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPaymentProofDTO {
    pub id: String,
    pub order_id: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub proof_asset_url: Option<String>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitOrderPaymentProofInput {
    pub order_id: String,
    pub customer_id: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub proof_asset_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewOrderPaymentProofInput {
    pub proof_id: String,
    pub reviewer_id: String,
    pub decision: String,
    pub note: Option<String>,
}

/// Three distinct COD amounts carried on an order (P24-02, BP8):
/// - order_total: gross order charge invoiced to customer
/// - collected_amount: cash collected by courier rider
/// - remitted_amount: net payout transferred by courier to merchant
///
/// Never conflated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodAmounts {
    pub order_total: Decimal,
    pub collected_amount: Decimal,
    pub remitted_amount: Decimal,
    pub courier_fee: Decimal,
}

impl CodAmounts {
    pub fn new(
        order_total: Decimal,
        collected: Decimal,
        remitted: Decimal,
        courier_fee: Decimal,
    ) -> Result<Self, stheno_types::Error> {
        if remitted + courier_fee != collected {
            return Err(stheno_types::Error::Invalid {
                code: "cod_amounts_mismatch",
                message: format!(
                    "courier remittance ({remitted}) + courier fee ({courier_fee}) does not balance collected amount ({collected})"
                ),
                field: Some("cod_amounts".to_string()),
            });
        }
        Ok(Self {
            order_total,
            collected_amount: collected,
            remitted_amount: remitted,
            courier_fee,
        })
    }

    pub fn order_total(&self) -> Decimal {
        self.order_total
    }

    pub fn collected_amount(&self) -> Decimal {
        self.collected_amount
    }

    pub fn remitted_amount(&self) -> Decimal {
        self.remitted_amount
    }

    pub fn courier_fee(&self) -> Decimal {
        self.courier_fee
    }

    /// Asserts that order total, collected amount, and remitted amount are distinct and never conflated.
    pub fn assert_never_conflated(&self) -> Result<(), stheno_types::Error> {
        if self.courier_fee > Decimal::ZERO && self.collected_amount == self.remitted_amount {
            return Err(stheno_types::Error::Invalid {
                code: "cod_amounts_conflated",
                message:
                    "remitted amount conflated with collected amount despite courier fee deduction"
                        .to_string(),
                field: Some("remitted_amount".to_string()),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrderInput {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub items: Option<Vec<CreateLineItemInput>>,
    pub shipping_methods: Option<Vec<CreateShippingMethodInput>>,
    pub is_draft_order: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateOrderInput {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── 2. OrderItem DTOs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemDTO {
    pub id: String,
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
}

// ── 3. LineItem & Adjustments DTOs ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemDTO {
    pub id: String,
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
    pub adjustments: Vec<LineItemAdjustmentDTO>,
    pub tax_lines: Vec<LineItemTaxLineDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateLineItemInput {
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnail: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_tax_inclusive: Option<bool>,
    pub compare_at_unit_price: Option<Decimal>,
    pub requires_shipping: Option<bool>,
    pub is_discountable: Option<bool>,
    pub is_custom_price: Option<bool>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemAdjustmentDTO {
    pub id: String,
    pub item_id: String,
    pub code: Option<String>,
    pub amount: Decimal,
    pub description: Option<String>,
    pub promotion_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemTaxLineDTO {
    pub id: String,
    pub item_id: String,
    pub code: String,
    pub rate: Decimal,
    pub provider_id: Option<String>,
    pub tax_rate_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── 4. ShippingMethod DTOs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodDTO {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub is_tax_inclusive: bool,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub adjustments: Vec<ShippingMethodAdjustmentDTO>,
    pub tax_lines: Vec<ShippingMethodTaxLineDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateShippingMethodInput {
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub is_tax_inclusive: Option<bool>,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodAdjustmentDTO {
    pub id: String,
    pub shipping_method_id: String,
    pub code: Option<String>,
    pub amount: Decimal,
    pub description: Option<String>,
    pub promotion_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodTaxLineDTO {
    pub id: String,
    pub shipping_method_id: String,
    pub code: String,
    pub rate: Decimal,
    pub provider_id: Option<String>,
    pub tax_rate_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── 5. OrderSummary DTO ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSummaryDTO {
    pub id: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChangePreview {
    pub current_total: stheno_types::Money,
    pub difference_total: stheno_types::Money,
    pub new_total: stheno_types::Money,
    pub actions_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChangeDTO {
    pub id: String,
    pub order_id: String,
    pub version: i32,
    pub change_type: String,
    pub status: String,
    pub actions: Vec<OrderChangeActionDTO>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderChangeInput {
    pub order_id: String,
    pub change_type: String,
    pub requested_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChangeActionDTO {
    pub id: String,
    pub order_change_id: String,
    pub order_id: String,
    pub action: String,
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Option<Decimal>,
    pub applied: bool,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderChangeActionInput {
    pub action: String,
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Option<Decimal>,
    pub details: Option<serde_json::Value>,
}

// ── 7. Transaction & CreditLine DTOs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDTO {
    pub id: String,
    pub order_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub reference: String,
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionInput {
    pub amount: Decimal,
    pub currency_code: String,
    pub reference: String,
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditLineDTO {
    pub id: String,
    pub order_id: String,
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Decimal,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreditLineInput {
    pub reference: Option<String>,
    pub reference_id: Option<String>,
    pub amount: Decimal,
    pub metadata: Option<serde_json::Value>,
}

// ── 8. Address DTOs ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressDTO {
    pub id: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateAddressInput {
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
}

// ── 9. Return, Claim, Exchange DTOs ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnDTO {
    pub id: String,
    pub order_id: String,
    pub claim_id: Option<String>,
    pub exchange_id: Option<String>,
    pub status: String,
    pub refund_amount: Option<Decimal>,
    pub location_id: Option<String>,
    pub items: Vec<ReturnItemDTO>,
    pub requested_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnItemDTO {
    pub id: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnReasonDTO {
    pub id: String,
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub parent_return_reason_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDTO {
    pub id: String,
    pub order_id: String,
    pub r#type: String,
    pub order_change_id: Option<String>,
    pub status: String,
    pub refund_amount: Option<Decimal>,
    pub items: Vec<ClaimItemDTO>,
    pub requested_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimItemDTO {
    pub id: String,
    pub claim_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub reason: String,
    pub raw_quantity: Option<serde_json::Value>,
    pub images: Vec<ClaimItemImageDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimItemImageDTO {
    pub id: String,
    pub claim_item_id: String,
    pub url: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeDTO {
    pub id: String,
    pub order_id: String,
    pub order_change_id: Option<String>,
    pub status: String,
    pub difference_due: Option<Decimal>,
    pub allow_backorder: bool,
    pub items: Vec<ExchangeItemDTO>,
    pub created_by: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeItemDTO {
    pub id: String,
    pub exchange_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderFilter {
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub display_id: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Privacy-safe product aggregates for store intelligence. These contain no buyer fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreOrderSignal {
    pub product_id: String,
    pub product_name: String,
    pub category: Option<String>,
    pub current_units: i64,
    pub comparison_units: i64,
    pub current_orders: i64,
    pub comparison_orders: i64,
}

/// Tenant-local category totals, counted independently from the product result cap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreCategoryOrderSignal {
    pub category: String,
    pub current_units: i64,
    pub comparison_units: i64,
    pub current_orders: i64,
    pub comparison_orders: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnInput {
    pub order_id: String,
    pub location_id: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReturnItemInput {
    pub return_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub reason_id: Option<String>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveReturnItemsInput {
    pub item_id: String,
    pub received_quantity: i32,
    pub damaged_quantity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClaimInput {
    pub order_id: String,
    pub r#type: String, // "replace", "refund"
    pub refund_amount: Option<Decimal>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddClaimItemInput {
    pub claim_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub reason: String,
    pub images: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExchangeInput {
    pub order_id: String,
    pub allow_backorder: Option<bool>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExchangeItemInput {
    pub exchange_id: String,
    pub item_id: String,
    pub quantity: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTimelineEventDTO {
    pub id: String,
    pub order_id: String,
    pub event_type: String,
    pub description: String,
    pub actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnReasonInput {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

/// Partial update for a return reason. Absent means unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReturnReasonInput {
    pub value: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
}
