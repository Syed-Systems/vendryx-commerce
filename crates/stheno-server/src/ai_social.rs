//! Production composition for the tenant-isolated AI social assistant.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use stheno_ai::{
    AiConfigurationService, ChatOfferService, ConversationMemoryService, KnowledgeService,
    MetaRuntimeConfig, WebhookEventService,
};
use stheno_api_store::social_webhook::{
    MetaInboundHandler, MetaInboundMessage, MetaWebhookContext, MetaWebhookRegistration,
    MetaWebhookRegistry,
};
use stheno_flows::{
    AiToolExecutor, ConversationEngine, ConversationInput, ConversationTurn, ProductOfferRequest,
    ProductOfferTool, ReadOnlyCommerceTools, ShippingQuoteTool,
};
use stheno_flows_order::{SocialDraftInput, SocialDraftWorkflow};
use stheno_notification::NotificationService;
use stheno_order::{CreateAddressInput, CreateLineItemInput};
use stheno_types::{Error, Result};

#[derive(Clone)]
struct ConfigurationRegistry {
    configurations: Arc<AiConfigurationService>,
}

#[async_trait]
impl MetaWebhookRegistry for ConfigurationRegistry {
    async fn resolve(&self, webhook_id: &str) -> Result<Option<MetaWebhookRegistration>> {
        let Some((tenant_id, kind, account_id)) = self
            .configurations
            .resolve_webhook_owner(webhook_id)
            .await?
        else {
            return Ok(None);
        };
        let Some(config) = self
            .configurations
            .load_meta_webhook(&tenant_id, webhook_id)
            .await?
        else {
            return Ok(None);
        };
        if config.channel != kind || config.account_id != account_id {
            return Ok(None);
        }
        Ok(Some(registration(config)))
    }
}

#[derive(Clone)]
struct SocialAssistant {
    pool: stheno_db::DbPool,
    configurations: Arc<AiConfigurationService>,
    products: Arc<stheno_product::ProductService>,
    pricing: Arc<stheno_pricing::PricingService>,
    inventory: Arc<stheno_inventory::InventoryService>,
    regions: Arc<stheno_region::RegionService>,
    currencies: Arc<stheno_currency::CurrencyService>,
    fulfillment: Arc<stheno_fulfillment::FulfillmentService>,
    store: Arc<stheno_store::StoreService>,
    social_flows: Arc<SocialDraftWorkflow>,
    graph_api_version: String,
    conversation_memory: Arc<ConversationMemoryService>,
    notifications: Arc<NotificationService>,
}

