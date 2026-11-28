//! ADR-032 phase 32.2: AI catalog search only returns bounded live published products.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateProductInput, ProductService, UpdateProductInput};
use stheno_types::Error;

#[tokio::test]
async fn search_is_bounded_literal_and_excludes_unpublished_or_deleted_products() {
    let db = TestDb::migrated().await;
    let service = ProductService::new(create_pool(&db.url, 5).expect("pool"));
    let published = service
        .create_product(CreateProductInput {
            title: "Desk Lamp".to_string(),
            handle: Some("desk-lamp".to_string()),
            subtitle: None,
            description: Some("Adjustable reading light".to_string()),
            discountable: None,
        })
        .await
        .expect("create published product");
    service
        .update_product(
            &published.id.to_string(),
            UpdateProductInput {
                status: Some("published".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("publish product");

    let hidden = service
        .create_product(CreateProductInput {
            title: "Desk Lamp Internal".to_string(),
            handle: Some("desk-lamp-internal".to_string()),
            subtitle: None,
            description: None,
            discountable: None,
        })
        .await
        .expect("create draft product");
    let deleted = service
        .create_product(CreateProductInput {
            title: "Desk Lamp Deleted".to_string(),
            handle: Some("desk-lamp-deleted".to_string()),
            subtitle: None,
            description: None,
            discountable: None,
        })
        .await
        .expect("create product to delete");
    service
        .update_product(
            &deleted.id.to_string(),
            UpdateProductInput {
                status: Some("published".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("publish product to delete");
    service
        .delete_product(&deleted.id.to_string())
        .await
        .expect("soft-delete product");

    let results = service
        .search_published_products("Desk Lamp")
        .await
        .expect("search published catalog");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id.to_string(), published.id.to_string());
    assert!(service
        .search_published_products("   ")
        .await
        .expect("empty query returns no results")
        .is_empty());
    assert!(matches!(
        service.search_published_products(&"x".repeat(129)).await,
        Err(Error::Invalid {
            code: "product_search_query_too_long",
            ..
        })
    ));
    assert!(matches!(
        service.search_published_products("Desk_%").await,
        Err(Error::Invalid {
            code: "product_search_query_invalid",
            ..
        })
    ));
    assert_eq!(hidden.status, "draft");

    db.drop().await;
}
