//! Internal entity models for stheno-customer.
//!
//! Rule 2.9: Entities are strictly pub(crate) and never derive deserialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Customer {
    pub id: Id<Customer>,
    pub email: String,
    pub default_billing_address_id: Option<String>,
    pub default_shipping_address_id: Option<String>,
    pub company_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub has_account: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CustomerGroup {
    pub id: Id<CustomerGroup>,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CustomerGroupCustomer {
    pub customer_id: String,
    pub customer_group_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CustomerAddress {
    pub id: Id<CustomerAddress>,
    pub customer_id: String,
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: String,
    pub country_code: String,
    pub province: Option<String>,
    pub postal_code: String,
    pub phone: Option<String>,
    pub is_default_shipping: bool,
    pub is_default_billing: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
