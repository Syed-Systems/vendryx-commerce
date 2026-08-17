use rust_decimal_macros::dec;
use stheno_cart::{
    CartService, CreateCartInput, CreateLineItemAdjustmentInput, CreateLineItemInput,
    CreateLineItemTaxLineInput, UpdateLineItemInput,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

async fn service() -> (TestDb, CartService) {
    let db = TestDb::migrated().await;
    let svc = CartService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

#[tokio::test]
async fn test_line_items_crud_and_denormalized_fields() {
    let (db, service) = service().await;

    let cart = service
        .create_cart(CreateCartInput {
            currency_code: "eur".to_string(),
            ..Default::default()
        })
        .await
        .expect("cart created");

    // Add item with full denormalized product metadata
    let item = service
        .add_line_item(
            &cart.id,
            CreateLineItemInput {
                title: "Merino Wool Beanie".to_string(),
                subtitle: Some("Winter Edition".to_string()),
                thumbnail: Some("https://cdn.example.com/beanie.jpg".to_string()),
                quantity: 3,
                variant_id: Some("var_wool_black".to_string()),
                product_id: Some("prod_wool_beanie".to_string()),
                product_title: Some("Merino Wool Beanie".to_string()),
                product_description: Some("100% fine merino wool".to_string()),
                product_subtitle: Some("Thermal warmth".to_string()),
                product_type: Some("Headwear".to_string()),
                product_collection: Some("Winter 2026".to_string()),
                product_handle: Some("merino-wool-beanie".to_string()),
                variant_sku: Some("BEANIE-BLK-OS".to_string()),
                variant_barcode: Some("1234567890123".to_string()),
                variant_title: Some("Black / One Size".to_string()),
                variant_option_values: Some(serde_json::json!({
                    "Color": "Black",
                    "Size": "One Size"
                })),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_tax_inclusive: Some(false),
                is_custom_price: Some(false),
                unit_price: dec!(35.00),
                compare_at_unit_price: Some(dec!(45.00)),
                metadata: None,
            },
        )
        .await
        .expect("item added");

    assert_eq!(item.title, "Merino Wool Beanie");
    assert_eq!(item.variant_sku.as_deref(), Some("BEANIE-BLK-OS"));
    assert_eq!(item.product_handle.as_deref(), Some("merino-wool-beanie"));
    assert_eq!(item.compare_at_unit_price, Some(dec!(45.00)));

    // Update item quantity
    let updated = service
        .update_line_item(
            &cart.id,
            &item.id,
            UpdateLineItemInput {
                quantity: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("item updated");

    assert_eq!(updated.quantity, 5);
    assert_eq!(updated.totals.unwrap().subtotal, dec!(175.00)); // 35 * 5

    // Add adjustments & tax lines
    let _ = service
        .add_line_item_adjustment(
            &item.id,
            CreateLineItemAdjustmentInput {
                amount: dec!(25.00),
                promotion_id: Some("promo_bulk".to_string()),
                code: Some("BULK5".to_string()),
                description: Some("Bulk discount".to_string()),
            },
        )
        .await
        .expect("adjustment added");

    let _ = service
        .add_line_item_tax_line(
            &item.id,
            CreateLineItemTaxLineInput {
                rate: dec!(0.20),
                code: "VAT20".to_string(),
                tax_rate_id: None,
                provider_id: None,
                description: None,
            },
        )
        .await
        .expect("tax line added");

    let reloaded = service
        .get_line_item(&item.id)
        .await
        .expect("reloaded item");
    assert_eq!(reloaded.adjustments.len(), 1);
    assert_eq!(reloaded.tax_lines.len(), 1);
    // Subtotal = 175, Discount = 25 -> Taxable = 150 -> Tax = 30 -> Total = 180
    assert_eq!(reloaded.totals.as_ref().unwrap().subtotal, dec!(175.00));
    assert_eq!(
        reloaded.totals.as_ref().unwrap().discount_total,
        dec!(25.00)
    );
    assert_eq!(reloaded.totals.as_ref().unwrap().tax_total, dec!(30.00));
    assert_eq!(reloaded.totals.as_ref().unwrap().total, dec!(180.00));

    // Delete item
    service
        .delete_line_item(&cart.id, &item.id)
        .await
        .expect("item deleted");

    let cart_after = service.get_cart(&cart.id).await.expect("cart fetched");
    assert_eq!(cart_after.items.len(), 0);

    db.drop().await;
}
