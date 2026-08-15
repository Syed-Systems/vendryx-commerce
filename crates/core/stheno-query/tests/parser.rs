use stheno_query::FieldSelection;

#[test]
fn test_field_selection_parser_nested_relations() {
    let fields = vec![
        "id",
        "title",
        "variants.id",
        "variants.title",
        "variants.prices.amount",
        "variants.prices.currency_code",
    ];

    let selection = FieldSelection::parse(&fields).expect("must parse");
    assert_eq!(selection.depth(), 3);
    assert_eq!(selection.fields, vec!["id", "title"]);
    assert!(selection.relations.contains_key("variants"));
    assert!(selection.relations["variants"]
        .relations
        .contains_key("prices"));
}

#[test]
fn test_field_selection_depth_limit_enforced() {
    let fields = vec!["a.b.c.d.e.f.g"];
    let res = FieldSelection::parse(&fields);
    assert!(res.is_err());
}
