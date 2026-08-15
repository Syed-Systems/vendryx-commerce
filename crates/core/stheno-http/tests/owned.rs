use stheno_http::{Owned, ScopeChecker};

#[derive(Debug)]
struct CustomerOrder {
    pub id: String,
    pub customer_id: String,
}

impl ScopeChecker for CustomerOrder {
    fn verify_ownership(&self, customer_id: &str) -> bool {
        self.customer_id == customer_id
    }
}

#[test]
fn test_owned_success_for_authorized_customer() {
    let order = CustomerOrder {
        id: "ord_100".to_string(),
        customer_id: "cus_alice".to_string(),
    };

    let result = Owned::verify(order, "cus_alice", "order");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value.id, "ord_100");
}

#[test]
fn test_owned_returns_404_on_ownership_mismatch() {
    // AGENTS.md §2.10: Return 404, not 403, on ownership failure to prevent existence oracle
    let order = CustomerOrder {
        id: "ord_100".to_string(),
        customer_id: "cus_alice".to_string(),
    };

    let result = Owned::verify(order, "cus_bob_attacker", "order");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let response = stheno_http::HttpError::from(err);
    assert_eq!(
        axum::response::IntoResponse::into_response(response).status(),
        404
    );
}
