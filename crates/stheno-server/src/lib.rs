//! `stheno-server`: Server application, router mounting, and universal test harness.

pub mod db;
pub mod scheduler;
pub mod space_integration;
pub mod tenant_layer;

use axum::http::{HeaderValue, Method};
use axum::{middleware, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::tenant_layer::TenantLayerState;
use stheno_api_admin::{
    admin_api_key_routes, admin_catalog_routes, admin_config_routes, admin_courier_routes,
    admin_customer_routes, admin_fulfillment_settings_routes, admin_home_routes,
    admin_inventory_routes, admin_localisation_routes, admin_onboarding_routes, admin_order_routes,
    admin_pricing_routes, admin_promotion_routes, admin_rbac_routes, admin_reconciliation_routes,
    admin_reports_routes, admin_settings_routes, admin_store_routes, admin_upload_routes,
    admin_user_routes, admin_views_routes, auth_guard::require_admin,
};
use stheno_api_admin_platform::{admin_platform_routes, AdminPlatformContext, UsageSnapshot};
use stheno_api_auth::{auth_routes, AuthApiContext};
use stheno_api_core::{Entitlements, EntitlementsError, Feature, Tier, TierConfig};
use stheno_api_platform::{
    merchant_routes, platform_routes, MerchantApiContext, PlatformApiContext,
};
use stheno_api_router::{gate_buyer_auth, mount_gated_for_request, GatedGroup};
use stheno_api_store::{
    hooks::{hooks_routes, HooksContext},
    store_customer_routes, store_routes, StoreApiContext,
};
use stheno_auth::AuthService;
use stheno_cart::CartService;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_db::QuotaCounter;
use stheno_flows::{AuthWorkflows, OrderWorkflowEngine, ReturnWorkflowEngine};
use stheno_fulfillment::FulfillmentService;
use stheno_fulfillment_manual::ManualFulfillmentProvider;
use stheno_fulfillment_pathao::PathaoFulfillmentProvider;
use stheno_fulfillment_redx::RedxFulfillmentProvider;
use stheno_fulfillment_steadfast::SteadfastFulfillmentProvider;
use stheno_inventory::InventoryService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_payment_bkash::BkashPaymentProviderFactory;
use stheno_payment_nagad::NagadPaymentProviderFactory;
use stheno_payment_offline::OfflinePaymentProviderFactory;
use stheno_payment_sslcommerz::SslcommerzPaymentProviderFactory;
use stheno_payment_stripe::StripePaymentProviderFactory;
use stheno_platform::PlatformService;
use stheno_pricing::PricingService;
use stheno_product::{CreateCollectionInput, CreateProductInput, ProductService};
use stheno_promotion::PromotionService;
use stheno_region::{CreateRegionInput, RegionService};
use stheno_registry::AppContext;
use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelService};
use stheno_stock_location::{CreateStockLocationInput, StockLocationService};
use stheno_store::{CreateStoreInput, StoreService};
use stheno_tax::{CreateTaxRateInput, CreateTaxRegionInput, TaxService};
use stheno_translation::{CreateLocaleInput, TranslationService};
use stheno_user::{CreateUserInput, UserService};
use stheno_workflow::WorkflowStore;

/// Tier definitions ship with the binary, overridable per deployment (ENTITLEMENTS.md 5b).
const DEFAULT_TIERS_TOML: &str = include_str!("../../../config/tiers.toml");

/// The tier *definitions*, loaded once. What a tier means is deployment config
/// (ENTITLEMENTS.md 5b); which tier a tenant is on is a `platform.subscription` row.
pub fn load_tier_config() -> Result<TierConfig, EntitlementsError> {
    match std::fs::read_to_string("config/tiers.toml") {
        Ok(src) => TierConfig::from_toml(&src),
        Err(_) => TierConfig::from_toml(DEFAULT_TIERS_TOML),
    }
}

/// Resolves this deployment's entitlements from tier config plus env overrides.
///
/// An unknown key or a malformed quota is an error, never a silent grant or deny --
/// callers abort startup rather than serving a deployment nobody can reason about.
pub fn load_entitlements() -> Result<Entitlements, EntitlementsError> {
    let config = match std::fs::read_to_string("config/tiers.toml") {
        Ok(src) => TierConfig::from_toml(&src)?,
        Err(_) => TierConfig::from_toml(DEFAULT_TIERS_TOML)?,
    };

    let tier: Tier = std::env::var("STHENO_TIER")
        .unwrap_or_else(|_| "free".to_string())
        .parse()?;

    let env: std::collections::BTreeMap<String, String> = std::env::vars()
        .filter(|(k, _)| k.starts_with("STHENO_QUOTA_") || k.starts_with("STHENO_WINDOW_"))
        .collect();

    config.resolve(tier, &env)
}

