use chrono::Utc;
use stheno_cart::{CartService, CreateCartInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

async fn service() -> (TestDb, CartService) {
    let db = TestDb::migrated().await;
    let svc = CartService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

#[tokio::test]
async fn test_cart_creation_rate_limiting() {
    let (db, service) = service().await;
    let ip = "192.168.1.50";

    // Rate limit: 5 creations per minute
    for _ in 0..5 {
        assert!(service.check_creation_rate_limit(ip, 5));
    }

    // 6th attempt should be rate limited
    assert!(!service.check_creation_rate_limit(ip, 5));

    // Another IP should still be allowed
    let other_ip = "192.168.1.99";
    assert!(service.check_creation_rate_limit(other_ip, 5));

    db.drop().await;
}

#[tokio::test]
async fn test_cart_ttl_cleanup_job() {
    let (db, service) = service().await;

    // Create 3 carts
    let c1 = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let c2 = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let c3 = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    // Mark c3 as completed
    service
        .update_cart(
            &c3.id,
            stheno_cart::UpdateCartInput {
                completed_at: Some(Utc::now()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    // Small delay to ensure timestamp is strictly in the past
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // With ttl_days = 0, any cart created before now should be expired, but completed carts are preserved
    let cleaned = service.cleanup_expired_carts(0).await.unwrap();
    // c1 and c2 should be cleaned up, c3 preserved because completed
    assert_eq!(cleaned, 2);

    // c1 and c2 should now return NotFound (soft-deleted)
    assert!(service.get_cart(&c1.id).await.is_err());
    assert!(service.get_cart(&c2.id).await.is_err());
    assert!(service.get_cart(&c3.id).await.is_ok());

    db.drop().await;
}
