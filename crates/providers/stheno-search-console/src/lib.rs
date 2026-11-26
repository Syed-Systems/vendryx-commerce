//! Read-only Google Search Console Search Analytics adapter.

use std::str::FromStr;
use std::time::Duration;

use chrono::{Days, NaiveDate, Utc};
use reqwest::header::CONTENT_TYPE;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_ai::{GoogleSearchConsoleRuntimeConfig, MetricCoverage, SourceMetricObservation};
use url::Url;

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_DAYS: i64 = 92;
const MAX_ROWS: u32 = 1000;
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const API_BASE_URL: &str = "https://www.googleapis.com/webmasters/v3/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchConsoleError {
    InvalidRequest,
    CredentialsRejected,
    PermissionDenied,
    RateLimited,
    ProviderUnavailable,
    MalformedResponse,
    RequestFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchConsoleBatch {
    pub source: &'static str,
    pub property: String,
    pub window_started_on: NaiveDate,
    /// Exclusive end date, in the provider's Pacific calendar.
    pub window_ended_on: NaiveDate,
    pub window_timezone: &'static str,
    pub ingested_at: chrono::DateTime<Utc>,
    pub coverage: MetricCoverage,
    pub rows_received: usize,
    pub observations: Vec<SourceMetricObservation>,
}

#[derive(Clone)]
pub struct GoogleSearchConsoleAdapter {
    client: reqwest::Client,
    token_url: Url,
    api_base_url: Url,
}

