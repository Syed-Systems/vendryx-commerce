//! Owner-configured exact chat prices and short-lived, conversation-bound quotes.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use diesel::result::DatabaseErrorKind;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::diesel::result::OptionalExtension;
use stheno_db::diesel::sql_types::{Bool, Integer, Nullable, Numeric, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{Conn, Error, Result};

const MAX_QUANTITY: i32 = 1000;
const QUOTE_TTL_MINUTES: i32 = 10;

/// Merchant-authored fixed final unit price for one published variant and currency.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveChatOffer {
    pub id: Option<String>,
    pub variant_id: String,
    pub currency_code: String,
    /// Decimal string avoids routing money through JavaScript binary floating point.
    pub chat_unit_price: String,
    #[serde(default = "default_max_quantity")]
    pub max_quantity: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub enabled: bool,
}

fn default_max_quantity() -> i32 {
    1
}

/// Safe dashboard representation of an owner-configured chat offer.
#[derive(Debug, Clone, Serialize)]
pub struct ChatOffer {
    pub id: String,
    pub variant_id: String,
    pub currency_code: String,
    pub chat_unit_price: String,
    pub max_quantity: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub version: i32,
    pub updated_at: DateTime<Utc>,
}

/// Server-generated quote returned to the model; callers cannot provide its price.
#[derive(Debug, Clone, Serialize)]
pub struct ChatOfferQuote {
    pub quote_id: String,
    pub offer_id: String,
    pub offer_version: i32,
    pub variant_id: String,
    pub quantity: i32,
    pub currency_code: String,
    pub live_unit_price: String,
    pub chat_unit_price: String,
    pub savings_amount: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct OfferRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    variant_id: String,
    #[diesel(sql_type = Text)]
    currency_code: String,
    #[diesel(sql_type = Numeric)]
    chat_unit_price: Decimal,
    #[diesel(sql_type = Integer)]
    max_quantity: i32,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    starts_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    ends_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    enabled: bool,
    #[diesel(sql_type = Integer)]
    version: i32,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct QuoteRow {
    #[diesel(sql_type = Text)]
    quote_id: String,
    #[diesel(sql_type = Text)]
    offer_id: String,
    #[diesel(sql_type = Integer)]
    offer_version: i32,
    #[diesel(sql_type = Text)]
    variant_id: String,
    #[diesel(sql_type = Integer)]
    quantity: i32,
    #[diesel(sql_type = Text)]
    currency_code: String,
    #[diesel(sql_type = Numeric)]
    live_unit_price: Decimal,
    #[diesel(sql_type = Numeric)]
    chat_unit_price: Decimal,
    #[diesel(sql_type = Numeric)]
    savings_amount: Decimal,
    #[diesel(sql_type = Timestamptz)]
    expires_at: DateTime<Utc>,
}

/// CRUD and quote validation for tenant-owned chat prices.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChatOfferService;

impl ChatOfferService {
    /// Saves an owner-authored offer and its audit record in one tenant transaction.
    pub async fn save_with_actor(
        &self,
        conn: &mut Conn<'_>,
        input: SaveChatOffer,
        actor_id: &str,
    ) -> Result<ChatOffer> {
        validate_actor_id(actor_id)?;
        let offer = self.save(conn, input).await?;
        self.record_offer_audit(conn, actor_id, &offer.id, "saved", offer.version)
            .await?;
        Ok(offer)
    }

