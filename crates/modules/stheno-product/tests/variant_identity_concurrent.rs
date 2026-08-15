use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_product::{CreateProductInput, CreateVariantInput, ProductService};

#[tokio::test]
async fn test_variant_identity_concurrent_creation_no_duplicates() {
    let service = Arc::new(ProductService::new());
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
    let success_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for i in 0..20 {
        let s = service.clone();
        let pid = prod_id.clone();
        let sc = success_count.clone();
        handles.push(tokio::spawn(async move {
            let res = s
                .create_variant(CreateVariantInput {
                    product_id: pid,
                    title: format!("Blue / 43 - Task {}", i),
                    sku: Some(format!("RUN-BLU-43-{}", i)),
                    barcode: None,
                    option_value_ids: vec!["opt_color_blue".to_string(), "opt_size_43".to_string()],
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
}
