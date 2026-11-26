//! Typed, read-only commerce tools used by the social assistant (ADR-032 phase 32.2).

use std::sync::Arc;
use std::time::Duration;

use crate::ai_conversation::AiToolExecutor;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stheno_fulfillment::{
    CalculatePriceContext, EligibilityContext, FulfillmentService, ShippingOptionDTO,
};
use stheno_inventory::InventoryService;
use stheno_pricing::{CalculatedPriceDTO, PricingContext, PricingService};
use stheno_product::{ProductDTO, ProductService, ProductVariantDTO};
use stheno_region::{RegionDTO, RegionService};
use stheno_types::{Error, Result};

const TOOL_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_QUANTITY: i32 = 1000;
const MAX_SHIPPING_QUOTES: usize = 8;

/// Explicit allowlist for resolving a current, published offer for one variant.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductOfferRequest {
    pub variant_id: String,
    pub currency_code: String,
    pub region_id: Option<String>,
    #[serde(default = "default_quantity")]
    pub quantity: i32,
}

/// Bounded literal prefix search request for the live catalog.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSearchRequest {
    pub query: String,
}

/// Minimal product facts safe to use as catalog search results.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogSearchItem {
    pub product_id: String,
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
}

/// Destination and basket facts needed for a current shipping quote.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShippingQuoteRequest {
    pub country_code: String,
    pub province_code: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub currency_code: String,
    pub region_id: Option<String>,
    pub items_count: usize,
    pub total_weight: Option<Decimal>,
}

/// One eligible shipping option with a live server-side amount.
#[derive(Debug, Clone, Serialize)]
pub struct ShippingQuote {
    pub option_id: String,
    pub name: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub is_calculated: bool,
}

/// Bounded quote results make omitted options explicit.
#[derive(Debug, Clone, Serialize)]
pub struct ShippingQuoteResult {
    pub quotes: Vec<ShippingQuote>,
    pub truncated: bool,
}

/// Explicit adapter exposing only the four ADR-approved read-only commerce tools.
#[derive(Clone)]
pub struct ReadOnlyCommerceTools {
    offers: ProductOfferTool,
    shipping: ShippingQuoteTool,
}

impl ReadOnlyCommerceTools {
    /// Creates the model-facing tool set from the already-composed live services.
    pub fn new(offers: ProductOfferTool, shipping: ShippingQuoteTool) -> Self {
        Self { offers, shipping }
    }
}

#[derive(Serialize)]
struct StockCheckResult {
    variant_id: String,
    available_quantity: Option<i32>,
    inventory_tracked: bool,
    allow_backorder: bool,
}

#[async_trait]
impl AiToolExecutor for ReadOnlyCommerceTools {
    async fn execute(&self, name: &str, arguments: Value) -> Result<Value> {
        let result = match name {
            "search_catalog" => {
                let request: CatalogSearchRequest = parse_tool_args(arguments)?;
                serde_json::to_value(self.offers.search_catalog(request).await?)
            }
            "get_product_details" => {
                let request: ProductOfferRequest = parse_tool_args(arguments)?;
                serde_json::to_value(self.offers.execute(request).await?)
            }
            "check_variant_stock" => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Request {
                    variant_id: String,
                    currency_code: String,
                }
                let request: Request = parse_tool_args(arguments)?;
                let offer = self
                    .offers
                    .execute(ProductOfferRequest {
                        variant_id: request.variant_id,
                        currency_code: request.currency_code,
                        region_id: None,
                        quantity: 1,
                    })
                    .await?;
                serde_json::to_value(StockCheckResult {
                    variant_id: offer.variant_id,
                    available_quantity: offer.available_quantity,
                    inventory_tracked: offer.inventory_tracked,
                    allow_backorder: offer.allow_backorder,
                })
            }
            "calculate_shipping_rates" => {
                let request: ShippingQuoteRequest = parse_tool_args(arguments)?;
                serde_json::to_value(self.shipping.calculate(request).await?)
            }
            _ => {
                return Err(Error::Invalid {
                    code: "ai_tool_not_allowed",
                    message: "The requested tool is not available.".to_string(),
                    field: Some("tool_name".to_string()),
                });
            }
        }
        .map_err(|_| {
            Error::Internal(Box::new(std::io::Error::other(
                "AI tool result serialization failed",
            )))
        })?;
        Ok(result)
    }
}

