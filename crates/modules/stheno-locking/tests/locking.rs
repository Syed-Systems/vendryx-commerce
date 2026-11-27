use std::sync::Arc;
use std::time::Duration;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_locking::{LockManager, LockProvider, PostgresAdvisoryLockProvider, RedisLockProvider};

#[tokio::test]
async fn test_distributed_locking_and_outage_fallback() {
    // A migrated per-test database, like every other DB test. The hardcoded
    // localhost:5432 URL this used to carry only worked where Postgres happened to listen
    // there, and the lock manager swallows fallback errors, so a wrong URL showed up as a
    // lock timeout in step 5 rather than as a connection failure.
    let db = TestDb::migrated().await;
    let redis_provider = Arc::new(RedisLockProvider::new());
    let pg_provider = Arc::new(PostgresAdvisoryLockProvider::new(
        create_pool(&db.url, 5).expect("pool"),
    ));

    let lock_manager = LockManager::new(redis_provider.clone(), pg_provider.clone());

    let resource = "inventory:variant_v101";

    // 1. Acquire lock
    let lock1 = lock_manager
        .acquire(resource, 60, Duration::from_millis(500))
        .await
        .unwrap();

    assert_eq!(lock1.resource, resource);

    // 2. Second acquire should timeout
    let attempt2 = lock_manager
        .acquire(resource, 60, Duration::from_millis(150))
        .await;

    assert!(attempt2.is_err());
    match attempt2.unwrap_err() {
        stheno_types::Error::Conflict { code, .. } => {
            assert_eq!(code, "lock_acquisition_timeout")
        }
        other => panic!("Expected Conflict error, got {other:?}"),
    }

    // 3. Release lock
    let released = lock_manager.release(&lock1).await.unwrap();
    assert!(released);

    // 4. Now second acquire succeeds
    let lock2 = lock_manager
        .acquire(resource, 60, Duration::from_millis(500))
        .await
        .unwrap();

    assert_eq!(lock2.resource, resource);
    lock_manager.release(&lock2).await.unwrap();

    // 5. Test Redis Outage Resilience (Task P8-08)
    redis_provider.simulate_outage(true);

    // Acquire lock during Redis outage: seamlessly falls back to Postgres advisory locks!
    let fallback_lock = lock_manager
        .acquire("cart:checkout_c99", 60, Duration::from_millis(500))
        .await
        .unwrap();

    assert_eq!(fallback_lock.resource, "cart:checkout_c99");
    let fallback_released = lock_manager.release(&fallback_lock).await.unwrap();
    assert!(fallback_released);

    db.drop().await;
}

#[tokio::test]
async fn test_postgres_advisory_lock_contention() {
    // P14-16 & P14-17: Two processes contend for one advisory lock on Postgres; exactly one proceeds
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 10).expect("pool");

    let pg_provider = Arc::new(PostgresAdvisoryLockProvider::new(pool));
    let resource = "product:sku_contended_99";

    // Process 1 acquires lock
    let lock1 = pg_provider.try_acquire(resource, 60).await.unwrap();
    assert!(lock1.is_some(), "Process 1 must acquire the lock");
    let token1 = lock1.unwrap();

    // Process 2 tries to acquire same lock simultaneously -> must be denied (None)
    let lock2 = pg_provider.try_acquire(resource, 60).await.unwrap();
    assert!(
        lock2.is_none(),
        "Process 2 must be blocked by advisory lock contention"
    );

    // Process 1 releases lock
    let released = pg_provider.release(resource, &token1.token).await.unwrap();
    assert!(released, "Process 1 must successfully release lock");

    // Process 2 can now acquire lock
    let lock2_retry = pg_provider.try_acquire(resource, 60).await.unwrap();
    assert!(
        lock2_retry.is_some(),
        "Process 2 must acquire lock after release"
    );
    let token2 = lock2_retry.unwrap();

    pg_provider.release(resource, &token2.token).await.unwrap();

    db.drop().await;
}
