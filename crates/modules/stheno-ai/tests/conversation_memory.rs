//! Encrypted short-lived multi-turn conversation contracts.

use stheno_ai::ConversationMemoryService;
use stheno_db::create_pool;
use stheno_db::diesel::sql_types::{Binary, Text};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";

#[derive(QueryableByName)]
struct CiphertextRow {
    #[diesel(sql_type = Binary)]
    ciphertext: Vec<u8>,
}

#[tokio::test]
async fn history_is_encrypted_bounded_deduplicated_and_tenant_isolated() {
    let _guard = ENV_LOCK.lock().await;
    let previous = std::env::var("STHENO_CREDENTIAL_KEY").ok();
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let memory = ConversationMemoryService::new(pool.clone());
    let user = stheno_db::tenant::scope("ten_memory_one", async {
        let first = memory
            .append(
                "ten_memory_one",
                "whatsapp",
                "phone_1",
                "buyer_1",
                "event_1",
                "user",
                "My name is Amina and I need size medium.",
            )
            .await
            .expect("append first user turn");
        let duplicate = memory
            .append(
                "ten_memory_one",
                "whatsapp",
                "phone_1",
                "buyer_1",
                "event_1",
                "user",
                "My name is Amina and I need size medium.",
            )
            .await
            .expect("duplicate delivery is idempotent");
        assert_eq!(first.len(), 1);
        assert_eq!(duplicate.len(), 1);
        let history = memory
            .append(
                "ten_memory_one",
                "whatsapp",
                "phone_1",
                "buyer_1",
                "event_1",
                "assistant",
                "Which color would you like?",
            )
            .await
            .expect("append assistant turn");
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0].content,
            "My name is Amina and I need size medium."
        );
        assert_eq!(history[1].role, "assistant");

        let pool_for_query = pool.clone();
        let raw_ciphertext = stheno_db::tenant::with_tenant_conn(&pool_for_query, |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT ciphertext FROM ai.channel_conversation \
                     WHERE channel = $1 AND external_account_id = $2 LIMIT 1",
                )
                .bind::<Text, _>("whatsapp")
                .bind::<Text, _>("phone_1")
                .get_result::<CiphertextRow>(conn)
                .await
                .map(|row| row.ciphertext)
                .map_err(|error| stheno_types::Error::Database(error.to_string()))
            })
        })
        .await
        .expect("read ciphertext under tenant RLS");
        assert!(!String::from_utf8_lossy(&raw_ciphertext).contains("Amina"));
        history
    })
    .await;

    let other_tenant_history = stheno_db::tenant::scope("ten_memory_two", async {
        memory
            .append(
                "ten_memory_two",
                "whatsapp",
                "phone_1",
                "buyer_1",
                "event_2",
                "user",
                "A separate tenant conversation.",
            )
            .await
            .expect("other tenant gets isolated history")
    })
    .await;
    assert_eq!(user.len(), 2);
    assert_eq!(other_tenant_history.len(), 1);

    db.drop().await;
    if let Some(value) = previous {
        std::env::set_var("STHENO_CREDENTIAL_KEY", value);
    } else {
        std::env::remove_var("STHENO_CREDENTIAL_KEY");
    }
}
