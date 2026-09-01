//! `stheno-api-admin`: Medusa v2 wire-compatible Admin API routes.

pub mod api_keys;
pub mod auth_guard;
pub mod catalog;
pub mod config;
pub mod customers;
pub mod fulfillment_settings;
pub mod inventory;
pub mod localisation;
pub mod orders;
pub mod pricing;
pub mod promotions;
pub mod rbac;
pub mod reports;
pub mod store;
pub mod uploads;
pub mod users;
pub mod views;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_fulfillment::FulfillmentService;
use stheno_inventory::InventoryService;
use stheno_links::PgLinkStore;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::ProductService;
use stheno_promotion::PromotionService;
use stheno_region::RegionService;
use stheno_sales_channel::SalesChannelService;
use stheno_stock_location::StockLocationService;
use stheno_store::StoreService;
use stheno_tax::TaxService;
use stheno_user::UserService;

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
    pub links: Arc<PgLinkStore>,
    pub reporting_service: Option<Arc<stheno_reporting::ReportingService>>,
}

/// Mounts admin catalog API routes (products, categories, variants).
pub fn admin_catalog_routes(
    product_service: Arc<ProductService>,
    pricing_service: Arc<stheno_pricing::PricingService>,
    inventory_service: Arc<InventoryService>,
) -> Router {
    // The product-detail routes carry their own state for the same reason the variant
    // routes below do: the dashboard's variant table reads inventory off each variant, and
    // a router has exactly one state type.
    let detail_routes: Router = Router::new()
        .route(
            "/admin/products/{id}",
            get(catalog::get_product)
                .post(catalog::update_product)
                .delete(catalog::delete_product),
        )
        .with_state(catalog::ProductDetailContext {
            product_service: product_service.clone(),
            inventory_service: inventory_service.clone(),
        });

    // Variant routes need pricing — a variant's price is set on the variant form — so
    // they carry their own state. Everything else here is products only.
    let variant_routes: Router = Router::new()
        .route(
            "/admin/products/{id}/variants",
            get(catalog::list_product_variants).post(catalog::create_variant),
        )
        .route(
            "/admin/products/{id}/variants/{variant_id}",
            get(catalog::get_variant)
                .post(catalog::update_variant)
                .delete(catalog::delete_variant),
        )
        .with_state(catalog::VariantContext {
            product_service: product_service.clone(),
            pricing_service,
            inventory_service,
        });

    Router::new()
        .route(
            "/admin/products",
            get(catalog::list_products).post(catalog::create_product),
        )
        .route(
            "/admin/products/{id}/options",
            get(catalog::list_product_options).post(catalog::create_product_option),
        )
        .route(
            "/admin/products/{id}/options/{option_id}",
            get(catalog::get_product_option)
                .post(catalog::update_product_option)
                .delete(catalog::delete_product_option),
        )
        .route(
            "/admin/product-options",
            get(catalog::list_global_product_options).post(catalog::create_global_product_option),
        )
        .route(
            "/admin/product-variants",
            get(catalog::list_global_product_variants),
        )
        .route(
            "/admin/product-categories",
            get(catalog::list_categories).post(catalog::create_category),
        )
        .route(
            "/admin/product-categories/{id}",
            get(catalog::get_category)
                .post(catalog::update_category)
                .delete(catalog::delete_category),
        )
        .route(
            "/admin/product-categories/{id}/products",
            post(catalog::update_category_products),
        )
        .route(
            "/admin/collections",
            get(catalog::list_collections).post(catalog::create_collection),
        )
        .route(
            "/admin/collections/{id}",
            get(catalog::get_collection)
                .post(catalog::update_collection)
                .delete(catalog::delete_collection),
        )
        .route(
            "/admin/collections/{id}/products",
            post(catalog::update_collection_products),
        )
        .route(
            "/admin/product-types",
            get(catalog::list_product_types).post(catalog::create_product_type),
        )
        .route(
            "/admin/product-types/{id}",
            get(catalog::get_product_type)
                .post(catalog::update_product_type)
                .delete(catalog::delete_product_type),
        )
        .route(
            "/admin/product-tags",
            get(catalog::list_product_tags).post(catalog::create_product_tag),
        )
        .route(
            "/admin/product-tags/{id}",
            get(catalog::get_product_tag)
                .post(catalog::update_product_tag)
                .delete(catalog::delete_product_tag),
        )
        .with_state(product_service)
        .merge(variant_routes)
        .merge(detail_routes)
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
            "/admin/inventory-items/export",
            post(inventory::export_inventory_items),
        )
        .route(
            "/admin/inventory-items/location-levels/batch",
            post(inventory::batch_inventory_location_levels),
        )
        .route(
            "/admin/inventory-items/{id}",
            get(inventory::get_inventory_item)
                .post(inventory::update_inventory_item)
                .delete(inventory::delete_inventory_item),
        )
        .route(
            "/admin/inventory-items/{id}/location-levels",
            get(inventory::list_location_levels).post(inventory::create_location_level),
        )
        .route(
            "/admin/inventory-items/{id}/location-levels/batch",
            post(inventory::batch_item_location_levels),
        )
        // Stock adjustment: the operation the whole inventory surface exists for, and
        // the one that had no endpoint at all.
        .route(
            "/admin/inventory-items/{id}/location-levels/{location_id}",
            post(inventory::set_stock_level).delete(inventory::delete_location_level),
        )
        .route(
            "/admin/reservations",
            get(inventory::list_reservations).post(inventory::create_reservation),
        )
        .route(
            "/admin/reservations/{id}",
            get(inventory::get_reservation)
                .post(inventory::update_reservation)
                .delete(inventory::delete_reservation),
        )
        .route(
            "/admin/stock-locations",
            get(inventory::list_stock_locations).post(inventory::create_stock_location),
        )
        .route(
            "/admin/stock-locations/{id}",
            get(inventory::get_stock_location)
                .post(inventory::update_stock_location)
                .delete(inventory::delete_stock_location),
        )
        .with_state(ctx)
}

