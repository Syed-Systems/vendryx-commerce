use stheno_flows::QuotaStore;

#[test]
fn test_grandfathering_merchant_keeps_all_existing_products_on_downgrade() {
    let quota_store = QuotaStore::new();

    // Merchant originally had 412 products created under Pro tier
    quota_store.set_quota("merchant_1", "max_products", 1000, 412);
    assert_eq!(quota_store.get_count("merchant_1", "max_products"), 412);

    // Plan downgraded to Free tier with a limit of 100 products
    quota_store.downgrade_limit("merchant_1", "max_products", 100);

    // Grandfathering Rule (ENTITLEMENTS.md §5.3):
    // 1. All 412 existing products remain live (current_count is NOT truncated or deleted)
    assert_eq!(
        quota_store.get_count("merchant_1", "max_products"),
        412,
        "All 412 existing live products must be preserved"
    );

    // 2. Creating any new product (even 1) is blocked by the quota check
    let create_attempt = quota_store.try_increment("merchant_1", "max_products", 1);
    assert!(create_attempt.is_err());
    let err_str = create_attempt.unwrap_err().to_string();
    assert!(err_str.contains("quota_exceeded"));
}
