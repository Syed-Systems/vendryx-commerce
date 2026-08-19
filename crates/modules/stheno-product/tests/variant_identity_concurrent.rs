use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService};

#[tokio::test]
async fn test_variant_identity_concurrent_creation_no_duplicates() {
    let db = TestDb::migrated().await;
    let service = Arc::new(ProductService::new(create_pool(&db.url, 10).expect("pool")));
    let product = service
        .create_product(CreateProductInput {
            title: "Running Shoe".to_string(),
            handle: None,
            subtitle: None,
            description: None,
            discountable: None,
        })
        .await
        .unwrap();

    let prod_id = product.id.to_string();
    let colour = option_values(&service, &prod_id, "Colour", &["Blue"]).await;
    let size = option_values(&service, &prod_id, "Size", &["43"]).await;
    let success_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for i in 0..20 {
        let s = service.clone();
        let pid = prod_id.clone();
        let sc = success_count.clone();
        let opts = vec![colour[0].clone(), size[0].clone()];
        handles.push(tokio::spawn(async move {
            let res = s
                .create_variant(CreateVariantInput {
                    product_id: pid,
                    title: format!("Blue / 43 - Task {}", i),
                    sku: Some(format!("RUN-BLU-43-{}", i)),
                    barcode: None,
                    option_value_ids: opts,
                })
                .await;

            if res.is_ok() {
                sc.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    // Verified: Exactly 1 concurrent creation succeeds, preventing duplicate variant option identity
    assert_eq!(success_count.load(Ordering::SeqCst), 1);

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
