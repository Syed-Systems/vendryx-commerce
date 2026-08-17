//! Customer Service managing customers, groups, addresses, and guest account upgrades (P7-14, P14-01).

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateCustomerAddressInput, CreateCustomerGroupInput, CreateCustomerInput, CustomerAddressDTO,
    CustomerDTO, CustomerGroupDTO, GuestUpgradeInput, UpdateCustomerAddressInput,
    UpdateCustomerGroupInput, UpdateCustomerInput,
};
use crate::models::{Customer, CustomerAddress, CustomerGroup};

#[derive(Clone)]
pub struct CustomerService {
    customers: PgCrudRepository<Customer>,
    groups: PgCrudRepository<CustomerGroup>,
    addresses: PgCrudRepository<CustomerAddress>,
}

impl CustomerService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            customers: PgCrudRepository::new(pool.clone()),
            groups: PgCrudRepository::new(pool.clone()),
            addresses: PgCrudRepository::new(pool),
        }
    }

    async fn customer_to_dto(&self, c: &Customer) -> Result<CustomerDTO> {
        let addrs = self.list_addresses_by_customer(&c.id.to_string()).await?;
        Ok(CustomerDTO {
            id: c.id.to_string(),
            email: c.email.clone(),
            default_billing_address_id: c.default_billing_address_id.clone(),
            default_shipping_address_id: c.default_shipping_address_id.clone(),
            company_name: c.company_name.clone(),
            first_name: c.first_name.clone(),
            last_name: c.last_name.clone(),
            phone: c.phone.clone(),
            has_account: c.has_account,
            groups: Vec::new(),
            addresses: addrs,
            metadata: c.metadata.clone(),
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
    }

    fn address_to_dto(a: &CustomerAddress) -> CustomerAddressDTO {
        CustomerAddressDTO {
            id: a.id.to_string(),
            customer_id: a.customer_id.clone(),
            company: a.company.clone(),
            first_name: a.first_name.clone(),
            last_name: a.last_name.clone(),
            address_1: a.address_1.clone(),
            address_2: a.address_2.clone(),
            city: a.city.clone(),
            country_code: a.country_code.clone(),
            province: a.province.clone(),
            postal_code: a.postal_code.clone(),
            phone: a.phone.clone(),
            is_default_shipping: a.is_default_shipping,
            is_default_billing: a.is_default_billing,
            metadata: a.metadata.clone(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }

    fn group_to_dto(g: &CustomerGroup) -> CustomerGroupDTO {
        CustomerGroupDTO {
            id: g.id.to_string(),
            name: g.name.clone(),
            customers_count: None,
            metadata: g.metadata.clone(),
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }

    // ==========================================
    // 1. Customer CRUD
    // ==========================================

    pub async fn create_customer(&self, input: CreateCustomerInput) -> Result<CustomerDTO> {
        let id = Id::new();
        let now = Utc::now();
        let has_account = input.has_account.unwrap_or(false);

        if has_account {
            let filter = FilterSpec::new()
                .with_predicate("email", FilterOp::Eq(input.email.clone()))
                .with_predicate("has_account", FilterOp::Eq(true.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let existing = self.customers.list(None, &filter).await?;
            if !existing.is_empty() {
                return Err(Error::Conflict {
                    code: "customer_email_exists",
                    message: format!(
                        "Registered customer with email {} already exists",
                        input.email
                    ),
                });
            }
        }

        let customer = Customer {
            id,
            email: input.email,
            default_billing_address_id: None,
            default_shipping_address_id: None,
            company_name: input.company_name,
            first_name: input.first_name,
            last_name: input.last_name,
            phone: input.phone,
            has_account,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.customers.create(None, customer).await?;
        self.customer_to_dto(&created).await
    }

    pub async fn get_customer(&self, id: &str) -> Result<CustomerDTO> {
        let customer = self.customers.retrieve(None, id).await?;
        self.customer_to_dto(&customer).await
    }

    pub async fn get_customer_by_email(&self, email: &str) -> Result<CustomerDTO> {
        let filter = FilterSpec::new()
            .with_predicate("email", FilterOp::Eq(email.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, 1);

        let customers = self.customers.list(None, &filter).await?;
        let customer = customers
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound {
                entity: "customer",
                id: email.to_string(),
            })?;

        self.customer_to_dto(&customer).await
    }

    pub async fn list_customers(&self) -> Result<Vec<CustomerDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.customers.list(None, &filter).await?;
        let mut result = Vec::with_capacity(list.len());
        for c in &list {
            result.push(self.customer_to_dto(c).await?);
        }
        Ok(result)
    }

    pub async fn update_customer(
        &self,
        id: &str,
        input: UpdateCustomerInput,
    ) -> Result<CustomerDTO> {
        let mut customer = self.customers.retrieve(None, id).await?;

        if let Some(email) = input.email {
            customer.email = email;
        }
        if let Some(first_name) = input.first_name {
            customer.first_name = Some(first_name);
        }
        if let Some(last_name) = input.last_name {
            customer.last_name = Some(last_name);
        }
        if let Some(company_name) = input.company_name {
            customer.company_name = Some(company_name);
        }
        if let Some(phone) = input.phone {
            customer.phone = Some(phone);
        }
        if let Some(has_account) = input.has_account {
            customer.has_account = has_account;
        }
        if let Some(metadata) = input.metadata {
            customer.metadata = Some(metadata);
        }
        if let Some(billing_id) = input.default_billing_address_id {
            customer.default_billing_address_id = Some(billing_id);
        }
        if let Some(shipping_id) = input.default_shipping_address_id {
            customer.default_shipping_address_id = Some(shipping_id);
        }
        customer.updated_at = Utc::now();

        let updated = self
            .customers
            .update(None, &customer.id.to_string(), customer)
            .await?;
        self.customer_to_dto(&updated).await
    }

    pub async fn delete_customer(&self, id: &str) -> Result<CustomerDTO> {
        let customer = self.customers.retrieve(None, id).await?;
        let dto = self.customer_to_dto(&customer).await?;
        self.customers
            .soft_delete(None, &customer.id.to_string())
            .await?;
        Ok(dto)
    }

    pub async fn upgrade_guest_to_registered(
        &self,
        input: GuestUpgradeInput,
    ) -> Result<CustomerDTO> {
        let mut customer = self
            .customers
            .retrieve(None, &input.guest_customer_id)
            .await?;

        if customer.has_account {
            return Err(Error::Conflict {
                code: "customer_already_registered",
                message: "Customer already has a registered account".to_string(),
            });
        }

        let filter = FilterSpec::new()
            .with_predicate("email", FilterOp::Eq(input.registered_email.clone()))
            .with_predicate("has_account", FilterOp::Eq(true.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        let existing = self.customers.list(None, &filter).await?;
        if !existing.is_empty() {
            return Err(Error::Conflict {
                code: "customer_email_exists",
                message: format!(
                    "Registered account with email {} already exists",
                    input.registered_email
                ),
            });
        }

        customer.email = input.registered_email;
        customer.has_account = true;
        if let Some(first_name) = input.first_name {
            customer.first_name = Some(first_name);
        }
        if let Some(last_name) = input.last_name {
            customer.last_name = Some(last_name);
        }
        if let Some(phone) = input.phone {
            customer.phone = Some(phone);
        }
        customer.updated_at = Utc::now();

        let updated = self
            .customers
            .update(None, &customer.id.to_string(), customer)
            .await?;
        self.customer_to_dto(&updated).await
    }

    // ==========================================
    // 2. Customer Groups
    // ==========================================

    pub async fn create_customer_group(
        &self,
        input: CreateCustomerGroupInput,
    ) -> Result<CustomerGroupDTO> {
        let id = Id::new();
        let now = Utc::now();

        let group = CustomerGroup {
            id,
            name: input.name,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.groups.create(None, group).await?;
        Ok(Self::group_to_dto(&created))
    }

    pub async fn get_customer_group(&self, id: &str) -> Result<CustomerGroupDTO> {
        let group = self.groups.retrieve(None, id).await?;
        Ok(Self::group_to_dto(&group))
    }

    pub async fn list_customer_groups(&self) -> Result<Vec<CustomerGroupDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let list = self.groups.list(None, &filter).await?;
        Ok(list.into_iter().map(|g| Self::group_to_dto(&g)).collect())
    }

    pub async fn update_customer_group(
        &self,
        id: &str,
        input: UpdateCustomerGroupInput,
    ) -> Result<CustomerGroupDTO> {
        let mut group = self.groups.retrieve(None, id).await?;

        if let Some(name) = input.name {
            group.name = name;
        }
        if let Some(metadata) = input.metadata {
            group.metadata = Some(metadata);
        }
        group.updated_at = Utc::now();

        let updated = self
            .groups
            .update(None, &group.id.to_string(), group)
            .await?;
        Ok(Self::group_to_dto(&updated))
    }

    pub async fn delete_customer_group(&self, id: &str) -> Result<()> {
        let group = self.groups.retrieve(None, id).await?;
        self.groups.soft_delete(None, &group.id.to_string()).await
    }

    pub async fn add_customer_to_group(&self, _customer_id: &str, _group_id: &str) -> Result<()> {
        Ok(())
    }

    pub async fn remove_customer_from_group(
        &self,
        _customer_id: &str,
        _group_id: &str,
    ) -> Result<()> {
        Ok(())
    }

    // ==========================================
    // 3. Customer Addresses
    // ==========================================

    pub async fn create_customer_address(
        &self,
        input: CreateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        let _customer = self.customers.retrieve(None, &input.customer_id).await?;

        let id = Id::new();
        let now = Utc::now();
        let is_shipping = input.is_default_shipping.unwrap_or(false);
        let is_billing = input.is_default_billing.unwrap_or(false);

        let address = CustomerAddress {
            id,
            customer_id: input.customer_id.clone(),
            company: input.company,
            first_name: input.first_name,
            last_name: input.last_name,
            address_1: input.address_1,
            address_2: input.address_2,
            city: input.city,
            country_code: input.country_code,
            province: input.province,
            postal_code: input.postal_code,
            phone: input.phone,
            is_default_shipping: is_shipping,
            is_default_billing: is_billing,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.addresses.create(None, address).await?;
        let addr_id_str = created.id.to_string();

        if is_shipping || is_billing {
            let mut cust = self.customers.retrieve(None, &input.customer_id).await?;
            if is_shipping {
                cust.default_shipping_address_id = Some(addr_id_str.clone());
            }
            if is_billing {
                cust.default_billing_address_id = Some(addr_id_str);
            }
            cust.updated_at = Utc::now();
            self.customers
                .update(None, &cust.id.to_string(), cust)
                .await?;
        }

        Ok(Self::address_to_dto(&created))
    }

    pub async fn get_customer_address(&self, id: &str) -> Result<CustomerAddressDTO> {
        let addr = self.addresses.retrieve(None, id).await?;
        Ok(Self::address_to_dto(&addr))
    }

    pub async fn list_addresses_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerAddressDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("customer_id", FilterOp::Eq(customer_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, 100);

        let addrs = self.addresses.list(None, &filter).await?;
        Ok(addrs.iter().map(Self::address_to_dto).collect())
    }

    pub async fn list_customer_addresses(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerAddressDTO>> {
        self.list_addresses_by_customer(customer_id).await
    }

    pub async fn update_customer_address(
        &self,
        id: &str,
        input: UpdateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        let mut addr = self.addresses.retrieve(None, id).await?;

        if let Some(company) = input.company {
            addr.company = Some(company);
        }
        if let Some(first_name) = input.first_name {
            addr.first_name = Some(first_name);
        }
        if let Some(last_name) = input.last_name {
            addr.last_name = Some(last_name);
        }
        if let Some(address_1) = input.address_1 {
            addr.address_1 = address_1;
        }
        if let Some(address_2) = input.address_2 {
            addr.address_2 = Some(address_2);
        }
        if let Some(city) = input.city {
            addr.city = city;
        }
        if let Some(country_code) = input.country_code {
            addr.country_code = country_code;
        }
        if let Some(province) = input.province {
            addr.province = Some(province);
        }
        if let Some(postal_code) = input.postal_code {
            addr.postal_code = postal_code;
        }
        if let Some(phone) = input.phone {
            addr.phone = Some(phone);
        }
        if let Some(is_shipping) = input.is_default_shipping {
            addr.is_default_shipping = is_shipping;
        }
        if let Some(is_billing) = input.is_default_billing {
            addr.is_default_billing = is_billing;
        }
        if let Some(metadata) = input.metadata {
            addr.metadata = Some(metadata);
        }
        addr.updated_at = Utc::now();

        let updated = self
            .addresses
            .update(None, &addr.id.to_string(), addr)
            .await?;
        Ok(Self::address_to_dto(&updated))
    }

    pub async fn delete_customer_address(&self, id: &str) -> Result<()> {
        let addr = self.addresses.retrieve(None, id).await?;
        self.addresses.soft_delete(None, &addr.id.to_string()).await
    }
}
