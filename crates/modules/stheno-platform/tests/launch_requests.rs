//! Merchant account and launch request invariants: no trial request before verified email,
//! idempotent submission, reserved subdomains, and Pro payment before provisioning.

use chrono::Utc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{PlatformService, SubmitLaunchRequestInput};

fn request(merchant_id: &str, tier: &str) -> SubmitLaunchRequestInput {
    SubmitLaunchRequestInput {
        merchant_id: merchant_id.to_string(),
        store_name: "Pahar Goods".to_string(),
        subdomain: "pahar-goods".to_string(),
        tier: tier.to_string(),
        logo_url: None,
        accent_color: "#BEE75C".to_string(),
        icon_choice: "rise".to_string(),
        buyer_accounts: false,
        bespoke_brief: None,
        bespoke_assets: Vec::new(),
    }
}

#[tokio::test]
async fn launch_request_requires_verified_email_and_is_idempotent() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));

    service
        .upsert_merchant_account(
            "auth_unverified".to_string(),
            "owner@example.com".to_string(),
            None,
            serde_json::json!({}),
            "free".to_string(),
        )
        .await
        .expect("draft account");
    let error = service
        .submit_launch_request(request("auth_unverified", "free"))
        .await
        .expect_err("unverified account must not submit");
    assert!(matches!(
        error,
        stheno_types::Error::Invalid {
            code: "email_verification_required",
            ..
        }
    ));

    service
        .upsert_merchant_account(
            "auth_verified".to_string(),
            "owner@example.com".to_string(),
            Some(Utc::now()),
            serde_json::json!({ "business_name": "Pahar Goods" }),
            "free".to_string(),
        )
        .await
        .expect("verified account");
    let first = service
        .submit_launch_request(request("auth_verified", "free"))
        .await
        .expect("first request");
    let second = service
        .submit_launch_request(request("auth_verified", "free"))
        .await
        .expect("repeat submission returns the original request");

    assert_eq!(first.id, second.id);
    assert_eq!(first.launch_status, "submitted");
    assert_eq!(first.payment_status, "not_required");
    assert_eq!(
        service
            .list_launch_request_events(&first.id)
            .await
            .expect("timeline")
            .len(),
        1,
        "repeat submission must not start a second trial or timeline"
    );
}

#[tokio::test]
async fn subdomain_is_reserved_and_pro_cannot_provision_before_payment() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));
    for id in ["auth_one", "auth_two"] {
        service
            .upsert_merchant_account(
                id.to_string(),
                format!("{id}@example.com"),
                Some(Utc::now()),
                serde_json::json!({}),
                "pro".to_string(),
            )
            .await
            .expect("verified account");
    }

    let pro = service
        .submit_launch_request(request("auth_one", "pro"))
        .await
        .expect("Pro request");
    assert_eq!(pro.payment_status, "pending");
    let error = service
        .transition_launch_request(
            &pro.id,
            "begin_provisioning",
            "op_1".to_string(),
            "Operator".to_string(),
            None,
            None,
        )
        .await
        .expect_err("unpaid Pro request must not provision");
    assert!(matches!(
        error,
        stheno_types::Error::Invalid {
            code: "pro_payment_required",
            ..
        }
    ));

    let error = service
        .submit_launch_request(request("auth_two", "free"))
        .await
        .expect_err("other merchant cannot reserve the same subdomain");
    assert!(matches!(
        error,
        stheno_types::Error::Invalid {
            code: "subdomain_taken",
            ..
        }
    ));
}

#[tokio::test]
async fn provisioning_is_idempotent_binds_the_owner_and_hands_off_only_when_activated() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));

    service
        .upsert_merchant_account(
            "auth_owner".to_string(),
            "owner@example.com".to_string(),
            Some(Utc::now()),
            serde_json::json!({ "business_name": "Pahar Goods" }),
            "free".to_string(),
        )
        .await
        .expect("verified account");
    let request = service
        .submit_launch_request(request("auth_owner", "free"))
        .await
        .expect("launch request");

    service
        .transition_launch_request(
            &request.id,
            "begin_provisioning",
            "op_1".to_string(),
            "Operator".to_string(),
            None,
            None,
        )
        .await
        .expect("begin provisioning");

    let first = service
        .provision_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("first provisioning run");
    let second = service
        .provision_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("retry returns the same durable handoff");

    assert_eq!(first.tenant_id, second.tenant_id);
    assert_eq!(first.launch_status, "provisioning");
    let tenant_id = first.tenant_id.expect("provisioning binds a tenant");
    assert_eq!(
        service.get_tenant(&tenant_id).await.expect("tenant").status,
        "pending_launch",
        "a provisioned tenant cannot serve before explicit activation"
    );
    assert!(
        service
            .merchant_owns_tenant("auth_owner", &tenant_id)
            .await
            .expect("owner binding reads"),
        "the authenticated merchant owns the deployment before handoff"
    );
    let subscription = service
        .get_subscription(&tenant_id)
        .await
        .expect("subscription");
    assert_eq!(subscription.tier, "trial");
    assert!(subscription.trial_started_at.is_some());
    assert!(subscription.trial_ends_at.is_some());

    let online = service
        .activate_launch_request(&request.id, "op_1", "Operator")
        .await
        .expect("activate after provisioning");
    assert_eq!(online.launch_status, "online");
    assert_eq!(
        service
            .get_tenant(&tenant_id)
            .await
            .expect("online tenant")
            .status,
        "active"
    );
}

#[tokio::test]
async fn concurrent_provisioning_reuses_the_same_tenant_handoff() {
    let db = TestDb::migrated().await;
    let service = PlatformService::new(create_pool(&db.url, 5).expect("pool"));
    service
        .upsert_merchant_account(
            "auth_race".to_string(),
            "race@example.com".to_string(),
            Some(Utc::now()),
            serde_json::json!({}),
            "free".to_string(),
        )
        .await
        .expect("verified account");
    let mut input = request("auth_race", "free");
    input.subdomain = "pahar-race".to_string();
    let request = service.submit_launch_request(input).await.expect("request");
    service
        .transition_launch_request(
            &request.id,
            "begin_provisioning",
            "op_1".to_string(),
            "Operator".to_string(),
            None,
            None,
        )
        .await
        .expect("begin provisioning");

    let (left, right) = tokio::join!(
        service.provision_launch_request(&request.id, "op_1", "Operator"),
        service.provision_launch_request(&request.id, "op_1", "Operator")
    );
    let left = left.expect("first concurrent provisioning call");
    let right = right.expect("second concurrent provisioning call");
    assert_eq!(left.tenant_id, right.tenant_id);
}
