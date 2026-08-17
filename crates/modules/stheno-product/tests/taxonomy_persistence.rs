//! P11-20: `product` taxonomy on Postgres — categories with materialized paths,
//! collections, tags, types.
//!
//! The category tree is the part worth testing hard. A materialized path is only
//! worth its write cost if descendant queries are a single indexed prefix match,
//! so these tests assert both that the paths are built correctly and that the
//! subtree query returns exactly the subtree.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateCategoryInput, CreateCollectionInput, ProductService};

async fn service() -> (TestDb, ProductService) {
    let db = TestDb::migrated().await;
    let svc = ProductService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

fn category(name: &str, parent: Option<&str>) -> CreateCategoryInput {
    CreateCategoryInput {
        name: name.to_string(),
        handle: None,
        parent_category_id: parent.map(str::to_string),
        rank: None,
    }
}

/// Builds:  Root ─┬─ Apparel ─┬─ Shirts
///                │           └─ Trousers
///                └─ Footwear
async fn tree(service: &ProductService) -> (String, String, String, String, String) {
    let root = service
        .create_category(category("Root", None))
        .await
        .expect("root")
        .id
        .to_string();
    let apparel = service
        .create_category(category("Apparel", Some(&root)))
        .await
        .expect("apparel")
        .id
        .to_string();
    let shirts = service
        .create_category(category("Shirts", Some(&apparel)))
        .await
        .expect("shirts")
        .id
        .to_string();
    let trousers = service
        .create_category(category("Trousers", Some(&apparel)))
        .await
        .expect("trousers")
        .id
        .to_string();
    let footwear = service
        .create_category(category("Footwear", Some(&root)))
        .await
        .expect("footwear")
        .id
        .to_string();
    (root, apparel, shirts, trousers, footwear)
}

#[tokio::test]
async fn a_root_category_persists_with_its_own_path() {
    let (db, service) = service().await;

    let created = service
        .create_category(category("Root", None))
        .await
        .expect("create");
    let id = created.id.to_string();

    assert!(id.starts_with("pcat_"), "id lost its prefix: {id}");
    assert_eq!(
        created.mpath,
        format!("{id}."),
        "a root category's path is its own id"
    );
    assert!(created.is_active);
    assert_eq!(created.parent_category_id, None);

    let fetched = service.get_category(&id).await.expect("retrieve");
    assert_eq!(fetched.mpath, created.mpath);

    db.drop().await;
}

#[tokio::test]
async fn a_child_extends_its_parents_path() {
    let (db, service) = service().await;
    let (root, apparel, shirts, _, _) = tree(&service).await;

    let shirts_cat = service.get_category(&shirts).await.expect("retrieve");
    assert_eq!(
        shirts_cat.mpath,
        format!("{root}.{apparel}.{shirts}."),
        "the path must spell out the whole ancestry"
    );
    assert_eq!(
        shirts_cat.parent_category_id.as_deref(),
        Some(apparel.as_str())
    );

    db.drop().await;
}

#[tokio::test]
async fn a_child_of_a_missing_parent_is_rejected() {
    let (db, service) = service().await;
    assert!(
        service
            .create_category(category("Orphan", Some("pcat_01ARZ3NDEKTSV4RRFFQ69G5FAV")))
            .await
            .is_err(),
        "a category must not be parented to something that does not exist"
    );
    db.drop().await;
}

#[tokio::test]
async fn descendants_returns_the_whole_subtree_and_excludes_the_target() {
    let (db, service) = service().await;
    let (root, apparel, shirts, trousers, footwear) = tree(&service).await;

    let under_root: Vec<String> = service
        .list_descendants(&root)
        .await
        .expect("descendants")
        .into_iter()
        .map(|c| c.id.to_string())
        .collect();

    assert_eq!(
        under_root.len(),
        4,
        "every descendant of root must be returned"
    );
    for expected in [&apparel, &shirts, &trousers, &footwear] {
        assert!(under_root.contains(expected), "missing {expected}");
    }
    assert!(
        !under_root.contains(&root),
        "a category is not its own descendant"
    );

    db.drop().await;
}

#[tokio::test]
async fn descendants_of_a_branch_excludes_its_siblings() {
    // The assertion that catches a prefix match built from the wrong path.
    let (db, service) = service().await;
    let (_, apparel, shirts, trousers, footwear) = tree(&service).await;

    let under_apparel: Vec<String> = service
        .list_descendants(&apparel)
        .await
        .expect("descendants")
        .into_iter()
        .map(|c| c.id.to_string())
        .collect();

    assert_eq!(under_apparel.len(), 2);
    assert!(under_apparel.contains(&shirts));
    assert!(under_apparel.contains(&trousers));
    assert!(
        !under_apparel.contains(&footwear),
        "Footwear is a sibling of Apparel, not a descendant"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_leaf_has_no_descendants() {
    let (db, service) = service().await;
    let (_, _, shirts, _, _) = tree(&service).await;

    assert!(service
        .list_descendants(&shirts)
        .await
        .expect("descendants")
        .is_empty());

    db.drop().await;
}

#[tokio::test]
async fn the_tree_outlives_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let (root, apparel, shirts, trousers, footwear) = {
        let first = ProductService::new(pool.clone());
        tree(&first).await
    };

    let second = ProductService::new(pool);
    let under_root = second.list_descendants(&root).await.expect("descendants");
    assert_eq!(
        under_root.len(),
        4,
        "the category tree did not survive the service that built it"
    );
    for expected in [apparel, shirts, trousers, footwear] {
        assert!(under_root.iter().any(|c| c.id.to_string() == expected));
    }

    db.drop().await;
}

#[tokio::test]
async fn categories_list_in_path_order() {
    let (db, service) = service().await;
    let (root, _, _, _, _) = tree(&service).await;

    let all = service.list_categories().await.expect("list");
    assert_eq!(all.len(), 5);
    assert_eq!(
        all[0].id.to_string(),
        root,
        "path ordering puts an ancestor before its descendants"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_collection_round_trips() {
    let (db, service) = service().await;

    let created = service
        .create_collection(CreateCollectionInput {
            title: "Summer Sale".to_string(),
            handle: None,
            metadata: None,
        })
        .await
        .expect("create");

    assert_eq!(created.handle, "summer-sale");
    assert!(
        created.id.starts_with("pcol_"),
        "id lost its prefix: {}",
        created.id
    );

    let fetched = service.get_collection(&created.id).await.expect("retrieve");
    assert_eq!(fetched.title, "Summer Sale");

    let rows = db
        .count("SELECT count(*) FROM product.product_collection")
        .await;
    assert_eq!(rows, 1);

    db.drop().await;
}

#[tokio::test]
async fn collection_metadata_survives_the_round_trip() {
    // jsonb through the JSON bridge is the case most likely to be mangled.
    let (db, service) = service().await;

    let created = service
        .create_collection(CreateCollectionInput {
            title: "Featured".to_string(),
            handle: Some("featured".to_string()),
            metadata: Some(serde_json::json!({ "banner": "hero.png", "rank": 3 })),
        })
        .await
        .expect("create");

    let fetched = service.get_collection(&created.id).await.expect("retrieve");
    let metadata = fetched.metadata.expect("metadata must survive");
    assert_eq!(metadata["banner"], "hero.png");
    assert_eq!(metadata["rank"], 3);

    db.drop().await;
}

#[tokio::test]
async fn collections_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    ProductService::new(pool.clone())
        .create_collection(CreateCollectionInput {
            title: "Persistent".to_string(),
            handle: None,
            metadata: None,
        })
        .await
        .expect("create");

    let listed = ProductService::new(pool)
        .list_collections()
        .await
        .expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Persistent");

    db.drop().await;
}

#[tokio::test]
async fn tag_and_type_tables_exist_and_enforce_uniqueness() {
    // Tags and types have no service surface yet, but their tables are part of
    // the migrated schema and their uniqueness is what later phases rely on.
    let db = TestDb::migrated().await;

    for table in ["product_tag", "product_type"] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'product' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "product.{table} is missing");

        db.execute(&format!(
            "INSERT INTO product.{table} (id, value) VALUES ('{table}_1', 'duplicate')"
        ))
        .await;

        let mut conn = db.conn().await;
        let second = {
            use diesel_async::RunQueryDsl;
            diesel::sql_query(format!(
                "INSERT INTO product.{table} (id, value) VALUES ('{table}_2', 'duplicate')"
            ))
            .execute(&mut conn)
            .await
        };
        assert!(
            second.is_err(),
            "product.{table}.value must be unique, but a duplicate was accepted"
        );
    }

    db.drop().await;
}
