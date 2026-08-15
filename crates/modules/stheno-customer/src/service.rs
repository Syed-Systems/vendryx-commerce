//! Customer Service managing customers, groups, addresses, and guest account upgrades (P7-14).

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateCustomerAddressInput, CreateCustomerGroupInput, CreateCustomerInput, CustomerAddressDTO,
    CustomerDTO, CustomerGroupDTO, GuestUpgradeInput, UpdateCustomerAddressInput,
    UpdateCustomerGroupInput, UpdateCustomerInput,
};
use crate::models::{Customer, CustomerAddress, CustomerGroup, CustomerGroupCustomer};

#[derive(Clone, Default)]
pub struct CustomerService {
    customers: Arc<RwLock<HashMap<String, Customer>>>,
    groups: Arc<RwLock<HashMap<String, CustomerGroup>>>,
    group_customers: Arc<RwLock<Vec<CustomerGroupCustomer>>>,
    addresses: Arc<RwLock<HashMap<String, CustomerAddress>>>,
}

impl CustomerService {
    pub fn new() -> Self {
        Self::default()
    }

    // ==========================================
    // 1. Customer CRUD
    // ==========================================

    pub async fn create_customer(&self, input: CreateCustomerInput) -> Result<CustomerDTO> {
        let id = Id::new();
        let now = Utc::now();
        let has_account = input.has_account.unwrap_or(false);

        {
            let mut map = self.customers.write().unwrap();

            // If registered account, verify email uniqueness among active accounts
            if has_account
                && map
                    .values()
                    .any(|c| c.email == input.email && c.has_account && c.deleted_at.is_none())
            {
                return Err(Error::Conflict {
                    code: "customer_email_exists",
                    message: format!(
                        "Registered customer with email {} already exists",
                        input.email
                    ),
                });
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

            map.insert(id.to_string(), customer);
        }

        self.get_customer(&id.to_string()).await
    }

    pub async fn update_customer(
        &self,
        id: &str,
        input: UpdateCustomerInput,
    ) -> Result<CustomerDTO> {
        {
            let mut map = self.customers.write().unwrap();
            let customer = map.get_mut(id).ok_or_else(|| Error::NotFound {
                entity: "customer",
                id: id.to_string(),
            })?;

            if customer.deleted_at.is_some() {
                return Err(Error::NotFound {
                    entity: "customer",
                    id: id.to_string(),
                });
            }

            if let Some(email) = input.email {
                customer.email = email;
            }
            if let Some(company) = input.company_name {
                customer.company_name = Some(company);
            }
            if let Some(fname) = input.first_name {
                customer.first_name = Some(fname);
            }
            if let Some(lname) = input.last_name {
                customer.last_name = Some(lname);
            }
            if let Some(phone) = input.phone {
                customer.phone = Some(phone);
            }
            if let Some(has_acc) = input.has_account {
                customer.has_account = has_acc;
            }
            if let Some(def_bill) = input.default_billing_address_id {
                customer.default_billing_address_id = Some(def_bill);
            }
            if let Some(def_ship) = input.default_shipping_address_id {
                customer.default_shipping_address_id = Some(def_ship);
            }
            if let Some(meta) = input.metadata {
                customer.metadata = Some(meta);
            }

            customer.updated_at = Utc::now();
        }

        self.get_customer(id).await
    }

    pub async fn get_customer(&self, id: &str) -> Result<CustomerDTO> {
        let customer = {
            let map = self.customers.read().unwrap();
            map.get(id)
                .filter(|c| c.deleted_at.is_none())
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "customer",
                    id: id.to_string(),
                })?
        };

