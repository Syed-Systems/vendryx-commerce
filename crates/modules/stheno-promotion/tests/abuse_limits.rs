use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{constant_time_eq_str, PromotionService};

#[tokio::test]
async fn test_promo_code_abuse_limits_and_constant_time_compare() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PromotionService::new(pool);

    // 1. Per-cart failed attempts limit (F10, F31): max 5 failed attempts per cart per hour
    let cart_id = "cart_test_abuse";
    for _ in 0..5 {
        assert!(
            service.check_cart_promo_abuse_limit(cart_id, 5),
            "should allow under 5 failures"
        );
        service.record_failed_promo_attempt(cart_id);
    }

    // 6th attempt is blocked
    assert!(
        !service.check_cart_promo_abuse_limit(cart_id, 5),
        "should block after 5 failures"
    );

    // Another cart should not be affected
    let clean_cart = "cart_clean";
    assert!(service.check_cart_promo_abuse_limit(clean_cart, 5));

    // 2. Per-IP rate limiting: max 10 requests per minute
    let ip = "10.0.0.50";
    for _ in 0..10 {
        assert!(service.check_ip_promo_rate_limit(ip, 10));
    }
    // 11th request from same IP is rate limited
    assert!(!service.check_ip_promo_rate_limit(ip, 10));

    // 3. Constant-time string comparison
    assert!(constant_time_eq_str("SUMMER2026", "SUMMER2026"));
    assert!(!constant_time_eq_str("SUMMER2026", "WINTER2026"));
    assert!(!constant_time_eq_str("SUMMER2026", "SUMMER2025"));
    assert!(!constant_time_eq_str("SHORT", "LONGER_CODE"));
}
