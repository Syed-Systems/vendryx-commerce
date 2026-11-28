//! Encrypted credential storage, redacted reads, and rotation contracts.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use stheno_ai::{
    AiConfigurationService, MetricCoverage, SaveAiConfiguration, SourceMetricObservation,
    StoreIntelligenceSourceService,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";

fn secrets(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[tokio::test]
async fn provider_and_meta_secrets_are_encrypted_redacted_and_rotatable() {
    let _guard = ENV_LOCK.lock().await;
    let previous = std::env::var("STHENO_CREDENTIAL_KEY").ok();
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AiConfigurationService::new(pool.clone());

    stheno_db::tenant::scope("ten_ai_one", async {
        let saved = service
            .save(SaveAiConfiguration {
                kind: "llm".to_string(),
                account_id: "default".to_string(),
                secrets: secrets(&[
                    ("provider", "deepseek"),
                    ("model", "deepseek-chat"),
                    ("api_key", "first-provider-secret"),
                ]),
                enabled: true,
            })
            .await
            .expect("save provider secrets");
        assert!(saved.configured && saved.enabled);
        assert_eq!(saved.version, 1);
        let encoded = serde_json::to_value(&saved).expect("status JSON");
        assert!(!encoded.to_string().contains("first-provider-secret"));
        let runtime = service
            .load_provider()
            .await
            .expect("load provider")
            .expect("enabled provider");
        assert_eq!(
            runtime.endpoint,
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(runtime.api_key, "first-provider-secret");

        let rotated = service
            .save(SaveAiConfiguration {
                kind: "llm".to_string(),
                account_id: "default".to_string(),
                secrets: secrets(&[("api_key", "rotated-provider-secret")]),
                enabled: true,
            })
            .await
            .expect("rotate provider secret");
        assert_eq!(rotated.version, 2);
        let runtime = service
            .load_provider()
            .await
            .expect("load rotated provider")
            .expect("configured provider");
        assert_eq!(runtime.api_key, "rotated-provider-secret");

        let meta = service
            .save(SaveAiConfiguration {
                kind: "whatsapp".to_string(),
                account_id: "phone_123".to_string(),
                secrets: secrets(&[
                    ("app_secret", "meta-app-secret"),
                    ("verify_token", "meta-verify-token"),
                    ("access_token", "meta-access-token"),
                ]),
                enabled: true,
            })
            .await
            .expect("save Meta secrets");
        let webhook_id = meta.webhook_id.expect("public webhook path id");
        assert_eq!(webhook_id.len(), 26);
        assert!(webhook_id.bytes().all(|byte| byte.is_ascii_alphanumeric()));
        assert_eq!(
            service
                .resolve_webhook_owner(&webhook_id)
                .await
                .expect("resolve opaque webhook owner"),
            Some((
                "ten_ai_one".to_string(),
                "whatsapp".to_string(),
                "phone_123".to_string()
            ))
        );
        let loaded = service
            .load_meta_webhook("ten_ai_one", &webhook_id)
            .await
            .expect("load webhook credentials")
            .expect("active WhatsApp connection");
        assert_eq!(loaded.app_secret, "meta-app-secret");
        assert_eq!(loaded.verify_token, "meta-verify-token");
        assert_eq!(loaded.access_token, "meta-access-token");

        assert!(service
            .load_meta_insights("messenger", "phone_123")
            .await
            .expect("messaging credentials are not insights credentials")
            .is_none());
        service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "meta_insights_facebook".to_string(),
                    account_id: "998".to_string(),
                    secrets: secrets(&[
                        ("app_secret", "insights-app-secret"),
                        ("access_token", "insights-access-token"),
                    ]),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("save dedicated Facebook insights credentials");
        let insights_runtime = service
            .load_meta_insights("messenger", "998")
            .await
            .expect("load separate Meta insights credentials")
            .expect("connected insights account");
        assert_eq!(insights_runtime.channel, "messenger");
        assert_eq!(insights_runtime.app_secret, "insights-app-secret");
        assert_eq!(insights_runtime.access_token, "insights-access-token");
        service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "meta_insights_facebook".to_string(),
                    account_id: "998".to_string(),
                    secrets: BTreeMap::new(),
                    enabled: false,
                },
                "usr_owner",
            )
            .await
            .expect("disable the insights connection without deleting credentials");
        assert!(service
            .load_meta_insights("messenger", "998")
            .await
            .expect("read disabled insights connection")
            .is_none());
        service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "meta_insights_facebook".to_string(),
                    account_id: "998".to_string(),
                    secrets: BTreeMap::new(),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("re-enable saved insights credentials");
        assert!(service
            .load_meta_insights("messenger", "998")
            .await
            .expect("load re-enabled insights account")
            .is_some());

        let property = "sc-domain:shop.example.com";
        let search_console = service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "search_console".to_string(),
                    account_id: property.to_string(),
                    secrets: secrets(&[
                        ("client_id", "google-client-id"),
                        ("client_secret", "google-client-secret"),
                        ("refresh_token", "google-refresh-token"),
                    ]),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("save Search Console credentials");
        assert!(search_console.webhook_id.is_none());
        let runtime = service
            .load_search_console(property)
            .await
            .expect("load Search Console credentials")
            .expect("enabled Search Console source");
        assert_eq!(runtime.client_id, "google-client-id");
        assert_eq!(runtime.client_secret, "google-client-secret");
        assert_eq!(runtime.refresh_token, "google-refresh-token");
        let list = service.list().await.expect("redacted configuration list");
        assert_eq!(list.len(), 4);
        assert!(serde_json::to_string(&list)
            .expect("status JSON")
            .find("meta-access-token")
            .is_none());
        assert!(serde_json::to_string(&list)
            .expect("status JSON")
            .find("google-refresh-token")
            .is_none());
        let observation = SourceMetricObservation {
            tenant_id: "ten_ai_one".to_string(),
            source: "search_console".to_string(),
            account_id: property.to_string(),
            scope: "merchant_property".to_string(),
            metric: "clicks".to_string(),
            definition: "search-console-clicks-v1".to_string(),
            unit: "count".to_string(),
            value: Decimal::from(12),
            denominator: None,
            sample_size: Some(7),
            window_started_on: NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
            window_ended_on: NaiveDate::from_ymd_opt(2026, 10, 8).expect("date"),
            window_timezone: "America/Los_Angeles".to_string(),
            ingested_at: Utc::now(),
            coverage: MetricCoverage::Complete,
        };
        let mut prior_observation = observation.clone();
        prior_observation.value = Decimal::from(10);
        prior_observation.ingested_at -= chrono::TimeDelta::seconds(1);
        prior_observation.window_started_on = NaiveDate::from_ymd_opt(2026, 9, 24).expect("date");
        prior_observation.window_ended_on = NaiveDate::from_ymd_opt(2026, 10, 1).expect("date");
        let observations = vec![prior_observation, observation];
        let mut facebook_observation = observations[1].clone();
        facebook_observation.source = "facebook_page".to_string();
        facebook_observation.account_id = "998".to_string();
        facebook_observation.scope = "meta_page".to_string();
        let facebook_observations = vec![facebook_observation];
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .record_snapshot(&mut conn, "facebook_page", "998", &facebook_observations)
                    .await?;
                Ok(())
            })
        })
        .await
        .expect("persist Facebook insights snapshot");
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .record_snapshot(&mut conn, "search_console", property, &observations)
                    .await?;
                Ok(())
            })
        })
        .await
        .expect("persist current source snapshot");
        let snapshots = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .list_recent(&mut conn, "search_console", property)
                    .await
            })
        })
        .await
        .expect("read source snapshot");
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].value, Decimal::from(12));
        assert_eq!(snapshots[1].value, Decimal::from(10));
        service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "search_console".to_string(),
                    account_id: property.to_string(),
                    secrets: BTreeMap::new(),
                    enabled: false,
                },
                "usr_owner",
            )
            .await
            .expect("disable Search Console source");
        service
            .save_with_actor(
                SaveAiConfiguration {
                    kind: "search_console".to_string(),
                    account_id: property.to_string(),
                    secrets: BTreeMap::new(),
                    enabled: true,
                },
                "usr_owner",
            )
            .await
            .expect("re-enable Search Console source");
        service
            .disconnect_search_console_with_actor(property, "usr_owner")
            .await
            .expect("disconnect Search Console source");
        assert!(service
            .load_search_console(property)
            .await
            .expect("check disconnected source")
            .is_none());
        let snapshots = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .list_recent(&mut conn, "search_console", property)
                    .await
            })
        })
        .await
        .expect("check disconnected snapshot");
        assert!(snapshots.is_empty());
        let history = service
            .list_search_console_audit(property, 10)
            .await
            .expect("source connection audit");
        assert_eq!(history.len(), 4);
        for action in ["connected", "updated", "disabled", "disconnected"] {
            assert!(history.iter().any(|entry| entry.action == action));
        }
        assert!(history.iter().all(|entry| entry.actor_id == "usr_owner"));

        service
            .disconnect_meta_insights_with_actor("messenger", "998", "usr_owner")
            .await
            .expect("disconnect Facebook insights and delete saved history");
        assert!(service
            .load_meta_insights("messenger", "998")
            .await
            .expect("check disconnected Facebook insights")
            .is_none());
        let facebook_history = stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .list_recent(&mut conn, "facebook_page", "998")
                    .await
            })
        })
        .await
        .expect("read deleted Facebook snapshot");
        assert!(facebook_history.is_empty());
        let meta_audit = service
            .list_meta_insights_audit("messenger", "998", 10)
            .await
            .expect("Facebook insights connection audit");
        assert!(meta_audit.iter().any(|entry| entry.action == "connected"));
        assert!(meta_audit
            .iter()
            .any(|entry| entry.action == "disconnected"));
        assert!(meta_audit.iter().any(|entry| entry.action == "disabled"));
        assert!(meta_audit.iter().all(|entry| entry.actor_id == "usr_owner"));
    })
    .await;

    let other_tenant = stheno_db::tenant::scope("ten_ai_two", async {
        service
            .load_meta_webhook("ten_ai_two", "ten_ai_one--opaque")
            .await
            .expect("foreign hook is invisible")
            .is_none()
    })
    .await;
    assert!(other_tenant);
    let foreign_snapshots = stheno_db::tenant::scope("ten_ai_two", async {
        stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                StoreIntelligenceSourceService
                    .list_recent(&mut conn, "search_console", "sc-domain:shop.example.com")
                    .await
            })
        })
        .await
        .expect("foreign tenant cannot read Search Console observations")
    })
    .await;
    assert!(foreign_snapshots.is_empty());
    let foreign_audit = stheno_db::tenant::scope("ten_ai_two", async {
        service
            .list_search_console_audit("sc-domain:shop.example.com", 10)
            .await
            .expect("foreign tenant source audit is empty")
    })
    .await;
    assert!(foreign_audit.is_empty());
    let oauth_nonce = "01JBCDEFGHJKMNPQRSTVWXYZ12";
    stheno_db::tenant::scope("ten_ai_one", async {
        service
            .create_search_console_oauth_state(
                oauth_nonce,
                "sc-domain:shop.example.com",
                "usr_owner",
            )
            .await
            .expect("create one-time OAuth state");
    })
    .await;
    let foreign_oauth_state = stheno_db::tenant::scope("ten_ai_two", async {
        service
            .consume_search_console_oauth_state(
                oauth_nonce,
                "sc-domain:shop.example.com",
                "usr_owner",
            )
            .await
            .expect("foreign tenant cannot consume OAuth state")
    })
    .await;
    assert!(!foreign_oauth_state);
    stheno_db::tenant::scope("ten_ai_one", async {
        assert!(service
            .consume_search_console_oauth_state(
                oauth_nonce,
                "sc-domain:shop.example.com",
                "usr_owner",
            )
            .await
            .expect("consume owner OAuth state"));
        assert!(!service
            .consume_search_console_oauth_state(
                oauth_nonce,
                "sc-domain:shop.example.com",
                "usr_owner",
            )
            .await
            .expect("OAuth state is one-time"));
    })
    .await;
    db.drop().await;
    if let Some(value) = previous {
        std::env::set_var("STHENO_CREDENTIAL_KEY", value);
    } else {
        std::env::remove_var("STHENO_CREDENTIAL_KEY");
    }
}

