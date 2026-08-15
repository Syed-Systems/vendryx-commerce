use serde_json::json;
use stheno_db::FilterSpec;

#[test]
fn test_product_filterspec_building_and_limits() {
    let raw_filter = json!({
        "status": "published",
        "title": { "$ilike": "hoodie%" },
        "collection_id": { "$in": ["pcol_1", "pcol_2"] }
    });

    let spec = FilterSpec::from_json(&raw_filter).expect("valid product filter spec");
    assert_eq!(spec.predicates.len(), 3);

    // Verify limit rejection ($in > 100)
    let huge_in: Vec<String> = (0..105).map(|i| format!("id_{}", i)).collect();
    let invalid_in_filter = json!({
        "id": { "$in": huge_in }
    });
    assert!(FilterSpec::from_json(&invalid_in_filter).is_err());

    // Verify leading wildcard rejection (P0-13, P2-09)
    let invalid_like_filter = json!({
        "title": { "$ilike": "%hoodie%" }
    });
    assert!(FilterSpec::from_json(&invalid_like_filter).is_err());
}
