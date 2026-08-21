//! P30-11: generic subscription patching is not a second dunning writer.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{PlatformService, SubscriptionPatch};

#[tokio::test]
async fn generic_subscription_patch_cannot_change_status() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    let err = service
        .update_subscription(
            TestDb::TEST_TENANT,
            SubscriptionPatch {
                status: Some("grace".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect_err("status is not patched through the generic path");

    assert!(
        matches!(
            err,
            stheno_types::Error::Invalid {
                code: "subscription_status_dunning_owned",
                ..
            }
        ),
        "got {err:?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn operator_override_is_explicit() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    let sub = service
        .override_subscription_status(TestDb::TEST_TENANT, "grace")
        .await
        .expect("explicit override");

    assert_eq!(sub.status, "grace");

    db.drop().await;
}