fn parse_tool_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|_| Error::Invalid {
        code: "ai_tool_invalid_argument",
        message: "Tool arguments do not match the required schema.".to_string(),
        field: Some("arguments".to_string()),
    })
}

fn default_quantity() -> i32 {
    1
}

/// Current product, price, and inventory facts returned by the read-only tool.
#[derive(Debug, Clone, Serialize)]
pub struct ProductOffer {
    pub product_id: String,
    pub product_title: String,
    pub variant_id: String,
    pub variant_title: String,
    pub sku: Option<String>,
    pub quantity: i32,
    /// Missing means no live price matched the requested currency/context.
    pub price: Option<CalculatedPriceDTO>,
    /// Missing means inventory is untracked or no inventory linkage is configured.
    pub available_quantity: Option<i32>,
    pub inventory_tracked: bool,
    pub allow_backorder: bool,
}

#[async_trait]
trait CatalogReader: Send + Sync {
    async fn variant(&self, id: &str) -> Result<ProductVariantDTO>;
    async fn product(&self, id: &str) -> Result<ProductDTO>;
    async fn search_published(&self, query: &str) -> Result<Vec<ProductDTO>>;
}

#[async_trait]
impl CatalogReader for ProductService {
    async fn variant(&self, id: &str) -> Result<ProductVariantDTO> {
        self.get_variant(id).await
    }

    async fn product(&self, id: &str) -> Result<ProductDTO> {
        self.get_product(id).await
    }

    async fn search_published(&self, query: &str) -> Result<Vec<ProductDTO>> {
        self.search_published_products(query).await
    }
}

#[async_trait]
trait PricingReader: Send + Sync {
    async fn prices_for_variants(
        &self,
        variant_ids: &[String],
        context: &PricingContext,
    ) -> Result<std::collections::HashMap<String, CalculatedPriceDTO>>;
}

#[async_trait]
impl PricingReader for PricingService {
    async fn prices_for_variants(
        &self,
        variant_ids: &[String],
        context: &PricingContext,
    ) -> Result<std::collections::HashMap<String, CalculatedPriceDTO>> {
        self.calculate_for_variants(variant_ids, context).await
    }
}

#[async_trait]
trait InventoryReader: Send + Sync {
    async fn availability(
        &self,
        variant_ids: &[String],
    ) -> Result<std::collections::HashMap<String, i32>>;
}

#[async_trait]
impl InventoryReader for InventoryService {
    async fn availability(
        &self,
        variant_ids: &[String],
    ) -> Result<std::collections::HashMap<String, i32>> {
        self.availability_for_variants(variant_ids).await
    }
}

#[async_trait]
trait RegionReader: Send + Sync {
    async fn region(&self, id: &str) -> Result<RegionDTO>;
}

#[async_trait]
impl RegionReader for RegionService {
    async fn region(&self, id: &str) -> Result<RegionDTO> {
        self.get_region(id).await
    }
}

#[async_trait]
trait ShippingReader: Send + Sync {
    async fn eligible_options(
        &self,
        context: &EligibilityContext,
    ) -> Result<Vec<ShippingOptionDTO>>;
    async fn calculate_rate(
        &self,
        option_id: &str,
        context: &CalculatePriceContext,
    ) -> Result<Decimal>;
}

#[async_trait]
impl ShippingReader for FulfillmentService {
    async fn eligible_options(
        &self,
        context: &EligibilityContext,
    ) -> Result<Vec<ShippingOptionDTO>> {
        self.resolve_eligible_shipping_options(context).await
    }

    async fn calculate_rate(
        &self,
        option_id: &str,
        context: &CalculatePriceContext,
    ) -> Result<Decimal> {
        self.calculate_shipping_price(option_id, Some(context))
            .await
    }
}

/// Cross-module shipping quote composition with only eligibility and pricing reads exposed.
#[derive(Clone)]
pub struct ShippingQuoteTool {
    fulfillment: Arc<dyn ShippingReader>,
    regions: Arc<dyn RegionReader>,
}