    /// Creates or updates one offer. An enabled offer for the same variant/currency conflicts.
    pub async fn save(&self, conn: &mut Conn<'_>, input: SaveChatOffer) -> Result<ChatOffer> {
        let variant_id = validate_variant(&input.variant_id)?;
        let currency = validate_currency(&input.currency_code)?;
        let price = parse_price(&input.chat_unit_price)?;
        if !(1..=MAX_QUANTITY).contains(&input.max_quantity) {
            return Err(invalid(
                "max_quantity",
                "Maximum quantity must be between 1 and 1000.",
            ));
        }
        if input
            .starts_at
            .zip(input.ends_at)
            .is_some_and(|(start, end)| start >= end)
        {
            return Err(invalid(
                "ends_at",
                "Offer end must be later than its start.",
            ));
        }
        let id = match input.id {
            Some(id) => validate_id(&id)?,
            None => format!("aioffer_{}", ulid::Ulid::new()),
        };
        let row = diesel::sql_query(
            "INSERT INTO ai.chat_offer (id, variant_id, currency_code, chat_unit_price, max_quantity, starts_at, ends_at, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET variant_id = EXCLUDED.variant_id, currency_code = EXCLUDED.currency_code, \
               chat_unit_price = EXCLUDED.chat_unit_price, max_quantity = EXCLUDED.max_quantity, starts_at = EXCLUDED.starts_at, \
               ends_at = EXCLUDED.ends_at, enabled = EXCLUDED.enabled, version = ai.chat_offer.version + 1, updated_at = now(), deleted_at = NULL \
             RETURNING id, variant_id, currency_code, chat_unit_price, max_quantity, starts_at, ends_at, enabled, version, updated_at",
        )
        .bind::<Text, _>(&id).bind::<Text, _>(&variant_id).bind::<Text, _>(&currency)
        .bind::<Numeric, _>(price).bind::<Integer, _>(input.max_quantity)
        .bind::<Nullable<Timestamptz>, _>(input.starts_at).bind::<Nullable<Timestamptz>, _>(input.ends_at)
        .bind::<Bool, _>(input.enabled).get_result::<OfferRow>(*conn).await
        .map_err(map_write_error)?;
        Ok(row.into())
    }

