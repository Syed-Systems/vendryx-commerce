//! `stheno-http::owned`: Secure Ownership extractor preventing IDOR vulnerabilities (SECURITY §7, AGENTS.md §2.10).

use serde::{Deserialize, Serialize};
use stheno_types::{Error, Result};

pub trait ScopeChecker {
    fn verify_ownership(&self, customer_id: &str) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owned<T> {
    pub value: T,
}

impl<T: ScopeChecker> Owned<T> {
    pub fn verify(
        entity: T,
        authenticated_customer_id: &str,
        entity_name: &'static str,
    ) -> Result<Self> {
        if entity.verify_ownership(authenticated_customer_id) {
            Ok(Self { value: entity })
        } else {
            // AGENTS.md §2.10: Return 404, not 403, on ownership failure to avoid existence oracle
            Err(Error::NotFound {
                entity: entity_name,
                id: "restricted".to_string(),
            })
        }
    }
}
