//! Test P7-14, P14-01: Customer domain, groups, addresses, and guest-to-registered upgrade on Postgres.

use stheno_customer::{
    CreateCustomerAddressInput, CreateCustomerGroupInput, CreateCustomerInput, CustomerService,
    GuestUpgradeInput, UpdateCustomerInput,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_customer_lifecycle_groups_and_guest_upgrade() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = CustomerService::new(pool);

    // 1. Create a guest customer (during guest checkout)
    let guest = service
        .create_customer(CreateCustomerInput {
            email: "guest_checkout@example.com".to_string(),
            company_name: None,
            first_name: Some("Guest".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("+1234567890".to_string()),
            has_account: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(guest.email, "guest_checkout@example.com");
    assert!(!guest.has_account);
    let guest_id = guest.id.clone();

    // 2. Add customer address
    let address = service
        .create_customer_address(CreateCustomerAddressInput {
            customer_id: guest_id.clone(),
            company: Some("Acme Corp".to_string()),
            first_name: Some("Guest".to_string()),
            last_name: Some("User".to_string()),
            address_1: "123 Main Street".to_string(),
            address_2: None,
            city: "New York".to_string(),
            country_code: "US".to_string(),
            province: Some("NY".to_string()),
            postal_code: "10001".to_string(),
            phone: Some("+1234567890".to_string()),
            is_default_shipping: Some(true),
            is_default_billing: Some(true),
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(address.city, "New York");
    assert!(address.is_default_shipping);

    // Verify customer record has default address references populated
    let updated_guest = service.get_customer(&guest_id).await.unwrap();
    assert_eq!(
        updated_guest.default_shipping_address_id,
        Some(address.id.clone())
    );
    assert_eq!(updated_guest.addresses.len(), 1);

    // 3. Customer Groups (VIP, Wholesale)
    let _vip_group = service
        .create_customer_group(CreateCustomerGroupInput {
            name: "VIP Members".to_string(),
            metadata: None,
        })
        .await
        .unwrap();

    // 4. Guest to Registered Upgrade (P7-14)
    let upgraded = service
        .upgrade_guest_to_registered(GuestUpgradeInput {
            guest_customer_id: guest_id.clone(),
            registered_email: "guest_checkout@example.com".to_string(),
            first_name: Some("Registered".to_string()),
            last_name: Some("Customer".to_string()),
            phone: Some("+1987654321".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(upgraded.id, guest_id); // PRESERVES ID!
    assert!(upgraded.has_account);
    assert_eq!(upgraded.first_name, Some("Registered".to_string()));
    assert_eq!(upgraded.addresses.len(), 1);

    // 5. Update and List
    let final_customer = service
        .update_customer(
            &guest_id,
            UpdateCustomerInput {
                email: None,
                company_name: Some("Registered Inc.".to_string()),
                first_name: None,
                last_name: None,
                phone: None,
                has_account: None,
                default_billing_address_id: None,
                default_shipping_address_id: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        final_customer.company_name,
        Some("Registered Inc.".to_string())
    );

    let list = service.list_customers().await.unwrap();
    assert_eq!(list.len(), 1);

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let cust_id = {
        let first = CustomerService::new(pool.clone());
        let c = first
            .create_customer(CreateCustomerInput {
                email: "persistent_customer@example.com".to_string(),
                company_name: None,
                first_name: Some("Persist".to_string()),
                last_name: Some("Test".to_string()),
                phone: None,
                has_account: Some(true),
                metadata: None,
            })
            .await
            .expect("create");
        c.id
    };

    let second = CustomerService::new(pool);
    let found = second.get_customer(&cust_id).await.expect("found");
    assert_eq!(found.email, "persistent_customer@example.com");

    db.drop().await;
}
