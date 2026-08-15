use stheno_db::{FilterOp, FilterSpec, OrderDirection};

#[test]
fn test_hard_case_01_admin_orders_list_filter() {
    let spec = FilterSpec::new()
        .with_predicate("customer_id", FilterOp::Eq("cus_123".to_string()))
        .with_predicate(
            "status",
            FilterOp::In(vec!["pending".to_string(), "completed".to_string()]),
        )
        .with_predicate(
            "created_at",
            FilterOp::Gte("2026-01-01T00:00:00Z".to_string()),
        )
        .with_order("created_at", OrderDirection::Desc)
        .with_pagination(20, 20);

    assert!(spec.validate().is_ok());
}

#[test]
fn test_hard_case_02_products_by_sales_channel_and_price_range() {
    let spec = FilterSpec::new()
        .with_predicate("sales_channel_id", FilterOp::Eq("sc_web".to_string()))
        .with_predicate("status", FilterOp::Eq("published".to_string()))
        .with_predicate("price_min", FilterOp::Gte("500".to_string()))
        .with_predicate("price_max", FilterOp::Lte("5000".to_string()))
        .with_predicate("title", FilterOp::Like("Pro%".to_string()))
        .with_order("title", OrderDirection::Asc);

    assert!(spec.validate().is_ok());
}

#[test]
fn test_hard_case_03_inventory_items_multi_location_availability() {
    let location_ids = (1..=25).map(|i| format!("loc_{}", i)).collect();
    let spec = FilterSpec::new()
        .with_predicate("location_id", FilterOp::In(location_ids))
        .with_predicate("stocked_quantity", FilterOp::Gt("0".to_string()))
        .with_order("updated_at", OrderDirection::Desc);

    assert!(spec.validate().is_ok());
}