#[async_trait]
impl MetaInboundHandler for SocialAssistant {
    async fn handle(&self, tenant_id: &str, message: MetaInboundMessage) -> Result<()> {
        stheno_db::tenant::scope(tenant_id.to_string(), async {
            if message.channel == "whatsapp" && is_whatsapp_opt_out(&message.text) {
                if let Some(recipient) = normalize_meta_phone(&message.sender_id) {
                    let _ = self
                        .notifications
                        .revoke_whatsapp_consent(tenant_id, &recipient)
                        .await?;
                }
                return Ok(());
            }
            let provider_config = self
                .configurations
                .load_provider()
                .await?
                .ok_or_else(|| invalid("AI provider is not configured."))?;
            let history = self
                .conversation_memory
                .append(
                    tenant_id,
                    &message.channel,
                    &message.account_id,
                    &message.sender_id,
                    &message.event_id,
                    "user",
                    &message.text,
                )
                .await?;
            let knowledge_service = KnowledgeService;
            let query = message.text.clone();
            let pool = self.pool.clone();
            let hits = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                Box::pin(async move { knowledge_service.search(&mut conn, &query, 6).await })
            })
            .await?;
            let provider_config = stheno_ai_openai_compatible::OpenAiCompatibleConfig {
                endpoint: provider_config.endpoint,
                model: provider_config.model.clone(),
                api_key: provider_config.api_key,
            };
            let model = provider_config.model.clone();
            let provider = Arc::new(
                if provider_config.endpoint.starts_with("http://127.0.0.1:") {
                    stheno_ai_openai_compatible::OpenAiCompatibleProvider::new_local(
                        provider_config,
                    )?
                } else {
                    stheno_ai_openai_compatible::OpenAiCompatibleProvider::new(provider_config)?
                },
            );
            let offers = ProductOfferTool::new(
                self.products.clone(),
                self.pricing.clone(),
                self.inventory.clone(),
                self.regions.clone(),
            );
            let shipping = ShippingQuoteTool::new(self.fulfillment.clone(), self.regions.clone());
            let read_only = ReadOnlyCommerceTools::new(offers.clone(), shipping);
            let store = self.store.get_default_store().await?;
            let currency_code = store
                .default_currency_code
                .ok_or_else(|| invalid("The store has no default currency configured."))?;
            let conversation_key =
                conversation_key(&message.channel, &message.account_id, &message.sender_id);
            let pool = self.pool.clone();
            let chat_offers = ChatOfferService;
            let quote_key = conversation_key.clone();
            let authorized_chat_quotes =
                stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                    Box::pin(async move { chat_offers.active_quotes(&mut conn, &quote_key).await })
                })
                .await?;
            let tools = Arc::new(SocialCommerceTools {
                read_only,
                offers,
                store: self.store.clone(),
                social_flows: self.social_flows.clone(),
                event_id: message.event_id.clone(),
                currency_code,
                region_id: store.default_region_id,
                pool: self.pool.clone(),
                chat_offers: ChatOfferService,
                conversation_key,
                currencies: self.currencies.clone(),
            });
            let engine = ConversationEngine::new(provider, tools, model)?;
            let answer = match history.last() {
                Some(turn) if turn.event_id == message.event_id && turn.role == "assistant" => {
                    turn.content.clone()
                }
                _ => {
                    engine
                        .run(ConversationInput {
                            messages: history
                                .into_iter()
                                .map(|turn| ConversationTurn {
                                    role: turn.role,
                                    content: turn.content,
                                })
                                .collect(),
                            knowledge: hits.into_iter().map(|hit| hit.entry).collect(),
                            authorized_chat_quotes,
                        })
                        .await?
                        .answer
                }
            };
            self.conversation_memory
                .append(
                    tenant_id,
                    &message.channel,
                    &message.account_id,
                    &message.sender_id,
                    &message.event_id,
                    "assistant",
                    &answer,
                )
                .await?;
            send_reply(&message, &answer, &self.graph_api_version).await
        })
        .await
    }
}

/// Creates verified webhook ingress and the live provider/tool response path.
// Composition requires all services used by the inbound assistant path.
#[allow(clippy::too_many_arguments)]
pub fn build_meta_webhook_context(
    pool: stheno_db::DbPool,
    configurations: Arc<AiConfigurationService>,
    events: Arc<WebhookEventService>,
    products: Arc<stheno_product::ProductService>,
    pricing: Arc<stheno_pricing::PricingService>,
    inventory: Arc<stheno_inventory::InventoryService>,
    regions: Arc<stheno_region::RegionService>,
    currencies: Arc<stheno_currency::CurrencyService>,
    fulfillment: Arc<stheno_fulfillment::FulfillmentService>,
    store: Arc<stheno_store::StoreService>,
    social_flows: Arc<SocialDraftWorkflow>,
    notifications: Arc<NotificationService>,
) -> MetaWebhookContext {
    let mut graph_api_version =
        std::env::var("STHENO_META_GRAPH_API_VERSION").unwrap_or_else(|_| "v21.0".to_string());
    if graph_api_version.len() > 16
        || !graph_api_version.strip_prefix('v').is_some_and(|version| {
            version.split_once('.').is_some_and(|(major, minor)| {
                !major.is_empty()
                    && !minor.is_empty()
                    && major.bytes().all(|byte| byte.is_ascii_digit())
                    && minor.bytes().all(|byte| byte.is_ascii_digit())
            })
        })
    {
        graph_api_version.clear();
        graph_api_version.push_str("v21.0");
    }
    let registry = Arc::new(ConfigurationRegistry {
        configurations: configurations.clone(),
    });
    let handler = Arc::new(SocialAssistant {
        conversation_memory: Arc::new(ConversationMemoryService::new(pool.clone())),
        notifications,
        pool,
        configurations,
        products,
        pricing,
        inventory,
        regions,
        currencies,
        fulfillment,
        store,
        social_flows,
        graph_api_version,
    });
    MetaWebhookContext::new(registry, events, handler)
}

