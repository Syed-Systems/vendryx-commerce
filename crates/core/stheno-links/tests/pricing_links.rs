use stheno_links::{LinkStore, ALL_LINKS};

#[test]
fn test_pricing_link_definitions_exist() {
    // P3-16: Verify all 3 pricing-related link definitions exist
    let names: Vec<&str> = ALL_LINKS.iter().map(|l| l.name).collect();

    assert!(
        names.contains(&"product_variant_price_set"),
        "product_variant_price_set link missing"
    );
    assert!(
        names.contains(&"shipping_option_price_set"),
        "shipping_option_price_set link missing"
    );
    assert!(
        names.contains(&"region_payment_provider"),
        "region_payment_provider link missing"
    );
}

#[test]
fn test_pricing_link_crud_operations() {
    let store = LinkStore::new();

    // 1. Link variant -> price_set
    store.link("product_variant_price_set", "variant_01abc", "pset_01xyz");
    assert_eq!(
        store.get_by_left("product_variant_price_set", "variant_01abc"),
        vec!["pset_01xyz"]
    );
    assert_eq!(
        store.get_by_right("product_variant_price_set", "pset_01xyz"),
        vec!["variant_01abc"]
    );

    // 2. Link shipping_option -> price_set
    store.link("shipping_option_price_set", "so_01flat", "pset_02ship");
    assert_eq!(
        store.get_by_left("shipping_option_price_set", "so_01flat"),
        vec!["pset_02ship"]
    );

    // 3. Link region -> payment_provider
    store.link("region_payment_provider", "reg_01us", "pp_01stripe");
    store.link("region_payment_provider", "reg_01us", "pp_02paypal");
    let providers = store.get_by_left("region_payment_provider", "reg_01us");
    assert_eq!(providers.len(), 2);
    assert!(providers.contains(&"pp_01stripe".to_string()));
    assert!(providers.contains(&"pp_02paypal".to_string()));

    // Reverse: stripe is linked to US region
    assert_eq!(
        store.get_by_right("region_payment_provider", "pp_01stripe"),
        vec!["reg_01us"]
    );

    // 4. Cascade delete variant link
    let deleted = store.cascade_delete_left("product_variant_price_set", "variant_01abc");
    assert_eq!(deleted, vec!["pset_01xyz"]);
    assert!(store
        .get_by_left("product_variant_price_set", "variant_01abc")
        .is_empty());
}
