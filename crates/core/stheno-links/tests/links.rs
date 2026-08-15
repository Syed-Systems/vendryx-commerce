use stheno_links::LinkStore;

#[test]
fn test_link_store_bidirectional_resolution() {
    let store = LinkStore::new();

    store.link("product_sales_channel", "prod_1", "sc_web");
    store.link("product_sales_channel", "prod_1", "sc_mobile");
    store.link("product_sales_channel", "prod_2", "sc_web");

    // Forward resolution (left -> right)
    let channels = store.get_by_left("product_sales_channel", "prod_1");
    assert_eq!(channels.len(), 2);
    assert!(channels.contains(&"sc_web".to_string()));
    assert!(channels.contains(&"sc_mobile".to_string()));

    // Reverse resolution (right -> left)
    let products = store.get_by_right("product_sales_channel", "sc_web");
    assert_eq!(products.len(), 2);
    assert!(products.contains(&"prod_1".to_string()));
    assert!(products.contains(&"prod_2".to_string()));
}

#[test]
fn test_link_store_cascade_delete() {
    let store = LinkStore::new();

    store.link("product_variant_price_set", "var_1", "pset_1");
    store.link("product_variant_price_set", "var_2", "pset_2");

    let deleted_pset = store.cascade_delete_left("product_variant_price_set", "var_1");
    assert_eq!(deleted_pset, vec!["pset_1".to_string()]);

    assert!(store
        .get_by_left("product_variant_price_set", "var_1")
        .is_empty());
    assert!(store
        .get_by_right("product_variant_price_set", "pset_1")
        .is_empty());

    // var_2 remains untouched
    assert_eq!(
        store.get_by_left("product_variant_price_set", "var_2"),
        vec!["pset_2".to_string()]
    );
}
