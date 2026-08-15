use chrono::Utc;
use stheno_cart::{CartService, CreateCartInput};

#[tokio::test]
async fn test_cart_creation_rate_limiting() {
    let service = CartService::new();
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
}

#[tokio::test]
async fn test_cart_ttl_cleanup_job() {
    let service = CartService::new();

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

    // With ttl_days = 0, any cart created before now (or equal) should be expired, but completed carts are preserved
    // Let's test with cleanup_expired_carts
    let cleaned = service.cleanup_expired_carts(0);
    // c1 and c2 should be cleaned up, c3 preserved because completed
    assert_eq!(cleaned, 2);

    // c1 and c2 should now return NotFound (soft-deleted)
    assert!(service.get_cart(&c1.id).await.is_err());
    assert!(service.get_cart(&c2.id).await.is_err());
    assert!(service.get_cart(&c3.id).await.is_ok());
}