fn is_whatsapp_opt_out(text: &str) -> bool {
    matches!(
        text.trim().to_ascii_lowercase().as_str(),
        "stop" | "unsubscribe" | "cancel" | "end"
    )
}

fn normalize_meta_phone(sender_id: &str) -> Option<String> {
    let digits = sender_id.strip_prefix('+').unwrap_or(sender_id);
    if (8..=15).contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(format!("+{digits}"))
    } else {
        None
    }
}

#[derive(Clone)]
struct SocialCommerceTools {
    read_only: ReadOnlyCommerceTools,
    offers: ProductOfferTool,
    store: Arc<stheno_store::StoreService>,
    social_flows: Arc<SocialDraftWorkflow>,
    event_id: String,
    currency_code: String,
    region_id: Option<String>,
    pool: stheno_db::DbPool,
    chat_offers: ChatOfferService,
    conversation_key: String,
    currencies: Arc<stheno_currency::CurrencyService>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftOrderRequest {
    buyer_name: String,
    buyer_phone: String,
    address_1: String,
    address_2: Option<String>,
    city: String,
    country_code: String,
    shipping_option_id: String,
    province: Option<String>,
    postal_code: Option<String>,
    payment_preference: String,
    items: Vec<DraftOrderItem>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftOrderItem {
    variant_id: String,
    quantity: i32,
    #[serde(default)]
    chat_offer_quote_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatOfferQuoteRequest {
    variant_id: String,
    quantity: i32,
}

#[async_trait]
impl AiToolExecutor for SocialCommerceTools {
    async fn execute(&self, name: &str, arguments: Value) -> Result<Value> {
        if name == "quote_chat_offer" {
            let request: ChatOfferQuoteRequest = match serde_json::from_value(arguments) {
                Ok(request) => request,
                Err(_) => return Ok(json!({"eligible": false, "reason": "invalid_request"})),
            };
            if request.quantity <= 0 || request.quantity > 1000 {
                return Ok(json!({"eligible": false, "reason": "invalid_quantity"}));
            }
            let product = self
                .offers
                .execute(ProductOfferRequest {
                    variant_id: request.variant_id.clone(),
                    currency_code: self.currency_code.clone(),
                    region_id: self.region_id.clone(),
                    quantity: request.quantity,
                })
                .await?;
            let price = product.price.as_ref().filter(|price| {
                price
                    .currency_code
                    .eq_ignore_ascii_case(&self.currency_code)
            });
            let Some(price) = price else {
                return Ok(json!({"eligible": false, "reason": "current_price_unavailable"}));
            };
            let live_price = price.calculated_amount;
            if price.original_amount != price.calculated_amount
                || price.price_list_type.is_some()
                || price.matched_rules_count > 0
                || price.min_quantity.is_some()
            {
                return Ok(json!({"eligible": false, "reason": "existing_price_adjustment"}));
            }
            let decimal_digits = self.currencies.decimal_digits(&self.currency_code).await?;
            if product
                .available_quantity
                .is_some_and(|available| available < request.quantity)
                && !product.allow_backorder
            {
                return Ok(json!({"eligible": false, "reason": "insufficient_stock"}));
            }
            let pool = self.pool.clone();
            let offers = self.chat_offers;
            let variant_id = request.variant_id;
            let currency = self.currency_code.clone();
            let region_id = self.region_id.clone();
            let conversation_key = self.conversation_key.clone();
            let quote = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                Box::pin(async move {
                    offers
                        .quote(
                            &mut conn,
                            &variant_id,
                            request.quantity,
                            &currency,
                            region_id.as_deref(),
                            &conversation_key,
                            live_price,
                            decimal_digits as u32,
                        )
                        .await
                })
            })
            .await?;
            return match quote {
                Some(quote) => Ok(json!({"eligible": true, "quote": quote})),
                None => Ok(
                    json!({"eligible": false, "live_unit_price": live_price.normalize().to_string(), "currency_code": self.currency_code}),
                ),
            };
        }
        if name != "create_social_draft_order" {
            return self.read_only.execute(name, arguments).await;
        }
        let input: DraftOrderRequest =
            serde_json::from_value(arguments).map_err(|_| Error::Invalid {
                code: "ai_social_draft_invalid",
                message: "Social draft arguments are incomplete or invalid.".to_string(),
                field: Some("draft".to_string()),
            })?;
        if let Some(existing) = self
            .social_flows
            .existing_ai_social_draft(&self.event_id)
            .await?
        {
            return social_draft_result(existing);
        }
        if !matches!(
            input.payment_preference.as_str(),
            "cash_on_delivery" | "bank_transfer" | "card_on_delivery"
        ) || input.items.is_empty()
            || input.items.len() > 20
            || input.country_code.len() != 2
            || !input
                .country_code
                .bytes()
                .all(|byte| byte.is_ascii_alphabetic())
        {
            return Err(invalid("Social draft details are invalid."));
        }
        if self.event_id.trim().is_empty()
            || input.shipping_option_id.trim().is_empty()
            || input.shipping_option_id.len() > 128
            || input.shipping_option_id.chars().any(char::is_control)
        {
            return Err(invalid("A quoted delivery option is required."));
        }
        let items_count = input
            .items
            .iter()
            .try_fold(0usize, |total, item| {
                let quantity = usize::try_from(item.quantity).ok()?;
                if quantity == 0 || quantity > 1000 {
                    return None;
                }
                total.checked_add(quantity)
            })
            .filter(|count| *count <= 1000)
            .ok_or_else(|| invalid("Social draft item count is out of range."))?;
        let shipping_quotes = self
            .read_only
            .execute(
                "calculate_shipping_rates",
                json!({
                    "country_code": input.country_code.clone(),
                    "province_code": input.province.clone(),
                    "city": input.city.clone(),
                    "postal_code": input.postal_code.clone(),
                    "currency_code": self.currency_code.clone(),
                    "region_id": self.region_id.clone(),
                    "items_count": items_count,
                    "total_weight": null
                }),
            )
            .await?;
        let selected_shipping = shipping_quotes
            .get("quotes")
            .and_then(Value::as_array)
            .and_then(|quotes| {
                quotes.iter().find(|quote| {
                    quote.get("option_id").and_then(Value::as_str)
                        == Some(input.shipping_option_id.as_str())
                })
            })
            .ok_or_else(|| invalid("The selected delivery option is no longer eligible."))?;
        let shipping_name = selected_shipping
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .ok_or_else(|| invalid("The selected delivery option has no valid name."))?
            .to_string();
        let shipping_amount = selected_shipping
            .get("amount")
            .map(Value::to_string)
            .ok_or_else(|| invalid("The selected delivery rate is unavailable."))?
            .parse::<rust_decimal::Decimal>()
            .map_err(|_| invalid("The selected delivery rate is invalid."))?;
        let shipping_currency = selected_shipping
            .get("currency_code")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("The selected delivery currency is unavailable."))?;
        if !shipping_currency.eq_ignore_ascii_case(&self.currency_code)
            || shipping_amount.is_sign_negative()
        {
            return Err(invalid(
                "The selected delivery rate does not match the store currency.",
            ));
        }
        let store = self.store.get_default_store().await?;
        let sales_channel_id = store
            .default_sales_channel_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| invalid("The store has no default sales channel configured."))?;
        let mut items = Vec::with_capacity(input.items.len());
        let mut seen_variants = std::collections::HashSet::new();
        for item in input.items {
            if !seen_variants.insert(item.variant_id.clone()) {
                return Err(invalid(
                    "Duplicate product variants must be combined into one order line.",
                ));
            }
            if item.quantity <= 0 || item.quantity > 1000 {
                return Err(invalid("Social draft quantity is invalid."));
            }
            let offer = self
                .offers
                .execute(ProductOfferRequest {
                    variant_id: item.variant_id.clone(),
                    currency_code: self.currency_code.clone(),
                    region_id: self.region_id.clone(),
                    quantity: item.quantity,
                })
                .await?;
            let mut line = draft_line_item(offer.clone(), item.quantity, &self.currency_code)?;
            if let Some(quote_id) = item.chat_offer_quote_id {
                let price = offer
                    .price
                    .as_ref()
                    .filter(|price| {
                        price
                            .currency_code
                            .eq_ignore_ascii_case(&self.currency_code)
                    })
                    .ok_or_else(|| invalid("A live price is unavailable for the quoted offer."))?;
                if price.original_amount != price.calculated_amount
                    || price.price_list_type.is_some()
                    || price.matched_rules_count > 0
                    || price.min_quantity.is_some()
                {
                    return Err(invalid(
                        "A chat offer cannot be combined with another price adjustment.",
                    ));
                }
                let live_price = price.calculated_amount;
                let pool = self.pool.clone();
                let chat_offers = self.chat_offers;
                let quote_id_for_validation = quote_id.clone();
                let conversation_key = self.conversation_key.clone();
                let event_id = self.event_id.clone();
                let variant_id = item.variant_id;
                let quantity = item.quantity;
                let currency = self.currency_code.clone();
                let decimal_digits = self.currencies.decimal_digits(&currency).await?;
                if live_price.normalize().scale() > decimal_digits as u32 {
                    return Err(invalid(
                        "The live price has unsupported currency precision.",
                    ));
                }
                let region_id = self.region_id.clone();
                let quote_price = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
                    Box::pin(async move {
                        chat_offers
                            .validate_quote(
                                &mut conn,
                                &quote_id_for_validation,
                                &conversation_key,
                                &event_id,
                                &variant_id,
                                quantity,
                                &currency,
                                region_id.as_deref(),
                                live_price,
                            )
                            .await
                    })
                })
                .await?;
                line.unit_price = quote_price;
                line.metadata = Some(json!({
                    "inventory_tracked": offer.inventory_tracked,
                    "chat_offer_quote_id": quote_id,
                    "chat_offer_price": quote_price.normalize().to_string()
                }));
            }
            items.push(line);
        }
        let draft = self
            .social_flows
            .create_ai_social_draft(
                SocialDraftInput {
                    sales_channel_id: Some(sales_channel_id),
                    region_id: self.region_id.clone(),
                    currency_code: self.currency_code.clone(),
                    items,
                    buyer_name: Some(input.buyer_name.clone()),
                    buyer_phone: Some(input.buyer_phone.clone()),
                    delivery_address: Some(CreateAddressInput {
                        first_name: Some(input.buyer_name),
                        address_1: Some(input.address_1),
                        address_2: input.address_2,
                        city: Some(input.city),
                        country_code: Some(input.country_code.to_ascii_lowercase()),
                        province: input.province,
                        postal_code: input.postal_code,
                        phone: Some(input.buyer_phone),
                        ..Default::default()
                    }),
                    shipping_method: Some(stheno_order::CreateShippingMethodInput {
                        name: shipping_name,
                        amount: shipping_amount,
                        shipping_option_id: Some(input.shipping_option_id),
                        ..Default::default()
                    }),
                    payment_preference: Some(input.payment_preference),
                    source_event_id: None,
                    source_ref: None,
                },
                &self.event_id,
            )
            .await?;
        social_draft_result(draft)
    }
}