impl GoogleSearchConsoleAdapter {
    /// Creates a client with bounded connect and total request timeouts.
    pub fn new() -> Result<Self, SearchConsoleError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|_| SearchConsoleError::RequestFailed)?;
        Ok(Self {
            client,
            token_url: Url::parse(TOKEN_URL).map_err(|_| SearchConsoleError::RequestFailed)?,
            api_base_url: Url::parse(API_BASE_URL)
                .map_err(|_| SearchConsoleError::RequestFailed)?,
        })
    }

    /// Exchanges a short-lived Google authorization code for a merchant refresh token.
    ///
    /// The configured redirect URI must exactly match the URI used to start authorization.
    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<String, SearchConsoleError> {
        if code.is_empty()
            || code.len() > 4096
            || client_id.is_empty()
            || client_secret.is_empty()
            || redirect_uri.is_empty()
        {
            return Err(SearchConsoleError::InvalidRequest);
        }
        let response = self
            .client
            .post(self.token_url.clone())
            .form(&[
                ("code", code),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|_| SearchConsoleError::RequestFailed)?;
        match response.status().as_u16() {
            200..=299 => {}
            400 | 401 => return Err(SearchConsoleError::CredentialsRejected),
            429 => return Err(SearchConsoleError::RateLimited),
            500..=599 => return Err(SearchConsoleError::ProviderUnavailable),
            _ => return Err(SearchConsoleError::RequestFailed),
        }
        let token: AuthorizationCodeToken = read_json_bounded(response).await?;
        if token.refresh_token.is_empty() || token.refresh_token.len() > 8192 {
            return Err(SearchConsoleError::CredentialsRejected);
        }
        Ok(token.refresh_token)
    }

    #[cfg(test)]
    async fn with_endpoints(token_url: Url, api_base_url: Url) -> Result<Self, SearchConsoleError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|_| SearchConsoleError::RequestFailed)?;
        Ok(Self {
            client,
            token_url,
            api_base_url,
        })
    }

    /// Refreshes the OAuth access token and fetches bounded daily property aggregates.
    pub async fn collect(
        &self,
        config: &GoogleSearchConsoleRuntimeConfig,
        tenant_id: &str,
        start_date: NaiveDate,
        end_date_inclusive: NaiveDate,
    ) -> Result<SearchConsoleBatch, SearchConsoleError> {
        let days = end_date_inclusive
            .signed_duration_since(start_date)
            .num_days()
            + 1;
        if days <= 0 || days > MAX_DAYS {
            return Err(SearchConsoleError::InvalidRequest);
        }

        let access_token = self.refresh_access_token(config).await?;
        let mut endpoint = self.api_base_url.clone();
        endpoint
            .path_segments_mut()
            .map_err(|_| SearchConsoleError::RequestFailed)?
            .pop_if_empty()
            .extend([
                "sites",
                config.property.as_str(),
                "searchAnalytics",
                "query",
            ]);
        let body = serde_json::json!({
            "startDate": start_date.format("%Y-%m-%d").to_string(),
            "endDate": end_date_inclusive.format("%Y-%m-%d").to_string(),
            "dimensions": ["date"],
            "rowLimit": MAX_ROWS,
            "startRow": 0,
            "dataState": "final"
        });
        let response = self
            .client
            .post(endpoint)
            .bearer_auth(access_token)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|_| SearchConsoleError::RequestFailed)?;
        ensure_success(response.status().as_u16())?;
        let payload: SearchAnalyticsResponse = read_json_bounded(response).await?;
        let ingested_at = Utc::now();
        let mut rows = std::collections::BTreeMap::new();
        let mut partial = false;
        for row in payload.rows {
            if row.keys.len() != 1 {
                return Err(SearchConsoleError::MalformedResponse);
            }
            let date = NaiveDate::parse_from_str(&row.keys[0], "%Y-%m-%d")
                .map_err(|_| SearchConsoleError::MalformedResponse)?;
            let reported_ctr = decimal(&row.ctr)?;
            if !(Decimal::ZERO..=Decimal::ONE).contains(&reported_ctr) {
                return Err(SearchConsoleError::MalformedResponse);
            }
            if date < start_date || date > end_date_inclusive {
                partial = true;
                continue;
            }
            if rows.insert(date, row).is_some() {
                partial = true;
            }
        }
        if rows.is_empty() {
            return Ok(SearchConsoleBatch {
                source: "google_search_console",
                property: config.property.clone(),
                window_started_on: start_date,
                window_ended_on: end_date_inclusive
                    .checked_add_days(Days::new(1))
                    .ok_or(SearchConsoleError::InvalidRequest)?,
                window_timezone: "America/Los_Angeles",
                ingested_at,
                coverage: MetricCoverage::Unavailable,
                rows_received: 0,
                observations: Vec::new(),
            });
        }
        for offset in 0..days {
            let day = start_date
                .checked_add_days(Days::new(offset as u64))
                .ok_or(SearchConsoleError::InvalidRequest)?;
            if !rows.contains_key(&day) {
                partial = true;
            }
        }

        let clicks = rows.values().try_fold(Decimal::ZERO, |sum, row| {
            checked_metric_add(sum, decimal(&row.clicks)?)
        })?;
        let impressions = rows.values().try_fold(Decimal::ZERO, |sum, row| {
            checked_metric_add(sum, decimal(&row.impressions)?)
        })?;
        let position_weighted_sum = rows.values().try_fold(Decimal::ZERO, |sum, row| {
            let position = decimal(&row.position)?;
            let row_impressions = decimal(&row.impressions)?;
            let weighted_position = position
                .checked_mul(row_impressions)
                .ok_or(SearchConsoleError::MalformedResponse)?;
            checked_metric_add(sum, weighted_position)
        })?;
        let coverage = if partial {
            MetricCoverage::Partial
        } else {
            MetricCoverage::Complete
        };
        let end_exclusive = end_date_inclusive
            .checked_add_days(Days::new(1))
            .ok_or(SearchConsoleError::InvalidRequest)?;
        let context = ObservationContext {
            tenant_id,
            config,
            sample_size: u64::try_from(rows.len()).ok(),
            start: start_date,
            end: end_exclusive,
            ingested_at,
            coverage,
        };
        let mut observations = vec![
            observation(
                &context,
                "clicks",
                "search-console-clicks-v1",
                "count",
                clicks,
                None,
            ),
            observation(
                &context,
                "impressions",
                "search-console-impressions-v1",
                "count",
                impressions,
                None,
            ),
        ];
        if impressions > Decimal::ZERO {
            observations.push(observation(
                &context,
                "click_through_rate",
                "search-console-ctr-v1",
                "ratio",
                clicks / impressions,
                Some(impressions),
            ));
            observations.push(observation(
                &context,
                "average_position",
                "search-console-position-v1",
                "position",
                position_weighted_sum / impressions,
                Some(impressions),
            ));
        }

        Ok(SearchConsoleBatch {
            source: "google_search_console",
            property: config.property.clone(),
            window_started_on: start_date,
            window_ended_on: end_exclusive,
            window_timezone: "America/Los_Angeles",
            ingested_at,
            coverage,
            rows_received: rows.len(),
            observations,
        })
    }

    async fn refresh_access_token(
        &self,
        config: &GoogleSearchConsoleRuntimeConfig,
    ) -> Result<String, SearchConsoleError> {
        let response = self
            .client
            .post(self.token_url.clone())
            .form(&[
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("refresh_token", config.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|_| SearchConsoleError::RequestFailed)?;
        if matches!(response.status().as_u16(), 400 | 401) {
            return Err(SearchConsoleError::CredentialsRejected);
        }
        ensure_success(response.status().as_u16())?;
        let token: TokenResponse = read_json_bounded(response).await?;
        if token.access_token.is_empty() || token.access_token.len() > 8192 {
            return Err(SearchConsoleError::MalformedResponse);
        }
        Ok(token.access_token)
    }
}

struct ObservationContext<'a> {
    tenant_id: &'a str,
    config: &'a GoogleSearchConsoleRuntimeConfig,
    sample_size: Option<u64>,
    start: NaiveDate,
    end: NaiveDate,
    ingested_at: chrono::DateTime<Utc>,
    coverage: MetricCoverage,
}

