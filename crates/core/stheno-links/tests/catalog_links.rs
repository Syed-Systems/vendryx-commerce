use stheno_links::LinkStore;

#[test]
fn test_catalog_links_bidirectional_and_cascade() {
    let store = LinkStore::new();

    // 1. Product <-> SalesChannel link
    store.link("product_sales_channel", "prod_123", "sc_web");
    // 2. ProductVariant <-> InventoryItem link
    store.link("product_variant_inventory_item", "var_123", "inv_123");
    // 3. SalesChannel <-> StockLocation link
    store.link("sales_channel_stock_location", "sc_web", "loc_wh1");

    // Forward lookups (get_by_left)
    assert_eq!(
        store.get_by_left("product_sales_channel", "prod_123"),
        vec!["sc_web".to_string()]
    );
    assert_eq!(
        store.get_by_left("product_variant_inventory_item", "var_123"),
        vec!["inv_123".to_string()]
    );
    assert_eq!(
        store.get_by_left("sales_channel_stock_location", "sc_web"),
        vec!["loc_wh1".to_string()]
    );

    // Reverse lookups (get_by_right)
    assert_eq!(
        store.get_by_right("product_sales_channel", "sc_web"),
        vec!["prod_123".to_string()]
    );

    // Cascade delete on prod_123
    let deleted = store.cascade_delete_left("product_sales_channel", "prod_123");
    assert_eq!(deleted, vec!["sc_web".to_string()]);
    assert!(store
        .get_by_left("product_sales_channel", "prod_123")
        .is_empty());
}
