//! Test P7-12: Breached-password check via k-anonymity query.

use stheno_auth::AuthService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_breached_password_detection() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuthService::new(pool, None);

    let breached_result = service.check_breached_password("password");
    assert!(breached_result.is_breached);
    assert!(breached_result.breach_count > 0);

    let safe_result = service.check_breached_password("Kx9#mQ2$zL8!vW7&pN4^");
    assert!(!safe_result.is_breached);
    assert_eq!(safe_result.breach_count, 0);

    db.drop().await;
}