/// Origins allowed to call this deployment, from `STORE_CORS`, `ADMIN_CORS`, and
/// `AUTH_CORS` (SECURITY.md 6). Each is a comma-separated list.
///
/// An unset variable contributes nothing, so a deployment that configures none of
/// them accepts no cross-origin browser traffic at all. That is the safe direction
/// to fail: a same-origin deployment and every server-to-server client keep working,
/// and a misconfigured browser client fails visibly in preflight rather than silently
/// exposing the admin surface to every origin on the internet.
fn cors_layer() -> CorsLayer {
    let mut origins: Vec<HeaderValue> = ["STORE_CORS", "ADMIN_CORS", "AUTH_CORS"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .flat_map(|value| {
            value
                .split(',')
                .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
                .collect::<Vec<_>>()
        })
        .collect();

    for dev_origin in [
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://localhost:8000",
        "http://127.0.0.1:8000",
        "http://localhost:5175",
        "http://127.0.0.1:5175",
    ] {
        if let Ok(hv) = HeaderValue::from_str(dev_origin) {
            if !origins.contains(&hv) {
                origins.push(hv);
            }
        }
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // The requested headers are mirrored rather than allowlisted. The origin list
        // above is the security boundary — only origins on it get a response at all —
        // and a hand-maintained header list silently breaks clients as they evolve.
        // A fixed list here omitted `x-medusa-locale`, which `@medusajs/js-sdk` sends
        // on every call, so the dashboard could not even read its own auth providers
        // and rendered a login screen with no form on it.
        .allow_headers(AllowHeaders::mirror_request())
}

/// Populates an empty deployment with a demo catalogue, regions, tax, locales, and an
/// owner account. Invoked by `stheno-server --seed-demo`, never on boot.
///
/// This used to run unconditionally inside `AppServer::new`, which meant every restart
/// created another copy of everything and every failure was discarded. Seeding is an
/// operator action, so it is a command with an exit code.
/// Medusa's built-in manual/offline payment provider. The dashboard shows it as
/// "System Default" and the storefront labels it "Manual Payment".
const OFFLINE_PAYMENT_PROVIDER_ID: &str = "pp_system_default";

pub async fn seed_demo(pool: stheno_db::DbPool) {
    use chrono::Datelike;
    let product_service = ProductService::new(pool.clone());
    let region_service = RegionService::new(pool.clone());
    let user_service = UserService::new(pool.clone());
    let translation_service = TranslationService::new(pool.clone());
    let sales_channel_service = SalesChannelService::new(pool.clone());
    let stock_location_service = StockLocationService::new(pool.clone());
    let store_service = StoreService::new(pool.clone());
    let pricing_service = stheno_pricing::PricingService::new(pool.clone());
    let inventory_service = stheno_inventory::InventoryService::new(pool.clone());
    let link_store = stheno_links::PgLinkStore::new(pool.clone());
    let tax_service = TaxService::new(pool.clone());

    let sc_res = sales_channel_service
        .create_channel(CreateSalesChannelInput {
            name: "Default Web Channel".to_string(),
            description: Some("Main online storefront".to_string()),
            is_disabled: Some(false),
        })
        .await;

    let sl_res = stock_location_service
        .create_location(CreateStockLocationInput {
            name: "Main Fulfillment Center".to_string(),
            address_id: None,
        })
        .await;

    // `let _ =` makes a failed create harmless, but it does not make a *successful* one
    // idempotent: create_region has no uniqueness to collide with, so every re-run of the
    // seed minted another "North America" and another "Europe". Three seeds in, the admin's
    // region picker listed seven identical "North America / USD" entries with nothing in the
    // markup to tell them apart, and only the first one ever held the countries — the global
    // unique index on ISO code gave them to whoever seeded first. Reuse by name instead.
    let existing_regions = region_service.list_regions().await.unwrap_or_default();
    let existing_region = |name: &str| existing_regions.iter().find(|r| r.name == name).cloned();

    let reg_na = match existing_region("North America") {
        Some(region) => Ok(region),
        None => {
            region_service
                .create_region(CreateRegionInput {
                    name: "North America".to_string(),
                    currency_code: "USD".to_string(),
                    automatic_taxes: Some(true),
                    countries: vec!["us".to_string(), "ca".to_string()],
                })
                .await
        }
    };

    if existing_region("Europe").is_none() {
        let _ = region_service
            .create_region(CreateRegionInput {
                name: "Europe".to_string(),
                currency_code: "EUR".to_string(),
                automatic_taxes: Some(true),
                countries: vec!["de".to_string(), "fr".to_string(), "gb".to_string()],
            })
            .await;
    }

    // The seed is written `let _ =` throughout so re-running it is a no-op. That is
    // deliberate, but it hid a total outage once: a global unique index on country ISO
    // codes meant only the first tenant to seed got any, the rest silently got regions
    // with no countries, and the storefront's middleware — which builds its region map
    // from exactly this — answered 500 on every page. Regions without countries are not
    // a partial seed, they are an unusable storefront, so this one result is checked.
    if let Ok(region) = &reg_na {
        if region.countries.is_empty() {
            eprintln!(
                "seed: region {} has no countries — the storefront cannot route any page.                  Check region.country for a constraint rejecting this tenant's rows.",
                region.id
            );
        }
    }

    // Delivery. Without a fulfillment set, a service zone covering the region's countries
    // and a shipping option inside it, `GET /store/shipping-options` returns an empty list
    // and the storefront's checkout stops dead at the delivery step — the buyer can fill in
    // an address and then has nothing to pick. Every piece of this existed as an admin
    // route; nothing seeded it, so no checkout had ever been completed end to end.
    let fulfillment_service = stheno_fulfillment::FulfillmentService::new(pool.clone());
    let shipping = async {
        let profile = fulfillment_service
            .create_shipping_profile(stheno_fulfillment::CreateShippingProfileInput {
                name: "Default".to_string(),
                r#type: Some("default".to_string()),
                metadata: None,
            })
            .await
            .ok()?;
        let set = fulfillment_service
            .create_fulfillment_set(stheno_fulfillment::CreateFulfillmentSetInput {
                name: "Default Delivery".to_string(),
                r#type: "shipping".to_string(),
                metadata: None,
            })
            .await
            .ok()?;
        let zone = fulfillment_service
            .create_service_zone(stheno_fulfillment::CreateServiceZoneInput {
                fulfillment_set_id: set.id.clone(),
                name: "Everywhere".to_string(),
                metadata: None,
            })
            .await
            .ok()?;

        // The countries the two seeded regions cover, so a cart in either resolves an
        // option rather than only the US one.
        for country in ["us", "ca", "de", "fr", "gb"] {
            let _ = fulfillment_service
                .create_geo_zone(stheno_fulfillment::CreateGeoZoneInput {
                    service_zone_id: zone.id.clone(),
                    r#type: "country".to_string(),
                    country_code: country.to_string(),
                    province_code: None,
                    city: None,
                    postal_expression: None,
                    metadata: None,
                })
                .await;
        }

        // Priced, in both seeded currencies. An unpriced shipping option is legitimate
        // (free delivery) but makes "is shipping actually charged" untestable, and the
        // cart used to hardcode zero regardless.
        for (name, usd, eur) in [
            (
                "Standard Delivery",
                rust_decimal_macros::dec!(10.00),
                rust_decimal_macros::dec!(9.00),
            ),
            (
                "Express Delivery",
                rust_decimal_macros::dec!(25.00),
                rust_decimal_macros::dec!(22.00),
            ),
        ] {
            let Ok(option) = fulfillment_service
                .create_shipping_option(stheno_fulfillment::CreateShippingOptionInput {
                    name: name.to_string(),
                    price_type: "flat".to_string(),
                    service_zone_id: zone.id.clone(),
                    shipping_profile_id: profile.id.clone(),
                    provider_id: "manual".to_string(),
                    shipping_option_type_id: None,
                    data: None,
                    metadata: None,
                })
                .await
            else {
                continue;
            };

            let _ = pricing_service
                .set_shipping_option_prices(
                    &option.id,
                    vec![
                        stheno_pricing::CreatePriceInput {
                            currency_code: "usd".to_string(),
                            amount: usd,
                            min_quantity: None,
                            max_quantity: None,
                            price_list_id: None,
                            rules: None,
                        },
                        stheno_pricing::CreatePriceInput {
                            currency_code: "eur".to_string(),
                            amount: eur,
                            min_quantity: None,
                            max_quantity: None,
                            price_list_id: None,
                            rules: None,
                        },
                    ],
                )
                .await;
        }
        Some(())
    }
    .await;
    if shipping.is_none() {
        eprintln!("seed: no shipping option was created — checkout cannot reach payment.");
    }

    // Payment. The seed built the whole delivery chain and stopped exactly where the line
    // above says it stops, so checkout reached the payment step and found nothing to pick:
    // `GET /store/payment-providers` lists a provider only when the tenant has a credential
    // row for it *and* a region link, and the seed created neither. The offline provider
    // needs no keys — its credential schema is empty — but it still needs the row.
    let payment_service = stheno_payment::PaymentService::new(pool.clone());
    payment_service.register_factory(std::sync::Arc::new(OfflinePaymentProviderFactory));

    if let Ok(region) = &reg_na {
        match stheno_db::tenant::current() {
            Ok(tenant_id) => {
                match payment_service
                    .upsert_provider_credential(
                        &tenant_id,
                        OFFLINE_PAYMENT_PROVIDER_ID,
                        &stheno_payment::CredentialMap::new(),
                    )
                    .await
                {
                    Ok(_) => {
                        let _ = link_store
                            .link(
                                "region_payment_provider",
                                &region.id.to_string(),
                                OFFLINE_PAYMENT_PROVIDER_ID,
                            )
                            .await;
                    }
                    // Credentials are encrypted before storage, so this is where a missing
                    // STHENO_CREDENTIAL_KEY surfaces. Say so: the symptom is otherwise a
                    // checkout that reaches payment and offers no method.
                    Err(e) => eprintln!(
                        "seed: could not configure the offline payment provider ({e}) — checkout will reach payment with no method to select."
                    ),
                }
            }
            Err(e) => eprintln!("seed: no tenant in scope for payment provider setup ({e})."),
        }
    }

    let tr_res = tax_service
        .create_tax_region(CreateTaxRegionInput {
            country_code: "us".to_string(),
            province_code: None,
            parent_id: None,
            provider_id: None,
        })
        .await;

    if let Ok(tr) = tr_res {
        let _ = tax_service
            .create_tax_rate(CreateTaxRateInput {
                tax_region_id: tr.id.to_string(),
                rate: rust_decimal_macros::dec!(0.10),
                code: "STANDARD_VAT".to_string(),
                name: "Standard Sales Tax (10%)".to_string(),
                is_default: Some(true),
                is_combinable: Some(false),
                rules: None,
            })
            .await;
    }

    let _ = store_service
        .create_store(CreateStoreInput {
            name: "Stheno Store".to_string(),
            default_sales_channel_id: sc_res.ok().map(|s| s.id.to_string()),
            default_region_id: reg_na.ok().map(|r| r.id.to_string()),
            default_location_id: sl_res.as_ref().ok().map(|l| l.id.to_string()),
            supported_currencies: Some(vec!["usd".to_string(), "eur".to_string()]),
            default_currency_code: Some("usd".to_string()),
            supported_locales: Some(vec!["en".to_string()]),
            default_locale: Some("en".to_string()),
        })
        .await;

    // Products are seeded with a size option, one variant per size, and a price per
    // variant. Previously they were seeded bare: no options, no variants, no prices — so
    // the storefront rendered a catalogue nothing could be selected from or added to a
    // cart, and every checkout journey was untestable end to end.
    // `thumbnail` was `None` on both, so the demo catalogue rendered two blank
    // product cards and the Featured / Latest Drops rails looked empty even
    // though they had products. Served as storefront-relative paths so the demo
    // needs no asset host: the storefront's `public/brand/editorial` serves them.
    let demo_products = [
        (
            "Stheno Performance Hoodie",
            "stheno-performance-hoodie",
            "Heavyweight Organic Cotton Fleece",
            "Engineered for comfort and durability with signature flatlock stitching.",
            "HOODIE",
            "/brand/editorial/product-hoodie-1.jpg",
            rust_decimal_macros::dec!(120.00),
            rust_decimal_macros::dec!(110.00),
        ),
        (
            "Stheno Signature Tee",
            "stheno-signature-tee",
            "240 GSM Combed Jersey",
            "Relaxed fit premium daily essential.",
            "TEE",
            "/brand/editorial/product-tee-1.jpg",
            rust_decimal_macros::dec!(45.00),
            rust_decimal_macros::dec!(40.00),
        ),
    ];

    for (title, handle, subtitle, description, sku_prefix, thumbnail, usd, eur) in demo_products {
        let Ok(product) = product_service
            .create_product(CreateProductInput {
                title: title.to_string(),
                handle: Some(handle.to_string()),
                subtitle: Some(subtitle.to_string()),
                description: Some(description.to_string()),
                discountable: Some(true),
            })
            .await
        else {
            // Already seeded — the handle is unique per tenant.
            continue;
        };

        let product_id = product.id.to_string();

        // Products are created as drafts, and the store API serves published ones only.
        // A demo catalog nobody can see is not a demo, so publish what we just seeded.
        let _ = product_service
            .update_product(
                &product_id,
                stheno_product::UpdateProductInput {
                    title: None,
                    handle: None,
                    subtitle: None,
                    description: None,
                    status: Some("published".to_string()),
                    thumbnail: Some(thumbnail.to_string()),
                    discountable: None,
                },
            )
            .await;

        let Ok(option_id) = product_service.create_option(&product_id, "Size").await else {
            continue;
        };

        for size in ["S", "M", "L"] {
            let Ok(value_id) = product_service.create_option_value(&option_id, size).await else {
                continue;
            };
            let Ok(variant) = product_service
                .create_variant(stheno_product::CreateVariantInput {
                    product_id: product_id.clone(),
                    title: format!("{title} / {size}"),
                    sku: Some(format!("{sku_prefix}-{size}")),
                    barcode: None,
                    option_value_ids: vec![value_id],
                })
                .await
            else {
                continue;
            };

            // Stock, linked to the variant. Without an inventory item every variant is
            // stock-tracked with an unknown quantity, which the storefront renders as
            // "Out of stock" — so the catalogue looked entirely sold out.
            if let Ok(location) = &sl_res {
                if let Ok(item) = inventory_service
                    .create_item(stheno_inventory::CreateInventoryItemInput {
                        sku: Some(format!("{sku_prefix}-{size}")),
                        requires_shipping: Some(true),
                    })
                    .await
                {
                    let item_id = item.id.to_string();
                    let _ = link_store
                        .link(
                            "product_variant_inventory_item",
                            &variant.id.to_string(),
                            &item_id,
                        )
                        .await;
                    let _ = inventory_service
                        .create_level(stheno_inventory::CreateInventoryLevelInput {
                            inventory_item_id: item_id,
                            location_id: location.id.to_string(),
                            stocked_quantity: 100,
                        })
                        .await;
                }
            }

            // Both seeded currencies, because the storefront prices from the shopper's
            // region and a variant priced only in USD is invisible in the EU store.
            let _ = pricing_service
                .set_variant_prices(
                    &variant.id.to_string(),
                    vec![
                        stheno_pricing::CreatePriceInput {
                            currency_code: "usd".to_string(),
                            amount: usd,
                            min_quantity: None,
                            max_quantity: None,
                            price_list_id: None,
                            rules: None,
                        },
                        stheno_pricing::CreatePriceInput {
                            currency_code: "eur".to_string(),
                            amount: eur,
                            min_quantity: None,
                            max_quantity: None,
                            price_list_id: None,
                            rules: None,
                        },
                    ],
                )
                .await;
        }
    }

    for (title, handle) in [("Featured", "featured"), ("Latest Drops", "latest-drops")] {
        let _ = product_service
            .create_collection(CreateCollectionInput {
                title: title.to_string(),
                handle: Some(handle.to_string()),
                metadata: None,
            })
            .await;
    }

    for (code, name, is_default) in [("en", "English", true), ("de", "Deutsch", false)] {
        let _ = translation_service
            .create_locale(CreateLocaleInput {
                code: code.to_string(),
                name: name.to_string(),
                is_default: Some(is_default),
                is_enabled: Some(true),
            })
            .await;
    }

    // Reporting rollups, so a demo deployment's analytics screen has something to plot.
    // Without these the charts render their empty state and nothing about the reporting
    // path is exercised end to end.
    let reporting_service = stheno_reporting::ReportingService::new(pool.clone());
    let today = chrono::Utc::now().date_naive();
    let titles = [
        "Stheno Performance Hoodie",
        "Stheno Signature Tee",
        "Selvedge Denim Overshirt",
    ];
    for day_offset in 0..45i64 {
        let date = today - chrono::Duration::days(day_offset);
        // A weekly rhythm rather than a flat line, so the chart shows shape.
        let weekday_factor = match date.weekday() {
            chrono::Weekday::Sat | chrono::Weekday::Sun => 60i64,
            chrono::Weekday::Wed | chrono::Weekday::Thu => 130,
            _ => 100,
        };
        let orders_today = 2 + (day_offset * 7 % 5) + weekday_factor / 65;
        for order in 0..orders_today {
            let revenue = rust_decimal::Decimal::from(
                (45 + order * 17 + (day_offset % 11) * 6) * weekday_factor / 100,
            );
            let units = 1 + order % 3;
            let index = (order % 3) as usize;
            let _ = reporting_service
                .record_order_placed(stheno_reporting::RecordOrderPlacedInput {
                    date,
                    currency: "USD".to_string(),
                    total_revenue: revenue,
                    units,
                    items: vec![stheno_reporting::OrderPlacedRollupItem {
                        product_id: format!("prod_demo_{index}"),
                        title: titles[index].to_string(),
                        units,
                        total_revenue: revenue,
                    }],
                })
                .await;
        }
    }

    // Default RBAC roles per tenant. `seed_default_roles` has existed since P7-18 and
    // was never called, which is why the roles screen had nothing real to show.
    let rbac_service = stheno_rbac::RbacService::new(pool.clone());
    if let Err(e) = rbac_service.seed_default_roles().await {
        tracing::warn!("could not seed default roles: {e}");
    }

    let _ = user_service
        .create_user(CreateUserInput {
            email: "admin@stheno.store".to_string(),
            first_name: Some("System".to_string()),
            last_name: Some("Administrator".to_string()),
            avatar_url: None,
            role: Some("admin".to_string()),
            metadata: None,
        })
        .await;
}

/// Every 15 minutes. Frequent enough that an ambiguous payment is resolved while the buyer
/// is still in the conversation; infrequent enough that it is not a load source of its own.
pub const PAYMENT_SWEEP_SCHEDULE: &str = "*/15 * * * *";

/// Daily at 03:00. The dunning ladder moves in days, so a finer schedule would re-read every
/// subscription to change nothing.
pub const DUNNING_SCHEDULE: &str = "0 3 * * *";

/// Daily at 01:00. Fetch FX rates for platform subscriptions.
pub const FX_FETCH_SCHEDULE: &str = "0 1 * * *";

/// Requests older than this are considered stranded rather than merely in progress.
const SWEEP_MIN_AGE_SECONDS: i64 = 300;
const SWEEP_LIMIT: usize = 500;

/// The periodic work the engine owes itself.
///
/// Both jobs iterate tenants explicitly. Neither underlying method scopes itself: they read
/// tenant-owned tables under RLS, so a sweep run without a tenant scope sees nothing and
/// reports a clean pass over an empty result — the silent no-op this scheduler exists to end.
pub fn scheduled_jobs(
    pool: stheno_db::DbPool,
    payment: Arc<PaymentService>,
) -> Vec<scheduler::Job> {
    let platform = Arc::new(PlatformService::new(pool.clone()));

    let sweep_platform = platform.clone();
    let sweep = scheduler::Job::new("payment-reconcile", PAYMENT_SWEEP_SCHEDULE, move || {
        let platform = sweep_platform.clone();
        let payment = payment.clone();
        async move {
            let tenants = platform.list_tenants().await.map_err(|e| e.to_string())?;
            let (mut scanned, mut converged) = (0usize, 0usize);
            for tenant in &tenants {
                let report = stheno_db::tenant::scope(
                    tenant.id.clone(),
                    payment.reconcile_unsettled_requests(SWEEP_MIN_AGE_SECONDS, SWEEP_LIMIT),
                )
                .await
                .map_err(|e| format!("tenant {}: {e}", tenant.id))?;
                scanned += report.scanned;
                converged += report.converged;
            }
            Ok(format!(
                "swept {} tenant(s): {scanned} stranded request(s) scanned, {converged} converged",
                tenants.len()
            ))
        }
    });

    let dunning_platform = platform.clone();
    let dunning = scheduler::Job::new("subscription-dunning", DUNNING_SCHEDULE, move || {
        let platform = dunning_platform.clone();
        async move {
            let subscriptions = platform
                .list_subscriptions()
                .await
                .map_err(|e| e.to_string())?;
            let now = chrono::Utc::now();
            let mut advanced = 0usize;
            let mut failed = 0usize;
            for sub in &subscriptions {
                let input = stheno_platform::DunningInput {
                    tenant_id: sub.tenant_id.clone(),
                    now,
                };
                // One tenant's ladder failing must not stop the rest: a single bad
                // subscription row would otherwise freeze collection for everybody.
                match platform.advance_dunning(input).await {
                    Ok(_) => advanced += 1,
                    Err(e) => {
                        failed += 1;
                        tracing::warn!(tenant = %sub.tenant_id, error = %e, "dunning advance failed");
                    }
                }
            }
            Ok(format!(
                "advanced {advanced} subscription(s), {failed} failed, of {} total",
                subscriptions.len()
            ))
        }
    });

    let fx_platform = platform.clone();
    let fx_fetch = scheduler::Job::new("fx-rate-fetch", FX_FETCH_SCHEDULE, move || {
        let platform = fx_platform.clone();
        async move {
            let fetcher = stheno_flows_platform::fx::BangladeshBankRateFetcher::new(platform);
            match fetcher.fetch_and_store().await {
                Ok(_) => Ok("FX rate fetched successfully".to_string()),
                Err(e) => Err(format!("FX rate fetch failed: {}", e)),
            }
        }
    });

    vec![sweep, dunning, fx_fetch]
}

pub struct AppServer {
    pub context: AppContext,
    pub router: Router,
    pub entitlements: Arc<Entitlements>,
    pub pool: stheno_db::DbPool,
    pub payment_service: Arc<PaymentService>,
    pub notification_service: Arc<stheno_notification::NotificationService>,
}

impl AppServer {
    /// Every path needs a database: migrated services hold repositories, not
    /// maps, so there is no longer a meaningful server without one.
    pub fn new(context: AppContext, entitlements: Entitlements, pool: stheno_db::DbPool) -> Self {
        let entitlements = Arc::new(entitlements);
        let product_service = Arc::new(ProductService::new(pool.clone()));
        let region_service = Arc::new(RegionService::new(pool.clone()));
        let currency_service = Arc::new(CurrencyService::new(pool.clone()));
        let pricing_service = Arc::new(PricingService::new(pool.clone()));
        let cart_service = Arc::new(CartService::new(pool.clone()));
        let payment_service = Arc::new(PaymentService::new(pool.clone()));
        payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));
        payment_service.register_factory(Arc::new(StripePaymentProviderFactory));
        payment_service.register_factory(Arc::new(SslcommerzPaymentProviderFactory));
        payment_service.register_factory(Arc::new(BkashPaymentProviderFactory));
        payment_service.register_factory(Arc::new(NagadPaymentProviderFactory));
        let order_service = Arc::new(OrderService::new(pool.clone()));
        let fulfillment_service = Arc::new(FulfillmentService::new(pool.clone()));
        fulfillment_service.register_provider(Arc::new(ManualFulfillmentProvider::new()));
        fulfillment_service.register_provider(Arc::new(PathaoFulfillmentProvider::default()));
        fulfillment_service.register_provider(Arc::new(SteadfastFulfillmentProvider::default()));
        fulfillment_service.register_provider(Arc::new(RedxFulfillmentProvider::default()));
        let tax_service = Arc::new(TaxService::new(pool.clone()));
        let customer_service = Arc::new(CustomerService::new(pool.clone()));
        let user_service =
            Arc::new(UserService::new(pool.clone()).with_event_bus(context.events.clone()));
        let inventory_service = Arc::new(InventoryService::new(pool.clone()));
        let stock_location_service = Arc::new(StockLocationService::new(pool.clone()));
        let sales_channel_service = Arc::new(SalesChannelService::new(pool.clone()));
        let store_service = Arc::new(StoreService::new(pool.clone()));
        let promotion_service = Arc::new(PromotionService::new(pool.clone()));
        let auth_service =
            Arc::new(AuthService::new(pool.clone(), None).with_event_bus(context.events.clone()));
        let notification_service =
            Arc::new(stheno_notification::NotificationService::new(pool.clone()));
        let resend_config = stheno_notification_resend::ResendConfig {
            api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            from_email: std::env::var("RESEND_FROM_EMAIL")
                .unwrap_or_else(|_| "notifications@vendryx.store".to_string()),
            from_name: std::env::var("RESEND_FROM_NAME")
                .ok()
                .or_else(|| Some("Vendryx Commerce".to_string())),
            base_url: std::env::var("RESEND_BASE_URL").ok(),
        };
        let resend_provider = Arc::new(
            stheno_notification_resend::ResendNotificationProvider::new(resend_config),
        );
        notification_service.register_provider(resend_provider);

        let store_name =
            std::env::var("STHENO_STORE_NAME").unwrap_or_else(|_| "Vendryx".to_string());
        let subscriber_registry = stheno_subscribers::SubscriberRegistry::new();
        subscriber_registry.attach_subscriber(
            &*context.events,
            Arc::new(stheno_subscribers::IdentityCreatedSubscriber::new(
                notification_service.clone(),
                store_name.clone(),
            )),
        );
        subscriber_registry.attach_subscriber(
            &*context.events,
            Arc::new(stheno_subscribers::PasswordResetSubscriber::new(
                notification_service.clone(),
                store_name.clone(),
            )),
        );
        subscriber_registry.attach_subscriber(
            &*context.events,
            Arc::new(stheno_subscribers::UserInviteSubscriber::new(
                notification_service.clone(),
                store_name.clone(),
            )),
        );
        subscriber_registry.attach_subscriber(
            &*context.events,
            Arc::new(stheno_subscribers::OrderConfirmationEmailSubscriber::new(
                notification_service.clone(),
                store_name.clone(),
            )),
        );
        if let Ok(owner_push_sub) = std::env::var("OWNER_PUSH_SUBSCRIPTION") {
            subscriber_registry.attach_subscriber(
                &*context.events,
                Arc::new(stheno_subscribers::PushOrderPlacedSubscriber::new(
                    notification_service.clone(),
                    owner_push_sub,
                )),
            );
        }
        let translation_service = Arc::new(TranslationService::new(pool.clone()));
        let reporting_service = Arc::new(stheno_reporting::ReportingService::new(pool.clone()));
        let workflow_store = Arc::new(WorkflowStore::from_pool(pool.clone()));
        let platform_service = Arc::new(PlatformService::new(pool.clone()));
        let rbac_service = Arc::new(stheno_rbac::RbacService::new(pool.clone()));
        let api_key_service = Arc::new(stheno_api_key::ApiKeyService::new(pool.clone()));
        let settings_service = Arc::new(stheno_settings::SettingsService::new(pool.clone()));
        let auth_flows = Arc::new(AuthWorkflows::new(auth_service.clone()));
        let order_flow = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
        let social_flows = Arc::new(stheno_flows_order::SocialDraftWorkflow::new(
            order_service.clone(),
        ));
        // Turns a cart into an order. Built with tests since the cart phase and never
        // mounted, so "Place order" had nothing to call.
        let file_provider = Arc::new(stheno_file_local::LocalFileProvider::new(
            "uploads", "/uploads",
        ));
        let file_service = Arc::new(stheno_file::FileService::new(file_provider, pool.clone()));
        let complete_flows = Arc::new(stheno_flows::CompleteCartWorkflowEngine::new(
            cart_service.clone(),
            order_service.clone(),
            payment_service.clone(),
            promotion_service.clone(),
            Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
            // The same store the SAGA engine already uses, so checkout idempotency and
            // resume checkpoints survive a restart instead of living in a HashMap on this
            // process.
            workflow_store.clone(),
        ));
        let return_flow = Arc::new(ReturnWorkflowEngine::new(
            order_service.clone(),
            Some(inventory_service.clone()),
        ));

        // Quota usage is read from the counter table on every request, so the
        // dashboard's quota bars and grace banner reflect the database rather than
        // a value fixed at boot (ENTITLEMENTS.md 6.2).
        let usage_snapshot = Arc::new(UsageSnapshot::from_counter(QuotaCounter::new(pool.clone())));

        let cors = cors_layer();

        // The never-gateable admin surface (ENTITLEMENTS.md 4.1). None of it costs
        // marginal money, so none of it is sellable.
        let admin_core = Router::new()
            .merge(admin_catalog_routes(
                product_service.clone(),
                pricing_service.clone(),
                inventory_service.clone(),
            ))
            .merge(admin_order_routes(
                order_service.clone(),
                fulfillment_service.clone(),
                payment_service.clone(),
            ))
            .merge(admin_customer_routes(customer_service.clone()))
            .merge(admin_inventory_routes(
                inventory_service.clone(),
                stock_location_service.clone(),
            ))
            .merge(admin_pricing_routes(pricing_service.clone()))
            .merge(admin_promotion_routes(promotion_service.clone()))
            .merge(admin_user_routes(user_service.clone()))
            .merge(admin_upload_routes(file_service.clone()))
            .merge(admin_store_routes(
                store_service.clone(),
                translation_service.clone(),
            ))
            .merge(admin_config_routes(
                region_service.clone(),
                currency_service.clone(),
                tax_service.clone(),
                sales_channel_service.clone(),
                payment_service.clone(),
                Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
            ))
            .merge(admin_api_key_routes(api_key_service.clone()))
            .merge(admin_views_routes())
            .merge(admin_rbac_routes(rbac_service.clone()))
            .merge(admin_fulfillment_settings_routes(
                fulfillment_service.clone(),
            ))
            .merge(admin_localisation_routes(
                translation_service.clone(),
                settings_service.clone(),
            ))
            .merge(admin_settings_routes(settings_service.clone()))
            .merge(admin_home_routes(
                order_service.clone(),
                inventory_service.clone(),
            ))
            .merge(admin_courier_routes(
                fulfillment_service.clone(),
                settings_service.clone(),
            ))
            .merge(admin_onboarding_routes(
                store_service.clone(),
                product_service.clone(),
                fulfillment_service.clone(),
                settings_service.clone(),
            ))
            .merge(admin_reconciliation_routes(payment_service.clone()))
            // The platform surface is never gated: the dashboard has to be able to
            // read its own entitlements in order to hide what it may not show.
            .merge(admin_platform_routes(AdminPlatformContext {
                entitlements: entitlements.clone(),
                usage: usage_snapshot.clone(),
                workflows: workflow_store.clone(),
                platform: Some(platform_service.clone()),
            }));

        // A gate is a mount-point decision, never an `if` inside domain logic
        // (ENTITLEMENTS.md 6.1). An unentitled route 404s, which is axum's default
        // for something that was never mounted -- deliberately not a 403, because a
        // 403 confirms the feature exists and invites probing.
        //
        // Reporting carries `analytics_basic`, which every tier holds today, so this
        // group mounts everywhere. It is wired through the gate anyway because the
        // tier-matrix test (ENTITLEMENTS.md 8) asserts against the mounted set: the
        // day a tier drops the feature, this fails loudly instead of leaking it.
        // What actually separates Free from Pro on the reporting surface is the
        // window clamp inside `admin_reports_routes`, not the mount.
        let admin_routes = mount_gated_for_request(
            admin_core,
            vec![GatedGroup::new(
                Feature::AnalyticsBasic,
                admin_reports_routes(reporting_service.clone()),
            )],
        )
        // Every /admin route routes through one credential check rather than
        // eighty-five (SECURITY.md 6). Auth and store routes are mounted outside
        // it: /auth is how you obtain the token in the first place.
        .layer(middleware::from_fn_with_state(
            auth_service.clone(),
            require_admin,
        ));

        let tenant_routes = Router::new()
            .route("/health", get({
                let pool = pool.clone();
                move || {
                    let pool = pool.clone();
                    async move {
                        // Pool saturation is reported, not asserted: a deployment
                        // running out of connections must be visible here before
                        // it starts timing out (P11-03).
                        let h = crate::db::PoolHealth::of(&pool);
                        let database = json!({
                            "pool_size": h.size,
                            "pool_max_size": h.max_size,
                            "pool_available": h.available,
                            "pool_saturation": h.saturation(),
                        });
                        Json(json!({
                            "status": "healthy",
                            "engine": "Stheno Commerce Engine",
                            "version": "0.1.0",
                            "architecture": "Modular Monolith (Rust / Axum)",
                            "compatibility": "Commerce REST v2",
                            "database": database
                        }))
                    }
                }
            }))
            .route("/", get(|| async {
                Json(json!({
                    "service": "Stheno Commerce API Server",
                    "store_endpoints": "/store/products, /store/regions, /store/carts, /store/orders",
                    "admin_endpoints": "/admin/products, /admin/orders, /admin/customers, /admin/users",
                    "auth_endpoints": "/auth/token, /auth/refresh",
                    "docs": "https://docs.stheno.store"
                }))
            }))
            // The one bit the storefront cannot infer. Buyer accounts are a per-tenant
            // archetype, and a single build serves every tenant, so the shape of the
            // account routes cannot be decided at build time (FRONTEND_MATRIX 3) or read
            // from env. Defined here rather than in stheno-api-store because reporting an
            // entitlement needs `Entitlements`, and that crate has no dependency on
            // stheno-api-core — the same reason /health and / are defined here.
            //
            // Reports the archetype only, never the feature map: this route is public.
            .route(
                "/store/config",
                get(|request: axum::extract::Request| async move {
                    let buyer_accounts = request
                        .extensions()
                        .get::<Arc<Entitlements>>()
                        .is_some_and(|entitlements| entitlements.has(Feature::BuyerAccounts));
                    Json(json!({ "buyer_accounts": buyer_accounts }))
                }),
            )
            .merge(store_routes(StoreApiContext {
                social_flows: social_flows.clone(),
                store_service: store_service.clone(),
                store_name: std::env::var("STHENO_STORE_NAME")
                    .unwrap_or_else(|_| "Stheno Store".to_string()),
                product_service: product_service.clone(),
                region_service: region_service.clone(),
                currency_service: currency_service.clone(),
                inventory_service: inventory_service.clone(),
                links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
                complete_flows: complete_flows.clone(),
                pricing_service: pricing_service.clone(),
                cart_service: cart_service.clone(),
                payment_service: payment_service.clone(),
                order_service: order_service.clone(),
                order_flows: order_flow.clone(),
                return_flows: return_flow.clone(),
                fulfillment_service: fulfillment_service.clone(),
                auth_service: auth_service.clone(),
                translation_service: translation_service.clone(),
                promotion_service: Some(promotion_service.clone()),
                cache_service: Some(Arc::new(stheno_caching::CacheService::new(Arc::new(
                    stheno_caching::InMemoryCacheProvider::new(),
                )))),
            }))
            // Buyer accounts are an archetype the merchant chooses on the store request,
            // not something a plan sells (PRODUCT.md 2, ENTITLEMENTS.md 4.2). No tier in
            // config/tiers.toml grants `buyer_accounts`; it arrives as the per-merchant
            // override provisioning writes. Gated per request, so one process can serve
            // both archetypes, and unentitled means 404 — never 403.
            .merge(mount_gated_for_request(
                Router::new(),
                vec![GatedGroup::new(
                    Feature::BuyerAccounts,
                    store_customer_routes(customer_service.clone(), auth_service.clone()),
                )],
            ))
            .merge(admin_routes)
            .merge(gate_buyer_auth(auth_routes(AuthApiContext {
                auth_flows,
                // One limiter for the process, shared by every auth route. Without it the
                // login endpoint had no throttle at all and password grinding was free.
                login_limiter: Arc::new(stheno_http::rate_limit::RateLimiter::new(
                    stheno_api_auth::LOGIN_MAX_ATTEMPTS,
                    stheno_api_auth::LOGIN_WINDOW_SECS,
                )),
            })))
            // Inbound gateway callbacks belong inside the tenant scope, not beside it.
            // Mounted outside, `/hooks/payment/{provider}` answered every callback with
            // `tenant_not_established`: the handler resolves the tenant's credentials to
            // verify the signature, and there was no tenant to resolve them for. No bKash,
            // SSLCommerz or Stripe callback could ever be processed, so a payment
            // authorised at the gateway never became a paid order here.
            //
            // The route's own test wraps its calls in `tenant::scope` by hand, so it passed
            // throughout — the gap was in the mounting, which no test exercised.
            //
            // A gateway must therefore call the merchant's own host, exactly as a shopper
            // does. That is the same rule as every other route, and it is what makes one
            // callback URL per tenant resolve to that tenant.
            .merge(hooks_routes(HooksContext {
                payment_service: payment_service.clone(),
                fulfillment_service: Some(fulfillment_service.clone()),
            }))
            // Outermost after CORS: every route below this point runs inside a tenant
            // scope, so nothing downstream has to know tenancy exists (ADR-012).
            // `/health` and `/` opt out inside the layer — they report on the process,
            // not on a merchant.
            .layer(middleware::from_fn_with_state(
                TenantLayerState {
                    platform: platform_service.clone(),
                    tier_config: Arc::new(
                        load_tier_config().expect("config/tiers.toml must parse"),
                    ),
                    // The boot-time value, used only when a subscription cannot be read.
                    fallback_entitlements: entitlements.clone(),
                    // `STHENO_TENANT` keeps a single-merchant self-hosted install working
                    // with no domain registry, which is the deployment shape ADR-009
                    // described and which remains supported.
                    fallback_tenant: std::env::var("STHENO_TENANT").ok(),
                },
                crate::tenant_layer::resolve_tenant,
            ));

        let space_client = Arc::new(stheno_connector_space::SpaceClient::new(
            stheno_connector_space::SpaceConnectorConfig::from_env(),
        ));
        let space_routes =
            space_integration::space_integration_routes(space_integration::SpaceIntegrationState {
                client: space_client,
            });

        let router = Router::new()
            .merge(tenant_routes)
            // Mounted OUTSIDE the tenant layer on purpose: the control plane reads across
            // every merchant, so scoping it to one would defeat it. It carries its own
            // credential and 404s without it (ADR-012).
            .merge(platform_routes(PlatformApiContext {
                platform: platform_service.clone(),
                tier_config: Arc::new(load_tier_config().expect("config/tiers.toml must parse")),
                operator_token: std::env::var("STHENO_PLATFORM_TOKEN").ok(),
                payment_service: Some(payment_service.clone()),
                fulfillment_service: Some(fulfillment_service.clone()),
            }))
            .merge(merchant_routes(MerchantApiContext {
                platform: platform_service.clone(),
                auth_service: auth_service.clone(),
                tier_config: Arc::new(load_tier_config().expect("config/tiers.toml must parse")),
            }))
            .merge(space_routes)
            .layer(cors);

        Self {
            context,
            router,
            entitlements,
            pool,
            payment_service,
            notification_service,
        }
    }
}