fn social_draft_result(draft: stheno_order::OrderDTO) -> Result<Value> {
    let reference = draft
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("ref"))
        .and_then(Value::as_str)
        .map(|reference| reference.split('|').next().unwrap_or(reference))
        .ok_or_else(|| invalid("The social draft reference is unavailable."))?;
    Ok(json!({
        "draft_created": true,
        "display_id": draft.display_id,
        "reference": reference,
        "staff_confirmation_required": true,
        "inventory_reserved": false,
        "payment_collected": false,
        "courier_booked": false
    }))
}

fn conversation_key(channel: &str, account_id: &str, sender_id: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("{channel}\0{account_id}\0{sender_id}").as_bytes())
    )
}

fn draft_line_item(
    offer: stheno_flows::ProductOffer,
    quantity: i32,
    currency_code: &str,
) -> Result<CreateLineItemInput> {
    if quantity <= 0
        || offer
            .available_quantity
            .is_some_and(|available| available < quantity)
    {
        return Err(invalid(
            "The requested quantity is not currently available.",
        ));
    }
    let price = offer
        .price
        .as_ref()
        .filter(|price| price.currency_code.eq_ignore_ascii_case(currency_code))
        .ok_or_else(|| invalid("A live price is unavailable for this item."))?;
    let inventory_tracked = offer.inventory_tracked;
    Ok(CreateLineItemInput {
        title: format!("{} — {}", offer.product_title, offer.variant_title),
        variant_id: Some(offer.variant_id),
        product_id: Some(offer.product_id),
        quantity,
        unit_price: price.calculated_amount,
        is_tax_inclusive: Some(price.is_tax_inclusive),
        is_custom_price: Some(false),
        is_discountable: Some(false),
        requires_shipping: Some(true),
        variant_sku: offer.sku,
        variant_title: Some(offer.variant_title),
        product_title: Some(offer.product_title),
        metadata: Some(json!({ "inventory_tracked": inventory_tracked })),
        ..Default::default()
    })
}