impl ShippingQuoteTool {
    /// Creates a quote tool from the application-owned fulfillment and region services.
    pub fn new(fulfillment: Arc<FulfillmentService>, regions: Arc<RegionService>) -> Self {
        Self {
            fulfillment,
            regions,
        }
    }

    /// Quotes eligible options without creating fulfillment, payment, or inventory records.
    pub async fn calculate(&self, request: ShippingQuoteRequest) -> Result<ShippingQuoteResult> {
        validate_shipping_request(&request)?;
        let fulfillment = self.fulfillment.clone();
        let regions = self.regions.clone();
        tokio::time::timeout(TOOL_TIMEOUT, async move {
            if let Some(region_id) = request.region_id.as_deref() {
                let region = regions.region(region_id).await?;
                if !region
                    .currency_code
                    .eq_ignore_ascii_case(&request.currency_code)
                {
                    return Err(invalid(
                        "currency_code",
                        "Requested currency does not match the selected region.",
                    ));
                }
            }

            let currency_code = request.currency_code.to_ascii_lowercase();
            let eligibility = EligibilityContext {
                country_code: Some(request.country_code.to_ascii_lowercase()),
                province_code: request.province_code,
                city: request.city,
                postal_code: request.postal_code,
                currency_code: Some(currency_code.clone()),
                is_return: Some(false),
                customer_id: None,
                customer_group_id: None,
                sales_channel_id: None,
                items_count: Some(request.items_count),
                total_weight: request.total_weight,
            };
            let mut options = fulfillment.eligible_options(&eligibility).await?;
            options.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
            let truncated = options.len() > MAX_SHIPPING_QUOTES;
            options.truncate(MAX_SHIPPING_QUOTES);

            let price_context = CalculatePriceContext {
                items_count: request.items_count,
                total_weight: request.total_weight,
                currency_code: currency_code.clone(),
            };
            let mut quotes = Vec::with_capacity(options.len());
            for option in options {
                let amount = fulfillment
                    .calculate_rate(&option.id, &price_context)
                    .await?;
                if amount.is_sign_negative() {
                    return Err(invalid(
                        "shipping_rate",
                        "Shipping rate cannot be negative.",
                    ));
                }
                quotes.push(ShippingQuote {
                    option_id: option.id,
                    name: option.name,
                    amount,
                    currency_code: currency_code.clone(),
                    is_calculated: option.price_type == "calculated",
                });
            }
            Ok(ShippingQuoteResult { quotes, truncated })
        })
        .await
        .map_err(|_| Error::Database("AI shipping quote lookup timed out".to_string()))?
    }
}

/// Read-only composition boundary; domain modules remain independent leaves.
#[derive(Clone)]
pub struct ProductOfferTool {
    products: Arc<dyn CatalogReader>,
    pricing: Arc<dyn PricingReader>,
    inventory: Arc<dyn InventoryReader>,
    regions: Arc<dyn RegionReader>,
}

impl ProductOfferTool {
    /// Creates a tool from the services already owned by the application composition root.
    pub fn new(
        products: Arc<ProductService>,
        pricing: Arc<PricingService>,
        inventory: Arc<InventoryService>,
        regions: Arc<RegionService>,
    ) -> Self {
        let products: Arc<dyn CatalogReader> = products;
        let pricing: Arc<dyn PricingReader> = pricing;
        let inventory: Arc<dyn InventoryReader> = inventory;
        let regions: Arc<dyn RegionReader> = regions;
        Self {
            products,
            pricing,
            inventory,
            regions,
        }
    }

