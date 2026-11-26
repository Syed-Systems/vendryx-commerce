use axum::extract::State;
use axum::Extension;
use axum::Json;
use std::collections::BTreeMap;
use std::sync::Arc;
use stheno_api_admin::{catalog, users, AdminQuotaContext};
use stheno_api_core::{Tier, TierConfig};
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, QuotaCounter};
use stheno_product::ProductService;
use stheno_user::UserService;

fn quota_context(counter: QuotaCounter) -> AdminQuotaContext {
    quota_context_with_limits(counter, 1, 1)
}

fn quota_context_with_limits(
    counter: QuotaCounter,
    max_products: usize,
    max_staff_seats: usize,
) -> AdminQuotaContext {
    let config = TierConfig::from_toml(
        &format!(
            "[free]\nfeatures = []\nquotas = {{ max_products = {max_products}, max_staff_seats = {max_staff_seats} }}\n"
        ),
    )
    .expect("valid test tier config");
    AdminQuotaContext {
        entitlements: Arc::new(
            config
                .resolve(Tier::Free, &BTreeMap::new())
                .expect("free tier resolves"),
        ),
        counter,
    }
}

#[tokio::test]
async fn production_product_route_enforces_the_atomic_quota() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let quota = quota_context(counter.clone());
    let service = Arc::new(ProductService::new(pool.clone()));

    stheno_db::tenant::scope("quota_product_route", async {
        let request = || catalog::CreateProductAdminRequest {
            title: "Quota product".to_string(),
            subtitle: None,
            description: None,
            handle: None,
            discountable: None,
            status: None,
            options: None,
            variants: None,
            sales_channels: None,
            shipping_profile_id: None,
            collection_id: None,
            categories: None,
            tags: None,
            type_id: None,
        };

        assert!(catalog::create_product(
            State(service.clone()),
            Some(Extension(quota.clone())),
            Json(request()),
        )
        .await
        .is_ok());

        assert!(
            catalog::create_product(State(service), Some(Extension(quota)), Json(request()),)
                .await
                .is_err()
        );

        let usage_counter = counter.clone();
        let usage_pool = counter.pool().clone();
        let used = stheno_db::tenant::with_tenant_conn(&usage_pool, move |conn| {
            Box::pin(async move { usage_counter.get_usage(conn, "products").await })
        })
        .await
        .expect("usage query");
        assert_eq!(used, 1);
    })
    .await;

    db.drop().await;
}

#[tokio::test]
async fn production_staff_route_enforces_the_atomic_quota() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let quota = quota_context(counter.clone());
    let user_service = Arc::new(UserService::new(pool.clone()));
    let ctx = users::AdminUserContext {
        user_service: user_service.clone(),
    };

    stheno_db::tenant::scope("quota_staff_route", async {
        let first = users::create_user(
            State(ctx.clone()),
            Some(Extension(quota.clone())),
            Json(users::CreateUserRequest {
                email: "one@example.test".to_string(),
                first_name: None,
                last_name: None,
                avatar_url: None,
                role: None,
                metadata: None,
            }),
        )
        .await;
        assert!(first.is_ok());

        let second = users::create_user(
            State(ctx.clone()),
            Some(Extension(quota.clone())),
            Json(users::CreateUserRequest {
                email: "two@example.test".to_string(),
                first_name: None,
                last_name: None,
                avatar_url: None,
                role: None,
                metadata: None,
            }),
        )
        .await;
        assert!(second.is_err());

        let user = ctx
            .user_service
            .list_users()
            .await
            .expect("list staff users")
            .into_iter()
            .next()
            .expect("first staff user exists");
        TestDb::execute(
            &db,
            "CREATE FUNCTION fail_user_quota_release() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF OLD.resource = 'users' THEN RAISE EXCEPTION 'injected user quota failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER fail_user_quota_release BEFORE UPDATE ON entitlement_usage FOR EACH ROW EXECUTE FUNCTION fail_user_quota_release()",
        )
        .await;
        assert!(users::delete_user(
            State(ctx.clone()),
            Some(Extension(quota.clone())),
            axum::extract::Path(user.id.clone()),
        )
        .await
        .is_err());
        assert_eq!(ctx.user_service.list_users().await.expect("user rollback").len(), 1);
        assert_eq!(counter.usage_scoped("users").await.expect("usage"), 1);
        TestDb::execute(
            &db,
            "DROP TRIGGER fail_user_quota_release ON entitlement_usage; DROP FUNCTION fail_user_quota_release()",
        )
        .await;
        assert!(users::delete_user(
            State(ctx),
            Some(Extension(quota)),
            axum::extract::Path(user.id),
        )
        .await
        .is_ok());

        let usage_counter = counter.clone();
        let usage_pool = counter.pool().clone();
        let used = stheno_db::tenant::with_tenant_conn(&usage_pool, move |conn| {
            Box::pin(async move { usage_counter.get_usage(conn, "users").await })
        })
        .await
        .expect("usage query");
        assert_eq!(used, 0);
    })
    .await;

    db.drop().await;
}