/// Universal test harness: an isolated, migrated, ephemeral backend per test.
///
/// # Why a database and not a schema
///
/// P11-16 was written as "provisions an isolated schema". It provisions an
/// isolated **database** instead, because every module owns a named Postgres
/// schema of its own -- `product.product`, `cart.cart`, `order.order` -- so
/// "one schema per test" would collide with the schema names the migrations
/// themselves create. Isolating a level up is the only boundary that actually
/// separates two concurrent tests without rewriting all 25 modules' migrations
/// to be schema-relative.
#[cfg(feature = "testing")]
pub struct TestApp {
    pub context: AppContext,
    /// Name of this test's dedicated database.
    pub database_name: String,
    pub database_url: String,
    pub app: Router,
    db: stheno_db::testing::TestDb,
}

#[cfg(feature = "testing")]
impl TestApp {
    /// Creates a dedicated database, migrates it, and mounts the full router
    /// against a pool pointed at it.
    ///
    /// Phase 1's version generated a schema name and then ignored it, so every
    /// test in the suite shared one in-memory backend.
    pub async fn spawn() -> Self {
        let db = stheno_db::testing::TestDb::migrated().await;
        let context = AppContext::create_test_context();
        let entitlements = load_entitlements().expect("tier config must resolve");

        let pool = stheno_db::create_pool(&db.url, 5).expect("test pool must build");
        let server = AppServer::new(context.clone(), entitlements, pool);

        Self {
            context,
            database_name: db.name.clone(),
            database_url: db.url.clone(),
            app: server.router,
            db,
        }
    }

    /// Direct access to the test database, for assertions about persisted rows.
    pub fn db(&self) -> &stheno_db::testing::TestDb {
        &self.db
    }

    /// Drops this test's database. Async, so it cannot be a `Drop` impl.
    pub async fn teardown(self) {
        self.db.drop().await;
    }
}