    /// Resolves current catalog facts, with an overall timeout and no write-capable calls.
    pub async fn execute(&self, request: ProductOfferRequest) -> Result<ProductOffer> {
        validate_request(&request)?;
        let products = self.products.clone();
        let pricing = self.pricing.clone();
        let inventory = self.inventory.clone();
        let regions = self.regions.clone();

        tokio::time::timeout(TOOL_TIMEOUT, async move {
            let variant = products.variant(&request.variant_id).await?;
            let product = products.product(&variant.product_id).await?;
            if product.status != "published" {
                return Err(Error::NotFound {
                    entity: "published_product_variant",
                    id: request.variant_id,
                });
            }

            if let Some(region_id) = request.region_id.as_deref() {
                let region = regions.region(region_id).await?;
                if !region
                    .currency_code
                    .eq_ignore_ascii_case(&request.currency_code)
                {
                    return Err(Error::Invalid {
                        code: "ai_tool_region_currency_mismatch",
                        message: format!("Requested currency does not match region {}.", region.id),
                        field: Some("currency_code".to_string()),
                    });
                }
            }

            let context = PricingContext {
                currency_code: request.currency_code.to_ascii_lowercase(),
                region_id: request.region_id,
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(request.quantity),
                as_of: None,
            };
            let variant_id = variant.id.to_string();
            let prices = pricing
                .prices_for_variants(std::slice::from_ref(&variant_id), &context)
                .await?;
            let availability = if variant.manage_inventory {
                inventory
                    .availability(std::slice::from_ref(&variant_id))
                    .await?
                    .get(&variant_id)
                    .copied()
            } else {
                None
            };

            Ok(ProductOffer {
                product_id: product.id.to_string(),
                product_title: product.title,
                variant_id: variant_id.clone(),
                variant_title: variant.title,
                sku: variant.sku,
                quantity: request.quantity,
                price: prices.get(&variant_id).cloned(),
                available_quantity: availability,
                inventory_tracked: variant.manage_inventory,
                allow_backorder: variant.allow_backorder,
            })
        })
        .await
        .map_err(|_| Error::Database("AI product offer lookup timed out".to_string()))?
    }

    /// Searches the live product schema and returns at most eight published products.
    pub async fn search_catalog(
        &self,
        request: CatalogSearchRequest,
    ) -> Result<Vec<CatalogSearchItem>> {
        let query = request.query;
        let products = self.products.clone();
        tokio::time::timeout(TOOL_TIMEOUT, async move {
            products.search_published(&query).await.map(|products| {
                products
                    .into_iter()
                    .map(|product| CatalogSearchItem {
                        product_id: product.id.to_string(),
                        title: product.title,
                        handle: product.handle,
                        description: product.description,
                        thumbnail: product.thumbnail,
                    })
                    .collect()
            })
        })
        .await
        .map_err(|_| Error::Database("AI catalog search timed out".to_string()))?
    }
}

fn validate_request(request: &ProductOfferRequest) -> Result<()> {
    if request.variant_id.trim().is_empty() || request.variant_id.len() > 128 {
        return Err(invalid(
            "variant_id",
            "Variant ID is required and must be at most 128 bytes.",
        ));
    }
    if request.currency_code.len() != 3
        || !request
            .currency_code
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return Err(invalid(
            "currency_code",
            "Currency code must contain exactly three ASCII letters.",
        ));
    }
    if request.quantity < 1 || request.quantity > MAX_QUANTITY {
        return Err(invalid("quantity", "Quantity must be between 1 and 1000."));
    }
    if request
        .region_id
        .as_deref()
        .is_some_and(|region_id| region_id.trim().is_empty() || region_id.len() > 128)
    {
        return Err(invalid(
            "region_id",
            "Region ID must be non-empty and at most 128 bytes.",
        ));
    }
    Ok(())
}

fn validate_shipping_request(request: &ShippingQuoteRequest) -> Result<()> {
    if request.country_code.len() != 2
        || !request
            .country_code
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return Err(invalid(
            "country_code",
            "Country code must contain exactly two ASCII letters.",
        ));
    }
    if request.currency_code.len() != 3
        || !request
            .currency_code
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return Err(invalid(
            "currency_code",
            "Currency code must contain exactly three ASCII letters.",
        ));
    }
    if !(1..=1000).contains(&request.items_count) {
        return Err(invalid(
            "items_count",
            "Item count must be between 1 and 1000.",
        ));
    }
    if request
        .region_id
        .as_deref()
        .is_some_and(|value| !valid_bounded_text(value, 128))
    {
        return Err(invalid(
            "region_id",
            "Region ID must be non-empty and at most 128 bytes.",
        ));
    }
    for (field, value, max_len) in [
        ("province_code", request.province_code.as_deref(), 128),
        ("city", request.city.as_deref(), 128),
        ("postal_code", request.postal_code.as_deref(), 32),
    ] {
        if value.is_some_and(|value| !valid_bounded_text(value, max_len)) {
            return Err(invalid(
                field,
                "Destination value is empty, too long, or contains control characters.",
            ));
        }
    }
    if request
        .total_weight
        .is_some_and(|weight| weight.is_sign_negative() || weight > Decimal::new(1_000_000, 0))
    {
        return Err(invalid(
            "total_weight",
            "Total weight must be between zero and 1000000.",
        ));
    }
    Ok(())
}

