use stheno_server::TestApp;

#[tokio::test]
async fn test_test_app_harness_spawn() {
    let app1 = TestApp::spawn().await;
    let app2 = TestApp::spawn().await;

    // Verify ephemeral isolation
    assert_ne!(app1.schema_name, app2.schema_name);
    assert!(app1.schema_name.starts_with("test_schema_"));
}
