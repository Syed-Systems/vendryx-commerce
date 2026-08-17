//! `stheno-server`: Server application, router mounting, and universal test harness.

pub mod db;

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use stheno_api_admin::{
    admin_catalog_routes, admin_config_routes, admin_customer_routes, admin_inventory_routes,
    admin_order_routes, admin_pricing_routes, admin_promotion_routes, admin_reports_routes,
    admin_user_routes,
};
use stheno_api_admin_platform::{admin_platform_routes, AdminPlatformContext, UsageSnapshot};
use stheno_api_auth::{auth_routes, AuthApiContext};
use stheno_api_core::{Entitlements, EntitlementsError, Tier, TierConfig};
use stheno_api_store::{store_routes, StoreApiContext};
use stheno_auth::AuthService;
use stheno_cart::CartService;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_flows::{AuthWorkflows, OrderWorkflowEngine, ReturnWorkflowEngine};
use stheno_fulfillment::FulfillmentService;
use stheno_fulfillment_manual::ManualFulfillmentProvider;
use stheno_inventory::InventoryService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::{CreateCollectionInput, CreateProductInput, ProductService};
use stheno_promotion::PromotionService;
use stheno_region::{CreateRegionInput, RegionService};
use stheno_registry::AppContext;
use stheno_sales_channel::SalesChannelService;
use stheno_stock_location::StockLocationService;
use stheno_tax::TaxService;
use stheno_translation::{CreateLocaleInput, TranslationService};
use stheno_user::{CreateUserInput, UserService};
use stheno_workflow::WorkflowStore;

/// Tier definitions ship with the binary, overridable per deployment (ENTITLEMENTS.md 5b).
const DEFAULT_TIERS_TOML: &str = include_str!("../../../config/tiers.toml");

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

pub struct AppServer {
    pub context: AppContext,
    pub router: Router,
    pub entitlements: Arc<Entitlements>,
    pub pool: stheno_db::DbPool,
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
        let order_service = Arc::new(OrderService::new(pool.clone()));
        let fulfillment_service = Arc::new(FulfillmentService::new(pool.clone()));
        fulfillment_service.register_provider(Arc::new(ManualFulfillmentProvider::new()));
        let tax_service = Arc::new(TaxService::new(pool.clone()));
        let customer_service = Arc::new(CustomerService::new(pool.clone()));
        let user_service = Arc::new(UserService::new(pool.clone()));
        let inventory_service = Arc::new(InventoryService::new(pool.clone()));
        let stock_location_service = Arc::new(StockLocationService::new(pool.clone()));
        let sales_channel_service = Arc::new(SalesChannelService::new(pool.clone()));
        let promotion_service = Arc::new(PromotionService::new(pool.clone()));
        let auth_service = Arc::new(AuthService::new(pool.clone(), None));
        let translation_service = Arc::new(TranslationService::new(pool.clone()));
        let reporting_service = Arc::new(stheno_reporting::ReportingService::new(pool.clone()));
        let workflow_store = Arc::new(WorkflowStore::from_pool(pool.clone()));
        let auth_flows = Arc::new(AuthWorkflows::new(auth_service.clone()));
        let order_flow = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
        let return_flow = Arc::new(ReturnWorkflowEngine::new(
            order_service.clone(),
            Some(inventory_service.clone()),
        ));

        // Seed default store catalog & demo items
        let ps = product_service.clone();
        let rs = region_service.clone();
        let us = user_service.clone();
        let ts = translation_service.clone();
        tokio::spawn(async move {
            let _ = rs
                .create_region(CreateRegionInput {
                    name: "North America".to_string(),
                    currency_code: "USD".to_string(),
                    automatic_taxes: Some(true),
                    countries: vec!["us".to_string(), "ca".to_string()],
                })
                .await;
            let _ = rs
                .create_region(CreateRegionInput {
                    name: "Europe".to_string(),
                    currency_code: "EUR".to_string(),
                    automatic_taxes: Some(true),
                    countries: vec!["de".to_string(), "fr".to_string(), "gb".to_string()],
                })
                .await;

            let _ = ps
                .create_product(CreateProductInput {
                    title: "Stheno Performance Hoodie".to_string(),
                    handle: Some("stheno-performance-hoodie".to_string()),
                    subtitle: Some("Heavyweight Organic Cotton Fleece".to_string()),
                    description: Some(
                        "Engineered for comfort and durability with signature flatlock stitching."
                            .to_string(),
                    ),
                    discountable: Some(true),
                })
                .await;

            let _ = ps
                .create_product(CreateProductInput {
                    title: "Stheno Signature Tee".to_string(),
                    handle: Some("stheno-signature-tee".to_string()),
                    subtitle: Some("240 GSM Combed Jersey".to_string()),
                    description: Some("Relaxed fit premium daily essential.".to_string()),
                    discountable: Some(true),
                })
                .await;

            for (title, handle) in [("Featured", "featured"), ("Latest Drops", "latest-drops")] {
                let _ = ps
                    .create_collection(CreateCollectionInput {
                        title: title.to_string(),
                        handle: Some(handle.to_string()),
                        metadata: None,
                    })
                    .await;
            }

            for (code, name, is_default) in [("en", "English", true), ("de", "Deutsch", false)] {
                let _ = ts
                    .create_locale(CreateLocaleInput {
                        code: code.to_string(),
                        name: name.to_string(),
                        is_default: Some(is_default),
                        is_enabled: Some(true),
                    })
                    .await;
            }

            let _ = us
                .create_user(CreateUserInput {
                    email: "admin@stheno.store".to_string(),
                    first_name: Some("System".to_string()),
                    last_name: Some("Administrator".to_string()),
                    avatar_url: None,
                    role: Some("admin".to_string()),
                    metadata: None,
                })
                .await;
        });

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let router = Router::new()
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
                            "compatibility": "Medusa v2 Parity",
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
            .merge(store_routes(StoreApiContext {
                product_service: product_service.clone(),
                region_service: region_service.clone(),
                currency_service: currency_service.clone(),
                pricing_service: pricing_service.clone(),
                cart_service: cart_service.clone(),
                payment_service: payment_service.clone(),
                order_service: order_service.clone(),
                order_flows: order_flow.clone(),
                return_flows: return_flow.clone(),
                fulfillment_service: fulfillment_service.clone(),
                customer_service: customer_service.clone(),
                auth_service: auth_service.clone(),
                translation_service: translation_service.clone(),
                promotion_service: Some(promotion_service.clone()),
            }))
            .merge(admin_catalog_routes(product_service.clone()))
            .merge(admin_order_routes(order_service.clone(), fulfillment_service.clone(), payment_service.clone()))
            .merge(admin_customer_routes(customer_service.clone()))
            .merge(admin_inventory_routes(inventory_service.clone(), stock_location_service.clone()))
            .merge(admin_pricing_routes(pricing_service.clone()))
            .merge(admin_promotion_routes(promotion_service.clone()))
            .merge(admin_user_routes(user_service.clone()))
            .merge(admin_config_routes(region_service.clone(), currency_service.clone(), tax_service.clone(), sales_channel_service.clone()))
            .merge(admin_reports_routes(reporting_service.clone()))
            .merge(auth_routes(AuthApiContext { auth_flows }))
            .merge(admin_platform_routes(AdminPlatformContext {
                entitlements: entitlements.clone(),
                // ponytail: usage is reported as zero until the quota counter table
                // lands with persistence (ENTITLEMENTS.md 6.2).
                usage: Arc::new(UsageSnapshot::new()),
                workflows: workflow_store.clone(),
            }))
            .layer(cors);

        Self {
            context,
            router,
            entitlements,
            pool,
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
