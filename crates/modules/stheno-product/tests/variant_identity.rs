use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService};

#[tokio::test]
async fn test_variant_identity_unique_hash_enforcement() {
    let db = TestDb::migrated().await;
    let service = ProductService::new(create_pool(&db.url, 5).expect("pool"));
    let product = service
        .create_product(CreateProductInput {
            title: "Sneakers".to_string(),
            handle: None,
            subtitle: None,
            description: None,
            discountable: None,
        })
        .await
        .unwrap();

    let prod_id = product.id.to_string();

    // Create variant with options ["opt_color_red", "opt_size_42"]
    let v1 = service
        .create_variant(CreateVariantInput {
            product_id: prod_id.clone(),
            title: "Red / 42".to_string(),
            sku: Some("SNK-RED-42".to_string()),
            barcode: None,
            option_value_ids: vec!["opt_color_red".to_string(), "opt_size_42".to_string()],
        })
        .await;

    assert!(v1.is_ok());

    // Creating duplicate variant with same options (even in different permutation order) fails
    let v2_duplicate = service
        .create_variant(CreateVariantInput {
            product_id: prod_id.clone(),
            title: "42 / Red".to_string(),
            sku: Some("SNK-42-RED".to_string()),
            barcode: None,
            option_value_ids: vec!["opt_size_42".to_string(), "opt_color_red".to_string()],
        })
        .await;

    assert!(v2_duplicate.is_err());
    let err_str = v2_duplicate.unwrap_err().to_string();
    assert!(err_str.contains("variant_option_combination_exists"));

    db.drop().await;
}
