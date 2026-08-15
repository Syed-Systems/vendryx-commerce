use stheno_query::{FieldSelection, QueryPlanner};

#[test]
fn test_query_planner_generates_fetch_tree() {
    let fields = vec![
        "id",
        "customer.id",
        "customer.email",
        "items.id",
        "items.quantity",
    ];
    let selection = FieldSelection::parse(&fields).unwrap();

    let planner = QueryPlanner::new(4);
    let plan = planner.plan("order", &selection).unwrap();

    assert_eq!(plan.entity_name, "order");
    assert_eq!(plan.fields, vec!["id"]);
    assert_eq!(plan.children.len(), 2);
}
