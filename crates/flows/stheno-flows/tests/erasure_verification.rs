use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{CreateCustomerAddressInput, CreateCustomerInput, CustomerService};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::platform::ErasurePipelineWorkflow;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_erasure_verification_no_pii_and_financials_preserved() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let customer_service = Arc::new(CustomerService::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let order_service = Arc::new(OrderService::new(pool));

    let workflow = ErasurePipelineWorkflow::new(
        customer_service.clone(),
        auth_service.clone(),
        order_service.clone(),
    );

    let customer = customer_service
        .create_customer(CreateCustomerInput {
            email: "charlie.v@example.com".to_string(),
            company_name: None,
            first_name: Some("Charlie".to_string()),
            last_name: Some("Vance".to_string()),
            phone: Some("+1-555-0987".to_string()),
            has_account: Some(true),
            metadata: None,
        })
        .await
        .unwrap();

    let _addr = customer_service
        .create_customer_address(CreateCustomerAddressInput {
            customer_id: customer.id.clone(),
            company: None,
            first_name: Some("Charlie".to_string()),
            last_name: Some("Vance".to_string()),
            address_1: "789 Park Ave".to_string(),
            address_2: None,
            city: "Gotham".to_string(),
            country_code: "US".to_string(),
            province: Some("NY".to_string()),
            postal_code: "10021".to_string(),
            phone: Some("+1-555-0987".to_string()),
            metadata: None,
            is_default_shipping: Some(true),
            is_default_billing: Some(true),
        })
        .await
        .unwrap();

    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some(customer.id.clone()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("charlie.v@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                customer_id: Some(customer.id.clone()),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Vance".to_string()),
                address_1: Some("789 Park Ave".to_string()),
                address_2: None,
                city: Some("Gotham".to_string()),
                country_code: Some("US".to_string()),
                province: Some("NY".to_string()),
                postal_code: Some("10021".to_string()),
                phone: Some("+1-555-0987".to_string()),
                company: None,
                metadata: None,
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Leather Portfolio".to_string(),
                quantity: 1,
                unit_price: dec!(150.00),
                variant_id: Some("var_port".to_string()),
                product_id: Some("prod_portfolio".to_string()),
                ..Default::default()
            }]),
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // Execute cascade
    workflow
        .execute_erasure_cascade(&customer.id)
        .await
        .unwrap();

    // Verify Cleanliness (Task P8-19)
    let is_clean = workflow
        .verify_erasure_cleanliness(&customer.id, std::slice::from_ref(&order.id))
        .await
        .unwrap();

    assert!(
        is_clean,
        "Erasure verification failed: PII detected in store"
    );

    db.drop().await;
}
