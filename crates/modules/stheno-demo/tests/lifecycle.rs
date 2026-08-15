use stheno_demo::{CreateDemoInput, DemoService};

#[tokio::test]
async fn test_demo_service_lifecycle() {
    let service = DemoService::new();
    let created = service
        .create(CreateDemoInput {
            name: "Test Entity".to_string(),
        })
        .await
        .expect("must create item");

    assert_eq!(created.name, "Test Entity");
    let retrieved = service
        .get(&created.id.to_string())
        .await
        .expect("must retrieve item");
    assert_eq!(retrieved.name, "Test Entity");
}
