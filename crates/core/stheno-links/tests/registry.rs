use stheno_links::{LinkStore, ALL_LINKS};

#[test]
fn test_all_37_links_declared_and_registered() {
    assert_eq!(ALL_LINKS.len(), 37);

    let store = LinkStore::new();
    // Test link CRUD
    store.link("order_customer", "order_123", "cus_456");
    store.link("order_customer", "order_124", "cus_456");

    let orders = store.get_by_right("order_customer", "cus_456");
    assert_eq!(orders.len(), 2);
    assert!(orders.contains(&"order_123".to_string()));
    assert!(orders.contains(&"order_124".to_string()));

    let customer = store.get_by_left("order_customer", "order_123");
    assert_eq!(customer, vec!["cus_456".to_string()]);
}
