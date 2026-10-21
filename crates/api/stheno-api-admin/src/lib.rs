//! `stheno-api-admin`: Medusa v2 wire-compatible Admin API routes.

pub mod api_keys;
pub mod auth_guard;
pub mod catalog;
pub mod config;
pub mod couriers;
pub mod customers;
pub mod fulfillment_settings;
pub mod home;
pub mod inventory;
pub mod localisation;
pub mod onboarding;
pub mod orders;
pub mod pricing;
pub mod promotions;
pub mod rbac;
pub mod reconciliation;
pub mod reports;
pub mod settings;
pub mod store;
pub mod uploads;
pub mod users;
pub mod views;

pub use couriers::admin_courier_routes;
pub use onboarding::admin_onboarding_routes;
pub use reconciliation::admin_reconciliation_routes;

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
            "/admin/products/{id}/variants/batch",
            post(catalog::batch_product_variants),
        )
        .route(
            "/admin/products/{id}/variants/inventory-items/batch",
            post(catalog::batch_product_variants_inventory),
        )
        .route(
            "/admin/products/{id}/variants/{variant_id}",
            get(catalog::get_variant)
                .post(catalog::update_variant)
                .delete(catalog::delete_variant),
        )
        .route(
            "/admin/products/{id}/variants/{variant_id}/images/batch",
            post(catalog::batch_variant_images),
        )
        .route(
            "/admin/products/{id}/variants/{variant_id}/inventory-items",
            post(catalog::create_variant_inventory_item),
        )
        .route(
            "/admin/products/{id}/variants/{variant_id}/inventory-items/{inventory_item_id}",
            post(catalog::update_variant_inventory_item)
                .delete(catalog::delete_variant_inventory_item),
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
        .route("/admin/products/batch", post(catalog::batch_products))
        .route("/admin/products/export", post(catalog::export_products))
        .route("/admin/products/import", post(catalog::import_products))
        .route("/admin/products/imports", post(catalog::import_products))
        .route(
            "/admin/products/import/{transaction_id}/confirm",
            post(catalog::confirm_product_import),
        )
        .route(
            "/admin/products/imports/{transaction_id}/confirm",
            post(catalog::confirm_product_import),
        )
        .route(
            "/admin/products/{id}/options",
            get(catalog::list_product_options).post(catalog::create_product_option),
        )
        .route(
            "/admin/products/{id}/options/batch",
            post(catalog::batch_product_options),
        )
        .route(
            "/admin/products/{id}/options/{option_id}",
            get(catalog::get_product_option)
                .post(catalog::update_product_option)
                .delete(catalog::delete_product_option),
        )
        .route(
            "/admin/products/{id}/images/{image_id}/variants/batch",
            post(catalog::batch_product_image_variants),
        )
        .route(
            "/admin/product-options",
            get(catalog::list_global_product_options).post(catalog::create_global_product_option),
        )
        .route(
            "/admin/product-options/{id}",
            get(catalog::get_product_option_by_id)
                .post(catalog::update_product_option_by_id)
                .delete(catalog::delete_product_option_by_id),
        )
        .route(
            "/admin/product-options/{id}/values",
            get(catalog::list_product_option_values),
        )
        .route(
            "/admin/product-options/{id}/values/{value_id}",
            get(catalog::get_product_option_value)
                .post(catalog::update_product_option_value)
                .delete(catalog::delete_product_option_value),
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

/// Mounts the merchant owner home summary route.
///
/// `GET /admin/home/summary` — returns live pending-order and low-stock aggregates.
/// This is an additive Stheno extension (no Medusa v2 equivalent) — the forked
/// dashboard ignores unknown routes, so adding it does not break parity (ADR-002).
pub fn admin_home_routes(
    order_service: Arc<OrderService>,
    inventory_service: Arc<InventoryService>,
) -> Router {
    let ctx = home::HomeSummaryContext {
        order_service,
        inventory_service,
    };
    Router::new()
        .route("/admin/home/summary", get(home::get_home_summary))
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
        .route("/admin/payments", get(orders::list_payments))
        .route(
            "/admin/payments/payment-providers",
            get(orders::list_payment_providers),
        )
        .route("/admin/payments/{id}", get(orders::get_payment))
        .route(
            "/admin/payments/{id}/capture",
            post(orders::capture_payment_direct),
        )
        .route(
            "/admin/payments/{id}/refund",
            post(orders::refund_payment_direct),
        )
        .route(
            "/admin/payment-collections",
            get(orders::list_payment_collections).post(orders::create_payment_collection),
        )
        .route(
            "/admin/payment-collections/{id}",
            get(orders::get_payment_collection).delete(orders::delete_payment_collection),
        )
        .route(
            "/admin/payment-collections/{id}/payment-sessions",
            post(orders::create_payment_collection_session),
        )
        .route(
            "/admin/payment-collections/{id}/mark-as-paid",
            post(orders::mark_payment_collection_paid),
        )
        .route(
            "/admin/store-credit-accounts",
            get(orders::list_store_credit_accounts).post(orders::create_store_credit_account),
        )
        .route(
            "/admin/store-credit-accounts/{id}",
            get(orders::get_store_credit_account),
        )
        .route(
            "/admin/store-credit-accounts/{id}/credit",
            post(orders::add_store_credit_account_credit),
        )
        .route(
            "/admin/store-credit-accounts/{id}/transactions",
            get(orders::list_store_credit_account_transactions),
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
        .route("/admin/orders/export", post(orders::export_orders))
        .route("/admin/orders/{id}/changes", get(orders::get_order_changes))
        .route(
            "/admin/orders/{id}/line-items",
            get(orders::get_order_line_items),
        )
        .route(
            "/admin/orders/{id}/shipping-options",
            get(orders::get_order_shipping_options),
        )
        .route(
            "/admin/orders/{id}/complete",
            post(orders::complete_order_admin),
        )
        .route(
            "/admin/orders/{id}/credit-lines",
            post(orders::add_order_credit_lines),
        )
        .route(
            "/admin/orders/{id}/payment-sessions/authorize",
            post(orders::authorize_order_payment_sessions),
        )
        .route(
            "/admin/orders/{id}/transfer/cancel",
            post(orders::cancel_order_transfer),
        )
        .route(
            "/admin/orders/{id}/transfer/guest",
            post(orders::guest_order_transfer),
        )
        .route(
            "/admin/orders/{id}/fulfillments/{fulfillment_id}/shipments",
            post(orders::create_fulfillment_shipment),
        )
        .route(
            "/admin/orders/{id}/fulfillments/{fulfillment_id}/mark-as-delivered",
            post(orders::mark_fulfillment_delivered),
        )
        .route(
            "/admin/orders/{id}/fulfillments/{fulfillment_id}/cancel",
            post(orders::cancel_order_fulfillment),
        )
        .route(
            "/admin/returns/{id}",
            get(orders::get_return).post(orders::update_return),
        )
        .route("/admin/returns/{id}/cancel", post(orders::cancel_return))
        .route(
            "/admin/returns/{id}/dismiss-items",
            post(orders::dismiss_return_items),
        )
        .route(
            "/admin/returns/{id}/dismiss-items/{action_id}",
            post(orders::update_dismiss_return_item).delete(orders::delete_dismiss_return_item),
        )
        .route(
            "/admin/returns/{id}/receive-items",
            post(orders::receive_return_items),
        )
        .route(
            "/admin/returns/{id}/receive-items/{action_id}",
            post(orders::update_receive_return_item).delete(orders::delete_receive_return_item),
        )
        .route(
            "/admin/returns/{id}/receive/confirm",
            post(orders::confirm_return_receive),
        )
        .route(
            "/admin/returns/{id}/request",
            post(orders::confirm_return_request).delete(orders::cancel_return_request),
        )
        .route(
            "/admin/returns/{id}/request-items",
            post(orders::add_return_request_items),
        )
        .route(
            "/admin/returns/{id}/request-items/{action_id}",
            post(orders::update_return_request_item).delete(orders::delete_return_request_item),
        )
        .route(
            "/admin/returns/{id}/shipping-method",
            post(orders::add_return_shipping_method),
        )
        .route(
            "/admin/returns/{id}/shipping-method/{action_id}",
            post(orders::update_return_shipping_method)
                .delete(orders::delete_return_shipping_method),
        )
        .route(
            "/admin/order-edits",
            get(orders::list_order_edits).post(orders::create_order_edit),
        )
        .route(
            "/admin/order-edits/{id}",
            axum::routing::delete(orders::delete_order_edit),
        )
        .route(
            "/admin/order-edits/{id}/items",
            post(orders::add_order_edit_items),
        )
        .route(
            "/admin/order-edits/{id}/items/item/{item_id}",
            post(orders::update_order_edit_original_item),
        )
        .route(
            "/admin/order-edits/{id}/items/{action_id}",
            post(orders::update_order_edit_item_action)
                .delete(orders::delete_order_edit_item_action),
        )
        .route(
            "/admin/order-edits/{id}/shipping-method",
            post(orders::add_order_edit_shipping_method),
        )
        .route(
            "/admin/order-edits/{id}/shipping-method/{action_id}",
            post(orders::update_order_edit_shipping_method)
                .delete(orders::delete_order_edit_shipping_method),
        )
        .route(
            "/admin/order-edits/{id}/request",
            post(orders::request_order_edit),
        )
        .route(
            "/admin/order-edits/{id}/confirm",
            post(orders::confirm_order_edit),
        )
        .route(
            "/admin/claims",
            get(orders::list_claims).post(orders::create_claim),
        )
        .route("/admin/claims/{id}", get(orders::get_claim))
        .route("/admin/claims/{id}/cancel", post(orders::cancel_claim))
        .route(
            "/admin/claims/{id}/claim-items",
            post(orders::add_claim_items),
        )
        .route(
            "/admin/claims/{id}/claim-items/{action_id}",
            post(orders::update_claim_item).delete(orders::delete_claim_item),
        )
        .route(
            "/admin/claims/{id}/inbound/items",
            post(orders::add_claim_inbound_items),
        )
        .route(
            "/admin/claims/{id}/inbound/items/{action_id}",
            post(orders::update_claim_inbound_item).delete(orders::delete_claim_inbound_item),
        )
        .route(
            "/admin/claims/{id}/inbound/shipping-method",
            post(orders::add_claim_inbound_shipping_method),
        )
        .route(
            "/admin/claims/{id}/inbound/shipping-method/{action_id}",
            post(orders::update_claim_inbound_shipping_method)
                .delete(orders::delete_claim_inbound_shipping_method),
        )
        .route(
            "/admin/claims/{id}/outbound/items",
            post(orders::add_claim_outbound_items),
        )
        .route(
            "/admin/claims/{id}/outbound/items/{action_id}",
            post(orders::update_claim_outbound_item).delete(orders::delete_claim_outbound_item),
        )
        .route(
            "/admin/claims/{id}/outbound/shipping-method",
            post(orders::add_claim_outbound_shipping_method),
        )
        .route(
            "/admin/claims/{id}/outbound/shipping-method/{action_id}",
            post(orders::update_claim_outbound_shipping_method)
                .delete(orders::delete_claim_outbound_shipping_method),
        )
        .route(
            "/admin/claims/{id}/request",
            post(orders::request_claim).delete(orders::cancel_claim_request),
        )
        .route(
            "/admin/exchanges",
            get(orders::list_exchanges).post(orders::create_exchange),
        )
        .route("/admin/exchanges/{id}", get(orders::get_exchange))
        .route(
            "/admin/exchanges/{id}/cancel",
            post(orders::cancel_exchange),
        )
        .route(
            "/admin/exchanges/{id}/inbound/items",
            post(orders::add_exchange_inbound_items),
        )
        .route(
            "/admin/exchanges/{id}/inbound/items/{action_id}",
            post(orders::update_exchange_inbound_item).delete(orders::delete_exchange_inbound_item),
        )
        .route(
            "/admin/exchanges/{id}/inbound/shipping-method",
            post(orders::add_exchange_inbound_shipping_method),
        )
        .route(
            "/admin/exchanges/{id}/inbound/shipping-method/{action_id}",
            post(orders::update_exchange_inbound_shipping_method)
                .delete(orders::delete_exchange_inbound_shipping_method),
        )
        .route(
            "/admin/exchanges/{id}/outbound/items",
            post(orders::add_exchange_outbound_items),
        )
        .route(
            "/admin/exchanges/{id}/outbound/items/{action_id}",
            post(orders::update_exchange_outbound_item)
                .delete(orders::delete_exchange_outbound_item),
        )
        .route(
            "/admin/exchanges/{id}/outbound/shipping-method",
            post(orders::add_exchange_outbound_shipping_method),
        )
        .route(
            "/admin/exchanges/{id}/outbound/shipping-method/{action_id}",
            post(orders::update_exchange_outbound_shipping_method)
                .delete(orders::delete_exchange_outbound_shipping_method),
        )
        .route(
            "/admin/exchanges/{id}/request",
            post(orders::request_exchange).delete(orders::cancel_exchange_request),
        )
        .route(
            "/admin/draft-orders",
            get(orders::list_draft_orders).post(orders::create_draft_order),
        )
        .route(
            "/admin/draft-orders/{id}",
            get(orders::get_draft_order)
                .post(orders::update_draft_order)
                .delete(orders::delete_draft_order),
        )
        .route(
            "/admin/draft-orders/{id}/convert-to-order",
            post(orders::convert_draft_order_to_order),
        )
        .route(
            "/admin/draft-orders/{id}/edit",
            post(orders::create_draft_order_edit).delete(orders::delete_draft_order_edit),
        )
        .route(
            "/admin/draft-orders/{id}/edit/confirm",
            post(orders::confirm_draft_order_edit),
        )
        .route(
            "/admin/draft-orders/{id}/edit/items",
            post(orders::add_draft_order_edit_items),
        )
        .route(
            "/admin/draft-orders/{id}/edit/items/item/{item_id}",
            post(orders::update_draft_order_edit_original_item),
        )
        .route(
            "/admin/draft-orders/{id}/edit/items/{action_id}",
            post(orders::update_draft_order_edit_item_action)
                .delete(orders::delete_draft_order_edit_item_action),
        )
        .route(
            "/admin/draft-orders/{id}/edit/promotions",
            post(orders::add_draft_order_edit_promotions)
                .delete(orders::delete_draft_order_edit_promotions),
        )
        .route(
            "/admin/draft-orders/{id}/edit/request",
            post(orders::request_draft_order_edit),
        )
        .route(
            "/admin/draft-orders/{id}/edit/shipping-methods",
            post(orders::add_draft_order_edit_shipping_methods),
        )
        .route(
            "/admin/draft-orders/{id}/edit/shipping-methods/method/{method_id}",
            post(orders::update_draft_order_edit_shipping_method_by_id)
                .delete(orders::delete_draft_order_edit_shipping_method_by_id),
        )
        .route(
            "/admin/draft-orders/{id}/edit/shipping-methods/{action_id}",
            post(orders::update_draft_order_edit_shipping_method_action)
                .delete(orders::delete_draft_order_edit_shipping_method_action),
        )
        .route(
            "/admin/fulfillments",
            post(orders::create_fulfillment_direct),
        )
        .route(
            "/admin/fulfillments/{id}/cancel",
            post(orders::cancel_fulfillment_direct),
        )
        .route(
            "/admin/fulfillments/{id}/shipment",
            post(orders::create_fulfillment_shipment_direct),
        )
        .route(
            "/admin/order-changes/{id}",
            post(orders::apply_order_change),
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
            "/admin/customers/{id}/addresses",
            get(customers::list_customer_addresses).post(customers::add_customer_address),
        )
        .route(
            "/admin/customers/{id}/addresses/{address_id}",
            get(customers::get_customer_address)
                .post(customers::update_customer_address)
                .delete(customers::delete_customer_address),
        )
        .route(
            "/admin/customers/{id}/customer-groups",
            post(customers::manage_customer_groups),
        )
        .route(
            "/admin/customer-groups",
            get(customers::list_customer_groups).post(customers::create_customer_group),
        )
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
            "/admin/promotions/rule-attribute-options/{rule_type}",
            get(promotions::list_rule_attribute_options),
        )
        .route(
            "/admin/promotions/rule-value-options/{rule_type}/{rule_attribute_id}",
            get(promotions::list_rule_value_options),
        )
        .route(
            "/admin/promotions/{id}/rules/batch",
            post(promotions::batch_promotion_rules),
        )
        .route(
            "/admin/promotions/{id}/buy-rules/batch",
            post(promotions::batch_promotion_buy_rules),
        )
        .route(
            "/admin/promotions/{id}/target-rules/batch",
            post(promotions::batch_promotion_target_rules),
        )
        .route(
            "/admin/promotions/{id}/{rule_type}",
            get(promotions::list_promotion_rules_by_type),
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
        .route(
            "/admin/campaigns/{id}/promotions",
            post(promotions::add_campaign_promotions),
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
            "/admin/payment-providers/{provider_id}/verify",
            post(config::verify_payment_provider_credentials),
        )
        .route(
            "/admin/regions/{region_id}/payment-providers/{provider_id}",
            post(config::enable_region_payment_provider)
                .delete(config::disable_region_payment_provider),
        )
        // Gift Cards
        .route(
            "/admin/gift-cards",
            get(config::list_gift_cards).post(config::create_gift_card),
        )
        .route(
            "/admin/gift-cards/{id}",
            get(config::get_gift_card).post(config::update_gift_card),
        )
        .route(
            "/admin/gift-cards/{id}/orders",
            get(config::list_gift_card_orders),
        )
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
            ctx.payment_service.clone(),
            ctx.links,
        ))
        .merge(admin_reconciliation_routes(ctx.payment_service));

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
        .route(
            "/admin/reports/today",
            get(reports::today_handler),
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
        .route("/admin/invites/accept", post(users::accept_invite))
        .route(
            "/admin/invites/{id}",
            get(users::get_invite).delete(users::revoke_invite),
        )
        .route("/admin/invites/{id}/resend", post(users::resend_invite))
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
        .route("/admin/notifications/{id}", get(store::get_notification))
        .route("/admin/plugins", get(store::list_plugins))
        .route("/admin/search", get(store::admin_search))
        .route("/admin/index/details", get(store::get_index_details))
        .route("/admin/index/sync", post(store::sync_index))
        .route(
            "/admin/store/branding",
            get(store::get_branding).post(store::upsert_branding),
        )
        .with_state(ctx)
}

/// Mounts the view-configuration routes the dashboard's data tables read.
pub fn admin_views_routes() -> Router {
    Router::new()
        .route("/admin/views/entities", get(views::list_entities))
        .route("/admin/views/{entity}/columns", get(views::list_columns))
        .route(
            "/admin/views/{entity}/configurations",
            get(views::list_configurations).post(views::create_configuration),
        )
        .route(
            "/admin/views/{entity}/configurations/{id}",
            get(views::get_configuration)
                .post(views::update_configuration)
                .delete(views::delete_configuration),
        )
        .route(
            "/admin/views/{entity}/configurations/active",
            get(views::active_configuration).post(views::set_active_configuration),
        )
        // Layouts
        .route(
            "/admin/layouts/configurations",
            get(views::list_layout_configurations),
        )
        .route(
            "/admin/layouts/{zone}/configuration",
            get(views::get_layout_configuration)
                .post(views::set_layout_configuration)
                .delete(views::delete_layout_configuration),
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
            "/admin/api-keys/{id}/sales-channels",
            post(api_keys::manage_api_key_sales_channels),
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
            "/admin/publishable-api-keys/{id}/sales-channels",
            post(api_keys::manage_api_key_sales_channels),
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
        .route("/admin/rbac/me/permissions", get(rbac::get_me_permissions))
        .with_state(ctx)
}

/// Mounts store settings routes for merchant configuration (COD rules, policies, etc.).
pub fn admin_settings_routes(settings_service: Arc<stheno_settings::SettingsService>) -> Router {
    let ctx = settings::SettingsContext { settings_service };
    Router::new()
        .route(
            "/admin/settings",
            get(settings::list_settings).post(settings::set_setting_body),
        )
        .route(
            "/admin/settings/{key}",
            get(settings::get_setting).post(settings::set_setting),
        )
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
            get(fulfillment_settings::get_fulfillment_set_service_zone)
                .post(fulfillment_settings::update_fulfillment_set_service_zone)
                .delete(fulfillment_settings::delete_service_zone_for_set),
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
            "/admin/shipping-options/{id}",
            get(fulfillment_settings::get_shipping_option)
                .post(fulfillment_settings::update_shipping_option)
                .delete(fulfillment_settings::delete_shipping_option),
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
            "/admin/shipping-option-types/{id}",
            get(fulfillment_settings::get_shipping_option_type)
                .post(fulfillment_settings::update_shipping_option_type)
                .delete(fulfillment_settings::delete_shipping_option_type),
        )
        .route(
            "/admin/stock-locations/{id}/fulfillment-providers",
            post(fulfillment_settings::manage_location_fulfillment_providers),
        )
        .route(
            "/admin/stock-locations/{id}/fulfillment-sets",
            post(fulfillment_settings::add_location_fulfillment_set),
        )
        .route(
            "/admin/stock-locations/{id}/sales-channels",
            post(fulfillment_settings::manage_location_sales_channels),
        )
        .route(
            "/admin/fulfillment-providers",
            get(fulfillment_settings::list_fulfillment_providers),
        )
        .route(
            "/admin/fulfillment-providers/{id}/options",
            get(fulfillment_settings::list_fulfillment_provider_options),
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
            "/admin/translations/entities",
            get(store::list_translatable_entities),
        )
        .route("/admin/translations/batch", post(store::batch_translations))
        .route(
            "/admin/translations/settings/batch",
            post(store::batch_translation_settings),
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
            "/admin/property-labels/batch",
            post(store::batch_property_labels),
        )
        .route(
            "/admin/property-labels/{id}",
            get(store::get_property_label)
                .post(store::update_property_label)
                .delete(store::delete_property_label),
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
            "/admin/uploads/presigned-urls",
            post(uploads::create_presigned_url),
        )
        .route(
            "/admin/uploads/{id}",
            get(uploads::get_upload).delete(uploads::delete_upload),
        )
        .with_state(ctx)
}
