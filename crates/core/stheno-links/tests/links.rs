use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_links::{LinkStore, PgLinkStore, ALL_LINKS};

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

#[tokio::test]
async fn test_pg_link_store_all_22_writable_links() {
    // P14-18: Verify all 22 writable link tables persist and resolve in Postgres
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let pg_links = PgLinkStore::new(pool);

    let writable_links: Vec<_> = ALL_LINKS.iter().filter(|l| l.table.is_some()).collect();
    assert_eq!(
        writable_links.len(),
        22,
        "Must have exactly 22 writable link definitions"
    );

    for link_def in &writable_links {
        let left_id = format!("left_test_{}", link_def.name);
        let right_id = format!("right_test_{}", link_def.name);

        // Write link
        pg_links
            .link(link_def.name, &left_id, &right_id)
            .await
            .unwrap_or_else(|e| panic!("Failed to link on {}: {}", link_def.name, e));

        // Read forward
        let fwd = pg_links.get_by_left(link_def.name, &left_id).await.unwrap();
        assert_eq!(
            fwd,
            vec![right_id.clone()],
            "Failed forward read on {}",
            link_def.name
        );

        // Read reverse
        let rev = pg_links
            .get_by_right(link_def.name, &right_id)
            .await
            .unwrap();
        assert_eq!(
            rev,
            vec![left_id.clone()],
            "Failed reverse read on {}",
            link_def.name
        );
    }

    db.drop().await;
}

#[tokio::test]
async fn test_pg_link_store_soft_delete_and_hiding() {
    // P14-20: Soft delete on link endpoints hides associated links
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let pg_links = PgLinkStore::new(pool);

    let link_name = "product_sales_channel";
    pg_links
        .link(link_name, "prod_active_1", "sc_web_1")
        .await
        .unwrap();
    pg_links
        .link(link_name, "prod_active_1", "sc_pos_1")
        .await
        .unwrap();

    let before_delete = pg_links
        .get_by_left(link_name, "prod_active_1")
        .await
        .unwrap();
    assert_eq!(before_delete.len(), 2);

    // Unlink one
    pg_links
        .unlink(link_name, "prod_active_1", "sc_web_1")
        .await
        .unwrap();
    let after_unlink = pg_links
        .get_by_left(link_name, "prod_active_1")
        .await
        .unwrap();
    assert_eq!(after_unlink, vec!["sc_pos_1".to_string()]);

    // Cascade delete remaining
    let cascade_deleted = pg_links
        .cascade_delete_left(link_name, "prod_active_1")
        .await
        .unwrap();
    assert_eq!(cascade_deleted, vec!["sc_pos_1".to_string()]);

    let after_cascade = pg_links
        .get_by_left(link_name, "prod_active_1")
        .await
        .unwrap();
    assert!(
        after_cascade.is_empty(),
        "Cascade soft-deleted link must be hidden"
    );

    db.drop().await;
}

#[tokio::test]
async fn test_pg_link_store_gdpr_erasure_cascade() {
    // P14-21: Hard delete / GDPR erasure cascades across link tables
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let pg_links = PgLinkStore::new(pool);

    let customer_id = "cus_gdpr_target_999";
    pg_links
        .link("customer_account_holder", customer_id, "acc_holder_101")
        .await
        .unwrap();

    let before = pg_links
        .get_by_left("customer_account_holder", customer_id)
        .await
        .unwrap();
    assert_eq!(before.len(), 1);

    // GDPR erase entity across all link tables
    let deleted_count = pg_links.gdpr_erase_entity(customer_id).await.unwrap();
    assert!(deleted_count >= 1);

    let after = pg_links
        .get_by_left("customer_account_holder", customer_id)
        .await
        .unwrap();
    assert!(
        after.is_empty(),
        "All links for erased entity must be permanently removed"
    );

    db.drop().await;
}
