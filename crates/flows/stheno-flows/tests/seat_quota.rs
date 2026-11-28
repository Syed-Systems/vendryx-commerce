use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::build_create_seat_quota_saga;
use stheno_user::UserService;

#[tokio::test]
async fn test_staff_seat_creation_wired_with_check_quota_step() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let user_service = Arc::new(UserService::new(pool.clone()));

    let saga = build_create_seat_quota_saga(user_service.clone(), counter.clone(), Some(2));

    // 1. Create 1st seat -> OK
    let res1 = saga
        .execute(
            "tx_u1",
            "create-user",
            json!({ "email": "staff1@example.com", "role": "admin" }),
        )
        .await;
    assert!(res1.is_ok());

    // 2. Create 2nd seat -> OK
    let res2 = saga
        .execute(
            "tx_u2",
            "create-user",
            json!({ "email": "staff2@example.com", "role": "member" }),
        )
        .await;
    assert!(res2.is_ok());

    // 3. Create 3rd seat -> Exceeds limit (2)
    let res3 = saga
        .execute(
            "tx_u3",
            "create-user",
            json!({ "email": "staff3@example.com", "role": "member" }),
        )
        .await;
    assert!(res3.is_err());
    let err = res3.unwrap_err().to_string();
    assert!(err.contains("Staff seat limit reached (2)"));

    // Verify only 2 users exist in user_service
    let users = user_service.list_users().await.expect("list");
    assert_eq!(users.len(), 2);

    db.drop().await;
}

#[tokio::test]
async fn concurrent_staff_seat_creates_cannot_exceed_the_limit() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let user_service = Arc::new(UserService::new(pool));
    let first = build_create_seat_quota_saga(user_service.clone(), counter.clone(), Some(1));
    let second = build_create_seat_quota_saga(user_service.clone(), counter.clone(), Some(1));

    let (first_result, second_result) = tokio::join!(
        first.execute(
            "concurrent_seat_a",
            "create-user",
            json!({ "email": "staff-a@example.test", "role": "member" }),
        ),
        second.execute(
            "concurrent_seat_b",
            "create-user",
            json!({ "email": "staff-b@example.test", "role": "member" }),
        ),
    );

    assert_ne!(first_result.is_ok(), second_result.is_ok());
    let failure = first_result
        .err()
        .or_else(|| second_result.err())
        .expect("one creation must be rejected at the seat limit");
    assert!(failure.to_string().contains("Staff seat limit reached (1)"));
    assert_eq!(
        user_service.list_users().await.expect("list users").len(),
        1
    );
    assert_eq!(counter.usage_scoped("users").await.expect("seat usage"), 1);

    db.drop().await;
}
