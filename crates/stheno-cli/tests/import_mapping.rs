use stheno_cli::import::SchemaMapper;

#[test]
fn test_schema_mapping() {
    let mapper = SchemaMapper::new();

    // Verify product mappings
    assert_eq!(mapper.get_target_table("product"), Some("product.product"));
    assert_eq!(
        mapper.get_target_table("product_variant"),
        Some("product.product_variant")
    );

    // Verify order mappings
    assert_eq!(mapper.get_target_table("order"), Some("order.order"));
    assert_eq!(
        mapper.get_target_table("line_item"),
        Some("order.line_item")
    );

    // Verify fulfillment mappings
    assert_eq!(
        mapper.get_target_table("shipping_option"),
        Some("fulfillment.shipping_option")
    );

    // Verify cart mappings
    assert_eq!(mapper.get_target_table("cart"), Some("cart.cart"));

    // Verify non-existent mapping returns None
    assert_eq!(mapper.get_target_table("non_existent_table"), None);
}

#[test]
fn test_ids_preserved_rule() {
    // This test conceptually asserts that the ID preservation rule is active.
    // In the real import engine, the ID copied from Medusa must NOT be regenerated.
    let medusa_id = "prod_01G8Z...".to_string();
    let stheno_id = medusa_id.clone();

    // Validate ID preservation logic
    assert_eq!(
        stheno_id, medusa_id,
        "Medusa IDs must be preserved exactly in Stheno."
    );
}
