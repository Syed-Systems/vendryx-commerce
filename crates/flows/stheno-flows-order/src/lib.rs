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

pub mod social_message;
pub use social_message::{generate_ref_code, render_social_message, SocialMessageLine};

use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_ai::ChatOfferService;
use stheno_flows::{
    ProductOfferRequest, ProductOfferTool, ShippingQuoteRequest, ShippingQuoteTool,
};
use stheno_inventory::{CreateReservationInput, InventoryService};
use stheno_links::PgLinkStore;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput, OrderDTO,
    OrderService,
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
    pub delivery_address: Option<CreateAddressInput>,
    /// Server-quoted delivery method selected from `calculate_shipping_rates`.
    pub shipping_method: Option<CreateShippingMethodInput>,
    pub payment_preference: Option<String>,
    pub source_event_id: Option<String>,
    /// Optional `?ref=` attribution carried from the storefront.
    pub source_ref: Option<String>,
}

pub struct SocialDraftWorkflow {
    order_service: Arc<OrderService>,
    inventory_service: Option<Arc<InventoryService>>,
    links: Option<Arc<PgLinkStore>>,
    chat_offer_validation: Option<(stheno_db::DbPool, ChatOfferService, ProductOfferTool)>,
    shipping_validation: Option<ShippingQuoteTool>,
}