fn observation(
    context: &ObservationContext<'_>,
    metric: &str,
    definition: &str,
    unit: &str,
    value: Decimal,
    denominator: Option<Decimal>,
) -> SourceMetricObservation {
    SourceMetricObservation {
        tenant_id: context.tenant_id.to_string(),
        source: "google_search_console".to_string(),
        account_id: context.config.property.clone(),
        scope: "property_daily_aggregate".to_string(),
        metric: metric.to_string(),
        definition: definition.to_string(),
        unit: unit.to_string(),
        value,
        denominator,
        sample_size: context.sample_size,
        window_started_on: context.start,
        window_ended_on: context.end,
        window_timezone: "America/Los_Angeles".to_string(),
        ingested_at: context.ingested_at,
        coverage: context.coverage,
    }
}

fn decimal(number: &serde_json::Number) -> Result<Decimal, SearchConsoleError> {
    Decimal::from_str(&number.to_string()).map_err(|_| SearchConsoleError::MalformedResponse)
}

fn checked_metric_add(sum: Decimal, value: Decimal) -> Result<Decimal, SearchConsoleError> {
    sum.checked_add(value)
        .ok_or(SearchConsoleError::MalformedResponse)
}

fn ensure_success(status: u16) -> Result<(), SearchConsoleError> {
    match status {
        200..=299 => Ok(()),
        401 => Err(SearchConsoleError::CredentialsRejected),
        403 => Err(SearchConsoleError::PermissionDenied),
        429 => Err(SearchConsoleError::RateLimited),
        500..=599 => Err(SearchConsoleError::ProviderUnavailable),
        _ => Err(SearchConsoleError::RequestFailed),
    }
}

