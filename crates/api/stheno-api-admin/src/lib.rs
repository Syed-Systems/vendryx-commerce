//! `stheno-api-admin`: Medusa v2 wire-compatible Admin API routes.

pub mod catalog;
pub mod config;
pub mod customers;
pub mod inventory;
pub mod orders;
pub mod pricing;
pub mod promotions;
pub mod reports;
pub mod users;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use stheno_user::UserService;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_fulfillment::FulfillmentService;
use stheno_inventory::InventoryService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::ProductService;
use stheno_promotion::PromotionService;
use stheno_region::RegionService;
use stheno_sales_channel::SalesChannelService;
use stheno_stock_location::StockLocationService;
use stheno_tax::TaxService;

use config::ConfigContext;
use inventory::InventoryContext;
use orders::AdminOrderContext;

#[derive(Clone)]
pub struct AdminApiContext {
    pub product_service: Arc<ProductService>,
    pub inventory_service: Arc<InventoryService>,
    pub stock_location_service: Arc<StockLocationService>,
    pub pricing_service: Arc<PricingService>,
    pub order_service: Arc<OrderService>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub payment_service: Arc<PaymentService>,
    pub customer_service: Arc<CustomerService>,
    pub promotion_service: Arc<PromotionService>,
    pub region_service: Arc<RegionService>,
    pub currency_service: Arc<CurrencyService>,
    pub tax_service: Arc<TaxService>,
    pub sales_channel_service: Arc<SalesChannelService>,
    pub reporting_service: Option<Arc<stheno_reporting::ReportingService>>,
}

/// Mounts admin catalog API routes (products, categories, variants).
pub fn admin_catalog_routes(product_service: Arc<ProductService>) -> Router {
    Router::new()
        .route(
            "/admin/products",
            get(catalog::list_products).post(catalog::create_product),
        )
        .route(
            "/admin/products/{id}",
            get(catalog::get_product)
                .post(catalog::update_product)
                .delete(catalog::delete_product),
        )
        .route(
            "/admin/product-categories",
            get(catalog::list_categories).post(catalog::create_category),
        )
        .route(
            "/admin/product-categories/{id}",
            get(catalog::get_category).delete(catalog::delete_category),
        )
        .route(
            "/admin/products/{id}/variants",
            post(catalog::create_variant),
        )
        .with_state(product_service)
}

/// Mounts admin inventory, stock locations, and reservations routes.
pub fn admin_inventory_routes(
    inventory_service: Arc<InventoryService>,
    stock_location_service: Arc<StockLocationService>,
) -> Router {
    let ctx = InventoryContext {
        inventory_service,
        stock_location_service,
    };
    Router::new()
        .route(
            "/admin/inventory-items",
            get(inventory::list_inventory_items).post(inventory::create_inventory_item),
        )
        .route(
            "/admin/reservations",
            get(inventory::list_reservations).post(inventory::create_reservation),
        )
        .route(
            "/admin/stock-locations",
            get(inventory::list_stock_locations).post(inventory::create_stock_location),
        )
        .route(
            "/admin/stock-locations/{id}",
            axum::routing::delete(inventory::delete_stock_location),
        )
        .with_state(ctx)
}

/// Mounts admin pricing routes.
pub fn admin_pricing_routes(pricing_service: Arc<PricingService>) -> Router {
    Router::new()
        .route(
            "/admin/price-lists",
            get(pricing::list_price_lists).post(pricing::create_price_list),
        )
        .route(
            "/admin/price-lists/{id}",
            get(pricing::get_price_list)
                .post(pricing::update_price_list)
                .delete(pricing::delete_price_list),
        )
        .with_state(pricing_service)
}

/// Mounts admin orders, returns, fulfillments and payments routes.
pub fn admin_order_routes(
    order_service: Arc<OrderService>,
    fulfillment_service: Arc<FulfillmentService>,
    payment_service: Arc<PaymentService>,
) -> Router {
    let ctx = AdminOrderContext {
        order_service,
        fulfillment_service,
        payment_service,
    };
    Router::new()
        .route(
            "/admin/orders",
            get(orders::list_orders).post(orders::create_order),
        )
        .route("/admin/orders/{id}", get(orders::get_order))
        .route("/admin/orders/{id}/cancel", post(orders::cancel_order))
        .route("/admin/orders/{id}/archive", post(orders::archive_order))
        .route(
            "/admin/orders/{id}/fulfillments",
            post(orders::create_fulfillment),
        )
        .route("/admin/orders/{id}/captures", post(orders::capture_payment))
        .route("/admin/orders/{id}/refunds", post(orders::refund_payment))
        .route(
            "/admin/returns",
            get(orders::list_returns).post(orders::create_return),
        )
        .route("/admin/returns/{id}/receive", post(orders::receive_return))
        .route(
            "/admin/return-reasons",
            get(orders::list_return_reasons).post(orders::create_return_reason),
        )
        .with_state(ctx)
}

