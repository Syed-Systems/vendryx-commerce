//! Encrypted, short-lived tenant conversation context for multi-turn social intake.

use diesel::result::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stheno_db::diesel::sql_types::{Binary, Text};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{DbPool, Error, Result};

use crate::configuration::{decrypt_payload, encrypt_payload};

const MAX_MESSAGES: usize = 12;
const MAX_CONTEXT_CHARS: usize = 12_000;
const MAX_MESSAGE_CHARS: usize = 4_000;

/// One encrypted short-lived user/assistant turn returned to the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConversationTurn {
    pub event_id: String,
    pub role: String,
    pub content: String,
}

#[derive(QueryableByName)]
struct ConversationRow {
    #[diesel(sql_type = Binary)]
    ciphertext: Vec<u8>,
    #[diesel(sql_type = Binary)]
    nonce: Vec<u8>,
}

/// Tenant-owned encrypted conversation memory with a rolling one-day expiry.
#[derive(Clone)]
pub struct ConversationMemoryService {
    pool: DbPool,
}

impl ConversationMemoryService {
    /// Creates the memory service over the application database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Adds one idempotent event turn and returns the bounded recent context.
    // Keep tenant, channel, account, sender, and event bindings explicit at the trust boundary.
    #[allow(clippy::too_many_arguments)]
    pub async fn append(
        &self,
        tenant_id: &str,
        channel: &str,
        account_id: &str,
        sender_id: &str,
        event_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Vec<StoredConversationTurn>> {
        validate_input(channel, account_id, sender_id, event_id, role, content)?;
        stheno_db::tenant::validate(tenant_id)?;
        let tenant_id = tenant_id.to_string();
        let channel = channel.to_string();
        let account_id = account_id.to_string();
        let event_id = event_id.to_string();
        let role = role.to_string();
        let content = content.to_string();
        let conversation_key = format!("{:x}", Sha256::digest(sender_id.as_bytes()));
        let lock_key = format!("{tenant_id}|{channel}|{account_id}|{conversation_key}");
        let aad = format!("{tenant_id}\0conversation\0{channel}\0{account_id}");
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                        .bind::<Text, _>(&lock_key)
                        .execute(conn)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?;
                    let current = diesel::sql_query(
                        "SELECT ciphertext, nonce FROM ai.channel_conversation \
                         WHERE channel = $1 AND external_account_id = $2 \
                           AND conversation_key = $3 AND expires_at > now() LIMIT 1",
                    )
                    .bind::<Text, _>(&channel)
                    .bind::<Text, _>(&account_id)
                    .bind::<Text, _>(&conversation_key)
                    .get_result::<ConversationRow>(conn)
                    .await
                    .optional()
                    .map_err(|error| Error::Database(error.to_string()))?;
                    let mut turns = match current {
                        Some(row) => {
                            let plaintext = decrypt_payload(aad.as_bytes(), &row.ciphertext, &row.nonce)?;
                            serde_json::from_slice::<Vec<StoredConversationTurn>>(&plaintext)
                                .map_err(|_| invalid("Stored conversation payload is invalid."))?
                        }
                        None => Vec::new(),
                    };
                    if !turns.iter().any(|turn| turn.event_id == event_id && turn.role == role) {
                        turns.push(StoredConversationTurn {
                            event_id,
                            role,
                            content,
                        });
                    }
                    while turns.len() > MAX_MESSAGES
                        || turns.iter().map(|turn| turn.content.chars().count()).sum::<usize>()
                            > MAX_CONTEXT_CHARS
                    {
                        if turns.is_empty() {
                            break;
                        }
                        turns.remove(0);
                    }
                    let plaintext = serde_json::to_vec(&turns)
                        .map_err(|_| invalid("Conversation could not be serialized."))?;
                    let (ciphertext, nonce) = encrypt_payload(aad.as_bytes(), &plaintext)?;
                    diesel::sql_query(
                        "INSERT INTO ai.channel_conversation \
                         (tenant_id, channel, external_account_id, conversation_key, ciphertext, nonce, updated_at, expires_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, now(), now() + interval '24 hours') \
                         ON CONFLICT (tenant_id, channel, external_account_id, conversation_key) \
                         DO UPDATE SET ciphertext = EXCLUDED.ciphertext, nonce = EXCLUDED.nonce, \
                                       updated_at = now(), expires_at = EXCLUDED.expires_at",
                    )
                    .bind::<Text, _>(&tenant_id)
                    .bind::<Text, _>(&channel)
                    .bind::<Text, _>(&account_id)
                    .bind::<Text, _>(&conversation_key)
                    .bind::<Binary, _>(ciphertext)
                    .bind::<Binary, _>(nonce)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    diesel::sql_query(
                        "DELETE FROM ai.channel_conversation WHERE expires_at <= now()",
                    )
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    Ok(turns)
                })
            })
            .await
        })
        .await
    }
}

fn validate_input(
    channel: &str,
    account_id: &str,
    sender_id: &str,
    event_id: &str,
    role: &str,
    content: &str,
) -> Result<()> {
    if !matches!(channel, "whatsapp" | "messenger" | "instagram")
        || account_id.trim().is_empty()
        || account_id.len() > 128
        || sender_id.trim().is_empty()
        || sender_id.len() > 256
        || event_id.trim().is_empty()
        || event_id.len() > 256
        || !matches!(role, "user" | "assistant")
        || content.trim().is_empty()
        || content.chars().count() > MAX_MESSAGE_CHARS
        || content.chars().any(|character| {
            character == '\0'
                || (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        })
    {
        return Err(invalid(
            "Conversation turn is malformed or exceeds its limit.",
        ));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_conversation_memory_invalid",
        message: message.to_string(),
        field: None,
    }
}
