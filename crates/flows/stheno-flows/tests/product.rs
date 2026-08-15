use serde_json::json;
use std::sync::Arc;
use stheno_flows::build_create_product_saga;
use stheno_product::ProductService;

#[tokio::test]
async fn test_create_product_workflow_saga_execution() {
    let service = Arc::new(ProductService::new());
    let saga = build_create_product_saga(service);

    let result = saga
        .execute(
            "tx_prod_flow_123",
            "create-product-flow",
            json!({
                "title": "Minimalist Ceramic Mug",
                "handle": "minimalist-ceramic-mug",
                "subtitle": "Matte finish",
                "discountable": true
            }),
        )
        .await
        .expect("must execute product creation workflow");

    assert_eq!(result["title"], "Minimalist Ceramic Mug");
    assert_eq!(result["handle"], "minimalist-ceramic-mug");
}
