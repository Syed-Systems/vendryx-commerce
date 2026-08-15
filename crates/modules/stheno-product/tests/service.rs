use stheno_product::{CreateProductInput, CreateVariantInput, ProductService, UpdateProductInput};

#[tokio::test]
async fn test_product_service_lifecycle_and_domain_methods() {
    let service = ProductService::new();

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
    let v1 = service
        .create_variant(CreateVariantInput {
            product_id: prod_id.clone(),
            title: "Medium / Black".to_string(),
            sku: Some("TEE-M-BLK".to_string()),
            barcode: None,
            option_value_ids: vec!["optval_m".to_string(), "optval_blk".to_string()],
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
}
