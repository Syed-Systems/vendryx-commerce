use std::sync::Arc;
use std::time::Duration;
use stheno_locking::{LockManager, PostgresAdvisoryLockProvider, RedisLockProvider};

#[tokio::test]
async fn test_distributed_locking_and_outage_fallback() {
    let redis_provider = Arc::new(RedisLockProvider::new());
    let pg_provider = Arc::new(PostgresAdvisoryLockProvider::new());

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
        other => panic!("Expected Conflict error, got {:?}", other),
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
}