fn valid_bounded_text(value: &str, max_len: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

fn invalid(field: &'static str, message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_tool_invalid_argument",
        message: message.to_string(),
        field: Some(field.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use stheno_product::{ProductDTO, ProductVariantDTO};
    use stheno_region::RegionDTO;

    struct MockCatalog {
        product: ProductDTO,
        variant: ProductVariantDTO,
        slow: bool,
    }

    #[async_trait]
    impl CatalogReader for MockCatalog {
        async fn variant(&self, _id: &str) -> Result<ProductVariantDTO> {
            if self.slow {
                tokio::time::sleep(TOOL_TIMEOUT + Duration::from_millis(50)).await;
            }
            Ok(self.variant.clone())
        }

        async fn product(&self, _id: &str) -> Result<ProductDTO> {
            Ok(self.product.clone())
        }

        async fn search_published(&self, _query: &str) -> Result<Vec<ProductDTO>> {
            Ok(vec![self.product.clone()])
        }
    }

    struct MockPricing {
        calls: Arc<AtomicUsize>,
        context: Arc<Mutex<Option<PricingContext>>>,
        with_price: bool,
    }

    #[async_trait]
    impl PricingReader for MockPricing {
        async fn prices_for_variants(
            &self,
            variant_ids: &[String],
            context: &PricingContext,
        ) -> Result<HashMap<String, CalculatedPriceDTO>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.context.lock().expect("context lock") = Some(context.clone());
            let mut prices = HashMap::new();
            if self.with_price {
                prices.insert(
                    variant_ids[0].clone(),
                    CalculatedPriceDTO {
                        price_set_id: "pset_live".to_string(),
                        calculated_amount: Decimal::new(1234, 2),
                        original_amount: Decimal::new(1500, 2),
                        currency_code: "bdt".to_string(),
                        is_tax_inclusive: false,
                        price_list_type: None,
                        matched_rules_count: 0,
                        min_quantity: None,
                    },
                );
            }
            Ok(prices)
        }
    }

    struct MockInventory {
        calls: Arc<AtomicUsize>,
        available: Option<i32>,
    }

    #[async_trait]
    impl InventoryReader for MockInventory {
        async fn availability(&self, variant_ids: &[String]) -> Result<HashMap<String, i32>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self
                .available
                .map(|available| HashMap::from([(variant_ids[0].clone(), available)]))
                .unwrap_or_default())
        }
    }

    struct MockRegion {
        currency_code: String,
    }

    struct MockShipping;

    #[async_trait]
    impl ShippingReader for MockShipping {
        async fn eligible_options(
            &self,
            _context: &EligibilityContext,
        ) -> Result<Vec<ShippingOptionDTO>> {
            Ok(Vec::new())
        }

        async fn calculate_rate(
            &self,
            _option_id: &str,
            _context: &CalculatePriceContext,
        ) -> Result<Decimal> {
            Ok(Decimal::ZERO)
        }
    }

    #[async_trait]
    impl RegionReader for MockRegion {
        async fn region(&self, _id: &str) -> Result<RegionDTO> {
            Ok(RegionDTO {
                id: stheno_types::Id::new(),
                name: "Test region".to_string(),
                currency_code: self.currency_code.clone(),
                automatic_taxes: false,
                countries: Vec::new(),
                created_at: Utc::now(),
            })
        }
    }

    fn make_tool(
        status: &str,
        region_currency: &str,
        with_price: bool,
        available: Option<i32>,
        slow: bool,
    ) -> (
        ProductOfferTool,
        Arc<AtomicUsize>,
        Arc<AtomicUsize>,
        Arc<Mutex<Option<PricingContext>>>,
    ) {
        let now = Utc::now();
        let product_id = "prod_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string();
        let catalog = MockCatalog {
            product: ProductDTO {
                id: stheno_types::Id::from_string(&product_id).expect("product id"),
                title: "Desk lamp".to_string(),
                handle: "desk-lamp".to_string(),
                subtitle: None,
                description: None,
                status: status.to_string(),
                thumbnail: None,
                discountable: true,
                created_at: now,
                updated_at: now,
            },
            variant: ProductVariantDTO {
                id: stheno_types::Id::new(),
                product_id,
                title: "Default".to_string(),
                sku: Some("LAMP-1".to_string()),
                barcode: None,
                allow_backorder: false,
                manage_inventory: true,
                option_values_hash: "hash".to_string(),
                created_at: now,
            },
            slow,
        };
        let pricing_calls = Arc::new(AtomicUsize::new(0));
        let inventory_calls = Arc::new(AtomicUsize::new(0));
        let seen_context = Arc::new(Mutex::new(None));
        let tool = ProductOfferTool {
            products: Arc::new(catalog),
            pricing: Arc::new(MockPricing {
                calls: pricing_calls.clone(),
                context: seen_context.clone(),
                with_price,
            }),
            inventory: Arc::new(MockInventory {
                calls: inventory_calls.clone(),
                available,
            }),
            regions: Arc::new(MockRegion {
                currency_code: region_currency.to_string(),
            }),
        };
        (tool, pricing_calls, inventory_calls, seen_context)
    }

    #[test]
    fn rejects_malformed_or_unbounded_tool_arguments() {
        let valid = ProductOfferRequest {
            variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            currency_code: "bdt".to_string(),
            region_id: Some("reg_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            quantity: 1,
        };
        assert!(validate_request(&valid).is_ok());
        assert!(validate_request(&ProductOfferRequest {
            quantity: 0,
            ..valid.clone()
        })
        .is_err());
        assert!(validate_request(&ProductOfferRequest {
            quantity: 1001,
            ..valid.clone()
        })
        .is_err());
        assert!(validate_request(&ProductOfferRequest {
            currency_code: "US$".to_string(),
            ..valid.clone()
        })
        .is_err());
        assert!(validate_request(&ProductOfferRequest {
            variant_id: " ".to_string(),
            ..valid.clone()
        })
        .is_err());
        assert!(validate_request(&ProductOfferRequest {
            region_id: Some(" ".to_string()),
            ..valid
        })
        .is_err());
    }

    #[test]
    fn shipping_arguments_reject_bad_codes_ranges_and_control_characters() {
        let valid = ShippingQuoteRequest {
            country_code: "US".to_string(),
            province_code: Some("CA".to_string()),
            city: Some("Los Angeles".to_string()),
            postal_code: Some("90001".to_string()),
            currency_code: "USD".to_string(),
            region_id: None,
            items_count: 2,
            total_weight: Some(Decimal::new(150, 2)),
        };
        assert!(validate_shipping_request(&valid).is_ok());
        assert!(validate_shipping_request(&ShippingQuoteRequest {
            country_code: "U$".to_string(),
            ..valid.clone()
        })
        .is_err());
        assert!(validate_shipping_request(&ShippingQuoteRequest {
            currency_code: "US".to_string(),
            ..valid.clone()
        })
        .is_err());
        assert!(validate_shipping_request(&ShippingQuoteRequest {
            items_count: 0,
            ..valid.clone()
        })
        .is_err());
        assert!(validate_shipping_request(&ShippingQuoteRequest {
            postal_code: Some("90\n001".to_string()),
            ..valid.clone()
        })
        .is_err());
        assert!(validate_shipping_request(&ShippingQuoteRequest {
            total_weight: Some(Decimal::NEGATIVE_ONE),
            ..valid
        })
        .is_err());
    }

    #[tokio::test]
    async fn offer_uses_live_price_context_and_preserves_zero_stock() {
        let (tool, _, _, seen_context) = make_tool("published", "BDT", true, Some(0), false);
        let offer = tool
            .execute(ProductOfferRequest {
                variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                currency_code: "BDT".to_string(),
                region_id: Some("reg_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
                quantity: 3,
            })
            .await
            .expect("resolve live offer");

        assert_eq!(
            offer.price.expect("merchant price").calculated_amount,
            Decimal::new(1234, 2)
        );
        assert_eq!(offer.available_quantity, Some(0));
        assert_eq!(offer.quantity, 3);
        let context = seen_context
            .lock()
            .expect("context lock")
            .clone()
            .expect("pricing called");
        assert_eq!(context.currency_code, "bdt");
        assert_eq!(context.quantity, Some(3));
        assert_eq!(
            context.region_id.as_deref(),
            Some("reg_01ARZ3NDEKTSV4RRFFQ69G5FAV")
        );
    }

    #[tokio::test]
    async fn unpublished_products_and_currency_mismatches_never_reach_pricing_or_stock() {
        let (tool, pricing_calls, inventory_calls, _) =
            make_tool("draft", "BDT", true, Some(8), false);
        assert!(matches!(
            tool.execute(ProductOfferRequest {
                variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                currency_code: "BDT".to_string(),
                region_id: None,
                quantity: 1,
            })
            .await,
            Err(Error::NotFound { .. })
        ));
        assert_eq!(pricing_calls.load(Ordering::SeqCst), 0);
        assert_eq!(inventory_calls.load(Ordering::SeqCst), 0);

        let (tool, pricing_calls, inventory_calls, _) =
            make_tool("published", "EUR", true, Some(8), false);
        assert!(matches!(
            tool.execute(ProductOfferRequest {
                variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                currency_code: "BDT".to_string(),
                region_id: Some("reg_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
                quantity: 1,
            })
            .await,
            Err(Error::Invalid {
                code: "ai_tool_region_currency_mismatch",
                ..
            })
        ));
        assert_eq!(pricing_calls.load(Ordering::SeqCst), 0);
        assert_eq!(inventory_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn missing_price_and_inventory_stay_unknown_instead_of_becoming_zero() {
        let (tool, _, _, _) = make_tool("published", "BDT", false, None, false);
        let offer = tool
            .execute(ProductOfferRequest {
                variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                currency_code: "BDT".to_string(),
                region_id: None,
                quantity: 1,
            })
            .await
            .expect("resolve incomplete offer");
        assert!(offer.price.is_none());
        assert!(offer.available_quantity.is_none());
    }

    #[tokio::test]
    async fn offer_timeout_is_reported_without_continuing_to_pricing_or_stock() {
        let (tool, pricing_calls, inventory_calls, _) =
            make_tool("published", "BDT", true, Some(8), true);
        assert!(matches!(
            tool.execute(ProductOfferRequest {
                variant_id: "variant_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                currency_code: "BDT".to_string(),
                region_id: None,
                quantity: 1,
            })
            .await,
            Err(Error::Database(message)) if message.contains("timed out")
        ));
        assert_eq!(pricing_calls.load(Ordering::SeqCst), 0);
        assert_eq!(inventory_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn model_dispatch_accepts_only_typed_read_only_tool_arguments() {
        let (offers, pricing_calls, _, _) = make_tool("published", "USD", true, Some(3), false);
        let shipping = ShippingQuoteTool {
            fulfillment: Arc::new(MockShipping),
            regions: Arc::new(MockRegion {
                currency_code: "USD".to_string(),
            }),
        };
        let tools = ReadOnlyCommerceTools::new(offers, shipping);
        let result = tools
            .execute("search_catalog", serde_json::json!({"query":"lamp"}))
            .await
            .expect("search via typed adapter");
        assert_eq!(result[0]["title"], "Desk lamp");

        assert!(tools
            .execute(
                "get_product_details",
                serde_json::json!({
                    "variant_id":"variant_01ARZ3NDEKTSV4RRFFQ69G5FAV",
                    "currency_code":"USD",
                    "quantity":1,
                    "unexpected":"field"
                })
            )
            .await
            .is_err());
        assert!(tools
            .execute("create_social_draft_order", serde_json::json!({}))
            .await
            .is_err());
        assert_eq!(pricing_calls.load(Ordering::SeqCst), 0);
    }
}