/// Mounts admin pricing and price preferences routes.
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
        .route(
            "/admin/price-lists/{id}/prices",
            get(pricing::list_price_list_prices),
        )
        .route(
            "/admin/price-lists/{id}/prices/batch",
            post(pricing::batch_price_list_prices),
        )
        .route(
            "/admin/price-lists/{id}/products",
            post(pricing::update_price_list_products),
        )
        .route(
            "/admin/price-preferences",
            get(pricing::list_price_preferences).post(pricing::set_price_preference),
        )
        .route(
            "/admin/price-preferences/{id}",
            get(pricing::get_price_preference)
                .post(pricing::update_price_preference)
                .delete(pricing::delete_price_preference),
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
        .route(
            "/admin/orders/{id}",
            get(orders::get_order).post(orders::update_order),
        )
        .route("/admin/orders/{id}/cancel", post(orders::cancel_order))
        .route("/admin/orders/{id}/archive", post(orders::archive_order))
        .route("/admin/orders/{id}/preview", get(orders::preview_order))
        .route("/admin/orders/{id}/transfer", post(orders::transfer_order))
        .route(
            "/admin/orders/{id}/fulfillments",
            post(orders::create_fulfillment),
        )
        .route("/admin/orders/{id}/captures", post(orders::capture_payment))
        .route("/admin/orders/{id}/refunds", post(orders::refund_payment))
        .route(
            "/admin/payments/{id}",
            get(orders::get_payment),
        )
        .route(
            "/admin/payments/{id}/capture",
            post(orders::capture_payment_direct),
        )
        .route(
            "/admin/payments/{id}/refund",
            post(orders::refund_payment_direct),
        )
        .route(
            "/admin/payment-collections/{id}",
            get(orders::get_payment_collection),
        )
        .route(
            "/admin/payment-collections/{id}/mark-as-paid",
            post(orders::mark_payment_collection_paid),
        )
        .route(
            "/admin/returns",
            get(orders::list_returns).post(orders::create_return),
        )
        .route(
            "/admin/returns/{id}/receive",
            post(orders::receive_return).delete(orders::cancel_return_receipt),
        )
        .route(
            "/admin/return-reasons",
            get(orders::list_return_reasons).post(orders::create_return_reason),
        )
        .route(
            "/admin/return-reasons/{id}",
            get(orders::get_return_reason)
                .post(orders::update_return_reason)
                .delete(orders::delete_return_reason),
        )
        .route(
            "/admin/refund-reasons",
            get(orders::list_refund_reasons).post(orders::create_refund_reason),
        )
        .route(
            "/admin/refund-reasons/{id}",
            get(orders::get_refund_reason)
                .post(orders::update_refund_reason)
                .delete(orders::delete_refund_reason),
        )
        .route(
            "/admin/order-edits",
            get(orders::list_order_edits).post(orders::create_order_edit),
        )
        .route(
            "/admin/claims",
            get(orders::list_claims).post(orders::create_claim),
        )
        .route(
            "/admin/claims/{id}",
            get(orders::get_claim),
        )
        .route(
            "/admin/exchanges",
            get(orders::list_exchanges).post(orders::create_exchange),
        )
        .route(
            "/admin/exchanges/{id}",
            get(orders::get_exchange),
        )
        .route(
            "/admin/draft-orders",
            get(orders::list_draft_orders).post(orders::create_draft_order),
        )
        .route(
            "/admin/draft-orders/{id}",
            get(orders::get_draft_order),
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
        // `update_customer_group` extracts `Json<UpdateCustomerGroupInput>`, so binding it
        // to GET made updating a customer group impossible: a GET carrying a body is not
        // something axum will extract, and there was no other verb on the route.
        .route(
            "/admin/customer-groups/{id}",
            get(customers::get_customer_group)
                .post(customers::update_customer_group)
                .delete(customers::delete_customer_group),
        )
        .route(
            "/admin/customer-groups/{id}/customers",
            post(customers::batch_customer_group_customers),
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

/// Mounts admin configuration routes — regions, currencies, tax, sales channels, shipping, payments, rbac.
pub fn admin_config_routes(
    region_service: Arc<RegionService>,
    currency_service: Arc<CurrencyService>,
    tax_service: Arc<TaxService>,
    sales_channel_service: Arc<SalesChannelService>,
    payment_service: Arc<PaymentService>,
    links: Arc<PgLinkStore>,
) -> Router {
    let ctx = ConfigContext {
        region_service,
        currency_service,
        tax_service,
        sales_channel_service,
        payment_service,
        links,
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
        .route(
            "/admin/tax-regions",
            get(config::list_tax_regions).post(config::create_tax_region),
        )
        .route(
            "/admin/tax-regions/{id}",
            get(config::get_tax_region)
                .post(config::update_tax_region)
                .delete(config::delete_tax_region),
        )
        // Tax Rates & Rules
        .route(
            "/admin/tax-rates",
            get(config::list_tax_rates).post(config::create_tax_rate),
        )
        .route(
            "/admin/tax-rates/{id}",
            get(config::get_tax_rate)
                .post(config::update_tax_rate)
                .delete(config::delete_tax_rate),
        )
        .route(
            "/admin/tax-rates/{id}/rules",
            post(config::create_tax_rate_rule),
        )
        .route(
            "/admin/tax-rates/{id}/rules/{rule_id}",
            axum::routing::delete(config::delete_tax_rate_rule),
        )
        .route("/admin/tax-providers", get(config::list_tax_providers))
        // Sales Channels
        .route(
            "/admin/sales-channels",
            get(config::list_sales_channels).post(config::create_sales_channel),
        )
        .route(
            "/admin/sales-channels/{id}",
            get(config::get_sales_channel)
                .post(config::update_sales_channel)
                .delete(config::delete_sales_channel),
        )
        .route(
            "/admin/sales-channels/{id}/products",
            post(config::update_sales_channel_products),
        )
        // Payment Providers & Collections
        .route(
            "/admin/payment-providers",
            get(config::list_payment_providers),
        )
        .route(
            "/admin/payment-providers/{provider_id}/credentials",
            post(config::upsert_payment_provider_credentials),
        )
        .route(
            "/admin/regions/{region_id}/payment-providers/{provider_id}",
            post(config::enable_region_payment_provider)
                .delete(config::disable_region_payment_provider),
        )
        // RBAC Roles & Policies
        // Locales & Translations
        .with_state(ctx)
}

/// Mounts all Admin API routes.
pub fn admin_routes(ctx: AdminApiContext) -> Router {
    let mut router = Router::new()
        .merge(admin_catalog_routes(
            ctx.product_service,
            ctx.pricing_service.clone(),
            ctx.inventory_service.clone(),
        ))
        .merge(admin_inventory_routes(
            ctx.inventory_service,
            ctx.stock_location_service,
        ))
        .merge(admin_pricing_routes(ctx.pricing_service))
        .merge(admin_order_routes(
            ctx.order_service,
            ctx.fulfillment_service.clone(),
            ctx.payment_service.clone(),
        ))
        .merge(admin_fulfillment_settings_routes(ctx.fulfillment_service))
        .merge(admin_customer_routes(ctx.customer_service))
        .merge(admin_promotion_routes(ctx.promotion_service))
        .merge(admin_config_routes(
            ctx.region_service,
            ctx.currency_service,
            ctx.tax_service,
            ctx.sales_channel_service,
            ctx.payment_service,
            ctx.links,
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
        .route(
            "/admin/users",
            get(users::list_users).post(users::create_user),
        )
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
        .route(
            "/admin/invites/accept",
            post(users::accept_invite),
        )
        .route(
            "/admin/invites/{id}",
            axum::routing::delete(users::revoke_invite),
        )
        .route(
            "/admin/invites/{id}/resend",
            post(users::resend_invite),
        )
        .with_state(ctx)
}

/// Mounts admin store and price preferences routes.
pub fn admin_store_routes(
    store_service: Arc<StoreService>,
    translation_service: Arc<stheno_translation::TranslationService>,
) -> Router {
    let ctx = store::AdminStoreContext {
        store_service,
        translation_service,
    };
    Router::new()
        .route("/admin/stores", get(store::list_stores))
        .route("/admin/store", get(store::list_stores))
        .route(
            "/admin/stores/{id}",
            get(store::get_store).post(store::update_store),
        )
        .route("/admin/notifications", get(store::list_notifications))
        .with_state(ctx)
}

/// Mounts admin API key routes.
/// Mounts the view-configuration routes the dashboard's data tables read.
///
/// Stateless: the column definitions are static per entity and saved views are not stored
/// yet. Mounted unconditionally because `/admin/feature-flags` advertises
/// `view_configurations`, and a flag that promises an API the router does not serve is what
/// left every list table rendering zero columns.
pub fn admin_views_routes() -> Router {
    Router::new()
        .route("/admin/views/entities", get(views::list_entities))
        .route("/admin/views/{entity}/columns", get(views::list_columns))
        .route(
            "/admin/views/{entity}/configurations",
            get(views::list_configurations).post(views::create_configuration),
        )
        .route(
            "/admin/views/{entity}/configurations/active",
            get(views::active_configuration).post(views::set_active_configuration),
        )
}

/// Mounts admin API key routes.
///
/// Carries `ApiKeyService`: every handler in `api_keys.rs` used to answer from a literal
/// while the service and its table sat unused, so a created key was never stored and a
/// revoked key kept working.
pub fn admin_api_key_routes(api_key_service: Arc<stheno_api_key::ApiKeyService>) -> Router {
    let ctx = api_keys::AdminApiKeyContext { api_key_service };
    Router::new()
        .route(
            "/admin/api-keys",
            get(api_keys::list_api_keys).post(api_keys::create_api_key),
        )
        .route(
            "/admin/api-keys/{id}",
            get(api_keys::get_api_key)
                .post(api_keys::update_api_key)
                .delete(api_keys::delete_api_key),
        )
        .route(
            "/admin/api-keys/{id}/revoke",
            post(api_keys::revoke_api_key),
        )
        .route(
            "/admin/publishable-api-keys",
            get(api_keys::list_publishable_api_keys).post(api_keys::create_publishable_api_key),
        )
        .route(
            "/admin/publishable-api-keys/{id}",
            get(api_keys::get_publishable_api_key)
                .post(api_keys::update_publishable_api_key)
                .delete(api_keys::delete_api_key),
        )
        .route(
            "/admin/publishable-api-keys/{id}/revoke",
            post(api_keys::revoke_api_key),
        )
        .with_state(ctx)
}

/// Mounts the clean-room RBAC routes (ADR-008).
///
/// `stheno-rbac` had a complete service and no HTTP surface; these routes are what
/// connect the admin's roles and policy screens to real data instead of the stubs that
/// previously answered them.
pub fn admin_rbac_routes(rbac_service: Arc<stheno_rbac::RbacService>) -> Router {
    let ctx = rbac::RbacContext { rbac_service };
    Router::new()
        .route(
            "/admin/rbac/roles",
            get(rbac::list_roles).post(rbac::create_role),
        )
        .route(
            "/admin/rbac/roles/{name}",
            get(rbac::get_role)
                .post(rbac::update_role)
                .delete(rbac::delete_role),
        )
        .route(
            "/admin/rbac/policies",
            get(rbac::list_policies).post(rbac::create_policy),
        )
        .route(
            "/admin/rbac/policies/{id}",
            get(rbac::get_policy)
                .post(rbac::update_policy)
                .delete(rbac::delete_policy),
        )
        .route("/admin/rbac/evaluate", post(rbac::evaluate))
        .route("/admin/rbac/audit", get(rbac::list_audit_records))
        .with_state(ctx)
}

/// Mounts the fulfillment settings surface — shipping profiles, fulfillment sets,
/// service zones, option types and the provider registry.
///
/// Every route here previously answered from a hardcoded literal in `catalog.rs` or
/// `config.rs`; none of it was stored, so nothing the admin created could persist.
pub fn admin_fulfillment_settings_routes(fulfillment_service: Arc<FulfillmentService>) -> Router {
    let ctx = fulfillment_settings::FulfillmentSettingsContext {
        fulfillment_service,
    };
    Router::new()
        .route(
            "/admin/shipping-profiles",
            get(fulfillment_settings::list_shipping_profiles)
                .post(fulfillment_settings::create_shipping_profile),
        )
        .route(
            "/admin/shipping-profiles/{id}",
            get(fulfillment_settings::get_shipping_profile)
                .post(fulfillment_settings::update_shipping_profile)
                .delete(fulfillment_settings::delete_shipping_profile),
        )
        .route(
            "/admin/fulfillment-sets",
            get(fulfillment_settings::list_fulfillment_sets)
                .post(fulfillment_settings::create_fulfillment_set),
        )
        .route(
            "/admin/fulfillment-sets/{id}",
            get(fulfillment_settings::get_fulfillment_set)
                .delete(fulfillment_settings::delete_fulfillment_set),
        )
        .route(
            "/admin/fulfillment-sets/{id}/service-zones",
            get(fulfillment_settings::list_service_zones)
                .post(fulfillment_settings::create_service_zone_for_set),
        )
        .route(
            "/admin/fulfillment-sets/{id}/service-zones/{zone_id}",
            axum::routing::delete(fulfillment_settings::delete_service_zone_for_set),
        )
        .route(
            "/admin/service-zones",
            post(fulfillment_settings::create_service_zone),
        )
        .route(
            "/admin/shipping-options",
            get(fulfillment_settings::list_shipping_options)
                .post(fulfillment_settings::create_shipping_option),
        )
        .route(
            "/admin/shipping-options/{id}/rules/batch",
            post(fulfillment_settings::batch_shipping_option_rules),
        )
        .route(
            "/admin/shipping-option-types",
            get(fulfillment_settings::list_shipping_option_types)
                .post(fulfillment_settings::create_shipping_option_type),
        )
        .route(
            "/admin/fulfillment-providers",
            get(fulfillment_settings::list_fulfillment_providers),
        )
        .route(
            "/admin/geo-zones",
            get(fulfillment_settings::list_geo_zones),
        )
        .with_state(ctx)
}

/// Mounts locales, translations, property labels and store settings.
///
/// `stheno-translation` and `stheno-settings` both had complete services and no routes;
/// the endpoints that existed answered from hardcoded literals.
pub fn admin_localisation_routes(
    translation_service: Arc<stheno_translation::TranslationService>,
    settings_service: Arc<stheno_settings::SettingsService>,
) -> Router {
    let ctx = localisation::LocalisationContext {
        translation_service,
        settings_service,
    };
    Router::new()
        .route(
            "/admin/locales",
            get(localisation::list_locales).post(localisation::create_locale),
        )
        .route("/admin/locales/{code}", get(localisation::get_locale))
        .route(
            "/admin/translations",
            get(localisation::list_translations).post(localisation::set_translation),
        )
        .route(
            "/admin/translations/settings",
            get(localisation::translation_settings),
        )
        .route(
            "/admin/translations/statistics",
            get(localisation::translation_statistics),
        )
        .route(
            "/admin/translations/resolve",
            get(localisation::resolve_translation),
        )
        .route(
            "/admin/property-labels",
            get(localisation::get_property_label).post(localisation::set_property_label),
        )
        .route(
            "/admin/settings",
            get(localisation::list_settings).post(localisation::set_setting),
        )
        .with_state(ctx)
}

/// Mounts admin upload and file routes.
pub fn admin_upload_routes(file_service: Arc<stheno_file::FileService>) -> Router {
    let ctx = uploads::AdminUploadContext { file_service };
    Router::new()
        .route(
            "/admin/uploads",
            get(uploads::list_uploads).post(uploads::create_upload),
        )
        .route(
            "/admin/uploads/{id}",
            get(uploads::get_upload).delete(uploads::delete_upload),
        )
        .with_state(ctx)
}