async fn read_json_bounded<T: for<'de> Deserialize<'de>>(
    mut response: reqwest::Response,
) -> Result<T, SearchConsoleError> {
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
    {
        return Err(SearchConsoleError::MalformedResponse);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| SearchConsoleError::RequestFailed)?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(SearchConsoleError::MalformedResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| SearchConsoleError::MalformedResponse)
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct AuthorizationCodeToken {
    refresh_token: String,
}

#[derive(Deserialize)]
struct SearchAnalyticsResponse {
    #[serde(default)]
    rows: Vec<SearchAnalyticsRow>,
}

#[derive(Deserialize)]
struct SearchAnalyticsRow {
    keys: Vec<String>,
    clicks: serde_json::Number,
    impressions: serde_json::Number,
    ctr: serde_json::Number,
    position: serde_json::Number,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config() -> GoogleSearchConsoleRuntimeConfig {
        GoogleSearchConsoleRuntimeConfig {
            property: "sc-domain:shop.example.com".to_string(),
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            refresh_token: "refresh-token".to_string(),
        }
    }

    #[test]
    fn decimal_aggregation_overflow_returns_a_provider_error() {
        assert_eq!(
            checked_metric_add(Decimal::MAX, Decimal::ONE),
            Err(SearchConsoleError::MalformedResponse)
        );
        assert_eq!(
            Decimal::MAX.checked_mul(Decimal::from(2)),
            None,
            "weighted metric multiplication must be checked before aggregation"
        );
    }

    #[tokio::test]
    async fn exchanges_authorization_code_and_requires_offline_refresh_token() {
        let token = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(header("content-type", "application/x-www-form-urlencoded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token":"access-token","refresh_token":"refresh-token","expires_in":3600
            })))
            .mount(&token)
            .await;
        let adapter = GoogleSearchConsoleAdapter::with_endpoints(
            Url::parse(&format!("{}/token", token.uri())).unwrap(),
            Url::parse("https://example.test/api/").unwrap(),
        )
        .await
        .unwrap();
        let refresh_token = adapter
            .exchange_authorization_code(
                "one-time-code",
                "client-id",
                "client-secret",
                "https://store.example.test/oauth/callback",
            )
            .await
            .unwrap();
        assert_eq!(refresh_token, "refresh-token");
    }

    #[tokio::test]
    async fn collects_decimal_daily_metrics_and_marks_missing_dates_partial() {
        let token = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token":"access-token","expires_in":3600,"token_type":"Bearer"
            })))
            .mount(&token)
            .await;
        let api = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/webmasters/v3/sites/sc-domain:shop.example.com/searchAnalytics/query",
            ))
            .and(header("authorization", "Bearer access-token"))
            .and(body_json(serde_json::json!({
                "startDate":"2026-10-01","endDate":"2026-10-02","dimensions":["date"],
                "rowLimit":1000,"startRow":0,"dataState":"final"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "rows":[{"keys":["2026-10-01"],"clicks":2.5,"impressions":10,"ctr":0.25,"position":3.0}]
            })))
            .mount(&api)
            .await;
        let adapter = GoogleSearchConsoleAdapter::with_endpoints(
            Url::parse(&format!("{}/token", token.uri())).unwrap(),
            Url::parse(&format!("{}/webmasters/v3/", api.uri())).unwrap(),
        )
        .await
        .unwrap();
        let batch = adapter
            .collect(
                &config(),
                "tenant-a",
                NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 10, 2).unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(batch.coverage, MetricCoverage::Partial);
        assert_eq!(batch.rows_received, 1);
        assert_eq!(
            batch.observations[0].value,
            Decimal::from_str("2.5").unwrap()
        );
        assert_eq!(batch.observations[1].value, Decimal::from(10));
        assert_eq!(batch.observations[2].value, Decimal::new(25, 2));
        assert_eq!(batch.observations[2].denominator, Some(Decimal::from(10)));
        assert_eq!(batch.observations[3].value, Decimal::from(3));
        assert_eq!(batch.window_timezone, "America/Los_Angeles");
    }

    #[tokio::test]
    async fn permission_failure_and_oversized_window_fail_closed() {
        let token = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token":"access-token"
            })))
            .mount(&token)
            .await;
        let api = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&api)
            .await;
        let adapter = GoogleSearchConsoleAdapter::with_endpoints(
            Url::parse(&format!("{}/token", token.uri())).unwrap(),
            Url::parse(&format!("{}/webmasters/v3/", api.uri())).unwrap(),
        )
        .await
        .unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(
            adapter
                .collect(&config(), "tenant-a", start, start + Days::new(92))
                .await
                .unwrap_err(),
            SearchConsoleError::InvalidRequest
        );
        assert_eq!(
            adapter
                .collect(&config(), "tenant-a", start, start)
                .await
                .unwrap_err(),
            SearchConsoleError::PermissionDenied
        );
    }

    #[tokio::test]
    async fn revoked_refresh_credentials_and_oversized_provider_bodies_fail_closed() {
        let token = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error":"invalid_grant","error_description":"private provider details"
            })))
            .mount(&token)
            .await;
        let api = MockServer::start().await;
        let adapter = GoogleSearchConsoleAdapter::with_endpoints(
            Url::parse(&format!("{}/token", token.uri())).unwrap(),
            Url::parse(&format!("{}/webmasters/v3/", api.uri())).unwrap(),
        )
        .await
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        assert_eq!(
            adapter
                .collect(&config(), "tenant-a", date, date)
                .await
                .unwrap_err(),
            SearchConsoleError::CredentialsRejected
        );

        let token = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token":"access-token"
            })))
            .mount(&token)
            .await;
        let api = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(1024 * 1024 + 1)))
            .mount(&api)
            .await;
        let adapter = GoogleSearchConsoleAdapter::with_endpoints(
            Url::parse(&format!("{}/token", token.uri())).unwrap(),
            Url::parse(&format!("{}/webmasters/v3/", api.uri())).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(
            adapter
                .collect(&config(), "tenant-a", date, date)
                .await
                .unwrap_err(),
            SearchConsoleError::MalformedResponse
        );
    }
}
