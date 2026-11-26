//! ADR-035 Meta source credentials and history are owner/admin controls.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use stheno_ai::{
    AiConfigurationService, MetricCoverage, SaveAiConfiguration, SourceMetricObservation,
    StoreIntelligenceReviewService, StoreIntelligenceSourceService,
};
use stheno_notification::NotificationService;
use stheno_platform::{NewTenant, PlatformService, SubscriptionPatch};
use stheno_query::query_tenant_store_intelligence_export;
use stheno_server::TestApp;
use stheno_user::{CreateUserInput, UserService};
use tower::ServiceExt;

const PLATFORM_TOKEN: &str = "meta-insights-platform-token";
const CREDENTIAL_KEY: &str = "0123456789abcdef0123456789abcdef";
static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn secrets(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body reads");
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

async fn register_user(app: &axum::Router, email: &str, actor_id: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user/emailpass/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "Sup3rSecret!",
                        "actor_id": actor_id,
                    })
                    .to_string(),
                ))
                .expect("registration request"),
        )
        .await
        .expect("registration responds");
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("registration response reads");
    serde_json::from_slice::<Value>(&body).expect("registration returns JSON")["token"]
        .as_str()
        .expect("registration issues a token")
        .to_string()
}

#[tokio::test]
async fn meta_insights_audit_rejects_anonymous_customer_and_non_owner_staff() {
    let _guard = ENV_LOCK.lock().await;
    let previous_tenant = std::env::var("STHENO_TENANT").ok();
    std::env::set_var("STHENO_TENANT", stheno_db::testing::TestDb::TEST_TENANT);
    let harness = TestApp::spawn().await;
    match previous_tenant {
        Some(tenant) => std::env::set_var("STHENO_TENANT", tenant),
        None => std::env::remove_var("STHENO_TENANT"),
    }
    let pool = stheno_db::create_pool(&harness.database_url, 5).expect("test pool");
    PlatformService::new(pool.clone())
        .update_subscription(
            stheno_db::testing::TestDb::TEST_TENANT,
            SubscriptionPatch {
                feature_overrides: Some(json!({ "ai_assistant": true })),
                ..Default::default()
            },
        )
        .await
        .expect("enable the AI route for authorization coverage");

    let path = "/admin/ai/meta-insights/audit?channel=messenger&account_id=998";
    let (anonymous_status, _) = call(
        &harness.app,
        Request::builder()
            .uri(path)
            .body(Body::empty())
            .expect("anonymous request"),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);

    let (customer_status, customer_body) = call(
        &harness.app,
        Request::builder()
            .method("POST")
            .uri("/auth/customer/emailpass/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"email":"meta-customer@example.test","password":"Sup3rSecret!"}"#,
            ))
            .expect("customer registration request"),
    )
    .await;
    assert_eq!(customer_status, StatusCode::OK);
    let customer_token = customer_body["token"].as_str().expect("customer token");
    let (customer_audit_status, _) = call(
        &harness.app,
        Request::builder()
            .uri(path)
            .header("authorization", format!("Bearer {customer_token}"))
            .body(Body::empty())
            .expect("customer audit request"),
    )
    .await;
    assert_eq!(customer_audit_status, StatusCode::FORBIDDEN);

    let users = UserService::new(pool);
    let staff = stheno_db::tenant::scope(
        stheno_db::testing::TestDb::TEST_TENANT,
        users.create_user(CreateUserInput {
            email: "meta-staff@example.test".to_string(),
            first_name: None,
            last_name: None,
            avatar_url: None,
            role: Some("staff".to_string()),
            metadata: None,
        }),
    )
    .await
    .expect("staff user created");
    let staff_token = register_user(&harness.app, &staff.email, &staff.id).await;
    let (staff_audit_status, _) = call(
        &harness.app,
        Request::builder()
            .uri(path)
            .header("authorization", format!("Bearer {staff_token}"))
            .body(Body::empty())
            .expect("staff audit request"),
    )
    .await;
    assert_eq!(staff_audit_status, StatusCode::FORBIDDEN);

    let owner = stheno_db::tenant::scope(
        stheno_db::testing::TestDb::TEST_TENANT,
        users.create_user(CreateUserInput {
            email: "meta-owner@example.test".to_string(),
            first_name: None,
            last_name: None,
            avatar_url: None,
            role: Some("owner".to_string()),
            metadata: None,
        }),
    )
    .await
    .expect("owner user created");
    let owner_token = register_user(&harness.app, &owner.email, &owner.id).await;
    let (owner_audit_status, owner_body) = call(
        &harness.app,
        Request::builder()
            .uri(path)
            .header("authorization", format!("Bearer {owner_token}"))
            .body(Body::empty())
            .expect("owner audit request"),
    )
    .await;
    assert_eq!(
        owner_audit_status,
        StatusCode::OK,
        "owner is admitted: {owner_body}"
    );
    assert_eq!(owner_body["history"], json!([]));

    harness.teardown().await;
}

