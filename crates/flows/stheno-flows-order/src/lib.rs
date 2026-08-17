//! `stheno-flows-order`: social-checkout draft orders (PRODUCT.md 5.5).
//!
//! Social checkout deliberately does **not** go through `POST /admin/draft-orders`.
//! That endpoint's validator refuses a request carrying neither `email` nor
//! `customer_id`, and supplying `email` alone silently creates a guest customer that
//! then gets reused for every later order from that address. Buyers never have
//! accounts, so this flow calls the order workflow directly instead.
//!
//! The three traps this module exists to close:
//!
//! 1. No `email`, no `customer_id`. Buyer name and phone live in `metadata`.
//! 2. `sales_channel_id` is required **at creation**. Without it a draft looks healthy
//!    for days and then explodes at conversion, when reservation-location resolution
//!    reads it off the order — in front of a waiting buyer.
//! 3. `metadata.source = "social"` is stamped at creation so the push subscriber can
//!    skip it. Conversion fires `order.placed` just like a real checkout, and naive
//!    push logic would notify the merchant about an order their own staff just accepted.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_order::{
    CreateLineItemInput, CreateOrderInput, OrderDTO, OrderService, UpdateOrderInput,
};
use stheno_types::{Error, Result};

/// Marker written to `metadata.source` at draft creation. The `order.placed` push
/// subscriber skips orders carrying it (trap 3).
pub const SOCIAL_SOURCE: &str = "social";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialDraftInput {
    /// Required. Omitting it is trap 2, so this flow rejects at creation.
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub currency_code: String,
    pub items: Vec<CreateLineItemInput>,
    /// Buyer identity goes in metadata, never into `email` or `customer_id`.
    pub buyer_name: Option<String>,
    pub buyer_phone: Option<String>,
    /// Optional `?ref=` attribution carried from the storefront.
    pub source_ref: Option<String>,
}

pub struct SocialDraftWorkflow {
    order_service: Arc<OrderService>,
}

impl SocialDraftWorkflow {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self { order_service }
    }

    /// Creates a social draft order by invoking order creation directly.
    ///
    /// Rejects a missing `sales_channel_id` here rather than letting conversion fail
    /// later (trap 2). Failing at creation is recoverable; failing at conversion is not.
    pub async fn create_social_draft(&self, input: SocialDraftInput) -> Result<OrderDTO> {
        let sales_channel_id = input
            .sales_channel_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| Error::Invalid {
                code: "sales_channel_required",
                message: "A social draft needs sales_channel_id at creation: reservation \
                          location resolution reads it at conversion, so a draft without one \
                          fails when staff accepts it rather than now."
                    .to_string(),
                field: Some("sales_channel_id".to_string()),
            })?;

        if input.items.is_empty() {
            return Err(Error::Invalid {
                code: "empty_order",
                message: "A social draft needs at least one line item.".to_string(),
                field: Some("items".to_string()),
            });
        }

        let mut metadata = json!({ "source": SOCIAL_SOURCE });
        if let Some(map) = metadata.as_object_mut() {
            if let Some(name) = input.buyer_name {
                map.insert("buyer_name".to_string(), json!(name));
            }
            if let Some(phone) = input.buyer_phone {
                map.insert("buyer_phone".to_string(), json!(phone));
            }
            if let Some(source_ref) = input.source_ref {
                map.insert("ref".to_string(), json!(source_ref));
            }
        }

        self.order_service
            .create_order(CreateOrderInput {
                region_id: input.region_id,
                // Trap 1: neither of these is ever set for a social draft.
                customer_id: None,
                email: None,
                sales_channel_id: Some(sales_channel_id),
                currency_code: input.currency_code,
                shipping_address: None,
                billing_address: None,
                items: Some(input.items),
                shipping_methods: None,
                is_draft_order: Some(true),
                metadata: Some(metadata),
            })
            .await
    }

    /// Converts an accepted social draft into a live order.
    ///
    /// The draft must still carry `sales_channel_id`, which is what reservation-location
    /// resolution follows to the stock location. `metadata.source` is preserved so the
    /// push subscriber can still recognise and skip it.
    pub async fn convert_to_order(&self, order_id: &str) -> Result<OrderDTO> {
        let draft = self.order_service.get_order(order_id).await?;

        if !draft.is_draft_order {
            return Err(Error::Invalid {
                code: "not_a_draft",
                message: format!("Order {order_id} is not a draft order."),
                field: Some("id".to_string()),
            });
        }

        // Trap 2, checked again at the point it actually matters.
        if draft
            .sales_channel_id
            .as_ref()
            .map_or(true, |id| id.trim().is_empty())
        {
            return Err(Error::Invalid {
                code: "sales_channel_required",
                message: format!(
                    "Draft {order_id} has no sales_channel_id, so inventory reservation \
                     cannot resolve a stock location."
                ),
                field: Some("sales_channel_id".to_string()),
            });
        }

        self.order_service
            .update_order(
                order_id,
                UpdateOrderInput {
                    status: Some("pending".to_string()),
                    ..Default::default()
                },
            )
            .await
    }
}

/// Whether the `order.placed` push subscriber should skip this order (trap 3).
///
/// Order Now must push; a social conversion must not — staff already know, they just
/// accepted it by hand.
pub fn should_skip_placed_push(order: &OrderDTO) -> bool {
    order
        .metadata
        .as_ref()
        .and_then(|m| m.get("source"))
        .and_then(|s| s.as_str())
        == Some(SOCIAL_SOURCE)
}