impl SocialDraftWorkflow {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self {
            order_service,
            inventory_service: None,
            links: None,
            chat_offer_validation: None,
            shipping_validation: None,
        }
    }

    /// Enables atomic stock reservation when staff confirms a social draft.
    pub fn with_inventory(
        mut self,
        inventory_service: Arc<InventoryService>,
        links: Arc<PgLinkStore>,
    ) -> Self {
        self.inventory_service = Some(inventory_service);
        self.links = Some(links);
        self
    }

    /// Revalidates chat offer authorization and live pricing before staff confirmation.
    pub fn with_chat_offer_validation(
        mut self,
        pool: stheno_db::DbPool,
        chat_offers: ChatOfferService,
        offers: ProductOfferTool,
    ) -> Self {
        self.chat_offer_validation = Some((pool, chat_offers, offers));
        self
    }

    /// Revalidates selected social shipping options and exact rates before confirmation.
    pub fn with_shipping_validation(mut self, shipping: ShippingQuoteTool) -> Self {
        self.shipping_validation = Some(shipping);
        self
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
        if input
            .items
            .iter()
            .any(|item| item.quantity <= 0 || item.quantity > 1000)
        {
            return Err(Error::Invalid {
                code: "social_draft_quantity_invalid",
                message: "Social draft quantities must be between 1 and 1000.".to_string(),
                field: Some("items.quantity".to_string()),
            });
        }

        let chat_offer_quotes = input
            .items
            .iter()
            .filter_map(|item| {
                let metadata = item.metadata.as_ref()?;
                let quote_id = metadata.get("chat_offer_quote_id")?.as_str()?;
                Some(json!({
                    "quote_id": quote_id,
                    "variant_id": item.variant_id.clone(),
                    "quantity": item.quantity,
                    "chat_unit_price": metadata.get("chat_offer_price")
                }))
            })
            .collect::<Vec<_>>();
        let mut metadata = json!({ "source": SOCIAL_SOURCE });
        if let Some(map) = metadata.as_object_mut() {
            if input.shipping_method.is_some() {
                map.insert("social_shipping_quote_required".to_string(), json!(true));
            }
            if !chat_offer_quotes.is_empty() {
                map.insert("chat_offer_quotes".to_string(), json!(chat_offer_quotes));
            }
            if let Some(name) = input.buyer_name {
                map.insert("buyer_name".to_string(), json!(name));
            }
            if let Some(phone) = input.buyer_phone {
                map.insert("buyer_phone".to_string(), json!(phone));
            }
            if let Some(payment_preference) = input.payment_preference {
                map.insert("payment_preference".to_string(), json!(payment_preference));
            }
            if let Some(source_event_id) = input.source_event_id {
                map.insert("source_event_id".to_string(), json!(source_event_id));
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
                shipping_address: input.delivery_address,
                billing_address: None,
                items: Some(input.items),
                shipping_methods: input.shipping_method.map(|method| vec![method]),
                is_draft_order: Some(true),
                metadata: Some(metadata),
            })
            .await
    }

    /// Creates an AI-collected draft idempotently for one provider event.
    ///
    /// The request event key is tenant-scoped by the order table's RLS and unique index.
    /// A retry after an ambiguous create returns the already-created draft.
    pub async fn create_ai_social_draft(
        &self,
        mut input: SocialDraftInput,
        event_id: &str,
    ) -> Result<OrderDTO> {
        if let Some(existing) = self.existing_ai_social_draft(event_id).await? {
            return Ok(existing);
        }
        validate_ai_draft(&input, event_id)?;
        input.source_event_id = Some(event_id.to_string());
        let external_ref = input.source_ref.take();
        for _ in 0..5 {
            let ref_code = generate_ref_code();
            input.source_ref = Some(match external_ref.as_deref() {
                Some(referrer) if !referrer.trim().is_empty() => {
                    format!("{ref_code}|{}", referrer.trim())
                }
                _ => ref_code.clone(),
            });
            match self.create_social_draft(input.clone()).await {
                Ok(order) => return Ok(order),
                Err(create_error) => {
                    if let Some(existing) = self
                        .order_service
                        .find_social_draft_by_event(event_id)
                        .await?
                    {
                        return Ok(existing);
                    }
                    let is_ref_collision = matches!(
                        &create_error,
                        Error::Conflict { code, .. } if *code == "duplicate_social_ref"
                    );
                    if is_ref_collision {
                        continue;
                    }
                    return Err(create_error);
                }
            }
        }
        Err(Error::Conflict {
            code: "social_ref_exhausted",
            message: "Could not allocate a unique social order reference.".to_string(),
        })
    }

    /// Returns the draft for an already-seen provider event before quote revalidation.
    pub async fn existing_ai_social_draft(&self, event_id: &str) -> Result<Option<OrderDTO>> {
        if event_id.trim().is_empty()
            || event_id.len() > 256
            || event_id.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "social_event_id_invalid",
                message: "Social event ID is invalid.".to_string(),
                field: Some("event_id".to_string()),
            });
        }
        self.order_service
            .find_social_draft_by_event(event_id)
            .await
    }

    /// Converts an accepted social draft into a live order.
    ///
    /// The draft must still carry `sales_channel_id`, which is what reservation-location
    /// resolution follows to the stock location. `metadata.source` is preserved so the
    /// push subscriber can still recognise and skip it.
    pub async fn convert_to_order(&self, order_id: &str) -> Result<OrderDTO> {
        let draft = self.order_service.get_order(order_id).await?;

        // A retry after the order update commits must return the same social order.
        // Keep this check ahead of the draft flag so it also handles records converted
        // before conversion cleared `is_draft_order`.
        if should_skip_placed_push(&draft) && draft.status == "pending" {
            return Ok(draft);
        }

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

        self.validate_chat_offer_lines(&draft).await?;
        self.validate_social_shipping(&draft).await?;

        if let (Some(inventory), Some(links)) = (&self.inventory_service, &self.links) {
            self.reserve_social_inventory(&draft, inventory, links)
                .await?;
        }
        self.order_service.convert_draft_order(order_id).await
    }

    async fn validate_chat_offer_lines(&self, draft: &OrderDTO) -> Result<()> {
        let mut quoted_lines = Vec::new();
        for line in &draft.items {
            let Some(metadata) = line.metadata.as_ref() else {
                continue;
            };
            let quote_id = metadata.get("chat_offer_quote_id");
            let quote_price = metadata.get("chat_offer_price");
            if quote_id.is_some() || quote_price.is_some() {
                let quote_id = quote_id.and_then(serde_json::Value::as_str).ok_or_else(|| {
                    Error::Conflict {
                        code: "social_chat_quote_metadata_invalid",
                        message: "This discounted draft has invalid quote audit data. Hold it for staff review."
                            .to_string(),
                    }
                })?;
                quoted_lines.push((line, quote_id));
            }
        }
        if let (Some((pool, chat_offers, _)), Some(event_id)) = (
            &self.chat_offer_validation,
            draft
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("source_event_id"))
                .and_then(serde_json::Value::as_str),
        ) {
            let pool = pool.clone();
            let chat_offers = *chat_offers;
            let event_id = event_id.to_string();
            let persisted_ids = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                Box::pin(async move { chat_offers.claimed_quote_ids(&mut conn, &event_id).await })
            })
            .await?;
            let persisted_set = persisted_ids
                .iter()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>();
            let line_ids = quoted_lines
                .iter()
                .map(|(_, quote_id)| *quote_id)
                .collect::<std::collections::BTreeSet<_>>();
            if persisted_set.len() != persisted_ids.len()
                || line_ids.len() != quoted_lines.len()
                || persisted_set != line_ids
            {
                return Err(Error::Conflict {
                    code: "social_chat_quote_claim_mismatch",
                    message: "A discounted line's server-side quote binding changed. Hold this draft for staff price review."
                        .to_string(),
                });
            }
        }
        if quoted_lines.is_empty() {
            if draft
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("chat_offer_quotes"))
                .and_then(serde_json::Value::as_array)
                .is_some_and(|quotes| !quotes.is_empty())
            {
                return Err(Error::Conflict {
                    code: "social_chat_quote_line_missing",
                    message: "The discounted line was changed or removed. Hold this draft for staff price review."
                        .to_string(),
                });
            }
            return Ok(());
        }
        let expected_quotes = draft
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("chat_offer_quotes"))
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| Error::Conflict {
                code: "social_chat_quote_audit_missing",
                message: "This discounted draft is missing its quote audit. Hold it for staff price review."
                    .to_string(),
            })?;
        let expected_quote_ids = expected_quotes
            .iter()
            .filter_map(|quote| quote.get("quote_id").and_then(serde_json::Value::as_str))
            .collect::<std::collections::BTreeSet<_>>();
        let line_quote_ids = quoted_lines
            .iter()
            .map(|(_, quote_id)| *quote_id)
            .collect::<std::collections::BTreeSet<_>>();
        if expected_quote_ids.len() != expected_quotes.len()
            || line_quote_ids.len() != quoted_lines.len()
            || expected_quote_ids != line_quote_ids
        {
            return Err(Error::Conflict {
                code: "social_chat_quote_audit_mismatch",
                message: "The discounted line changed after it was quoted. Hold this draft for staff price review."
                    .to_string(),
            });
        }
        let Some((pool, chat_offers, offer_tool)) = &self.chat_offer_validation else {
            return Err(Error::Conflict {
                code: "social_chat_offer_validation_unavailable",
                message:
                    "This discounted draft cannot be confirmed until its chat price is revalidated."
                        .to_string(),
            });
        };
        let event_id = draft
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("source_event_id"))
            .and_then(serde_json::Value::as_str)
            .filter(|event_id| !event_id.is_empty())
            .ok_or_else(|| Error::Conflict {
                code: "social_chat_quote_event_missing",
                message: "This discounted draft is missing its quote audit binding. Hold it for staff review."
                    .to_string(),
            })?;
        for (line, quote_id) in quoted_lines {
            let metadata = line.metadata.as_ref().ok_or_else(|| Error::Conflict {
                code: "social_chat_quote_metadata_missing",
                message: "This discounted draft is missing its quote audit data. Hold it for staff review."
                    .to_string(),
            })?;
            let expected_chat_price = metadata
                .get("chat_offer_price")
                .and_then(serde_json::Value::as_str)
                .and_then(|value| rust_decimal::Decimal::from_str(value).ok())
                .ok_or_else(|| Error::Conflict {
                    code: "social_chat_quote_price_missing",
                    message: "This discounted draft is missing its quoted price. Hold it for staff review."
                        .to_string(),
                })?;
            if line.unit_price != expected_chat_price {
                return Err(Error::Conflict {
                    code: "social_chat_quote_price_changed",
                    message: "This discounted draft's customer-visible price changed. Hold it for staff price review."
                        .to_string(),
                });
            }
            let variant_id = line.variant_id.as_deref().ok_or_else(|| Error::Conflict {
                code: "social_chat_quote_variant_missing",
                message: "This discounted draft is missing its published variant. Hold it for staff review."
                    .to_string(),
            })?.to_string();
            let quantity = line.quantity;
            let currency_code = draft.currency_code.clone();
            let region_id = draft.region_id.clone();
            let (quoted_live_price, quoted_chat_price) = {
                let pool = pool.clone();
                let chat_offers = *chat_offers;
                let quote_id = quote_id.to_string();
                let event_id = event_id.to_string();
                let quote_variant_id = variant_id.clone();
                let quote_currency_code = currency_code.clone();
                let quote_region_id = region_id.clone();
                stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                    Box::pin(async move {
                        chat_offers
                            .validate_confirmation_quote(
                                &mut conn,
                                &quote_id,
                                &event_id,
                                expected_chat_price,
                                &quote_variant_id,
                                quantity,
                                &quote_currency_code,
                                quote_region_id.as_deref(),
                            )
                            .await
                    })
                })
                .await?
            };
            let live_offer = offer_tool
                .execute(ProductOfferRequest {
                    variant_id,
                    currency_code,
                    region_id,
                    quantity,
                })
                .await?;
            let live_price = live_offer
                .price
                .as_ref()
                .filter(|price| {
                    price
                        .currency_code
                        .eq_ignore_ascii_case(&draft.currency_code)
                })
                .ok_or_else(|| Error::Conflict {
                    code: "social_chat_quote_live_price_missing",
                    message:
                        "The current price cannot be verified. Hold this draft for staff review."
                            .to_string(),
                })?;
            if live_price.calculated_amount != quoted_live_price
                || live_price.calculated_amount <= quoted_chat_price
                || live_price.original_amount != live_price.calculated_amount
                || live_price.price_list_type.is_some()
                || live_price.matched_rules_count > 0
                || live_price.min_quantity.is_some()
            {
                return Err(Error::Conflict {
                    code: "social_chat_quote_live_price_changed",
                    message: "The published price or its adjustments changed. Hold this draft for staff price review."
                        .to_string(),
                });
            }
        }
        Ok(())
    }

    async fn validate_social_shipping(&self, draft: &OrderDTO) -> Result<()> {
        let required = draft
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("social_shipping_quote_required"))
            .and_then(serde_json::Value::as_bool)
            == Some(true);
        if !required {
            return Ok(());
        }
        let Some(shipping) = &self.shipping_validation else {
            return Err(Error::Conflict {
                code: "social_shipping_validation_unavailable",
                message:
                    "This social draft cannot be confirmed until its delivery rate is revalidated."
                        .to_string(),
            });
        };
        if draft.shipping_methods.len() != 1 {
            return Err(Error::Conflict {
                code: "social_shipping_method_changed",
                message: "This social draft must have exactly one customer-selected delivery option. Hold it for staff review."
                    .to_string(),
            });
        }
        let method = &draft.shipping_methods[0];
        let option_id = method
            .shipping_option_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| Error::Conflict {
                code: "social_shipping_option_missing",
                message: "This social draft is missing its quoted delivery option. Hold it for staff review."
                    .to_string(),
            })?;
        let address = draft
            .shipping_address
            .as_ref()
            .ok_or_else(|| Error::Conflict {
                code: "social_shipping_address_missing",
                message:
                    "This social draft is missing its delivery address. Hold it for staff review."
                        .to_string(),
            })?;
        let country_code = address
            .country_code
            .as_deref()
            .ok_or_else(|| Error::Conflict {
                code: "social_shipping_country_missing",
                message:
                    "This social draft is missing its delivery country. Hold it for staff review."
                        .to_string(),
            })?;
        let items_count = draft.items.iter().try_fold(0usize, |total, item| {
            let quantity = usize::try_from(item.quantity).ok()?;
            total.checked_add(quantity)
        });
        let items_count = items_count
            .filter(|count| (1..=1000).contains(count))
            .ok_or_else(|| Error::Conflict {
                code: "social_shipping_quantity_invalid",
                message:
                    "This social draft's delivery quantity is invalid. Hold it for staff review."
                        .to_string(),
            })?;
        let current = shipping
            .calculate(ShippingQuoteRequest {
                country_code: country_code.to_string(),
                province_code: address.province.clone(),
                city: address.city.clone(),
                postal_code: address.postal_code.clone(),
                currency_code: draft.currency_code.clone(),
                region_id: draft.region_id.clone(),
                items_count,
                total_weight: None,
            })
            .await?;
        let quote = current
            .quotes
            .into_iter()
            .find(|quote| quote.option_id == option_id)
            .ok_or_else(|| Error::Conflict {
                code: "social_shipping_option_unavailable",
                message: "The selected delivery option is no longer eligible. Hold this draft for staff review."
                    .to_string(),
            })?;
        if quote.amount != method.amount
            || !quote
                .currency_code
                .eq_ignore_ascii_case(&draft.currency_code)
        {
            return Err(Error::Conflict {
                code: "social_shipping_rate_changed",
                message: "The delivery rate changed after the shopper saw it. Hold this draft for staff review."
                    .to_string(),
            });
        }
        Ok(())
    }

    async fn reserve_social_inventory(
        &self,
        order: &OrderDTO,
        inventory: &InventoryService,
        links: &PgLinkStore,
    ) -> Result<()> {
        let mut reservation_ids = Vec::new();
        let result = async {
            let sales_channel_id =
                order
                    .sales_channel_id
                    .as_deref()
                    .ok_or_else(|| Error::Invalid {
                        code: "sales_channel_required",
                        message:
                            "A social order needs a sales channel before inventory can be reserved."
                                .to_string(),
                        field: Some("sales_channel_id".to_string()),
                    })?;
            let mut location_ids = links
                .get_by_left("sales_channel_location", sales_channel_id)
                .await?;
            location_ids.sort();
            location_ids.dedup();

            for line_item in &order.items {
                if !line_item.requires_shipping {
                    continue;
                }
                let Some(variant_id) = line_item.variant_id.as_deref() else {
                    continue;
                };
                let mut inventory_item_ids = links
                    .get_by_left("product_variant_inventory_item", variant_id)
                    .await?;
                inventory_item_ids.sort();
                inventory_item_ids.dedup();
                if inventory_item_ids.is_empty() {
                    if line_item
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.get("inventory_tracked"))
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                    {
                        return Err(Error::Conflict {
                            code: "social_inventory_link_missing",
                            message:
                                "A stock-managed social order item has no inventory item linked."
                                    .to_string(),
                        });
                    }
                    continue;
                }
                if location_ids.is_empty() {
                    return Err(Error::Conflict {
                        code: "social_stock_location_missing",
                        message: "The social order's sales channel has no stock location."
                            .to_string(),
                    });
                }

                let mut reserved = false;
                let mut last_error = None;
                'candidate: for location_id in &location_ids {
                    for inventory_item_id in &inventory_item_ids {
                        match inventory
                            .reserve(CreateReservationInput {
                                line_item_id: Some(line_item.id.clone()),
                                inventory_item_id: inventory_item_id.clone(),
                                location_id: location_id.clone(),
                                quantity: line_item.quantity,
                                description: Some("Staff-confirmed social order".to_string()),
                            })
                            .await
                        {
                            Ok(reservation) => {
                                reservation_ids.push(reservation.id.to_string());
                                reserved = true;
                                break 'candidate;
                            }
                            Err(error) if is_insufficient_inventory(&error) => {
                                last_error = Some(error)
                            }
                            Err(error) => return Err(error),
                        }
                    }
                }
                if !reserved {
                    return Err(last_error.unwrap_or_else(|| Error::Conflict {
                        code: "social_inventory_unavailable",
                        message: "No stock location can fulfill the social order item.".to_string(),
                    }));
                }
            }
            Ok(())
        }
        .await;
        if result.is_err() {
            for reservation_id in reservation_ids {
                let _ = inventory.release_reservation(&reservation_id).await;
            }
        }
        result
    }
}

