use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{CreateCustomerAddressInput, CreateCustomerInput, CustomerService};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::platform::ErasurePipelineWorkflow;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_erasure_cascade_pseudonymization_and_financial_integrity() {
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

    // 1. Create Customer with full PII
    let customer = customer_service
        .create_customer(CreateCustomerInput {
            email: "bob.smith@example.com".to_string(),
            company_name: Some("Smith Enterprises".to_string()),
            first_name: Some("Bob".to_string()),
            last_name: Some("Smith".to_string()),
            phone: Some("+1-555-0199".to_string()),
            has_account: Some(true),
            metadata: None,
        })
        .await
        .unwrap();

    let _addr = customer_service
        .create_customer_address(CreateCustomerAddressInput {
            customer_id: customer.id.clone(),
            company: Some("Smith Enterprises".to_string()),
            first_name: Some("Bob".to_string()),
            last_name: Some("Smith".to_string()),
            address_1: "123 Main Street".to_string(),
            address_2: Some("Apt 4B".to_string()),
            city: "Metropolis".to_string(),
            country_code: "US".to_string(),
            province: Some("NY".to_string()),
            postal_code: "10001".to_string(),
            phone: Some("+1-555-0199".to_string()),
            metadata: None,
            is_default_shipping: Some(true),
            is_default_billing: Some(true),
        })
        .await
        .unwrap();

    // 2. Create Order linked to customer
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some(customer.id.clone()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("bob.smith@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                customer_id: Some(customer.id.clone()),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                address_1: Some("123 Main Street".to_string()),
                address_2: Some("Apt 4B".to_string()),
                city: Some("Metropolis".to_string()),
                country_code: Some("US".to_string()),
                province: Some("NY".to_string()),
                postal_code: Some("10001".to_string()),
                phone: Some("+1-555-0199".to_string()),
                company: None,
                metadata: None,
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Premium Merino Sweater".to_string(),
                quantity: 2,
                unit_price: dec!(120.00),
                variant_id: Some("var_merino".to_string()),
                product_id: Some("prod_sweater".to_string()),
                ..Default::default()
            }]),
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    let initial_order_total = order.summary.as_ref().unwrap().current_order_total;
    assert_eq!(initial_order_total, dec!(240.00));

    // 3. Execute GDPR Erasure Cascade (Task P8-18)
    workflow
        .execute_erasure_cascade(&customer.id)
        .await
        .unwrap();

    // 4. Verify Customer account is deleted
    assert!(customer_service.get_customer(&customer.id).await.is_err());

    // 5. Verify Customer Addresses are purged
    let addresses_after = customer_service
        .list_customer_addresses(&customer.id)
        .await
        .unwrap();
    assert!(addresses_after.is_empty());

    // 6. Verify Order shipping PII is erased while FINANCIAL TOTALS REMAIN 100% INTACT
    let order_after = order_service.get_order(&order.id).await.unwrap();
    assert_eq!(
        order_after.email,
        Some("erased@pseudonym.local".to_string())
    );
    let shipping_addr_after = order_after.shipping_address.unwrap();
    assert_eq!(shipping_addr_after.first_name, Some("[ERASED]".to_string()));
    assert_eq!(shipping_addr_after.address_1, Some("[ERASED]".to_string()));

    // Financial integrity check (Must match exactly to the cent)
    assert_eq!(
        order_after.summary.unwrap().current_order_total,
        dec!(240.00)
    );

    db.drop().await;
}
