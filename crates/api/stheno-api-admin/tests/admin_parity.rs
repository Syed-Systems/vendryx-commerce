use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

use stheno_api_admin::{
    admin_catalog_routes, admin_config_routes, admin_customer_routes, admin_inventory_routes,
    admin_order_routes, admin_pricing_routes, admin_promotion_routes,
};
use stheno_product::{CreateCategoryInput, CreateProductInput, ProductService};
use stheno_inventory::{CreateInventoryItemInput, InventoryService};
use stheno_stock_location::{CreateStockLocationInput, StockLocationService};
use stheno_pricing::{CreatePriceListInput, PricingService};
use stheno_order::{CreateOrderInput, CreateReturnReasonInput, OrderService};
use stheno_fulfillment::FulfillmentService;
use stheno_fulfillment_manual::ManualFulfillmentProvider;
use stheno_payment::PaymentService;
use stheno_customer::{CreateCustomerInput, CustomerService};
use stheno_promotion::{CreatePromotionInput, PromotionService};
use stheno_region::{CreateRegionInput, RegionService};
use stheno_currency::CurrencyService;
use stheno_tax::{CreateTaxRegionInput, TaxService};
use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelService};

#[tokio::test]
async fn test_admin_catalog_endpoints_parity() {
    let service = Arc::new(ProductService::new());

    // 1. Seed product
    let prod = service
        .create_product(CreateProductInput {
            title: "Admin Parity Shirt".to_string(),
            handle: Some("admin-parity-shirt".to_string()),
            subtitle: Some("Heavy cotton".to_string()),
            description: Some("Durable cotton fabric".to_string()),
            discountable: Some(true),
        })
        .await
        .unwrap();

    // 2. Seed category
    let _cat = service
        .create_category(CreateCategoryInput {
            name: "Apparel".to_string(),
            handle: Some("apparel".to_string()),
            parent_category_id: None,
            rank: Some(1),
        })
        .await
        .unwrap();

    let app = admin_catalog_routes(service.clone());

    // 3. GET /admin/products
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["handle"], "admin-parity-shirt");

    // 4. GET /admin/products/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/products/{}", prod.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["handle"], "admin-parity-shirt");

    // 5. GET /admin/product-categories
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/product-categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["handle"], "apparel");

    // 6. DELETE /admin/products/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/admin/products/{}", prod.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["id"], prod.id.to_string());
    assert_eq!(body["object"], "product");
    assert_eq!(body["deleted"], true);
}

#[tokio::test]
async fn test_admin_inventory_endpoints_parity() {
    let inventory_service = Arc::new(InventoryService::new());
    let stock_location_service = Arc::new(StockLocationService::new());

    // Seed location
    let loc = stock_location_service
        .create_location(CreateStockLocationInput {
            name: "San Francisco Warehouse".to_string(),
            address_id: None,
        })
        .await
        .unwrap();

    let app = admin_inventory_routes(inventory_service.clone(), stock_location_service.clone());

    // 1. POST /admin/inventory-items
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/inventory-items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateInventoryItemInput {
                        sku: Some("SKU-MAIN".to_string()),
                        requires_shipping: Some(true),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["sku"], "SKU-MAIN");

    // 2. GET /admin/stock-locations
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/stock-locations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["name"], "San Francisco Warehouse");

    // 3. DELETE /admin/stock-locations/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/admin/stock-locations/{}", loc.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["id"], loc.id.to_string());
    assert_eq!(body["object"], "stock_location");
    assert_eq!(body["deleted"], true);
}

#[tokio::test]
async fn test_admin_pricing_endpoints_parity() {
    let service = Arc::new(PricingService::new());

    // Seed price list
    let list = service
        .create_price_list(CreatePriceListInput {
            title: "Black Friday Sale".to_string(),
            description: Some("BFCM discounts".to_string()),
            starts_at: None,
            ends_at: None,
            r#type: "sale".to_string(),
            rules: None,
        })
        .await
        .unwrap();

    let app = admin_pricing_routes(service.clone());

    // 1. GET /admin/price-lists
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/price-lists")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["title"], "Black Friday Sale");

    // 2. GET /admin/price-lists/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/price-lists/{}", list.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["title"], "Black Friday Sale");

    // 3. DELETE /admin/price-lists/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/admin/price-lists/{}", list.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["id"], list.id.to_string());
    assert_eq!(body["object"], "price_list");
    assert_eq!(body["deleted"], true);
}

#[tokio::test]
async fn test_admin_orders_endpoints_parity() {
    let order_service = Arc::new(OrderService::new());
    let fulfillment_service = Arc::new(FulfillmentService::new());
    let payment_service = Arc::new(PaymentService::new());

    // Seed return reason
    let _reason = order_service
        .create_return_reason(CreateReturnReasonInput {
            value: "damaged".to_string(),
            label: "Damaged item".to_string(),
            description: Some("Item arrived damaged".to_string()),
        })
        .await
        .unwrap();

    let app = admin_order_routes(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service.clone(),
    );

    // 1. POST /admin/orders
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/orders")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateOrderInput {
                        region_id: Some("reg_123".to_string()),
                        customer_id: Some("cust_123".to_string()),
                        sales_channel_id: Some("sc_123".to_string()),
                        email: Some("client@example.com".to_string()),
                        currency_code: "usd".to_string(),
                        shipping_address: None,
                        billing_address: None,
                        items: None,
                        shipping_methods: None,
                        is_draft_order: Some(false),
                        metadata: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    let order_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. GET /admin/orders/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/orders/{}", order_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["id"], order_id);

    // 3. GET /admin/return-reasons
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/return-reasons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["value"], "damaged");
}