#[tokio::test]
async fn unsupported_fields_and_incomplete_enabled_configs_fail_closed() {
    let _guard = ENV_LOCK.lock().await;
    let previous = std::env::var("STHENO_CREDENTIAL_KEY").ok();
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY);
    let db = TestDb::migrated().await;
    let service = AiConfigurationService::new(create_pool(&db.url, 5).expect("pool"));
    stheno_db::tenant::scope("ten_ai_invalid", async {
        let empty_disabled = service
            .save(SaveAiConfiguration {
                kind: "llm".to_string(),
                account_id: "default".to_string(),
                secrets: BTreeMap::new(),
                enabled: false,
            })
            .await
            .expect("empty disabled config is allowed");
        assert!(!empty_disabled.configured && !empty_disabled.enabled);

        assert!(service
            .save(SaveAiConfiguration {
                kind: "llm".to_string(),
                account_id: "default".to_string(),
                secrets: secrets(&[("api_key", "key"), ("endpoint", "http://internal")]),
                enabled: true,
            })
            .await
            .is_err());
        assert!(service
            .save(SaveAiConfiguration {
                kind: "messenger".to_string(),
                account_id: "page_1".to_string(),
                secrets: secrets(&[("app_secret", "secret")]),
                enabled: true,
            })
            .await
            .is_err());
    })
    .await;
    db.drop().await;
    if let Some(value) = previous {
        std::env::set_var("STHENO_CREDENTIAL_KEY", value);
    } else {
        std::env::remove_var("STHENO_CREDENTIAL_KEY");
    }
}
