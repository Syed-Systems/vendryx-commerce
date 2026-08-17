use std::sync::Arc;
use stheno_api_admin::*;
use stheno_currency::CurrencyService;
use stheno_customer::CustomerService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::FulfillmentService;
use stheno_inventory::InventoryService;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_pricing::PricingService;
use stheno_product::ProductService;
use stheno_promotion::PromotionService;
use stheno_region::RegionService;
use stheno_reporting::ReportingService;
use stheno_sales_channel::SalesChannelService;
use stheno_stock_location::StockLocationService;
use stheno_tax::TaxService;

#[tokio::test]
async fn test_admin_route_coverage_mounted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let ctx = AdminApiContext {
        product_service: Arc::new(ProductService::new(pool.clone())),
        inventory_service: Arc::new(InventoryService::new(pool.clone())),
        stock_location_service: Arc::new(StockLocationService::new(pool.clone())),
        pricing_service: Arc::new(PricingService::new(pool.clone())),
        order_service: Arc::new(OrderService::new(pool.clone())),
        fulfillment_service: Arc::new(FulfillmentService::new(pool.clone())),
        payment_service: Arc::new(PaymentService::new(pool.clone())),
        customer_service: Arc::new(CustomerService::new(pool.clone())),
        promotion_service: Arc::new(PromotionService::new(pool.clone())),
        region_service: Arc::new(RegionService::new(pool.clone())),
        currency_service: Arc::new(CurrencyService::new(pool.clone())),
        tax_service: Arc::new(TaxService::new(pool.clone())),
        sales_channel_service: Arc::new(SalesChannelService::new(pool.clone())),
        reporting_service: Some(Arc::new(ReportingService::new(pool.clone()))),
    };

    let router = admin_routes(ctx);

    // Verify router compiles and builds without panic
    let _ = router;

    db.drop().await;
}
