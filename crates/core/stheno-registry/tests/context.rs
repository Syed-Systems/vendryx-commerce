use serde_json::json;
use stheno_events::Event;
use stheno_registry::AppContext;

#[tokio::test]
async fn test_app_context_initialization_and_mocking() {
    let ctx = AppContext::create_test_context();

    // Verify events bus works via AppContext
    let event = Event::new("test.event", json!({ "key": "value" }));
    let res = ctx.events.emit(vec![event]).await;
    assert!(res.is_ok());

    // Verify links store works via AppContext
    ctx.links
        .link("product_variant_price_set", "var_1", "pset_1");
    let rights = ctx.links.get_by_left("product_variant_price_set", "var_1");
    assert_eq!(rights, vec!["pset_1".to_string()]);
}
