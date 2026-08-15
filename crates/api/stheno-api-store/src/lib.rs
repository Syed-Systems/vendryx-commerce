//! `stheno-api-store`: Medusa v2 wire-compatible Store API routes.

pub mod carts;
pub mod categories;
pub mod currencies;
pub mod orders;
pub mod payment_collections;
pub mod products;
pub mod regions;
pub mod returns;
pub mod shipping_options;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use stheno_cart::CartService;
use stheno_currency::CurrencyService;
use stheno_flows::{OrderWorkflowEngine, ReturnWorkflowEngine};
use stheno_fulfillment::FulfillmentService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::ProductService;
use stheno_region::RegionService;

use carts::StoreCartContext;
use orders::StoreOrderContext;
use payment_collections::StorePaymentContext;
use returns::StoreReturnContext;
use shipping_options::StoreShippingOptionContext;

#[derive(Clone)]
pub struct StoreApiContext {
    pub product_service: Arc<ProductService>,
    pub region_service: Arc<RegionService>,
    pub currency_service: Arc<CurrencyService>,
    pub pricing_service: Arc<PricingService>,
    pub cart_service: Arc<CartService>,
    pub payment_service: Arc<PaymentService>,
    pub order_service: Arc<OrderService>,
    pub order_flows: Arc<OrderWorkflowEngine>,
    pub return_flows: Arc<ReturnWorkflowEngine>,
    pub fulfillment_service: Arc<FulfillmentService>,
}

/// Mounts all store catalog API routes (products & categories).
pub fn store_catalog_routes(product_service: Arc<ProductService>) -> Router {
    Router::new()
        .route("/store/products", get(products::list_products))
        .route("/store/products/{id}", get(products::get_product))
        .route(
            "/store/product-categories",
            get(categories::list_categories),
        )
        .route(
            "/store/product-categories/{id}",
            get(categories::get_category),
        )
        .with_state(product_service)
}

/// Mounts store region API routes.
pub fn store_region_routes(region_service: Arc<RegionService>) -> Router {
    Router::new()
        .route("/store/regions", get(regions::list_regions))
        .route("/store/regions/{id}", get(regions::get_region))
        .with_state(region_service)
}

/// Mounts store currency API routes.
pub fn store_currency_routes(currency_service: Arc<CurrencyService>) -> Router {
    Router::new()
        .route("/store/currencies", get(currencies::list_currencies))
        .route("/store/currencies/{code}", get(currencies::get_currency))
        .with_state(currency_service)
}

/// Mounts store cart and line-item API routes (TASKS P4-29).
pub fn store_cart_routes(cart_service: Arc<CartService>) -> Router {
    let ctx = StoreCartContext { cart_service };
    Router::new()
        .route("/store/carts", post(carts::create_cart))
        .route(
            "/store/carts/{id}",
            get(carts::get_cart).post(carts::update_cart),
        )
        .route("/store/carts/{id}/line-items", post(carts::add_line_item))
        .route(
            "/store/carts/{id}/line-items/{line_id}",
            post(carts::update_line_item).delete(carts::delete_line_item),
        )
        .route(
            "/store/carts/{id}/shipping-methods",
            post(carts::add_shipping_method),
        )
        .with_state(ctx)
}

/// Mounts store payment collection and provider routes (TASKS P4-29).
pub fn store_payment_routes(payment_service: Arc<PaymentService>) -> Router {
    let ctx = StorePaymentContext { payment_service };
    Router::new()
        .route(
            "/store/payment-collections/{id}/payment-sessions",
            post(payment_collections::create_payment_session),
        )
        .route(
            "/store/payment-collections/{id}/payment-sessions/{session_id}",
            post(payment_collections::authorize_payment_session),
        )
        .route(
            "/store/payment-providers",
            get(payment_collections::list_payment_providers),
        )
        .with_state(ctx)
}

/// Mounts store order and transfer routes (TASKS P5-22).
pub fn store_order_routes(
    order_service: Arc<OrderService>,
    order_flows: Arc<OrderWorkflowEngine>,
) -> Router {
    let ctx = StoreOrderContext {
        order_service,
        order_flows,
    };
    Router::new()
        .route("/store/orders", get(orders::list_orders))
        .route("/store/orders/{id}", get(orders::get_order))
        .route("/store/orders/transfer", post(orders::request_transfer))
        .route(
            "/store/orders/transfer/accept",
            post(orders::accept_transfer),
        )
        .route(
            "/store/orders/transfer/decline",
            post(orders::decline_transfer),
        )
        .route(
            "/store/orders/transfer/cancel",
            post(orders::cancel_transfer),
        )
        .with_state(ctx)
}

/// Mounts store shipping options routes (TASKS P5-22).
pub fn store_shipping_option_routes(fulfillment_service: Arc<FulfillmentService>) -> Router {
    let ctx = StoreShippingOptionContext {
        fulfillment_service,
    };
    Router::new()
        .route(
            "/store/shipping-options",
            get(shipping_options::list_shipping_options),
        )
        .with_state(ctx)
}

/// Mounts store return routes (TASKS P6-20).
pub fn store_return_routes(
    order_service: Arc<OrderService>,
    return_flows: Arc<ReturnWorkflowEngine>,
) -> Router {
    let ctx = StoreReturnContext {
        order_service,
        return_flows,
    };
    Router::new()
        .route(
            "/store/returns",
            get(returns::list_returns).post(returns::create_return),
        )
        .route("/store/returns/{id}", get(returns::get_return))
        .with_state(ctx)
}

/// Mounts all Store API routes (catalog, regions, currencies, carts, payments, orders, shipping options, returns).
pub fn store_routes(ctx: StoreApiContext) -> Router {
    Router::new()
        .merge(store_catalog_routes(ctx.product_service))
        .merge(store_region_routes(ctx.region_service))
        .merge(store_currency_routes(ctx.currency_service))
        .merge(store_cart_routes(ctx.cart_service))
        .merge(store_payment_routes(ctx.payment_service))
        .merge(store_order_routes(
            ctx.order_service.clone(),
            ctx.order_flows,
        ))
        .merge(store_shipping_option_routes(ctx.fulfillment_service))
        .merge(store_return_routes(ctx.order_service, ctx.return_flows))
}

/// Mounts Store API routes for Phase P3 (catalog, regions, currencies).
pub fn store_pricing_routes(ctx: StoreApiContext) -> Router {
    Router::new()
        .merge(store_catalog_routes(ctx.product_service))
        .merge(store_region_routes(ctx.region_service))
        .merge(store_currency_routes(ctx.currency_service))
}