#[tokio::test]
async fn first_user_bootstrap_consumes_one_staff_seat_and_replays_without_double_counting() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let quota = quota_context(counter.clone());
    let user_service = Arc::new(UserService::new(pool.clone()));
    let ctx = users::AdminUserContext {
        user_service: user_service.clone(),
    };

    stheno_db::tenant::scope("quota_bootstrap_route", async {
        assert!(
            users::get_me(State(ctx.clone()), Some(Extension(quota.clone())))
                .await
                .is_ok()
        );
        assert!(
            users::get_me(State(ctx.clone()), Some(Extension(quota.clone())))
                .await
                .is_ok()
        );

        let usage_counter = counter.clone();
        let usage_pool = counter.pool().clone();
        let used = stheno_db::tenant::with_tenant_conn(&usage_pool, move |conn| {
            Box::pin(async move { usage_counter.get_usage(conn, "users").await })
        })
        .await
        .expect("usage query");
        assert_eq!(used, 1);
    })
    .await;

    stheno_db::tenant::scope("quota_bootstrap_full", async {
        counter
            .set_usage_scoped("users", 1)
            .await
            .expect("seed a full tenant seat counter");
        let full = quota_context_with_limits(counter.clone(), 1, 1);
        assert!(users::get_me(State(ctx), Some(Extension(full)))
            .await
            .is_err());
        assert!(user_service
            .list_users()
            .await
            .expect("list users")
            .is_empty());
        assert_eq!(counter.usage_scoped("users").await.expect("usage"), 1);
    })
    .await;

    db.drop().await;
}

#[tokio::test]
async fn invite_acceptance_consumes_quota_and_preserves_a_second_invite_when_full() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let quota = quota_context(counter.clone());
    let user_service = Arc::new(UserService::new(pool.clone()));
    let ctx = users::AdminUserContext {
        user_service: user_service.clone(),
    };

    stheno_db::tenant::scope("quota_invite_route", async {
        let first_invite = user_service
            .create_invite(stheno_user::dto::CreateInviteInput {
                email: "first-invite@example.test".to_string(),
                role: None,
                metadata: None,
            })
            .await
            .expect("create first invite");
        assert!(users::accept_invite(
            State(ctx.clone()),
            Some(Extension(quota.clone())),
            Json(users::AcceptInviteRequest {
                token: first_invite.token,
                first_name: None,
                last_name: None,
            }),
        )
        .await
        .is_ok());

        let second_invite = user_service
            .create_invite(stheno_user::dto::CreateInviteInput {
                email: "second-invite@example.test".to_string(),
                role: None,
                metadata: None,
            })
            .await
            .expect("create second invite");
        assert!(users::accept_invite(
            State(ctx),
            Some(Extension(quota.clone())),
            Json(users::AcceptInviteRequest {
                token: second_invite.token,
                first_name: None,
                last_name: None,
            }),
        )
        .await
        .is_err());

        let invites = user_service.list_invites().await.expect("list invites");
        assert!(invites
            .iter()
            .find(|invite| invite.email == "first-invite@example.test")
            .and_then(|invite| invite.accepted_at)
            .is_some());
        assert!(invites
            .iter()
            .find(|invite| invite.email == "second-invite@example.test")
            .is_some_and(|invite| invite.accepted_at.is_none()));
        assert_eq!(counter.usage_scoped("users").await.expect("usage"), 1);
    })
    .await;

    db.drop().await;
}

#[tokio::test]
async fn batch_product_route_enforces_the_quota_for_each_created_product() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let quota = quota_context(counter.clone());
    let service = Arc::new(ProductService::new(pool.clone()));

    stheno_db::tenant::scope("quota_batch_route", async {
        assert!(catalog::batch_products(
            State(service.clone()),
            Some(Extension(quota.clone())),
            Json(serde_json::json!({
                "create": [
                    {"title": "First batch product"},
                    {"title": "Over quota batch product"}
                ]
            })),
        )
        .await
        .is_ok());

        assert_eq!(
            service.list_products().await.expect("list products").len(),
            1
        );
        let usage_counter = counter.clone();
        let usage_pool = counter.pool().clone();
        let used = stheno_db::tenant::with_tenant_conn(&usage_pool, move |conn| {
            Box::pin(async move { usage_counter.get_usage(conn, "products").await })
        })
        .await
        .expect("usage query");
        assert_eq!(used, 1);

        let product_id = service
            .list_products()
            .await
            .expect("list products")
            .into_iter()
            .next()
            .expect("the first batch product exists")
            .id
            .to_string();
        TestDb::execute(
            &db,
            "CREATE FUNCTION fail_product_quota_release() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF OLD.resource = 'products' THEN RAISE EXCEPTION 'injected product quota failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER fail_product_quota_release BEFORE UPDATE ON entitlement_usage FOR EACH ROW EXECUTE FUNCTION fail_product_quota_release()",
        )
        .await;
        assert!(catalog::batch_products(
            State(service.clone()),
            Some(Extension(quota.clone())),
            Json(serde_json::json!({"delete": [product_id]})),
        )
        .await
        .is_ok());
        assert_eq!(service.list_products().await.expect("rolled-back deletion").len(), 1);
        assert_eq!(counter.usage_scoped("products").await.expect("usage"), 1);
        TestDb::execute(
            &db,
            "DROP TRIGGER fail_product_quota_release ON entitlement_usage; DROP FUNCTION fail_product_quota_release()",
        )
        .await;
        let product_id = service
            .list_products()
            .await
            .expect("product after rollback")
            .into_iter()
            .next()
            .expect("product remains active")
            .id
            .to_string();
        assert!(catalog::batch_products(
            State(service.clone()),
            Some(Extension(quota)),
            Json(serde_json::json!({"delete": [product_id]})),
        )
        .await
        .is_ok());
        assert_eq!(
            service
                .list_products()
                .await
                .expect("deleted product hidden")
                .len(),
            0
        );
        assert_eq!(counter.usage_scoped("products").await.expect("usage"), 0);
    })
    .await;

    db.drop().await;
}
