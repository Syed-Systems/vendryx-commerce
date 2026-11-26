//! `stheno-api-store`: Medusa v2 wire-compatible Store API routes.

pub mod branding;
pub mod cache;
pub mod carts;
pub mod categories;
pub mod collections;
pub mod currencies;
pub mod customers;
pub mod hooks;
pub mod locales;
pub mod orders;
pub mod payment_collections;
pub mod product_expand;
pub mod products;
pub mod regions;
pub mod returns;
pub mod shipping_options;
pub mod social;
pub mod social_webhook;
pub mod variants;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_cart::CartService;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_flows::{OrderWorkflowEngine, ReturnWorkflowEngine};
use stheno_fulfillment::FulfillmentService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::ProductService;
use stheno_region::RegionService;
use stheno_translation::TranslationService;

pub use carts::StoreCartContext;
use customers::StoreCustomerContext;
pub use orders::StoreOrderContext;
pub use payment_collections::StorePaymentContext;
use returns::StoreReturnContext;
use shipping_options::StoreShippingOptionContext;

#[derive(Clone)]
pub struct StoreApiContext {
    /// Queues social drafts (PRODUCT.md §5.2). `SocialDraftWorkflow` existed with tests
    /// and no HTTP surface until this was wired.
    pub social_flows: Arc<stheno_flows_order::SocialDraftWorkflow>,
    pub store_name: String,
    pub store_service: Arc<stheno_store::StoreService>,
    pub product_service: Arc<ProductService>,
    pub region_service: Arc<RegionService>,
    pub currency_service: Arc<CurrencyService>,
    pub pricing_service: Arc<PricingService>,
    pub inventory_service: Arc<stheno_inventory::InventoryService>,
    /// Resolves cross-module associations the store API has to expand — a cart's payment
    /// collection, a variant's price set.
    pub links: Arc<stheno_links::PgLinkStore>,
    pub complete_flows: Arc<stheno_flows::CompleteCartWorkflowEngine>,
    pub cart_service: Arc<CartService>,
    pub promotion_service: Option<Arc<stheno_promotion::PromotionService>>,
    pub payment_service: Arc<PaymentService>,
    pub order_service: Arc<OrderService>,
    pub order_flows: Arc<OrderWorkflowEngine>,
    pub return_flows: Arc<ReturnWorkflowEngine>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<stheno_notification::NotificationService>,
    pub translation_service: Arc<TranslationService>,
    pub cache_service: Option<Arc<stheno_caching::CacheService>>,
    /// Optional verified Meta ingress; absent registrations leave the webhook path unmounted.
    pub meta_webhook: Option<social_webhook::MetaWebhookContext>,
}

/// Mounts all store catalog API routes (products & categories).
pub fn store_catalog_routes(ctx: products::CatalogContext) -> Router {
    // Products carry prices, so this router needs the pricing and region services too.
    // Collections, variants and categories still only need the product service, so they
    // keep their own state rather than being widened along with it.
    let product_service = ctx.product_service.clone();
    Router::new()
        .route("/store/products", get(products::list_products))
        .route("/store/products/{id}", get(products::get_product))
        .with_state(ctx)
        .merge(store_catalog_secondary_routes(product_service))
}