#[tokio::test]
async fn platform_export_and_purge_include_store_intelligence_data() {
    let _guard = ENV_LOCK.lock().await;
    let previous_tenant = std::env::var("STHENO_TENANT").ok();
    let previous_platform_token = std::env::var("STHENO_PLATFORM_TOKEN").ok();
    let previous_credential_key = std::env::var("STHENO_CREDENTIAL_KEY").ok();
    std::env::set_var("STHENO_TENANT", stheno_db::testing::TestDb::TEST_TENANT);
    std::env::set_var("STHENO_PLATFORM_TOKEN", PLATFORM_TOKEN);
    std::env::set_var("STHENO_CREDENTIAL_KEY", CREDENTIAL_KEY);
    let harness = TestApp::spawn().await;

    let tenant_id = "ten_source_lifecycle";
    let pool = stheno_db::create_pool(&harness.database_url, 5).expect("test pool");
    let platform = PlatformService::new(pool.clone());
    platform
        .create_tenant(NewTenant {
            id: tenant_id.to_string(),
            name: "Source Lifecycle Merchant".to_string(),
            tier: "pro".to_string(),
            domain: Some("source-lifecycle.localhost".to_string()),
        })
        .await
        .expect("tenant created");
    let isolated_tenant_id = "ten_source_isolated";
    platform
        .create_tenant(NewTenant {
            id: isolated_tenant_id.to_string(),
            name: "Isolated Source Merchant".to_string(),
            tier: "pro".to_string(),
            domain: Some("source-isolated.localhost".to_string()),
        })
        .await
        .expect("isolated tenant created");

    let configuration = AiConfigurationService::new(pool.clone());
    let notification_service = NotificationService::new(pool.clone());
    stheno_db::tenant::scope(tenant_id, async {
        configuration
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "meta_insights_facebook".to_string(),
                    account_id: "998".to_string(),
                    secrets: secrets(&[
                        ("app_secret", "source-lifecycle-app-secret"),
                        ("access_token", "source-lifecycle-private-token"),
                    ]),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("Meta insights connection saved");
        let observation = SourceMetricObservation {
            tenant_id: tenant_id.to_string(),
            source: "facebook_page".to_string(),
            account_id: "998".to_string(),
            scope: "page".to_string(),
            metric: "page_post_engagements".to_string(),
            definition: "meta-page-post-engagements-v1".to_string(),
            unit: "count".to_string(),
            value: Decimal::from(11),
            denominator: None,
            sample_size: Some(11),
            window_started_on: chrono::NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
            window_ended_on: chrono::NaiveDate::from_ymd_opt(2026, 10, 8).expect("date"),
            window_timezone: "UTC".to_string(),
            ingested_at: chrono::Utc::now(),
            coverage: MetricCoverage::Complete,
        };
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .record_snapshot(&mut conn, "facebook_page", "998", &[observation])
                    .await?;
                let day = chrono::Utc::now().timestamp().div_euclid(86_400) - 7;
                StoreIntelligenceReviewService
                    .record(
                        &mut conn,
                        &format!("{day}:product:prod_export:review_feature"),
                        "approved",
                        "usr_owner",
                        "request-export-1",
                    )
                    .await?;
                Ok(())
            })
        })
        .await
        .expect("source observation saved");
        notification_service
            .record_whatsapp_analytics_consent(
                tenant_id,
                "+8801712345678",
                Some("cus_exported"),
                "order-exported",
            )
            .await
            .expect("analytics consent saved");
    })
    .await;
    stheno_db::tenant::scope(isolated_tenant_id, async {
        configuration
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "meta_insights_facebook".to_string(),
                    account_id: "997".to_string(),
                    secrets: secrets(&[
                        ("app_secret", "isolated-app-secret"),
                        ("access_token", "isolated-private-token"),
                    ]),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("isolated tenant source saved");
        notification_service
            .record_whatsapp_analytics_consent(
                isolated_tenant_id,
                "+8801712345679",
                Some("cus_isolated"),
                "order-isolated",
            )
            .await
            .expect("isolated analytics consent saved");
    })
    .await;

    let (export_status, export) = call(
        &harness.app,
        Request::builder()
            .uri(format!("/platform/tenants/{tenant_id}/export"))
            .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
            .body(Body::empty())
            .expect("tenant export request"),
    )
    .await;
    assert_eq!(export_status, StatusCode::OK, "export succeeds: {export}");
    assert_eq!(
        export["export"]["store_intelligence"]["connections"]
            .as_array()
            .expect("source connections")
            .len(),
        1,
        "export contains only the requested tenant's source configuration"
    );
    assert_eq!(
        export["export"]["store_intelligence"]["connection_audit"]
            .as_array()
            .expect("source connection audit")
            .len(),
        1,
        "export contains only the requested tenant's source audit"
    );
    assert_eq!(
        export["export"]["store_intelligence"]["whatsapp_analytics_consents"]
            .as_array()
            .expect("analytics consent ledger")
            .len(),
        1,
        "export contains only the requested tenant's consent state"
    );
    assert_eq!(
        export["export"]["store_intelligence"]["recommendation_reviews"]
            .as_array()
            .expect("recommendation review state")
            .len(),
        1
    );
    assert_eq!(
        export["export"]["store_intelligence"]["recommendation_review_audit"]
            .as_array()
            .expect("recommendation review audit")
            .len(),
        1
    );
    assert_eq!(
        export["export"]["store_intelligence"]["connections"][0]["source"],
        "facebook_page"
    );
    assert_eq!(
        export["export"]["store_intelligence"]["observations"][0]["value"],
        "11"
    );
    assert_eq!(
        export["export"]["store_intelligence"]["whatsapp_analytics_consents"][0]["customer_id"],
        "cus_exported"
    );
    let export_json = export.to_string();
    assert!(!export_json.contains("source-lifecycle-private-token"));
    assert!(!export_json.contains("source-lifecycle-app-secret"));

    platform
        .set_tenant_status(tenant_id, "closed", "tenant erasure retention test")
        .await
        .expect("tenant closed");
    let (purge_status, purge) = call(
        &harness.app,
        Request::builder()
            .method("POST")
            .uri("/platform/tenants/purge")
            .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "retention_days": 0 }).to_string()))
            .expect("tenant purge request"),
    )
    .await;
    assert_eq!(purge_status, StatusCode::OK, "purge succeeds: {purge}");
    assert_eq!(purge["purged_count"], 1);
    assert!(platform.get_tenant(tenant_id).await.is_err());

    let remaining = stheno_db::tenant::scope(tenant_id, async {
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .list_recent(&mut conn, "facebook_page", "998")
                    .await
            })
        })
        .await
    })
    .await
    .expect("tenant source rows remain readable under their scoped RLS policy");
    assert!(remaining.is_empty(), "purge removed retained source rows");

    let isolated_export = query_tenant_store_intelligence_export(&pool, isolated_tenant_id)
        .await
        .expect("purging one tenant preserves another tenant's source data");
    assert_eq!(isolated_export.connections.len(), 1);
    assert!(isolated_export.recommendation_reviews.is_empty());
    assert!(isolated_export.recommendation_review_audit.is_empty());
    assert_eq!(isolated_export.connections[0].account_id, "997");
    assert_eq!(isolated_export.whatsapp_analytics_consents.len(), 1);
    assert_eq!(
        isolated_export.whatsapp_analytics_consents[0]
            .customer_id
            .as_deref(),
        Some("cus_isolated")
    );

    let erased = query_tenant_store_intelligence_export(&pool, tenant_id)
        .await
        .expect("tenant analytics consents remain readable under its isolated scope");
    assert!(erased.connections.is_empty());
    assert!(erased.observations.is_empty());
    assert!(erased.connection_audit.is_empty());
    assert!(erased.recommendation_reviews.is_empty());
    assert!(erased.recommendation_review_audit.is_empty());
    assert!(
        erased.whatsapp_analytics_consents.is_empty(),
        "purge removed consent ledger rows"
    );

    harness.teardown().await;
    match previous_tenant {
        Some(tenant) => std::env::set_var("STHENO_TENANT", tenant),
        None => std::env::remove_var("STHENO_TENANT"),
    }
    match previous_platform_token {
        Some(token) => std::env::set_var("STHENO_PLATFORM_TOKEN", token),
        None => std::env::remove_var("STHENO_PLATFORM_TOKEN"),
    }
    match previous_credential_key {
        Some(key) => std::env::set_var("STHENO_CREDENTIAL_KEY", key),
        None => std::env::remove_var("STHENO_CREDENTIAL_KEY"),
    }
}
