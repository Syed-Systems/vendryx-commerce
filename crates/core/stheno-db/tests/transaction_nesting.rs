use stheno_db::create_pool;

#[tokio::test]
async fn test_pool_builder() {
    let pool = create_pool("postgres://postgres:postgres@localhost:5432/stheno_test", 5);
    assert!(
        pool.is_ok(),
        "Pool creation should succeed with valid config"
    );
}
