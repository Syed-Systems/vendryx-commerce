use stheno_db::{FilterOp, FilterSpec, OrderDirection};

#[test]
fn test_filterspec_construction_and_validation() {
    let spec = FilterSpec::new()
        .with_predicate("status", FilterOp::Eq("active".to_string()))
        .with_predicate("price", FilterOp::Gte("1000".to_string()))
        .with_predicate("title", FilterOp::Like("Sneaker%".to_string()))
        .with_order("created_at", OrderDirection::Desc)
        .with_pagination(0, 50);

    assert!(spec.validate().is_ok());
    assert_eq!(spec.page_limit(), 50);
    assert_eq!(spec.page_offset(), 0);
    assert_eq!(spec.predicates.len(), 3);
    assert_eq!(spec.order_by.len(), 1);
}
