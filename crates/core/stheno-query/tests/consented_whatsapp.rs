use chrono::{Days, Duration, Utc};
use sha2::{Digest, Sha256};
use stheno_db::diesel::sql_types::{Text, Timestamptz};
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, diesel, diesel_async::RunQueryDsl};
use stheno_query::query_consented_whatsapp_aggregate;

#[tokio::test]
async fn aggregate_includes_only_processed_events_during_active_whatsapp_consent() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let tenant_id = "ten_wa_consent";
    let account_id = "phone_number_123";
    let start = Utc::now().date_naive() - Days::new(2);
    let start_at = start.and_hms_opt(0, 0, 0).expect("start").and_utc();
    let active_hash = format!("{:x}", Sha256::digest("8801712345678"));
    let revoked_hash = format!("{:x}", Sha256::digest("8801812345678"));
    let late_hash = format!("{:x}", Sha256::digest("8801912345678"));
    let unknown_hash = format!("{:x}", Sha256::digest("8801012345678"));
    let event_at = start_at + Duration::hours(5);

    stheno_db::tenant::scope(tenant_id, async {
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                for (recipient_hash, source_event_id, consented, consented_at, revoked_at) in [
                    (active_hash.as_str(), "order-active", true, start_at, None),
                    (
                        revoked_hash.as_str(),
                        "order-revoked",
                        false,
                        start_at,
                        Some(event_at),
                    ),
                    (
                        late_hash.as_str(),
                        "order-late",
                        true,
                        event_at + Duration::hours(1),
                        None,
                    ),
                ] {
                    diesel::sql_query(
                        "INSERT INTO notification.whatsapp_analytics_consent \
                         (tenant_id, recipient_hash, source_event_id, consented, consent_source, consented_at, revoked_at) \
                         VALUES ($1, $2, $3, $4, 'checkout', $5, $6)",
                    )
                    .bind::<Text, _>(tenant_id)
                    .bind::<Text, _>(recipient_hash)
                    .bind::<Text, _>(source_event_id)
                    .bind::<diesel::sql_types::Bool, _>(consented)
                    .bind::<Timestamptz, _>(consented_at)
                    .bind::<diesel::sql_types::Nullable<Timestamptz>, _>(revoked_at)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| stheno_types::Error::Database(error.to_string()))?;
                }
                for (event_id, hash, created_at) in [
                    ("active-1", active_hash.as_str(), event_at),
                    ("active-2", active_hash.as_str(), event_at + Duration::minutes(1)),
                    ("revoked", revoked_hash.as_str(), event_at),
                    ("before-consent", late_hash.as_str(), event_at),
                    ("unknown", unknown_hash.as_str(), event_at),
                ] {
                    diesel::sql_query(
                        "INSERT INTO ai.channel_event \
                         (id, channel, external_account_id, event_id, conversation_key, provider_created_at, state) \
                         VALUES ($1, 'whatsapp', $2, $3, $4, $5, 'processed')",
                    )
                    .bind::<Text, _>(format!("event_{event_id}"))
                    .bind::<Text, _>(account_id)
                    .bind::<Text, _>(event_id)
                    .bind::<Text, _>(hash)
                    .bind::<Timestamptz, _>(created_at)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| stheno_types::Error::Database(error.to_string()))?;
                }
                Ok::<_, stheno_types::Error>(())
            })
        })
        .await
        .expect("insert consent and event fixtures");
    })
    .await;

    let aggregate = stheno_db::tenant::scope(tenant_id, async {
        query_consented_whatsapp_aggregate(
            &pool,
            tenant_id,
            account_id,
            start,
            start + Days::new(1),
        )
        .await
        .expect("aggregate WhatsApp events")
    })
    .await
    .expect("active consent has processed events");
    assert_eq!(aggregate.inbound_events, 2);
    assert_eq!(aggregate.consented_conversations, 1);
    assert_eq!(aggregate.window_timezone, "UTC");
    assert_eq!(aggregate.window_ended_on, start + Days::new(2));
    db.drop().await;
}

#[test]
fn consent_hash_matches_webhook_event_key_for_e164_sender_digits() {
    let phone = "+8801712345678";
    let consent_hash = format!("{:x}", Sha256::digest(phone.trim_start_matches('+')));
    let event_hash = format!("{:x}", Sha256::digest("8801712345678"));
    assert_eq!(consent_hash, event_hash);
}
