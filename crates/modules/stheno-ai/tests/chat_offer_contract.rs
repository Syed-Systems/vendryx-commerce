//! Owner-authored chat prices stay disabled until enabled and remain bound to
//! their exact conversation, event, and offer version.

use rust_decimal::Decimal;
use std::str::FromStr;
use stheno_ai::{ChatOfferService, SaveChatOffer};
use stheno_db::create_pool;
use stheno_db::diesel;
use stheno_db::diesel::sql_types::Text;
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn quote_is_conversation_bound_idempotent_and_revoked_by_offer_change() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 2).expect("pool");
    let service = ChatOfferService;

    stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
        Box::pin(async move {
            let mut conn = conn;
            let disabled = service
                .save(
                    &mut conn,
                    SaveChatOffer {
                        id: None,
                        variant_id: "variant_01TEST".to_string(),
                        currency_code: "BDT".to_string(),
                        chat_unit_price: "995".to_string(),
                        max_quantity: 2,
                        starts_at: None,
                        ends_at: None,
                        enabled: false,
                    },
                )
                .await?;
            assert!(!disabled.enabled);
            assert!(
                service
                    .quote(
                        &mut conn,
                        &disabled.variant_id,
                        1,
                        "BDT",
                        Some("reg_bd"),
                        &"a".repeat(64),
                        Decimal::from_str("1000").expect("live price"),
                        2,
                    )
                    .await?
                    .is_none()
            );

            let enabled = service
                .save(
                    &mut conn,
                    SaveChatOffer {
                        id: Some(disabled.id.clone()),
                        variant_id: disabled.variant_id.clone(),
                        currency_code: disabled.currency_code.clone(),
                        chat_unit_price: disabled.chat_unit_price.clone(),
                        max_quantity: 2,
                        starts_at: None,
                        ends_at: None,
                        enabled: true,
                    },
                )
                .await?;
            assert!(enabled.enabled);
            assert_eq!(enabled.version, disabled.version + 1);

            diesel::sql_query("SAVEPOINT overlapping_offer_case")
                .execute(&mut conn)
                .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;
            let overlapping = service
                .save(
                    &mut conn,
                    SaveChatOffer {
                        id: None,
                        variant_id: enabled.variant_id.clone(),
                        currency_code: enabled.currency_code.clone(),
                        chat_unit_price: "990".to_string(),
                        max_quantity: 2,
                        starts_at: None,
                        ends_at: None,
                        enabled: true,
                    },
                )
                .await;
            assert!(
                matches!(overlapping, Err(stheno_types::Error::Conflict { code: "ai_chat_offer_conflict", .. })),
                "two enabled policies for the same variant and currency must not coexist"
            );
            diesel::sql_query("ROLLBACK TO SAVEPOINT overlapping_offer_case")
                .execute(&mut conn)
                .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;
            diesel::sql_query("RELEASE SAVEPOINT overlapping_offer_case")
                .execute(&mut conn)
                .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;

            let conversation = "a".repeat(64);
            assert!(service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    3,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("1000").expect("live price"),
                    2,
                )
                .await?
                .is_none(), "owner quantity limit is enforced");
            assert!(service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("995").expect("same live price"),
                    2,
                )
                .await?
                .is_none(), "an offer cannot claim savings when it equals the live price");
            assert!(service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("1000.001").expect("over-precision live price"),
                    2,
                )
                .await?
                .is_none(), "prices unsupported by the store currency fail closed");
            let quote = service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("1000").expect("live price"),
                    2,
                )
                .await?
                .expect("enabled lower chat price is quoteable");
            assert_eq!(quote.chat_unit_price, "995");

            let quoted_price = service
                .validate_quote(
                    &mut conn,
                    &quote.quote_id,
                    &conversation,
                    "meta:event-1",
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    Decimal::from_str("1000").expect("live price"),
                )
                .await?;
            assert_eq!(quoted_price, Decimal::from_str("995").expect("chat price"));

            diesel::sql_query(
                "UPDATE ai.chat_offer SET starts_at = now(), ends_at = NULL WHERE id = $1",
            )
            .bind::<Text, _>(&enabled.id)
            .execute(&mut conn)
            .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;
            assert!(service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("1000").expect("live price"),
                    2,
                )
                .await?
                .is_some(), "start-time equality is eligible");
            diesel::sql_query(
                "UPDATE ai.chat_offer SET starts_at = NULL, ends_at = now() WHERE id = $1",
            )
            .bind::<Text, _>(&enabled.id)
            .execute(&mut conn)
            .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;
            assert!(service
                .quote(
                    &mut conn,
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    &conversation,
                    Decimal::from_str("1000").expect("live price"),
                    2,
                )
                .await?
                .is_none(), "end-time equality is expired");
            diesel::sql_query(
                "UPDATE ai.chat_offer SET starts_at = NULL, ends_at = NULL WHERE id = $1",
            )
            .bind::<Text, _>(&enabled.id)
            .execute(&mut conn)
            .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;

            let same_event_replay = service
                .validate_quote(
                    &mut conn,
                    &quote.quote_id,
                    &conversation,
                    "meta:event-1",
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    Decimal::from_str("1000").expect("live price"),
                )
                .await?;
            assert_eq!(same_event_replay, quoted_price);

            let different_event_replay = service
                .validate_quote(
                    &mut conn,
                    &quote.quote_id,
                    &conversation,
                    "meta:event-2",
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    Decimal::from_str("1000").expect("live price"),
                )
                .await;
            assert!(different_event_replay.is_err());

            let wrong_conversation = service
                .validate_quote(
                    &mut conn,
                    &quote.quote_id,
                    &"b".repeat(64),
                    "meta:event-1",
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                    Decimal::from_str("1000").expect("live price"),
                )
                .await;
            assert!(wrong_conversation.is_err());

            diesel::sql_query(
                "UPDATE ai.chat_offer_quote SET issued_at = now() - interval '10 minutes', expires_at = now() - interval '1 second' WHERE id = $1",
            )
            .bind::<Text, _>(&quote.quote_id)
            .execute(&mut conn)
            .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;
            let expired = service
                .validate_confirmation_quote(
                    &mut conn,
                    &quote.quote_id,
                    "meta:event-1",
                    Decimal::from_str("995").expect("chat price"),
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                )
                .await;
            assert!(expired.is_err(), "expired quote cannot authorize confirmation");
            diesel::sql_query(
                "UPDATE ai.chat_offer_quote SET issued_at = now(), expires_at = now() + interval '5 minutes' WHERE id = $1",
            )
            .bind::<Text, _>(&quote.quote_id)
            .execute(&mut conn)
            .await.map_err(|error| stheno_types::Error::Database(error.to_string()))?;

            service
                .delete_with_actor(&mut conn, &enabled.id, "user_01TEST")
                .await?;
            let revoked = service
                .validate_confirmation_quote(
                    &mut conn,
                    &quote.quote_id,
                    "meta:event-1",
                    Decimal::from_str("995").expect("chat price"),
                    &enabled.variant_id,
                    1,
                    "BDT",
                    Some("reg_bd"),
                )
                .await;
            assert!(
                revoked.is_err(),
                "disabled offer cannot authorize confirmation"
            );
            Ok(())
        })
    })
    .await
    .expect("offer quote contract");

    db.drop().await;
}
