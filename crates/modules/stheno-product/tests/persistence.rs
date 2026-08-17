//! P11-19: `product` on Postgres — products, variants, options, option values.
//!
//! `product` is the nastiest shape in the catalog, which is why P11 migrates it
//! as a reference rather than leaving it to P12: variant identity, option
//! values, and soft delete all have to survive a restart together.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService, UpdateProductInput};

async fn service() -> (TestDb, ProductService) {
    let db = TestDb::migrated().await;
    let svc = ProductService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

fn product(title: &str) -> CreateProductInput {
    CreateProductInput {
        title: title.to_string(),
        handle: None,
        subtitle: None,
        description: None,
        discountable: None,
    }
}

#[tokio::test]
async fn a_product_round_trips_through_the_database() {
    let (db, service) = service().await;

    let created = service
        .create_product(product("Desk Lamp"))
        .await
        .expect("create");
    assert_eq!(created.title, "Desk Lamp");
    assert_eq!(created.handle, "desk-lamp");

    let id = created.id.to_string();
    assert!(id.starts_with("prod_"), "id lost its prefix: {id}");

    let fetched = service.get_product(&id).await.expect("retrieve");
    assert_eq!(fetched.title, "Desk Lamp");

    let rows = db.count("SELECT count(*) FROM product.product").await;
    assert_eq!(rows, 1, "create did not write a row");

    db.drop().await;
}

#[tokio::test]
async fn products_outlive_the_service_instance() {
    // The assertion P11 exists to make.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let id = ProductService::new(pool.clone())
        .create_product(product("Persistent"))
        .await
        .expect("create")
        .id
        .to_string();

    let second = ProductService::new(pool);
    let found = second
        .get_product(&id)
        .await
        .expect("the product must survive the service that created it");
    assert_eq!(found.title, "Persistent");

    db.drop().await;
}

#[tokio::test]
async fn update_persists_and_rejects_an_unknown_status() {
    let (db, service) = service().await;
    let id = service
        .create_product(product("Old"))
        .await
        .expect("create")
        .id
        .to_string();

    let updated = service
        .update_product(
            &id,
            UpdateProductInput {
                title: Some("New".to_string()),
                status: Some("published".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("update");
    assert_eq!(updated.title, "New");
    assert_eq!(updated.status, "published");

    // A published product must not read back as a draft.
    let refetched = service.get_product(&id).await.expect("refetch");
    assert_eq!(refetched.status, "published");

    let err = service
        .update_product(
            &id,
            UpdateProductInput {
                status: Some("not-a-status".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect_err("an unknown status must be rejected");
    assert!(err.to_string().contains("not-a-status"), "got: {err}");

    db.drop().await;
}

#[tokio::test]
async fn a_deleted_product_is_hidden_but_retained() {
    let (db, service) = service().await;
    let id = service
        .create_product(product("Doomed"))
        .await
        .expect("create")
        .id
        .to_string();

    service.delete_product(&id).await.expect("delete");

    assert!(service.get_product(&id).await.is_err());
    assert!(service.list_products().await.expect("list").is_empty());

    let rows = db.count("SELECT count(*) FROM product.product").await;
    assert_eq!(rows, 1, "delete_product must be a soft delete");

    db.drop().await;
}

#[tokio::test]
async fn options_and_option_values_persist() {
    let (db, service) = service().await;
    let product_id = service
        .create_product(product("Tee"))
        .await
        .expect("create")
        .id
        .to_string();

    let size = service
        .create_option(&product_id, "Size")
        .await
        .expect("create option");
    let small = service
        .create_option_value(&size, "Small")
        .await
        .expect("create value");
    let large = service
        .create_option_value(&size, "Large")
        .await
        .expect("create value");
    assert_ne!(small, large);

    let options = service
        .list_options(&product_id)
        .await
        .expect("list options");
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].1, "Size");

    let values = service
        .list_option_values(&size)
        .await
        .expect("list values");
    assert_eq!(values.len(), 2, "both option values must persist");

    db.drop().await;
}

#[tokio::test]
async fn an_option_on_a_missing_product_is_rejected() {
    let (db, service) = service().await;
    assert!(
        service
            .create_option("prod_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Size")
            .await
            .is_err(),
        "an option must not be orphaned on a product that does not exist"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_variant_persists_with_its_option_hash() {
    let (db, service) = service().await;
    let product_id = service
        .create_product(product("Tee"))
        .await
        .expect("create")
        .id
        .to_string();

    let created = service
        .create_variant(CreateVariantInput {
            product_id: product_id.clone(),
            title: "Small / Red".to_string(),
            sku: Some("TEE-S-RED".to_string()),
            barcode: None,
            option_value_ids: vec!["optval_small".into(), "optval_red".into()],
        })
        .await
        .expect("create variant");

    assert!(
        !created.option_values_hash.is_empty(),
        "variant identity was not computed"
    );

    // The hash is a stored column now, so reading the variant back recovers it
    // rather than scanning a side map.
    let fetched = service
        .get_variant(&created.id.to_string())
        .await
        .expect("retrieve variant");
    assert_eq!(fetched.option_values_hash, created.option_values_hash);
    assert_eq!(fetched.sku.as_deref(), Some("TEE-S-RED"));

    db.drop().await;
}

#[tokio::test]
async fn variant_identity_is_order_independent() {
    let (db, service) = service().await;
    let product_id = service
        .create_product(product("Tee"))
        .await
        .expect("create")
        .id
        .to_string();

    let a = service
        .create_variant(CreateVariantInput {
            product_id: product_id.clone(),
            title: "A".to_string(),
            sku: None,
            barcode: None,
            option_value_ids: vec!["optval_red".into(), "optval_small".into()],
        })
        .await
        .expect("create");

    // Same set, different order: the same variant, so the second is rejected.
    let err = service
        .create_variant(CreateVariantInput {
            product_id,
            title: "B".to_string(),
            sku: None,
            barcode: None,
            option_value_ids: vec!["optval_small".into(), "optval_red".into()],
        })
        .await
        .expect_err("the same option set in a different order is the same variant");

    assert!(
        err.to_string().contains("already exists"),
        "expected a duplicate-variant conflict, got: {err}"
    );
    assert!(!a.option_values_hash.is_empty());

    db.drop().await;
}

#[tokio::test]
async fn the_same_option_set_on_a_different_product_is_allowed() {
    // Identity is per product, not global: two products may each have "Small".
    let (db, service) = service().await;
    let first = service
        .create_product(product("Tee"))
        .await
        .expect("create")
        .id
        .to_string();
    let second = service
        .create_product(product("Hoodie"))
        .await
        .expect("create")
        .id
        .to_string();

    for product_id in [first, second] {
        service
            .create_variant(CreateVariantInput {
                product_id,
                title: "Small".to_string(),
                sku: None,
                barcode: None,
                option_value_ids: vec!["optval_small".into()],
            })
            .await
            .expect("each product may have its own Small variant");
    }

    let rows = db
        .count("SELECT count(*) FROM product.product_variant")
        .await;
    assert_eq!(rows, 2);

    db.drop().await;
}

#[tokio::test]
async fn duplicate_variants_are_rejected_by_the_database_not_a_read_check() {
    // Proves the unique index exists. A read-then-write check in the service
    // would let two concurrent writers both observe "absent" and both insert.
    let db = TestDb::migrated().await;
    let indexed = db
        .count(
            "SELECT count(*) FROM pg_indexes \
             WHERE schemaname = 'product' AND indexname = 'idx_variant_product_option_hash'",
        )
        .await;
    assert_eq!(
        indexed, 1,
        "the variant option-hash unique index is missing"
    );
    db.drop().await;
}

#[tokio::test]
async fn variants_are_listed_per_product() {
    let (db, service) = service().await;
    let product_id = service
        .create_product(product("Tee"))
        .await
        .expect("create")
        .id
        .to_string();

    for value in ["small", "medium", "large"] {
        service
            .create_variant(CreateVariantInput {
                product_id: product_id.clone(),
                title: value.to_string(),
                sku: None,
                barcode: None,
                option_value_ids: vec![format!("optval_{value}")],
            })
            .await
            .expect("create variant");
    }

    let listed = service.list_variants(&product_id).await.expect("list");
    assert_eq!(listed.len(), 3);

    db.drop().await;
}
