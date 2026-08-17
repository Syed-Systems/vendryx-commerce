use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateCategoryInput, ProductService};

#[tokio::test]
async fn test_category_tree_materialized_path_descendants() {
    let db = TestDb::migrated().await;
    let service = ProductService::new(create_pool(&db.url, 5).expect("pool"));

    // 1. Root category: Apparel
    let apparel = service
        .create_category(CreateCategoryInput {
            name: "Apparel".to_string(),
            handle: Some("apparel".to_string()),
            parent_category_id: None,
            rank: Some(0),
        })
        .await
        .expect("must create root category");

    // 2. Child category: Menswear
    let menswear = service
        .create_category(CreateCategoryInput {
            name: "Menswear".to_string(),
            handle: Some("menswear".to_string()),
            parent_category_id: Some(apparel.id.to_string()),
            rank: Some(0),
        })
        .await
        .expect("must create child category");

    // 3. Grandchild category: Shoes
    let _shoes = service
        .create_category(CreateCategoryInput {
            name: "Shoes".to_string(),
            handle: Some("shoes".to_string()),
            parent_category_id: Some(menswear.id.to_string()),
            rank: Some(0),
        })
        .await
        .expect("must create grandchild category");

    // Query descendants of root Apparel -> should return Menswear and Shoes
    let descendants = service
        .list_descendants(&apparel.id.to_string())
        .await
        .expect("must list descendants");

    assert_eq!(descendants.len(), 2);
    let names: Vec<String> = descendants.into_iter().map(|d| d.name).collect();
    assert!(names.contains(&"Menswear".to_string()));
    assert!(names.contains(&"Shoes".to_string()));

    db.drop().await;
}
