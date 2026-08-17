use stheno_db::filter::FilterSpec;
use stheno_db::service::{CrudEntity, InMemoryCrudStore};
use stheno_db::stheno_service_crud;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct TestProduct {
    id: String,
    title: String,
    deleted: bool,
}

impl CrudEntity for TestProduct {
    const TABLE: &'static str = "product.product";
    const ENTITY: &'static str = "product";

    fn id(&self) -> String {
        self.id.clone()
    }

    fn is_deleted(&self) -> bool {
        self.deleted
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }
}

struct TestProductService {
    repo: InMemoryCrudStore<TestProduct>,
}

impl TestProductService {
    pub fn new() -> Self {
        Self {
            repo: InMemoryCrudStore::new("test_product"),
        }
    }
}

// Generate all standard 10 CRUD operations via macro
stheno_service_crud!(TestProductService, TestProduct, repo);

#[tokio::test]
async fn test_stheno_service_derive_all_10_crud_operations() {
    let service = TestProductService::new();

    // 1. create & 2. retrieve
    let item1 = TestProduct {
        id: "prod_001".to_string(),
        title: "T-Shirt".to_string(),
        deleted: false,
    };
    let created = service
        .create(None, item1.clone())
        .await
        .expect("create item1");
    assert_eq!(created.id, "prod_001");

    let retrieved = service
        .retrieve(None, "prod_001")
        .await
        .expect("retrieve item1");
    assert_eq!(retrieved.title, "T-Shirt");

    // 3. create_many
    let item2 = TestProduct {
        id: "prod_002".to_string(),
        title: "Jeans".to_string(),
        deleted: false,
    };
    let item3 = TestProduct {
        id: "prod_003".to_string(),
        title: "Jacket".to_string(),
        deleted: false,
    };
    let created_many = service
        .create_many(None, vec![item2, item3])
        .await
        .expect("create_many");
    assert_eq!(created_many.len(), 2);

    // 4. list & 5. list_and_count
    let filter = FilterSpec::default();
    let list = service.list(None, &filter).await.expect("list");
    assert_eq!(list.len(), 3);

    let (list_items, count) = service
        .list_and_count(None, &filter)
        .await
        .expect("list_and_count");
    assert_eq!(list_items.len(), 3);
    assert_eq!(count, 3);

    // 6. update
    let mut updated_item = retrieved;
    updated_item.title = "Organic T-Shirt".to_string();
    let updated = service
        .update(None, "prod_001", updated_item)
        .await
        .expect("update");
    assert_eq!(updated.title, "Organic T-Shirt");

    // 7. upsert
    let item4 = TestProduct {
        id: "prod_004".to_string(),
        title: "Cap".to_string(),
        deleted: false,
    };
    let upserted = service.upsert(None, item4).await.expect("upsert new");
    assert_eq!(upserted.title, "Cap");

    // 8. soft_delete
    service
        .soft_delete(None, "prod_004")
        .await
        .expect("soft_delete");
    let retrieve_deleted = service.retrieve(None, "prod_004").await;
    assert!(
        retrieve_deleted.is_err(),
        "Soft-deleted item should not be retrievable"
    );

    // 9. restore
    service.restore(None, "prod_004").await.expect("restore");
    let restored = service
        .retrieve(None, "prod_004")
        .await
        .expect("retrieve restored");
    assert_eq!(restored.title, "Cap");

    // 10. delete (hard delete)
    service.delete(None, "prod_004").await.expect("hard delete");
    let retrieve_hard_deleted = service.retrieve(None, "prod_004").await;
    assert!(retrieve_hard_deleted.is_err());
}
