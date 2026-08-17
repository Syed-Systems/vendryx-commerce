use stheno_links::{LinkStore, ALL_LINKS};

#[test]
fn test_all_36_links_declared_and_registered() {
    // DATA_MODEL.md §3: 22 writable link tables + 14 read-only link definitions.
    assert_eq!(ALL_LINKS.len(), 36, "expected 36 link definitions");

    let writable = ALL_LINKS.iter().filter(|l| l.table.is_some()).count();
    let read_only = ALL_LINKS.iter().filter(|l| l.table.is_none()).count();
    assert_eq!(writable, 22, "expected 22 writable links");
    assert_eq!(read_only, 14, "expected 14 read-only links");

    // Writable links own a table in the `links` schema, named after the link.
    for link in ALL_LINKS.iter().filter(|l| l.table.is_some()) {
        assert_eq!(
            link.table.unwrap(),
            format!("link_{}", link.name),
            "writable link {} has a mismatched table name",
            link.name
        );
    }

    // Names are unique.
    let mut names: Vec<&str> = ALL_LINKS.iter().map(|l| l.name).collect();
    names.sort_unstable();
    let unique = names.len();
    names.dedup();
    assert_eq!(names.len(), unique, "duplicate link names in ALL_LINKS");

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