    /// Lists active and disabled offers for the current tenant in a stable order.
    pub async fn list(&self, conn: &mut Conn<'_>) -> Result<Vec<ChatOffer>> {
        let rows = diesel::sql_query(
            "SELECT id, variant_id, currency_code, chat_unit_price, max_quantity, starts_at, ends_at, enabled, version, updated_at \
             FROM ai.chat_offer WHERE deleted_at IS NULL ORDER BY updated_at DESC, id LIMIT 100",
        ).load::<OfferRow>(*conn).await.map_err(db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Loads at most eight live, unclaimed quotes for one hashed conversation key.
    pub async fn active_quotes(
        &self,
        conn: &mut Conn<'_>,
        conversation_key: &str,
    ) -> Result<Vec<ChatOfferQuote>> {
        validate_conversation_key(conversation_key)?;
        let rows = diesel::sql_query(
            "SELECT q.id AS quote_id, q.offer_id, q.offer_version, q.variant_id, q.quantity, q.currency_code, \
                    q.live_unit_price, q.chat_unit_price, q.live_unit_price - q.chat_unit_price AS savings_amount, q.expires_at \
             FROM ai.chat_offer_quote q JOIN ai.chat_offer o ON o.id = q.offer_id \
             WHERE q.conversation_key = $1 AND q.expires_at > now() AND q.claimed_event_id IS NULL \
               AND o.enabled AND o.deleted_at IS NULL AND o.version = q.offer_version \
               AND o.variant_id = q.variant_id AND o.currency_code = q.currency_code AND o.max_quantity >= q.quantity \
               AND (o.starts_at IS NULL OR o.starts_at <= now()) AND (o.ends_at IS NULL OR o.ends_at > now()) \
             ORDER BY q.issued_at DESC, q.id LIMIT 8",
        )
        .bind::<Text, _>(conversation_key)
        .load::<QuoteRow>(*conn)
        .await
        .map_err(db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Soft-deletes an offer; any unconsumed quote becomes invalid immediately.
    pub async fn delete(&self, conn: &mut Conn<'_>, id: &str) -> Result<()> {
        validate_id(id)?;
        let changed = diesel::sql_query(
            "UPDATE ai.chat_offer SET enabled = false, deleted_at = now(), updated_at = now(), version = version + 1 \
             WHERE id = $1 AND deleted_at IS NULL",
        ).bind::<Text, _>(id).execute(*conn).await.map_err(db_error)?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "ai_chat_offer",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    /// Disables an offer and records the owner action atomically.
    pub async fn delete_with_actor(
        &self,
        conn: &mut Conn<'_>,
        id: &str,
        actor_id: &str,
    ) -> Result<()> {
        validate_actor_id(actor_id)?;
        let id = validate_id(id)?;
        self.delete(conn, &id).await?;
        let offer = diesel::sql_query("SELECT version FROM ai.chat_offer WHERE id = $1")
            .bind::<Text, _>(&id)
            .get_result::<VersionRow>(*conn)
            .await
            .map_err(db_error)?;
        self.record_offer_audit(conn, actor_id, &id, "disabled", offer.version)
            .await
    }

    /// Issues an exact quote only when owner policy and current server price both permit it.
    // Keep catalog, conversation, quantity, currency, and current-price bindings explicit.
    #[allow(clippy::too_many_arguments)]
    pub async fn quote(
        &self,
        conn: &mut Conn<'_>,
        variant_id: &str,
        quantity: i32,
        currency_code: &str,
        region_id: Option<&str>,
        conversation_key: &str,
        live_unit_price: Decimal,
        currency_digits: u32,
    ) -> Result<Option<ChatOfferQuote>> {
        let variant_id = validate_variant(variant_id)?;
        let currency = validate_currency(currency_code)?;
        validate_conversation_key(conversation_key)?;
        if !(1..=MAX_QUANTITY).contains(&quantity) || live_unit_price <= Decimal::ZERO {
            return Err(invalid(
                "quantity",
                "Quote quantity or live price is invalid.",
            ));
        }
        diesel::sql_query(
            "DELETE FROM ai.chat_offer_quote WHERE expires_at <= now() AND claimed_event_id IS NULL",
        )
            .execute(*conn)
            .await
            .map_err(db_error)?;
        let offer = diesel::sql_query(
            "SELECT id, variant_id, currency_code, chat_unit_price, max_quantity, starts_at, ends_at, enabled, version, updated_at \
             FROM ai.chat_offer WHERE variant_id = $1 AND currency_code = $2 AND enabled AND deleted_at IS NULL \
               AND (starts_at IS NULL OR starts_at <= now()) AND (ends_at IS NULL OR ends_at > now()) LIMIT 2 FOR SHARE",
        ).bind::<Text, _>(&variant_id).bind::<Text, _>(&currency).load::<OfferRow>(*conn).await.map_err(db_error)?;
        if offer.len() != 1 {
            return Ok(None);
        }
        let offer = offer.into_iter().next().ok_or_else(|| {
            Error::Internal(Box::new(std::io::Error::other(
                "offer lookup invariant failed",
            )))
        })?;
        if quantity > offer.max_quantity
            || offer.chat_unit_price >= live_unit_price
            || offer.chat_unit_price.normalize().scale() > currency_digits
            || live_unit_price.normalize().scale() > currency_digits
        {
            return Ok(None);
        }
        let quote_id = format!("aiquote_{}", ulid::Ulid::new());
        let row = diesel::sql_query(
            "INSERT INTO ai.chat_offer_quote (id, offer_id, offer_version, conversation_key, variant_id, quantity, currency_code, region_id, live_unit_price, chat_unit_price, expires_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,now() + ($11 * interval '1 minute')) \
             RETURNING id AS quote_id, offer_id, offer_version, variant_id, quantity, currency_code, live_unit_price, chat_unit_price, live_unit_price - chat_unit_price AS savings_amount, expires_at",
        ).bind::<Text, _>(&quote_id).bind::<Text, _>(&offer.id).bind::<Integer, _>(offer.version)
        .bind::<Text, _>(conversation_key).bind::<Text, _>(&variant_id).bind::<Integer, _>(quantity)
        .bind::<Text, _>(&currency).bind::<Nullable<Text>, _>(region_id).bind::<Numeric, _>(live_unit_price)
        .bind::<Numeric, _>(offer.chat_unit_price).bind::<Integer, _>(QUOTE_TTL_MINUTES)
        .get_result::<QuoteRow>(*conn).await.map_err(db_error)?;
        Ok(Some(row.into()))
    }

    /// Validates a previously quoted price before using it in a draft line.
    // Each argument rebinds one independent quote fact to the active cart event.
    #[allow(clippy::too_many_arguments)]
    pub async fn validate_quote(
        &self,
        conn: &mut Conn<'_>,
        quote_id: &str,
        conversation_key: &str,
        event_id: &str,
        variant_id: &str,
        quantity: i32,
        currency_code: &str,
        region_id: Option<&str>,
        live_unit_price: Decimal,
    ) -> Result<Decimal> {
        validate_id(quote_id)?;
        validate_conversation_key(conversation_key)?;
        if event_id.is_empty() || event_id.len() > 128 || event_id.chars().any(char::is_control) {
            return Err(invalid("event_id", "Quote event binding is invalid."));
        }
        let quote = diesel::sql_query(
            "SELECT q.id AS quote_id, q.offer_id, q.offer_version, q.variant_id, q.quantity, q.currency_code, q.live_unit_price, q.chat_unit_price, q.live_unit_price - q.chat_unit_price AS savings_amount, q.expires_at \
             FROM ai.chat_offer_quote q JOIN ai.chat_offer o ON o.id = q.offer_id \
             WHERE q.id = $1 AND q.conversation_key = $2 AND q.variant_id = $3 AND q.quantity = $4 \
               AND q.currency_code = $5 AND q.region_id IS NOT DISTINCT FROM $6 AND q.expires_at > now() \
               AND o.enabled AND o.deleted_at IS NULL AND o.version = q.offer_version \
               AND o.variant_id = q.variant_id AND o.currency_code = q.currency_code AND o.max_quantity >= q.quantity \
               AND (o.starts_at IS NULL OR o.starts_at <= now()) AND (o.ends_at IS NULL OR o.ends_at > now()) \
             LIMIT 1 FOR UPDATE OF q FOR SHARE OF o",
        ).bind::<Text, _>(quote_id).bind::<Text, _>(conversation_key).bind::<Text, _>(validate_variant(variant_id)?)
        .bind::<Integer, _>(quantity).bind::<Text, _>(validate_currency(currency_code)?)
        .bind::<Nullable<Text>, _>(region_id).get_result::<QuoteRow>(*conn).await.optional().map_err(db_error)?
            .ok_or_else(|| Error::Conflict { code: "ai_chat_quote_stale", message: "The chat offer is no longer valid. Refresh the price before continuing.".to_string() })?;
        if quote.live_unit_price != live_unit_price {
            return Err(Error::Conflict {
                code: "ai_chat_quote_stale",
                message: "The published price changed. Refresh the chat offer before continuing."
                    .to_string(),
            });
        }
        let claim = diesel::sql_query(
            "UPDATE ai.chat_offer_quote SET claimed_event_id = $2 WHERE id = $1 AND (claimed_event_id IS NULL OR claimed_event_id = $2)",
        ).bind::<Text, _>(quote_id).bind::<Text, _>(event_id).execute(*conn).await.map_err(db_error)?;
        if claim != 1 {
            return Err(Error::Conflict {
                code: "ai_chat_quote_replayed",
                message: "This chat offer quote was already used.".to_string(),
            });
        }
        Ok(quote.chat_unit_price)
    }

    /// Revalidates an already drafted offer before staff converts that draft to an order.
    // These explicit values are checked against the immutable quote and current owner policy.
    #[allow(clippy::too_many_arguments)]
    pub async fn validate_confirmation_quote(
        &self,
        conn: &mut Conn<'_>,
        quote_id: &str,
        event_id: &str,
        expected_chat_price: Decimal,
        variant_id: &str,
        quantity: i32,
        currency_code: &str,
        region_id: Option<&str>,
    ) -> Result<(Decimal, Decimal)> {
        validate_id(quote_id)?;
        if event_id.is_empty() || event_id.len() > 128 || event_id.chars().any(char::is_control) {
            return Err(invalid("event_id", "Quote event binding is invalid."));
        }
        let quote = diesel::sql_query(
            "SELECT q.live_unit_price, q.chat_unit_price FROM ai.chat_offer_quote q \
             JOIN ai.chat_offer o ON o.id = q.offer_id AND o.tenant_id = q.tenant_id \
             WHERE q.id = $1 AND q.claimed_event_id = $2 AND q.chat_unit_price = $3 \
               AND q.variant_id = $4 AND q.quantity = $5 AND q.currency_code = $6 \
               AND q.region_id IS NOT DISTINCT FROM $7 \
               AND q.expires_at > now() AND o.enabled AND o.deleted_at IS NULL \
               AND o.version = q.offer_version AND o.variant_id = q.variant_id \
               AND o.currency_code = q.currency_code AND o.max_quantity >= q.quantity \
               AND (o.starts_at IS NULL OR o.starts_at <= now()) \
               AND (o.ends_at IS NULL OR o.ends_at > now()) \
             LIMIT 1 FOR SHARE OF o",
        )
        .bind::<Text, _>(quote_id)
        .bind::<Text, _>(event_id)
        .bind::<Numeric, _>(expected_chat_price)
        .bind::<Text, _>(validate_variant(variant_id)?)
        .bind::<Integer, _>(quantity)
        .bind::<Text, _>(validate_currency(currency_code)?)
        .bind::<Nullable<Text>, _>(region_id)
        .get_result::<ConfirmationQuoteRow>(*conn)
        .await
        .optional()
        .map_err(db_error)?;
        let quote = quote.ok_or_else(|| Error::Conflict {
            code: "ai_chat_quote_stale",
            message:
                "The chat price is no longer authorized. Hold this draft for staff price review."
                    .to_string(),
        })?;
        Ok((quote.live_unit_price, quote.chat_unit_price))
    }

    /// Returns the tenant-scoped quote set claimed by one inbound event.
    pub async fn claimed_quote_ids(
        &self,
        conn: &mut Conn<'_>,
        event_id: &str,
    ) -> Result<Vec<String>> {
        if event_id.is_empty() || event_id.len() > 128 || event_id.chars().any(char::is_control) {
            return Err(invalid("event_id", "Quote event binding is invalid."));
        }
        let rows = diesel::sql_query(
            "SELECT id FROM ai.chat_offer_quote WHERE claimed_event_id = $1 ORDER BY id",
        )
        .bind::<Text, _>(event_id)
        .load::<QuoteIdRow>(*conn)
        .await
        .map_err(db_error)?;
        Ok(rows.into_iter().map(|row| row.id).collect())
    }

    async fn record_offer_audit(
        &self,
        conn: &mut Conn<'_>,
        actor_id: &str,
        offer_id: &str,
        action: &str,
        version: i32,
    ) -> Result<()> {
        diesel::sql_query(
            "INSERT INTO ai.chat_offer_audit (id, actor_id, offer_id, action, offer_version) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind::<Text, _>(format!("aiofferaudit_{}", ulid::Ulid::new()))
        .bind::<Text, _>(actor_id)
        .bind::<Text, _>(offer_id)
        .bind::<Text, _>(action)
        .bind::<Integer, _>(version)
        .execute(*conn)
        .await
        .map_err(db_error)?;
        Ok(())
    }
}

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = Integer)]
    version: i32,
}

#[derive(QueryableByName)]
struct QuoteIdRow {
    #[diesel(sql_type = Text)]
    id: String,
}

#[derive(QueryableByName)]
struct ConfirmationQuoteRow {
    #[diesel(sql_type = Numeric)]
    live_unit_price: Decimal,
    #[diesel(sql_type = Numeric)]
    chat_unit_price: Decimal,
}

impl From<OfferRow> for ChatOffer {
    fn from(row: OfferRow) -> Self {
        Self {
            id: row.id,
            variant_id: row.variant_id,
            currency_code: row.currency_code,
            chat_unit_price: row.chat_unit_price.normalize().to_string(),
            max_quantity: row.max_quantity,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            enabled: row.enabled,
            version: row.version,
            updated_at: row.updated_at,
        }
    }
}

impl From<QuoteRow> for ChatOfferQuote {
    fn from(row: QuoteRow) -> Self {
        Self {
            quote_id: row.quote_id,
            offer_id: row.offer_id,
            offer_version: row.offer_version,
            variant_id: row.variant_id,
            quantity: row.quantity,
            currency_code: row.currency_code,
            live_unit_price: row.live_unit_price.normalize().to_string(),
            chat_unit_price: row.chat_unit_price.normalize().to_string(),
            savings_amount: row.savings_amount.normalize().to_string(),
            expires_at: row.expires_at,
        }
    }
}

fn parse_price(value: &str) -> Result<Decimal> {
    if value.is_empty() || value.len() > 32 || value.trim() != value || value.starts_with('+') {
        return Err(invalid(
            "chat_unit_price",
            "Enter a positive decimal price.",
        ));
    }
    let price = Decimal::from_str(value)
        .map_err(|_| invalid("chat_unit_price", "Enter a positive decimal price."))?;
    if price <= Decimal::ZERO || price.normalize().scale() > 6 {
        return Err(invalid(
            "chat_unit_price",
            "Price must be positive and have at most six decimal places.",
        ));
    }
    Ok(price)
}

fn validate_variant(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(invalid("variant_id", "Variant ID is invalid."));
    }
    Ok(value.to_string())
}

fn validate_currency(value: &str) -> Result<String> {
    if value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(invalid(
            "currency_code",
            "Currency code must be three uppercase letters.",
        ));
    }
    Ok(value.to_string())
}

fn validate_id(value: &str) -> Result<String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(invalid("id", "Offer or quote ID is invalid."));
    }
    Ok(value.to_string())
}