fn store_catalog_secondary_routes(product_service: Arc<ProductService>) -> Router {
    Router::new()
        .route("/store/collections", get(collections::list_collections))
        .route("/store/collections/{id}", get(collections::get_collection))
        .route("/store/product-variants/{id}", get(variants::get_variant))
        .route(
            "/store/product-options",
            get(products::list_product_options),
        )
        .route(
            "/store/product-options/{id}",
            get(products::get_product_option),
        )
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

/// Mounts the store branding route (P26-03).
pub fn store_branding_routes(store_service: Arc<stheno_store::StoreService>) -> Router {
    Router::new()
        .route("/store/branding", get(branding::get_branding))
        .route("/store/appearance", get(branding::get_appearance))
        .with_state(branding::BrandingContext { store_service })
}

/// Mounts store currency API routes.
pub fn store_currency_routes(currency_service: Arc<CurrencyService>) -> Router {
    Router::new()
        .route("/store/currencies", get(currencies::list_currencies))
        .route("/store/currencies/{code}", get(currencies::get_currency))
        .with_state(currency_service)
}

/// Mounts store cart and line-item API routes (TASKS P4-29).
///
/// Takes the whole context rather than a cart service: line items are priced from the
/// catalogue now, so the router needs pricing, region and product alongside the cart.
pub fn store_cart_routes(ctx: StoreCartContext) -> Router {
    Router::new()
        .route("/store/carts", post(carts::create_cart))
        .route(
            "/store/carts/{id}",
            get(carts::get_cart).post(carts::update_cart),
        )
        .route(
            "/store/carts/{id}/customer",
            post(carts::associate_customer),
        )
        .route("/store/carts/{id}/line-items", post(carts::add_line_item))
        .route("/store/carts/{id}/complete", post(carts::complete_cart))
        .route(
            "/store/carts/{id}/line-items/{line_id}",
            post(carts::update_line_item).delete(carts::delete_line_item),
        )
        .route(
            "/store/carts/{id}/shipping-methods",
            post(carts::add_shipping_method),
        )
        .route(
            "/store/carts/{id}/promotions",
            post(carts::apply_promotions).delete(carts::remove_promotions),
        )
        .with_state(ctx)
}

/// Mounts store payment collection and provider routes (TASKS P4-29).
pub fn store_payment_routes(ctx: StorePaymentContext) -> Router {
    Router::new()
        .route(
            "/store/payment-collections",
            post(payment_collections::create_payment_collection),
        )
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
pub fn store_order_routes(ctx: StoreOrderContext) -> Router {
    Router::new()
        .route("/store/orders", get(orders::list_orders))
        .route("/store/orders/{id}", get(orders::get_order))
        .route(
            "/store/orders/{id}/payment-proof",
            get(orders::get_order_payment_proof).post(orders::submit_order_payment_proof),
        )
        .route("/store/orders/transfer", post(orders::request_transfer))
        .route(
            "/store/orders/{id}/transfer/request",
            post(orders::request_transfer_for_order),
        )
        .route(
            "/store/orders/{id}/transfer/accept",
            post(orders::accept_transfer_for_order),
        )
        .route(
            "/store/orders/{id}/transfer/decline",
            post(orders::decline_transfer_for_order),
        )
        .route(
            "/store/orders/{id}/transfer/cancel",
            post(orders::cancel_transfer_for_order),
        )
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
        .route(
            "/store/shipping-options/{id}/calculate",
            post(shipping_options::calculate_shipping_option),
        )
        .with_state(ctx)
}

/// Mounts store return routes (TASKS P6-20).
pub fn store_return_routes(
    order_service: Arc<OrderService>,
    return_flows: Arc<ReturnWorkflowEngine>,
    auth_service: Arc<AuthService>,
    notification_service: Arc<stheno_notification::NotificationService>,
) -> Router {
    let ctx = StoreReturnContext {
        order_service,
        return_flows,
        auth_service,
        notification_service,
    };
    Router::new()
        .route(
            "/store/returns",
            get(returns::list_returns).post(returns::create_return),
        )
        .route("/store/returns/{id}", get(returns::get_return))
        .with_state(ctx)
}

/// Mounts the store customer routes. Both are authenticated by bearer JWT.
///
/// `POST /store/customers` completes a registration the auth route can only half-finish:
/// the identity exists before the customer does, so something has to create the profile
/// and bind them. Without it the storefront's sign-up form failed outright.
pub fn store_customer_routes(
    customer_service: Arc<CustomerService>,
    auth_service: Arc<AuthService>,
    notification_service: Arc<stheno_notification::NotificationService>,
) -> Router {
    Router::new()
        .route("/store/customers", post(customers::create_customer))
        .route(
            "/store/customers/me",
            get(customers::get_current_customer).post(customers::update_current_customer),
        )
        .route(
            "/store/customers/me/addresses",
            get(customers::list_my_addresses).post(customers::create_my_address),
        )
        .route(
            "/store/customers/me/addresses/{address_id}",
            get(customers::get_my_address)
                .post(customers::update_my_address)
                .delete(customers::delete_my_address),
        )
        .route(
            "/store/customers/me/whatsapp-consent",
            axum::routing::delete(customers::revoke_my_whatsapp_consent),
        )
        .with_state(StoreCustomerContext {
            customer_service,
            auth_service,
            notification_service,
        })
}

/// Mounts the store locale routes (additive to Medusa's contract, ADR-002).
pub fn store_locale_routes(translation_service: Arc<TranslationService>) -> Router {
    Router::new()
        .route("/store/locales", get(locales::list_locales))
        .with_state(translation_service)
}

/// Mounts all Store API routes (catalog, regions, currencies, carts, payments, orders, shipping options, returns).
pub fn store_routes(ctx: StoreApiContext) -> Router {
    Router::new()
        .merge(
            ctx.meta_webhook
                .clone()
                .map(crate::social_webhook::meta_webhook_routes)
                .unwrap_or_default(),
        )
        .merge(crate::social::store_social_routes(
            crate::social::SocialContext {
                social_flows: ctx.social_flows,
                store_name: ctx.store_name.clone(),
                store_service: ctx.store_service.clone(),
            },
        ))
        .merge(store_catalog_routes(products::CatalogContext {
            product_service: ctx.product_service.clone(),
            pricing_service: ctx.pricing_service.clone(),
            region_service: ctx.region_service.clone(),
            inventory_service: ctx.inventory_service.clone(),
            cache_service: ctx.cache_service.clone(),
        }))
        .merge(store_region_routes(ctx.region_service.clone()))
        .merge(store_currency_routes(ctx.currency_service))
        .merge(store_cart_routes(StoreCartContext {
            store_name: ctx.store_name.clone(),
            cart_service: ctx.cart_service.clone(),
            promotion_service: ctx.promotion_service,
            pricing_service: ctx.pricing_service,
            region_service: ctx.region_service,
            product_service: ctx.product_service,
            fulfillment_service: ctx.fulfillment_service.clone(),
            links: ctx.links.clone(),
            payment_service: ctx.payment_service.clone(),
            notification_service: ctx.notification_service.clone(),
            complete_flows: ctx.complete_flows.clone(),
        }))
        .merge(store_payment_routes(StorePaymentContext {
            payment_service: ctx.payment_service.clone(),
            cart_service: ctx.cart_service.clone(),
            links: ctx.links.clone(),
        }))
        .merge(store_order_routes(StoreOrderContext {
            order_service: ctx.order_service.clone(),
            order_flows: ctx.order_flows,
            links: ctx.links.clone(),
            payment_service: ctx.payment_service.clone(),
            notification_service: ctx.notification_service.clone(),
            auth_service: ctx.auth_service.clone(),
        }))
        .merge(store_shipping_option_routes(ctx.fulfillment_service))
        .merge(store_return_routes(
            ctx.order_service,
            ctx.return_flows,
            ctx.auth_service,
            ctx.notification_service,
        ))
        // Customer routes are deliberately NOT merged here. Buyer accounts are an
        // archetype the merchant picks at store-request time (PRODUCT.md 2), so
        // `store_customer_routes` mounts through a GatedGroup in stheno-server, the
        // same way admin reporting does. Merging it here would make it ungateable.
        .merge(store_locale_routes(ctx.translation_service))
        .merge(store_branding_routes(ctx.store_service.clone()))
}

/// Mounts Store API routes for Phase P3 (catalog, regions, currencies).
pub fn store_pricing_routes(ctx: StoreApiContext) -> Router {
    Router::new()
        .merge(store_catalog_routes(products::CatalogContext {
            product_service: ctx.product_service,
            pricing_service: ctx.pricing_service.clone(),
            region_service: ctx.region_service.clone(),
            inventory_service: ctx.inventory_service.clone(),
            cache_service: ctx.cache_service.clone(),
        }))
        .merge(store_region_routes(ctx.region_service))
        .merge(store_currency_routes(ctx.currency_service))
}
