//! Customer workflows (WORKFLOWS §2, TASKS P7-21).

use std::sync::Arc;
use stheno_customer::*;
use stheno_types::Result;

pub struct CustomerWorkflows {
    customer_service: Arc<CustomerService>,
}

impl CustomerWorkflows {
    pub fn new(customer_service: Arc<CustomerService>) -> Self {
        Self { customer_service }
    }

    // 1. create_customer_workflow
    pub async fn create_customer(&self, input: CreateCustomerInput) -> Result<CustomerDTO> {
        self.customer_service.create_customer(input).await
    }

    // 2. update_customer_workflow
    pub async fn update_customer(
        &self,
        id: &str,
        input: UpdateCustomerInput,
    ) -> Result<CustomerDTO> {
        self.customer_service.update_customer(id, input).await
    }

    // 3. delete_customer_workflow
    pub async fn delete_customer(&self, id: &str) -> Result<CustomerDTO> {
        self.customer_service.delete_customer(id).await
    }

    // 4. get_customer_workflow
    pub async fn get_customer(&self, id: &str) -> Result<CustomerDTO> {
        self.customer_service.get_customer(id).await
    }

    // 5. list_customers_workflow
    pub async fn list_customers(&self) -> Result<Vec<CustomerDTO>> {
        self.customer_service.list_customers().await
    }

    // 6. create_customer_address_workflow
    pub async fn create_customer_address(
        &self,
        input: CreateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        self.customer_service.create_customer_address(input).await
    }

    // 7. update_customer_address_workflow
    pub async fn update_customer_address(
        &self,
        id: &str,
        input: UpdateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        self.customer_service
            .update_customer_address(id, input)
            .await
    }

    // 8. create_customer_group_workflow
    pub async fn create_customer_group(
        &self,
        input: CreateCustomerGroupInput,
    ) -> Result<CustomerGroupDTO> {
        self.customer_service.create_customer_group(input).await
    }

    // 9. update_customer_group_workflow
    pub async fn update_customer_group(
        &self,
        id: &str,
        input: UpdateCustomerGroupInput,
    ) -> Result<CustomerGroupDTO> {
        self.customer_service.update_customer_group(id, input).await
    }

    // 10. link_customer_to_group_workflow
    pub async fn link_customer_to_group(&self, customer_id: &str, group_id: &str) -> Result<()> {
        self.customer_service
            .add_customer_to_group(customer_id, group_id)
            .await
    }

    // 11. unlink_customer_from_group_workflow
    pub async fn unlink_customer_from_group(
        &self,
        customer_id: &str,
        group_id: &str,
    ) -> Result<()> {
        self.customer_service
            .remove_customer_from_group(customer_id, group_id)
            .await
    }

    // 12. upgrade_guest_to_registered_workflow
    pub async fn upgrade_guest_to_registered(
        &self,
        input: GuestUpgradeInput,
    ) -> Result<CustomerDTO> {
        self.customer_service
            .upgrade_guest_to_registered(input)
            .await
    }
}
