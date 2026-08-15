use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{CreateCustomerAddressInput, CreateCustomerInput, CustomerService};
use stheno_flows::platform::ErasurePipelineWorkflow;
use stheno_order::OrderService;

#[tokio::test]
async fn test_customer_data_export_archive() {
    let customer_service = Arc::new(CustomerService::new());
    let auth_service = Arc::new(AuthService::new(None));
    let order_service = Arc::new(OrderService::new());

    let workflow = ErasurePipelineWorkflow::new(
        customer_service.clone(),
        auth_service.clone(),
        order_service.clone(),
    );

    // 1. Create customer and address
    let customer = customer_service
        .create_customer(CreateCustomerInput {
            email: "daniela@example.com".to_string(),
            company_name: Some("Daniela Design Studio".to_string()),
            first_name: Some("Daniela".to_string()),
            last_name: Some("Reyes".to_string()),
            phone: Some("+1-555-4321".to_string()),
            has_account: Some(true),
            metadata: None,
        })
        .await
        .unwrap();

    let _addr = customer_service
        .create_customer_address(CreateCustomerAddressInput {
            customer_id: customer.id.clone(),
            company: Some("Daniela Design Studio".to_string()),
            first_name: Some("Daniela".to_string()),
            last_name: Some("Reyes".to_string()),
            address_1: "550 Broadway".to_string(),
            address_2: Some("Suite 400".to_string()),
            city: "New York".to_string(),
            country_code: "US".to_string(),
            province: Some("NY".to_string()),
            postal_code: "10012".to_string(),
            phone: Some("+1-555-4321".to_string()),
            metadata: None,
            is_default_shipping: Some(true),
            is_default_billing: Some(true),
        })
        .await
        .unwrap();

    // 2. Export customer data package (Task P8-21)
    let export = workflow.export_customer_data(&customer.id).await.unwrap();

    assert_eq!(export.customer.id, customer.id);
    assert_eq!(export.customer.email, "daniela@example.com");
    assert_eq!(export.addresses.len(), 1);
    assert_eq!(export.addresses[0].address_1, "550 Broadway");
}
