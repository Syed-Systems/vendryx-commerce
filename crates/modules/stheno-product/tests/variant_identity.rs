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

    let colour = option_values(&service, &prod_id, "Colour", &["Red"]).await;
    let size = option_values(&service, &prod_id, "Size", &["42"]).await;

    // Create variant with options [Red, 42]
    let v1 = service
        .create_variant(CreateVariantInput {
            product_id: prod_id.clone(),
            title: "Red / 42".to_string(),
            sku: Some("SNK-RED-42".to_string()),
            barcode: None,
            option_value_ids: vec![colour[0].clone(), size[0].clone()],
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
            option_value_ids: vec![size[0].clone(), colour[0].clone()],
        })
        .await;

    assert!(v2_duplicate.is_err());
    let err_str = v2_duplicate.unwrap_err().to_string();
    assert!(err_str.contains("variant_option_combination_exists"));

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
