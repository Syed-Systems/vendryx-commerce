use stheno_cli::import::run_post_import_reindex;

#[test]
fn test_post_import_reindex_execution() {
    let products_count = 250;
    let orders_count = 500;
    let customers_count = 120;

    let report = run_post_import_reindex(products_count, orders_count, customers_count)
        .expect("Post-import reindexing must succeed");

    assert!(
        report.is_consistent,
        "Search and query indexes must be fully consistent post-import"
    );
    assert_eq!(report.products_indexed, products_count);
    assert_eq!(report.orders_indexed, orders_count);
    assert_eq!(report.customers_indexed, customers_count);
    assert_eq!(
        report.total_indexed,
        products_count + 15 + orders_count + customers_count
    );
    assert!(report.duration_ms > 0);
}
