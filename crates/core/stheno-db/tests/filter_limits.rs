use stheno_db::{FilterOp, FilterSpec};

#[test]
fn test_in_clause_limit_enforced() {
    let mut items = Vec::new();
    for i in 0..101 {
        items.push(format!("item_{i}"));
    }

    let spec = FilterSpec::new().with_predicate("id", FilterOp::In(items));
    let result = spec.validate();
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("in_clause_limit_exceeded"));
}

#[test]
fn test_depth_limit_enforced() {
    let mut spec = FilterSpec::new();
    spec.depth = 4; // Exceeds max 3
    let result = spec.validate();
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("filter_depth_exceeded"));
}

#[test]
fn test_predicate_count_limit_enforced() {
    let mut spec = FilterSpec::new();
    for i in 0..21 {
        spec = spec.with_predicate(format!("field_{i}"), FilterOp::Eq("val".to_string()));
    }
    let result = spec.validate();
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("too_many_predicates"));
}

#[test]
fn test_leading_wildcard_rejected() {
    let spec = FilterSpec::new().with_predicate("title", FilterOp::Like("%shirt".to_string()));
    let result = spec.validate();
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("leading_wildcard_forbidden"));
}

#[test]
fn test_trailing_wildcard_accepted() {
    let spec = FilterSpec::new().with_predicate("title", FilterOp::Like("shirt%".to_string()));
    assert!(spec.validate().is_ok());
}

#[test]
fn test_page_limits() {
    let spec = FilterSpec::new().with_pagination(0, 500);
    assert_eq!(spec.page_limit(), 100); // Clamped to max 100

    let default_spec = FilterSpec::new();
    assert_eq!(default_spec.page_limit(), 20); // Default 20
}