/// Mounts admin customers and groups routes.
pub fn admin_customer_routes(customer_service: Arc<CustomerService>) -> Router {
    Router::new()
        .route(
            "/admin/customers",
            get(customers::list_customers).post(customers::create_customer),
        )
        .route(
            "/admin/customers/{id}",
            get(customers::get_customer)
                .post(customers::update_customer)
                .delete(customers::delete_customer),
        )
        .route(
            "/admin/customer-groups",
            get(customers::list_customer_groups).post(customers::create_customer_group),
        )
        .route(
            "/admin/customer-groups/{id}",
            get(customers::update_customer_group),
        )
        .with_state(customer_service)
}

/// Mounts admin promotions and campaigns routes.
pub fn admin_promotion_routes(promotion_service: Arc<PromotionService>) -> Router {
    Router::new()
        .route(
            "/admin/promotions",
            get(promotions::list_promotions).post(promotions::create_promotion),
        )
        .route(
            "/admin/promotions/{id}",
            get(promotions::get_promotion)
                .post(promotions::update_promotion)
                .delete(promotions::delete_promotion),
        )
        .route(
            "/admin/campaigns",
            get(promotions::list_campaigns).post(promotions::create_campaign),
        )
        .route(
            "/admin/campaigns/{id}",
            get(promotions::get_campaign)
                .post(promotions::update_campaign)
                .delete(promotions::delete_campaign),
        )
        .with_state(promotion_service)
}

/// Mounts admin configuration routes — regions, currencies, tax, sales channels.
pub fn admin_config_routes(
    region_service: Arc<RegionService>,
    currency_service: Arc<CurrencyService>,
    tax_service: Arc<TaxService>,
    sales_channel_service: Arc<SalesChannelService>,
) -> Router {
    let ctx = ConfigContext {
        region_service,
        currency_service,
        tax_service,
        sales_channel_service,
    };
    Router::new()
        // Regions
        .route(
            "/admin/regions",
            get(config::list_regions).post(config::create_region),
        )
        .route(
            "/admin/regions/{id}",
            get(config::get_region)
                .post(config::update_region)
                .delete(config::delete_region),
        )
        // Currencies
        .route("/admin/currencies", get(config::list_currencies))
        .route("/admin/currencies/{code}", get(config::get_currency))
        // Tax Regions
        .route("/admin/tax-regions", post(config::create_tax_region))
        .route(
            "/admin/tax-regions/{id}",
            get(config::get_tax_region)
                .post(config::update_tax_region)
                .delete(config::delete_tax_region),
        )
        // Tax Rates
        .route("/admin/tax-rates", post(config::create_tax_rate))
        .route(
            "/admin/tax-rates/{id}",
            get(config::get_tax_rate)
                .post(config::update_tax_rate)
                .delete(config::delete_tax_rate),
        )
        // Sales Channels
        .route(
            "/admin/sales-channels",
            get(config::list_sales_channels).post(config::create_sales_channel),
        )
        .route(
            "/admin/sales-channels/{id}",
            get(config::get_sales_channel).delete(config::delete_sales_channel),
        )
        .with_state(ctx)
}

/// Mounts all Admin API routes.
pub fn admin_routes(ctx: AdminApiContext) -> Router {
    let mut router = Router::new()
        .merge(admin_catalog_routes(ctx.product_service))
        .merge(admin_inventory_routes(
            ctx.inventory_service,
            ctx.stock_location_service,
        ))
        .merge(admin_pricing_routes(ctx.pricing_service))
        .merge(admin_order_routes(
            ctx.order_service,
            ctx.fulfillment_service,
            ctx.payment_service,
        ))
        .merge(admin_customer_routes(ctx.customer_service))
        .merge(admin_promotion_routes(ctx.promotion_service))
        .merge(admin_config_routes(
            ctx.region_service,
            ctx.currency_service,
            ctx.tax_service,
            ctx.sales_channel_service,
        ));

    if let Some(reporting) = ctx.reporting_service {
        router = router.merge(admin_reports_routes(reporting));
    }

    router
}

/// Mounts admin reporting routes (P16).
pub fn admin_reports_routes(reporting_service: Arc<stheno_reporting::ReportingService>) -> Router {
    let ctx = reports::AdminReportingContext { reporting_service };
    Router::new()
        .route(
            "/admin/reports/revenue-over-time",
            get(reports::revenue_over_time_handler),
        )
        .route(
            "/admin/reports/orders-over-time",
            get(reports::orders_over_time_handler),
        )
        .route(
            "/admin/reports/top-products",
            get(reports::top_products_handler),
        )
        .route(
            "/admin/reports/average-order-value",
            get(reports::average_order_value_handler),
        )
        .with_state(ctx)
}

/// Mounts admin user and invite routes.
pub fn admin_user_routes(user_service: Arc<UserService>) -> Router {
    let ctx = users::AdminUserContext { user_service };
    Router::new()
        .route("/admin/users/me", get(users::get_me))
        .route("/admin/users", get(users::list_users).post(users::create_user))
        .route(
            "/admin/users/{id}",
            get(users::get_user)
                .post(users::update_user)
                .delete(users::delete_user),
        )
        .route(
            "/admin/invites",
            get(users::list_invites).post(users::create_invite),
        )
        .with_state(ctx)
}