#[tokio::test]
async fn test_admin_order_mutations() {
    let order_service = Arc::new(OrderService::new());
    let fulfillment_service = Arc::new(FulfillmentService::new());
    let payment_service = Arc::new(PaymentService::new());

    let app = admin_order_routes(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service.clone(),
    );

    // Seed an order
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_123".to_string()),
            customer_id: Some("cust_123".to_string()),
            sales_channel_id: Some("sc_123".to_string()),
            email: Some("client@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: None,
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // 1. POST /admin/orders/{id}/cancel
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/admin/orders/{}/cancel", order.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["status"], "canceled");
}

#[tokio::test]
async fn test_admin_draft_orders() {
    let order_service = Arc::new(OrderService::new());
    let fulfillment_service = Arc::new(FulfillmentService::new());
    let payment_service = Arc::new(PaymentService::new());

    let app = admin_order_routes(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service.clone(),
    );

    // Create a draft order
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/orders")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateOrderInput {
                        region_id: Some("reg_123".to_string()),
                        customer_id: Some("cust_123".to_string()),
                        sales_channel_id: Some("sc_123".to_string()),
                        email: Some("client@example.com".to_string()),
                        currency_code: "usd".to_string(),
                        shipping_address: None,
                        billing_address: None,
                        items: None,
                        shipping_methods: None,
                        is_draft_order: Some(true),
                        metadata: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["status"], "draft");
    assert_eq!(body["data"]["is_draft_order"], true);
}

#[tokio::test]
async fn test_admin_fulfillment() {
    let order_service = Arc::new(OrderService::new());
    let fulfillment_service = Arc::new(FulfillmentService::new());
    fulfillment_service.register_provider(Arc::new(ManualFulfillmentProvider::new()));
    let payment_service = Arc::new(PaymentService::new());

    let app = admin_order_routes(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service.clone(),
    );

    // Seed order
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_123".to_string()),
            customer_id: Some("cust_123".to_string()),
            sales_channel_id: Some("sc_123".to_string()),
            email: Some("client@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: None,
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // POST /admin/orders/{id}/fulfillments
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/admin/orders/{}/fulfillments", order.id))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "provider_id": "manual",
                        "items": []
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_admin_payments() {
    let order_service = Arc::new(OrderService::new());
    let fulfillment_service = Arc::new(FulfillmentService::new());
    let payment_service = Arc::new(PaymentService::new());

    let app = admin_order_routes(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service.clone(),
    );

    // Seed order
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_123".to_string()),
            customer_id: Some("cust_123".to_string()),
            sales_channel_id: Some("sc_123".to_string()),
            email: Some("client@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: None,
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // We can also verify GET /admin/orders
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["id"], order.id);
}

#[tokio::test]
async fn test_admin_customers() {
    let service = Arc::new(CustomerService::new());
    let app = admin_customer_routes(service.clone());

    // 1. POST /admin/customers
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/customers")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateCustomerInput {
                        email: "jane@example.com".to_string(),
                        first_name: Some("Jane".to_string()),
                        last_name: Some("Doe".to_string()),
                        phone: None,
                        company_name: None,
                        has_account: Some(true),
                        metadata: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["email"], "jane@example.com");
    let customer_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. GET /admin/customers/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/customers/{}", customer_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["email"], "jane@example.com");
}

#[tokio::test]
async fn test_admin_promotions() {
    let service = Arc::new(PromotionService::new());
    let app = admin_promotion_routes(service.clone());

    // 1. POST /admin/promotions
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/promotions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreatePromotionInput {
                        code: "SUMMER10".to_string(),
                        promotion_type: Some("standard".to_string()),
                        is_automatic: Some(false),
                        campaign_id: None,
                        status: Some("active".to_string()),
                        application_method: None,
                        rules: None,
                        metadata: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["code"], "SUMMER10");
    let promo_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. GET /admin/promotions/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/promotions/{}", promo_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["code"], "SUMMER10");

    // 3. GET /admin/promotions (list)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/promotions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
}

#[tokio::test]
async fn test_admin_config() {
    let region_service = Arc::new(RegionService::new());
    let currency_service = Arc::new(CurrencyService::new());
    let tax_service = Arc::new(TaxService::new());
    let sales_channel_service = Arc::new(SalesChannelService::new());
    let app = admin_config_routes(
        region_service,
        currency_service,
        tax_service,
        sales_channel_service,
    );

    // 1. POST /admin/regions
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/regions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateRegionInput {
                        name: "North America".to_string(),
                        currency_code: "usd".to_string(),
                        countries: vec!["us".to_string(), "ca".to_string()],
                        tax_rate: None,
                        metadata: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["name"], "North America");
    let region_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. GET /admin/regions/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/admin/regions/{}", region_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["name"], "North America");

    // 3. GET /admin/regions (list)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/regions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);

    // 4. POST /admin/sales-channels
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/sales-channels")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&CreateSalesChannelInput {
                        name: "Default Storefront".to_string(),
                        description: Some("The default web storefront".to_string()),
                        is_disabled: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["data"]["name"], "Default Storefront");
}
