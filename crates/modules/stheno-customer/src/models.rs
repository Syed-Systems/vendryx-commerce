//! Internal entity models for stheno-customer.
//!
//! Rule 2.9: Entities are strictly pub(crate).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for Customer {
    const TABLE: &'static str = "customer.customer";
    const ENTITY: &'static str = "customer";

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CustomerGroup {
    pub id: Id<CustomerGroup>,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for CustomerGroup {
    const TABLE: &'static str = "customer.customer_group";
    const ENTITY: &'static str = "customer_group";

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for CustomerAddress {
    const TABLE: &'static str = "customer.customer_address";
    const ENTITY: &'static str = "customer_address";

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Customer => "cus",
    CustomerAddress => "cuaddr",
    CustomerGroup => "cusgroup",
}
