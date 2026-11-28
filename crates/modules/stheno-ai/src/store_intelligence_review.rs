//! Tenant-scoped, reversible owner review state for Store Intelligence candidates.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_db::diesel::result::OptionalExtension;
use stheno_db::diesel::sql_types::{Integer, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{Conn, Error, Result};

#[derive(Debug, Clone, Serialize)]
pub struct StoreIntelligenceReview {
    pub recommendation_id: String,
    pub decision: String,
    pub version: i32,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreIntelligenceReviewAudit {
    pub request_id: String,
    pub decision: String,
    pub actor_id: String,
    pub evidence_source: String,
    pub rule_version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct ReviewRow {
    #[diesel(sql_type = Text)]
    recommendation_id: String,
    #[diesel(sql_type = Text)]
    decision: String,
    #[diesel(sql_type = Integer)]
    version: i32,
    #[diesel(sql_type = Timestamptz)]
    changed_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct ReviewAuditRow {
    #[diesel(sql_type = Text)]
    recommendation_id: String,
    #[diesel(sql_type = Text)]
    decision: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
}

#[derive(QueryableByName)]
struct ReviewHistoryRow {
    #[diesel(sql_type = Text)]
    request_id: String,
    #[diesel(sql_type = Text)]
    decision: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Text)]
    evidence_source: String,
    #[diesel(sql_type = Text)]
    rule_version: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StoreIntelligenceReviewService;

impl StoreIntelligenceReviewService {
    /// Loads the tenant's bounded set of current review states.
    pub async fn list(&self, conn: &mut Conn<'_>) -> Result<Vec<StoreIntelligenceReview>> {
        diesel::sql_query(
            "SELECT recommendation_id, decision, version, changed_at \
             FROM ai.store_intelligence_review ORDER BY changed_at DESC LIMIT 1000",
        )
        .load::<ReviewRow>(conn)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(database_error)
    }

    /// Loads a bounded, tenant-scoped append-only history for one current recommendation.
    pub async fn history(
        &self,
        conn: &mut Conn<'_>,
        recommendation_id: &str,
        limit: i32,
    ) -> Result<Vec<StoreIntelligenceReviewAudit>> {
        validate_history_input(recommendation_id, limit)?;
        diesel::sql_query(
            "SELECT request_id, decision, actor_id, evidence_source, rule_version, created_at \
             FROM ai.store_intelligence_review_audit WHERE recommendation_id = $1 \
             ORDER BY created_at DESC, id DESC LIMIT $2",
        )
        .bind::<Text, _>(recommendation_id)
        .bind::<Integer, _>(limit)
        .load::<ReviewHistoryRow>(conn)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| StoreIntelligenceReviewAudit {
                    request_id: row.request_id,
                    decision: row.decision,
                    actor_id: row.actor_id,
                    evidence_source: row.evidence_source,
                    rule_version: row.rule_version,
                    created_at: row.created_at,
                })
                .collect()
        })
        .map_err(database_error)
    }

    /// Records an idempotent owner decision and its append-only audit event atomically.
    pub async fn record(
        &self,
        conn: &mut Conn<'_>,
        recommendation_id: &str,
        decision: &str,
        actor_id: &str,
        request_id: &str,
    ) -> Result<StoreIntelligenceReview> {
        validate_review_input(recommendation_id, decision, actor_id, request_id)?;
        let row = diesel::sql_query(
            "WITH accepted AS (\
               INSERT INTO ai.store_intelligence_review_audit \
                 (id, recommendation_id, request_id, decision, actor_id, evidence_source, rule_version) \
               VALUES ($1, $2, $3, $4, $5, 'tenant_order_snapshots', 'store-intelligence-recommendations-v1') \
               ON CONFLICT (tenant_id, request_id) DO NOTHING \
               RETURNING tenant_id, recommendation_id, decision, actor_id\
             ), updated AS (\
               INSERT INTO ai.store_intelligence_review \
                 (recommendation_id, decision, actor_id) \
               SELECT recommendation_id, decision, actor_id FROM accepted \
               ON CONFLICT (tenant_id, recommendation_id) DO UPDATE SET \
                 decision = EXCLUDED.decision, actor_id = EXCLUDED.actor_id, \
                 version = ai.store_intelligence_review.version + \
                   CASE WHEN ai.store_intelligence_review.decision IS DISTINCT FROM EXCLUDED.decision THEN 1 ELSE 0 END, \
                 changed_at = CASE WHEN ai.store_intelligence_review.decision IS DISTINCT FROM EXCLUDED.decision \
                   THEN now() ELSE ai.store_intelligence_review.changed_at END \
               RETURNING recommendation_id, decision, version, changed_at\
             ) SELECT recommendation_id, decision, version, changed_at FROM updated",
        )
        .bind::<Text, _>(format!("airev_{}", ulid::Ulid::new()))
        .bind::<Text, _>(recommendation_id)
        .bind::<Text, _>(request_id)
        .bind::<Text, _>(decision)
        .bind::<Text, _>(actor_id)
        .get_result::<ReviewRow>(conn)
        .await
        .optional()
        .map_err(database_error)?;
        if let Some(row) = row {
            return Ok(row.into());
        }
        let prior = diesel::sql_query(
            "SELECT recommendation_id, decision, actor_id \
             FROM ai.store_intelligence_review_audit WHERE request_id = $1",
        )
        .bind::<Text, _>(request_id)
        .get_result::<ReviewAuditRow>(conn)
        .await
        .optional()
        .map_err(database_error)?;
        let matches_request = prior.is_some_and(|prior| {
            prior.recommendation_id == recommendation_id
                && prior.decision == decision
                && prior.actor_id == actor_id
        });
        if !matches_request {
            return Err(Error::Conflict {
                code: "store_intelligence_request_id_reused",
                message: "The review request identifier was already used for a different decision."
                    .to_string(),
            });
        }
        diesel::sql_query(
            "SELECT recommendation_id, decision, version, changed_at \
             FROM ai.store_intelligence_review WHERE recommendation_id = $1",
        )
        .bind::<Text, _>(recommendation_id)
        .get_result::<ReviewRow>(conn)
        .await
        .map(Into::into)
        .map_err(database_error)
    }
}