        // Load groups
        let groups = {
            let gc_list = self.group_customers.read().unwrap();
            let g_map = self.groups.read().unwrap();
            gc_list
                .iter()
                .filter(|gc| gc.customer_id == id)
                .filter_map(|gc| g_map.get(&gc.customer_group_id))
                .filter(|g| g.deleted_at.is_none())
                .map(|g| CustomerGroupDTO {
                    id: g.id.to_string(),
                    name: g.name.clone(),
                    customers_count: None,
                    metadata: g.metadata.clone(),
                    created_at: g.created_at,
                    updated_at: g.updated_at,
                })
                .collect()
        };

        // Load addresses
        let addresses = {
            let a_map = self.addresses.read().unwrap();
            a_map
                .values()
                .filter(|a| a.customer_id == id && a.deleted_at.is_none())
                .map(|a| CustomerAddressDTO {
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
                })
                .collect()
        };

        Ok(CustomerDTO {
            id: customer.id.to_string(),
            email: customer.email,
            default_billing_address_id: customer.default_billing_address_id,
            default_shipping_address_id: customer.default_shipping_address_id,
            company_name: customer.company_name,
            first_name: customer.first_name,
            last_name: customer.last_name,
            phone: customer.phone,
            has_account: customer.has_account,
            groups,
            addresses,
            metadata: customer.metadata,
            created_at: customer.created_at,
            updated_at: customer.updated_at,
        })
    }

    pub async fn list_customers(&self) -> Result<Vec<CustomerDTO>> {
        let ids: Vec<String> = {
            let map = self.customers.read().unwrap();
            map.values()
                .filter(|c| c.deleted_at.is_none())
                .map(|c| c.id.to_string())
                .collect()
        };

        let mut results = Vec::new();
        for id in ids {
            results.push(self.get_customer(&id).await?);
        }
        Ok(results)
    }

    pub async fn delete_customer(&self, id: &str) -> Result<CustomerDTO> {
        let mut map = self.customers.write().unwrap();
        let customer = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "customer",
            id: id.to_string(),
        })?;

        let now = Utc::now();
        customer.deleted_at = Some(now);

        Ok(CustomerDTO {
            id: customer.id.to_string(),
            email: customer.email.clone(),
            default_billing_address_id: customer.default_billing_address_id.clone(),
            default_shipping_address_id: customer.default_shipping_address_id.clone(),
            company_name: customer.company_name.clone(),
            first_name: customer.first_name.clone(),
            last_name: customer.last_name.clone(),
            phone: customer.phone.clone(),
            has_account: customer.has_account,
            groups: vec![],
            addresses: vec![],
            metadata: customer.metadata.clone(),
            created_at: customer.created_at,
            updated_at: now,
        })
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

        let dto = CustomerGroupDTO {
            id: id.to_string(),
            name: group.name.clone(),
            customers_count: Some(0),
            metadata: group.metadata.clone(),
            created_at: group.created_at,
            updated_at: group.updated_at,
        };

        let mut map = self.groups.write().unwrap();
        map.insert(id.to_string(), group);
        Ok(dto)
    }

    pub async fn update_customer_group(
        &self,
        id: &str,
        input: UpdateCustomerGroupInput,
    ) -> Result<CustomerGroupDTO> {
        let mut map = self.groups.write().unwrap();
        let group = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "customer_group",
            id: id.to_string(),
        })?;

        if let Some(name) = input.name {
            group.name = name;
        }
        if let Some(meta) = input.metadata {
            group.metadata = Some(meta);
        }
        group.updated_at = Utc::now();

        Ok(CustomerGroupDTO {
            id: group.id.to_string(),
            name: group.name.clone(),
            customers_count: None,
            metadata: group.metadata.clone(),
            created_at: group.created_at,
            updated_at: group.updated_at,
        })
    }

    pub async fn add_customer_to_group(&self, customer_id: &str, group_id: &str) -> Result<()> {
        // Verify customer & group exist
        self.get_customer(customer_id).await?;
        {
            let map = self.groups.read().unwrap();
            if !map.contains_key(group_id) {
                return Err(Error::NotFound {
                    entity: "customer_group",
                    id: group_id.to_string(),
                });
            }
        }

        let mut gc_list = self.group_customers.write().unwrap();
        if !gc_list
            .iter()
            .any(|gc| gc.customer_id == customer_id && gc.customer_group_id == group_id)
        {
            gc_list.push(CustomerGroupCustomer {
                customer_id: customer_id.to_string(),
                customer_group_id: group_id.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        Ok(())
    }

    pub async fn list_customer_groups(&self) -> Result<Vec<CustomerGroupDTO>> {
        let map = self.groups.read().unwrap();
        let list = map
            .values()
            .map(|g| CustomerGroupDTO {
                id: g.id.to_string(),
                name: g.name.clone(),
                customers_count: Some(0),
                metadata: g.metadata.clone(),
                created_at: g.created_at,
                updated_at: g.updated_at,
            })
            .collect();
        Ok(list)
    }

    pub async fn remove_customer_from_group(
        &self,
        customer_id: &str,
        group_id: &str,
    ) -> Result<()> {
        let mut gc_list = self.group_customers.write().unwrap();
        gc_list.retain(|gc| !(gc.customer_id == customer_id && gc.customer_group_id == group_id));
        Ok(())
    }

    // ==========================================
    // 3. Customer Addresses
    // ==========================================

    pub async fn create_customer_address(
        &self,
        input: CreateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        let id = Id::new();
        let now = Utc::now();

        let is_default_shipping = input.is_default_shipping.unwrap_or(false);
        let is_default_billing = input.is_default_billing.unwrap_or(false);

        // If setting as default, unset existing default for this customer
        if is_default_shipping || is_default_billing {
            let mut a_map = self.addresses.write().unwrap();
            for addr in a_map.values_mut() {
                if addr.customer_id == input.customer_id {
                    if is_default_shipping {
                        addr.is_default_shipping = false;
                    }
                    if is_default_billing {
                        addr.is_default_billing = false;
                    }
                }
            }
        }

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
            is_default_shipping,
            is_default_billing,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let dto = CustomerAddressDTO {
            id: id.to_string(),
            customer_id: address.customer_id.clone(),
            company: address.company.clone(),
            first_name: address.first_name.clone(),
            last_name: address.last_name.clone(),
            address_1: address.address_1.clone(),
            address_2: address.address_2.clone(),
            city: address.city.clone(),
            country_code: address.country_code.clone(),
            province: address.province.clone(),
            postal_code: address.postal_code.clone(),
            phone: address.phone.clone(),
            is_default_shipping: address.is_default_shipping,
            is_default_billing: address.is_default_billing,
            metadata: address.metadata.clone(),
            created_at: address.created_at,
            updated_at: address.updated_at,
        };

        let mut a_map = self.addresses.write().unwrap();
        a_map.insert(id.to_string(), address);

        // Update default address IDs on customer if specified
        if is_default_shipping || is_default_billing {
            let mut c_map = self.customers.write().unwrap();
            if let Some(cust) = c_map.get_mut(&input.customer_id) {
                if is_default_shipping {
                    cust.default_shipping_address_id = Some(id.to_string());
                }
                if is_default_billing {
                    cust.default_billing_address_id = Some(id.to_string());
                }
            }
        }

        Ok(dto)
    }

    pub async fn update_customer_address(
        &self,
        address_id: &str,
        input: UpdateCustomerAddressInput,
    ) -> Result<CustomerAddressDTO> {
        let mut a_map = self.addresses.write().unwrap();
        let addr = a_map.get_mut(address_id).ok_or_else(|| Error::NotFound {
            entity: "customer_address",
            id: address_id.to_string(),
        })?;

        if let Some(company) = input.company {
            addr.company = Some(company);
        }
        if let Some(fname) = input.first_name {
            addr.first_name = Some(fname);
        }
        if let Some(lname) = input.last_name {
            addr.last_name = Some(lname);
        }
        if let Some(a1) = input.address_1 {
            addr.address_1 = a1;
        }
        if let Some(a2) = input.address_2 {
            addr.address_2 = Some(a2);
        }
        if let Some(city) = input.city {
            addr.city = city;
        }
        if let Some(cc) = input.country_code {
            addr.country_code = cc;
        }
        if let Some(prov) = input.province {
            addr.province = Some(prov);
        }
        if let Some(zip) = input.postal_code {
            addr.postal_code = zip;
        }
        if let Some(phone) = input.phone {
            addr.phone = Some(phone);
        }
        if let Some(def_s) = input.is_default_shipping {
            addr.is_default_shipping = def_s;
        }
        if let Some(def_b) = input.is_default_billing {
            addr.is_default_billing = def_b;
        }
        if let Some(meta) = input.metadata {
            addr.metadata = Some(meta);
        }
        addr.updated_at = Utc::now();

        Ok(CustomerAddressDTO {
            id: addr.id.to_string(),
            customer_id: addr.customer_id.clone(),
            company: addr.company.clone(),
            first_name: addr.first_name.clone(),
            last_name: addr.last_name.clone(),
            address_1: addr.address_1.clone(),
            address_2: addr.address_2.clone(),
            city: addr.city.clone(),
            country_code: addr.country_code.clone(),
            province: addr.province.clone(),
            postal_code: addr.postal_code.clone(),
            phone: addr.phone.clone(),
            is_default_shipping: addr.is_default_shipping,
            is_default_billing: addr.is_default_billing,
            metadata: addr.metadata.clone(),
            created_at: addr.created_at,
            updated_at: addr.updated_at,
        })
    }

    pub async fn list_customer_addresses(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerAddressDTO>> {
        let a_map = self.addresses.read().unwrap();
        let list: Vec<CustomerAddressDTO> = a_map
            .values()
            .filter(|a| a.customer_id == customer_id && a.deleted_at.is_none())
            .map(|addr| CustomerAddressDTO {
                id: addr.id.to_string(),
                customer_id: addr.customer_id.clone(),
                company: addr.company.clone(),
                first_name: addr.first_name.clone(),
                last_name: addr.last_name.clone(),
                address_1: addr.address_1.clone(),
                address_2: addr.address_2.clone(),
                city: addr.city.clone(),
                country_code: addr.country_code.clone(),
                province: addr.province.clone(),
                postal_code: addr.postal_code.clone(),
                phone: addr.phone.clone(),
                is_default_shipping: addr.is_default_shipping,
                is_default_billing: addr.is_default_billing,
                metadata: addr.metadata.clone(),
                created_at: addr.created_at,
                updated_at: addr.updated_at,
            })
            .collect();
        Ok(list)
    }

    pub async fn delete_customer_address(&self, address_id: &str) -> Result<()> {
        let mut a_map = self.addresses.write().unwrap();
        if let Some(addr) = a_map.get_mut(address_id) {
            addr.deleted_at = Some(Utc::now());
        }
        Ok(())
    }

    // ==========================================
    // 4. Guest to Registered Account Upgrade (P7-14)
    // ==========================================

    pub async fn upgrade_guest_to_registered(
        &self,
        input: GuestUpgradeInput,
    ) -> Result<CustomerDTO> {
        {
            let mut map = self.customers.write().unwrap();
            let customer =
                map.get_mut(&input.guest_customer_id)
                    .ok_or_else(|| Error::NotFound {
                        entity: "customer",
                        id: input.guest_customer_id.clone(),
                    })?;

            // Convert guest account to registered account while preserving ID
            customer.email = input.registered_email;
            customer.has_account = true;
            if let Some(fname) = input.first_name {
                customer.first_name = Some(fname);
            }
            if let Some(lname) = input.last_name {
                customer.last_name = Some(lname);
            }
            if let Some(phone) = input.phone {
                customer.phone = Some(phone);
            }
            customer.updated_at = Utc::now();
        }

        self.get_customer(&input.guest_customer_id).await
    }
}
