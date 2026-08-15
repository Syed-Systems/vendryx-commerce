use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelService};

#[tokio::test]
async fn test_sales_channel_creation_and_retrieval() {
    let service = SalesChannelService::new();

    let sc = service
        .create_channel(CreateSalesChannelInput {
            name: "Default Web Store".to_string(),
            description: Some("Primary B2C online shop".to_string()),
            is_disabled: Some(false),
        })
        .await
        .expect("must create sales channel");

    assert_eq!(sc.name, "Default Web Store");
    assert!(!sc.is_disabled);

    let retrieved = service
        .get_channel(&sc.id.to_string())
        .await
        .expect("must retrieve sales channel");

    assert_eq!(retrieved.name, "Default Web Store");
}
