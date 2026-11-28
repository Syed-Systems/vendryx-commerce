use std::sync::Arc;

use rust_decimal::Decimal;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::{ProductOfferRequest, ProductOfferTool};
use stheno_inventory::InventoryService;
use stheno_pricing::{CreatePriceInput, PricingService};
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService, UpdateProductInput};
use stheno_region::RegionService;

#[tokio::test]
async fn offer_tool_uses_live_configured_price_and_leaves_missing_stock_unknown() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let products = Arc::new(ProductService::new(pool.clone()));
    let pricing = Arc::new(PricingService::new(pool.clone()));
    let inventory = Arc::new(InventoryService::new(pool.clone()));
    let regions = Arc::new(RegionService::new(pool));

    let product = products
        .create_product(CreateProductInput {
            title: "Live test product".to_string(),
            ..Default::default()
        })
        .await
        .expect("create draft product");
    let variant = products
        .create_variant(CreateVariantInput {
            product_id: product.id.to_string(),
            title: "Default".to_string(),
            sku: Some("LIVE-1".to_string()),
            barcode: None,
            option_value_ids: Vec::new(),
        })
        .await
        .expect("create variant");
    pricing
        .set_variant_prices(
            &variant.id.to_string(),
            vec![CreatePriceInput {
                currency_code: "usd".to_string(),
                amount: Decimal::new(2599, 2),
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            }],
        )
        .await
        .expect("set merchant price");
    let tool = ProductOfferTool::new(products.clone(), pricing, inventory, regions);
    let request = ProductOfferRequest {
        variant_id: variant.id.to_string(),
        currency_code: "USD".to_string(),
        region_id: None,
        quantity: 1,
    };
    assert!(matches!(
        tool.execute(request.clone()).await,
        Err(stheno_types::Error::NotFound { .. })
    ));

    products
        .update_product(
            &product.id.to_string(),
            UpdateProductInput {
                status: Some("published".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("publish product");
    let offer = tool.execute(request).await.expect("resolve live offer");
    assert_eq!(offer.product_id, product.id.to_string());
    assert_eq!(offer.variant_id, variant.id.to_string());
    assert_eq!(
        offer
            .price
            .expect("server-calculated price")
            .calculated_amount,
        Decimal::new(2599, 2)
    );
    assert!(offer.available_quantity.is_none());
    assert!(offer.inventory_tracked);
    db.drop().await;
}