fn is_insufficient_inventory(error: &Error) -> bool {
    matches!(error, Error::Conflict { code, .. } if *code == "insufficient_inventory")
}

fn validate_ai_draft(input: &SocialDraftInput, event_id: &str) -> Result<()> {
    let valid_text = |value: Option<&str>, max: usize| {
        value.is_some_and(|value| {
            !value.trim().is_empty()
                && value.chars().count() <= max
                && !value.chars().any(char::is_control)
        })
    };
    let valid_shipping = input.shipping_method.as_ref().is_some_and(|method| {
        valid_text(Some(method.name.as_str()), 128)
            && method.amount >= rust_decimal::Decimal::ZERO
            && method.shipping_option_id.as_deref().is_some_and(|id| {
                !id.trim().is_empty() && id.len() <= 128 && !id.chars().any(char::is_control)
            })
    });
    if !valid_text(input.buyer_name.as_deref(), 120)
        || !valid_text(input.buyer_phone.as_deref(), 32)
        || !valid_shipping
        || input.items.is_empty()
        || input.items.len() > 20
        || event_id.trim().is_empty()
        || event_id.len() > 256
        || event_id.chars().any(char::is_control)
    {
        return Err(Error::Invalid {
            code: "social_draft_buyer_details_required",
            message: "A social draft needs buyer details, items, a quoted delivery option, and valid event key."
                .to_string(),
            field: Some("buyer".to_string()),
        });
    }
    let Some(address) = input.delivery_address.as_ref() else {
        return Err(Error::Invalid {
            code: "social_draft_address_required",
            message: "A social draft needs a delivery address.".to_string(),
            field: Some("delivery_address".to_string()),
        });
    };
    let phone = input.buyer_phone.as_deref().unwrap_or_default();
    let digit_count = phone.chars().filter(char::is_ascii_digit).count();
    if !valid_text(address.address_1.as_deref(), 255)
        || !valid_text(address.city.as_deref(), 128)
        || !address.country_code.as_deref().is_some_and(|code| {
            code.len() == 2 && code.bytes().all(|byte| byte.is_ascii_alphabetic())
        })
        || !matches!(
            input.payment_preference.as_deref(),
            Some("cash_on_delivery" | "bank_transfer" | "card_on_delivery")
        )
        || !(5..=20).contains(&digit_count)
        || phone.chars().any(|character| {
            !(character.is_ascii_digit() || matches!(character, '+' | ' ' | '-' | '(' | ')'))
        })
        || input.items.iter().any(|item| {
            item.variant_id.as_deref().map_or(true, |id| {
                id.trim().is_empty() || id.len() > 128 || id.chars().any(char::is_control)
            }) || item.quantity <= 0
                || item.quantity > 1000
        })
    {
        return Err(Error::Invalid {
            code: "social_draft_details_invalid",
            message: "Social draft address, payment preference, or items are invalid.".to_string(),
            field: Some("draft".to_string()),
        });
    }
    Ok(())
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
