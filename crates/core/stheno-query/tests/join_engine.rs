use serde_json::json;
use std::sync::Arc;
use stheno_links::LinkStore;
use stheno_query::{FieldSelection, QueryEngine};

#[test]
fn test_field_selection_parsing_and_depth() {
    let fields = vec!["id", "title", "variants.id", "variants.prices.amount"];
    let selection = FieldSelection::parse(&fields).expect("parse must succeed");

    assert!(selection.fields.contains(&"id".to_string()));
    assert!(selection.fields.contains(&"title".to_string()));

    let variants = selection
        .relations
        .get("variants")
        .expect("variants relation must exist");
    assert!(variants.fields.contains(&"id".to_string()));

    let prices = variants
        .relations
        .get("prices")
        .expect("prices relation must exist");
    assert!(prices.fields.contains(&"amount".to_string()));

    assert_eq!(selection.depth(), 3);
}

#[test]
fn test_query_depth_limit_enforced() {
    let deep_fields = vec!["a.b.c.d.e.f.g"];
    let err = FieldSelection::parse(&deep_fields);
    assert!(
        err.is_err(),
        "Exceeding max query depth must return an error"
    );
}

#[test]
fn test_remote_join_relation_attachment() {
    let links = Arc::new(LinkStore::new());
    links.link("product_variant_price_set", "var_100", "pset_200");
    links.link("product_variant_price_set", "var_100", "pset_201");

    let engine = QueryEngine::new(links);

    let mut variant_record = json!({
        "id": "var_100",
        "sku": "SHIRT-RED-L"
    });

    engine.attach_relations(
        "product_variant_price_set",
        "var_100",
        "price_set_ids",
        &mut variant_record,
    );

    let price_sets = variant_record["price_set_ids"].as_array().unwrap();
    assert_eq!(price_sets.len(), 2);
    assert_eq!(price_sets[0], "pset_200");
    assert_eq!(price_sets[1], "pset_201");
}