fn validate_actor_id(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(invalid("actor_id", "Owner identity is invalid."));
    }
    Ok(())
}

fn validate_conversation_key(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("conversation", "Conversation binding is invalid."));
    }
    Ok(())
}

fn map_write_error(error: diesel::result::Error) -> Error {
    match error {
        diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Error::Conflict { code: "ai_chat_offer_conflict", message: "An enabled chat offer already exists for this variant and currency. Disable it before creating another.".to_string() },
        other => db_error(other),
    }
}

fn db_error(error: diesel::result::Error) -> Error {
    Error::Database(error.to_string())
}

fn invalid(field: &'static str, message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_chat_offer_invalid",
        message: message.to_string(),
        field: Some(field.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_price, validate_actor_id, validate_conversation_key, validate_currency, validate_id,
        validate_variant,
    };
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn chat_price_parser_preserves_exact_decimal_and_rejects_unsafe_values() {
        for (input, expected) in [("995", "995"), ("995.00", "995"), ("0.000001", "0.000001")] {
            let parsed = parse_price(input).expect("valid exact decimal");
            assert_eq!(
                parsed,
                Decimal::from_str(expected).expect("expected decimal")
            );
        }

        for input in ["", " 995", "995 ", "+995", "0", "-0.01", "1.0000001"] {
            assert!(
                parse_price(input).is_err(),
                "accepted unsafe price {input:?}"
            );
        }
        assert!(parse_price(&"x".repeat(33)).is_err());
    }

    #[test]
    fn offer_identity_and_currency_validators_fail_closed() {
        assert_eq!(
            validate_variant(" variant_123 ").expect("valid variant"),
            "variant_123"
        );
        assert!(validate_variant("variant\n123").is_err());
        assert!(validate_variant(&"v".repeat(129)).is_err());

        assert_eq!(validate_currency("BDT").expect("valid currency"), "BDT");
        for currency in ["bdt", "US", "US1", " BDT"] {
            assert!(validate_currency(currency).is_err());
        }

        assert_eq!(
            validate_id("aioffer_01AB").expect("valid ID"),
            "aioffer_01AB"
        );
        assert!(validate_id("offer/other-tenant").is_err());
        assert!(validate_id(&"x".repeat(65)).is_err());
    }

    #[test]
    fn actor_and_conversation_bindings_reject_control_or_malformed_values() {
        assert!(validate_actor_id("user_01AB").is_ok());
        assert!(validate_actor_id("user\nforged").is_err());
        assert!(validate_conversation_key(&"a".repeat(64)).is_ok());
        assert!(validate_conversation_key(&"g".repeat(64)).is_err());
        assert!(validate_conversation_key(&"a".repeat(63)).is_err());
    }
}