fn validate_review_input(
    recommendation_id: &str,
    decision: &str,
    actor_id: &str,
    request_id: &str,
) -> Result<()> {
    if !valid_recommendation_id(recommendation_id)
        || actor_id.is_empty()
        || actor_id.len() > 128
        || actor_id.chars().any(char::is_control)
        || !matches!(decision, "approved" | "dismissed" | "reverted")
        || request_id.is_empty()
        || request_id.len() > 64
        || !request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(invalid_review_input());
    }
    Ok(())
}

fn valid_recommendation_id(recommendation_id: &str) -> bool {
    let parts = recommendation_id.splitn(4, ':').collect::<Vec<_>>();
    let current_day = Utc::now().timestamp().div_euclid(86_400);
    let valid_day = parts
        .as_slice()
        .first()
        .and_then(|day| day.parse::<i64>().ok())
        .is_some_and(|day| (current_day - 10..=current_day - 5).contains(&day));
    let valid_subject = parts.get(2).is_some_and(|subject| {
        !subject.is_empty() && subject.len() <= 128 && !subject.chars().any(char::is_control)
    });
    !recommendation_id.is_empty()
        && recommendation_id.len() <= 255
        && valid_day
        && parts
            .get(1)
            .is_some_and(|kind| *kind == "product" || *kind == "category")
        && valid_subject
        && parts.get(3).is_some_and(|action| {
            *action == "review_feature"
                || *action == "validate_new_signal"
                || *action == "review_visibility"
        })
}

fn validate_history_input(recommendation_id: &str, limit: i32) -> Result<()> {
    if !valid_recommendation_id(recommendation_id) || !(1..=50).contains(&limit) {
        return Err(invalid_review_input());
    }
    Ok(())
}

fn invalid_review_input() -> Error {
    Error::Invalid {
        code: "store_intelligence_review_invalid",
        message: "Recommendation review input is invalid.".to_string(),
        field: None,
    }
}

impl From<ReviewRow> for StoreIntelligenceReview {
    fn from(row: ReviewRow) -> Self {
        Self {
            recommendation_id: row.recommendation_id,
            decision: row.decision,
            version: row.version,
            changed_at: row.changed_at,
        }
    }
}

fn database_error(error: diesel::result::Error) -> Error {
    Error::Database(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{validate_history_input, validate_review_input};
    use chrono::Utc;

    fn valid_input() -> (String, &'static str, &'static str, &'static str) {
        let day = Utc::now().timestamp().div_euclid(86_400) - 7;
        (
            format!("{day}:product:prod_123:review_feature"),
            "approved",
            "user_123",
            "request-123",
        )
    }

    #[test]
    fn accepts_current_window_recommendation_and_owner_decision() {
        let (recommendation_id, decision, actor_id, request_id) = valid_input();
        assert!(validate_review_input(&recommendation_id, decision, actor_id, request_id).is_ok());
    }

    #[test]
    fn rejects_stale_future_malformed_or_oversized_recommendations() {
        let current_day = Utc::now().timestamp().div_euclid(86_400);
        let invalid_ids = [
            String::new(),
            format!("{}:product:prod_123:review_feature", current_day - 30),
            format!("{current_day}:product:prod_123:review_feature"),
            format!("{}:unknown:prod_123:review_feature", current_day - 7),
            format!("{}:product::review_feature", current_day - 7),
            format!(
                "{}:product:{}:review_feature",
                current_day - 7,
                "x".repeat(129)
            ),
            format!("{}:product:prod_123:delete_everything", current_day - 7),
            format!(
                "{}:product:{}:review_feature",
                current_day - 7,
                "x".repeat(256)
            ),
        ];
        for recommendation_id in invalid_ids {
            assert!(
                validate_review_input(&recommendation_id, "approved", "user_123", "req-1").is_err()
            );
        }
    }

    #[test]
    fn rejects_unknown_decisions_and_invalid_actor_or_idempotency_values() {
        let (recommendation_id, _, _, _) = valid_input();
        let long_actor = "x".repeat(129);
        let long_request = "x".repeat(65);
        for (decision, actor_id, request_id) in [
            ("executed", "user_123", "req-1"),
            ("approved", "", "req-1"),
            ("approved", "user\n123", "req-1"),
            ("approved", long_actor.as_str(), "req-1"),
            ("approved", "user_123", "not safe"),
            ("approved", "user_123", long_request.as_str()),
        ] {
            assert!(
                validate_review_input(&recommendation_id, decision, actor_id, request_id).is_err()
            );
        }
    }

    #[test]
    fn review_history_is_limited_to_current_recommendations_and_bounded_pages() {
        let (recommendation_id, _, _, _) = valid_input();
        assert!(validate_history_input(&recommendation_id, 1).is_ok());
        assert!(validate_history_input(&recommendation_id, 50).is_ok());
        for invalid_limit in [0, -1, 51, i32::MAX] {
            assert!(validate_history_input(&recommendation_id, invalid_limit).is_err());
        }
        assert!(validate_history_input("old:product:prod_123:review_feature", 20).is_err());
    }
}