fn registration(config: MetaRuntimeConfig) -> MetaWebhookRegistration {
    MetaWebhookRegistration {
        tenant_id: config.tenant_id,
        channel: config.channel,
        account_id: config.account_id,
        app_secret: config.app_secret,
        verify_token: config.verify_token,
        access_token: config.access_token,
    }
}

async fn send_reply(message: &MetaInboundMessage, answer: &str, version: &str) -> Result<()> {
    let url = format!(
        "https://graph.facebook.com/{version}/{}/messages",
        message.account_id
    );
    let payload = if message.channel == "whatsapp" {
        serde_json::json!({
            "messaging_product": "whatsapp",
            "to": message.sender_id,
            "type": "text",
            "text": { "body": answer }
        })
    } else {
        serde_json::json!({
            "recipient": { "id": message.sender_id },
            "message": { "text": answer }
        })
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(provider_error)?;
    let response = client
        .post(url)
        .bearer_auth(&message.access_token)
        .json(&payload)
        .send()
        .await
        .map_err(provider_error)?;
    if !response.status().is_success() {
        return Err(provider_error(std::io::Error::other(format!(
            "Meta Graph API returned HTTP {}",
            response.status().as_u16()
        ))));
    }
    Ok(())
}

fn provider_error(error: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::ProviderFailed {
        provider: "meta_graph".to_string(),
        source: Box::new(error),
    }
}

fn invalid(message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_social_configuration_invalid",
        message: message.to_string(),
        field: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use stheno_pricing::CalculatedPriceDTO;

    fn offer(
        price: Option<CalculatedPriceDTO>,
        available_quantity: Option<i32>,
    ) -> stheno_flows::ProductOffer {
        stheno_flows::ProductOffer {
            product_id: "prod_1".to_string(),
            product_title: "Blue Lamp".to_string(),
            variant_id: "variant_1".to_string(),
            variant_title: "Blue / Small".to_string(),
            sku: Some("LAMP-BLUE-S".to_string()),
            quantity: 2,
            price,
            available_quantity,
            inventory_tracked: true,
            allow_backorder: false,
        }
    }

    fn live_price(currency_code: &str) -> CalculatedPriceDTO {
        CalculatedPriceDTO {
            price_set_id: "pset_1".to_string(),
            calculated_amount: Decimal::new(2599, 2),
            original_amount: Decimal::new(2599, 2),
            currency_code: currency_code.to_string(),
            is_tax_inclusive: true,
            price_list_type: None,
            matched_rules_count: 0,
            min_quantity: None,
        }
    }

    #[test]
    fn social_draft_uses_live_price_and_rejects_unknown_or_wrong_currency() {
        let item = draft_line_item(offer(Some(live_price("usd")), Some(2)), 2, "USD")
            .expect("live offer becomes draft line");
        assert_eq!(item.unit_price, Decimal::new(2599, 2));
        assert_eq!(item.is_discountable, Some(false));
        assert!(draft_line_item(offer(None, Some(2)), 2, "USD").is_err());
        assert!(draft_line_item(offer(Some(live_price("eur")), Some(2)), 2, "USD").is_err());
        assert!(draft_line_item(offer(Some(live_price("usd")), Some(1)), 2, "USD").is_err());
        assert!(draft_line_item(offer(Some(live_price("usd")), None), 0, "USD").is_err());
    }
}
