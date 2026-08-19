use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService, UpdateProductInput};

#[tokio::test]
async fn test_product_service_lifecycle_and_domain_methods() {
    let db = TestDb::migrated().await;
    let service = ProductService::new(create_pool(&db.url, 5).expect("pool"));

    // 1. Create Product
    let product = service
        .create_product(CreateProductInput {
            title: "Performance Tee".to_string(),
            handle: Some("performance-tee".to_string()),
            subtitle: Some("Breathable fabric".to_string()),
            description: Some("Ideal for running".to_string()),
            discountable: Some(true),
        })
        .await
        .expect("must create product");

    let prod_id = product.id.to_string();
    assert_eq!(product.title, "Performance Tee");
    assert_eq!(product.handle, "performance-tee");

    // 2. Get Product
    let retrieved = service
        .get_product(&prod_id)
        .await
        .expect("must get product");
    assert_eq!(retrieved.title, "Performance Tee");

    // 3. Update Product
    let updated = service
        .update_product(
            &prod_id,
            UpdateProductInput {
                title: Some("Ultra Performance Tee".to_string()),
                status: Some("published".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("must update product");
    assert_eq!(updated.title, "Ultra Performance Tee");
    assert_eq!(updated.status, "published");

    // 4. Create Variants
    let size = option_values(&service, &prod_id, "Size", &["M"]).await;
    let colour = option_values(&service, &prod_id, "Colour", &["Black"]).await;
    let v1 = service
        .create_variant(CreateVariantInput {
            product_id: prod_id.clone(),
            title: "Medium / Black".to_string(),
            sku: Some("TEE-M-BLK".to_string()),
            barcode: None,
            option_value_ids: vec![size[0].clone(), colour[0].clone()],
        })
        .await
        .expect("must create variant 1");
    assert_eq!(v1.sku.as_deref(), Some("TEE-M-BLK"));

    // 5. List Products
    let list = service.list_products().await.expect("must list products");
    assert_eq!(list.len(), 1);

    // 6. Delete Product
    service
        .delete_product(&prod_id)
        .await
        .expect("must delete product");
    let get_deleted = service.get_product(&prod_id).await;
    assert!(
        get_deleted.is_err(),
        "Deleted product must not be retrievable"
    );

    db.drop().await;
}

/// Creates real option values on a product and returns their ids.
///
/// These suites used to pass synthetic ids like `"optval_small"`. Variant option values are
/// stored as rows now, with a foreign key, so an id that resolves to nothing is rejected —
/// which is the point: a variant referencing an option value that does not exist is one no
/// shopper can select.
async fn option_values(
    service: &ProductService,
    product_id: &str,
    option_title: &str,
    values: &[&str],
) -> Vec<String> {
    let option_id = service
        .create_option(product_id, option_title)
        .await
        .expect("create option");
    let mut ids = Vec::new();
    for value in values {
        ids.push(
            service
                .create_option_value(&option_id, value)
                .await
                .expect("create option value"),
        );
    }
    ids
}
