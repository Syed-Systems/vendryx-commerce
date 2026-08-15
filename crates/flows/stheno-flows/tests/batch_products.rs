use std::sync::Arc;
use stheno_flows::BatchProductService;
use stheno_product::{CreateProductInput, ProductService};

#[tokio::test]
async fn test_batch_product_ops_1000_items_with_per_row_errors() {
    let service = Arc::new(ProductService::new());
    let batch_service = BatchProductService::new(service);

    let mut batch = Vec::with_capacity(1000);
    for i in 0..1000 {
        if i % 100 == 0 {
            // Plant invalid row (empty title)
            batch.push(CreateProductInput {
                title: "  ".to_string(),
                handle: None,
                subtitle: None,
                description: None,
                discountable: None,
            });
        } else {
            batch.push(CreateProductInput {
                title: format!("Batch Product #{}", i),
                handle: Some(format!("batch-prod-{}", i)),
                subtitle: None,
                description: None,
                discountable: Some(true),
            });
        }
    }

    let result = batch_service
        .batch_create_products(batch)
        .await
        .expect("must execute batch creation");

    assert_eq!(result.total, 1000);
    assert_eq!(result.failed, 10); // Exactly 10 invalid rows
    assert_eq!(result.succeeded, 990); // Exactly 990 succeeded
    assert_eq!(result.errors.len(), 10);
    assert_eq!(result.errors[0].index, 0);
}
