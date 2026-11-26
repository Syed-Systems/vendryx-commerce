//! Read-only Facebook Page and Instagram professional-account insights.

use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveTime, Utc};
use hmac::{Hmac, Mac};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;
use stheno_ai::{MetaRuntimeConfig, MetricCoverage, SourceMetricObservation};
use url::Url;

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_DAYS: i64 = 90;
const MAX_PAGES: usize = 5;
const FACEBOOK_METRICS: &[(&str, &str, &str)] = &[
    (
        "page_post_engagements",
        "facebook-page-post-engagements-v1",
        "count",
    ),
    ("page_media_view", "facebook-page-media-views-v1", "count"),
];
const INSTAGRAM_METRICS: &[(&str, &str, &str)] = &[
    (
        "total_interactions",
        "instagram-account-total-interactions-v1",
        "count",
    ),
    ("reach", "instagram-account-reach-v1", "accounts"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaInsightsError {
    InvalidRequest,
    CredentialsRejected,
    PermissionDenied,
    RateLimited,
    ProviderUnavailable,
    MalformedResponse,
    RequestFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaInsightsKind {
    FacebookPage,
    InstagramAccount,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaInsightsBatch {
    pub source: &'static str,
    pub account_id: String,
    pub window_started_on: NaiveDate,
    /// Exclusive end date in UTC; the provider receives the inclusive prior day.
    pub window_ended_on: NaiveDate,
    pub window_timezone: &'static str,
    pub ingested_at: DateTime<Utc>,
    pub coverage: MetricCoverage,
    pub rows_received: usize,
    pub observations: Vec<SourceMetricObservation>,
}

#[derive(Clone)]
pub struct MetaInsightsAdapter {
    client: reqwest::Client,
    api_base_url: Url,
}

impl MetaInsightsAdapter {
    /// Creates a bounded client for the configured Graph API version (for example `v26.0`).
    pub fn new(graph_api_version: &str) -> Result<Self, MetaInsightsError> {
        let version = graph_api_version.trim_matches('/');
        if version.len() > 16
            || !version.strip_prefix('v').is_some_and(|part| {
                let mut sections = part.split('.');
                sections.next().is_some_and(|major| {
                    !major.is_empty() && major.bytes().all(|b| b.is_ascii_digit())
                }) && sections.next().is_some_and(|minor| {
                    !minor.is_empty() && minor.bytes().all(|b| b.is_ascii_digit())
                }) && sections.next().is_none()
            })
        {
            return Err(MetaInsightsError::InvalidRequest);
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|_| MetaInsightsError::RequestFailed)?;
        let api_base_url = Url::parse(&format!("https://graph.facebook.com/{version}/"))
            .map_err(|_| MetaInsightsError::InvalidRequest)?;
        Ok(Self {
            client,
            api_base_url,
        })
    }

    #[cfg(test)]
    async fn with_api_base(api_base_url: Url) -> Result<Self, MetaInsightsError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|_| MetaInsightsError::RequestFailed)?;
        Ok(Self {
            client,
            api_base_url,
        })
    }

    /// Reads fixed account-level aggregates without posts, messages, or audience demographics.
    pub async fn collect(
        &self,
        kind: MetaInsightsKind,
        config: &MetaRuntimeConfig,
        tenant_id: &str,
        start_date: NaiveDate,
        end_date_inclusive: NaiveDate,
    ) -> Result<MetaInsightsBatch, MetaInsightsError> {
        if config.tenant_id != tenant_id
            || config.access_token.trim().is_empty()
            || config.account_id.is_empty()
            || config.account_id.len() > 128
            || config
                .account_id
                .chars()
                .any(|c| !c.is_ascii_alphanumeric())
            || start_date > end_date_inclusive
            || end_date_inclusive
                .signed_duration_since(start_date)
                .num_days()
                + 1
                > MAX_DAYS
            || (kind == MetaInsightsKind::InstagramAccount && config.channel != "instagram")
            || (kind == MetaInsightsKind::FacebookPage && config.channel != "messenger")
        {
            return Err(MetaInsightsError::InvalidRequest);
        }

        let (source, metric_specs) = match kind {
            MetaInsightsKind::FacebookPage => ("facebook_page", FACEBOOK_METRICS),
            MetaInsightsKind::InstagramAccount => ("instagram_account", INSTAGRAM_METRICS),
        };
        let since = start_date
            .and_time(NaiveTime::MIN)
            .and_utc()
            .timestamp()
            .to_string();
        let until = end_date_inclusive
            .and_hms_opt(23, 59, 59)
            .ok_or(MetaInsightsError::InvalidRequest)?
            .and_utc()
            .timestamp()
            .to_string();
        let mut endpoint = self.api_base_url.clone();
        endpoint
            .path_segments_mut()
            .map_err(|_| MetaInsightsError::RequestFailed)?
            .pop_if_empty()
            .push(&config.account_id)
            .push("insights");

        let metric_names = metric_specs
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>()
            .join(",");
        let mut after: Option<String> = None;
        let mut seen_cursors = HashSet::new();
        let mut values: BTreeMap<(&'static str, &'static str, &'static str), Decimal> =
            BTreeMap::new();
        let mut returned_dates: BTreeMap<&str, HashSet<NaiveDate>> = BTreeMap::new();
        let mut partial = false;
        let mut rows_received = 0_usize;
        let proof = appsecret_proof(&config.access_token, &config.app_secret)?;

        for page in 0..MAX_PAGES {
            let mut request = self
                .client
                .get(endpoint.clone())
                .bearer_auth(&config.access_token)
                .query(&[("appsecret_proof", proof.as_str())]);
            match kind {
                MetaInsightsKind::FacebookPage => {
                    request = request.query(&[
                        ("metric", metric_names.as_str()),
                        ("period", "day"),
                        ("since", since.as_str()),
                        ("until", until.as_str()),
                    ]);
                }
                MetaInsightsKind::InstagramAccount => {
                    request = request.query(&[
                        ("metric", metric_names.as_str()),
                        ("period", "day"),
                        ("metric_type", "total_value"),
                        ("since", since.as_str()),
                        ("until", until.as_str()),
                    ]);
                }
            }
            if let Some(cursor) = after.as_deref() {
                request = request.query(&[("after", cursor)]);
            }
            let response = request
                .send()
                .await
                .map_err(|_| MetaInsightsError::RequestFailed)?;
            ensure_success(response.status().as_u16())?;
            let payload = read_json_bounded(response).await?;
            let data = payload
                .get("data")
                .and_then(Value::as_array)
                .ok_or(MetaInsightsError::MalformedResponse)?;
            for metric_data in data {
                let name = metric_data
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or(MetaInsightsError::MalformedResponse)?;
                let Some(spec) = metric_specs.iter().find(|spec| spec.0 == name) else {
                    return Err(MetaInsightsError::MalformedResponse);
                };
                let metric_name = spec.0;
                let definition = spec.1;
                let unit = spec.2;
                rows_received += 1;
                match kind {
                    MetaInsightsKind::FacebookPage => {
                        let entries = metric_data
                            .get("values")
                            .and_then(Value::as_array)
                            .ok_or(MetaInsightsError::MalformedResponse)?;
                        for entry in entries {
                            let number = entry
                                .get("value")
                                .and_then(Value::as_number)
                                .ok_or(MetaInsightsError::MalformedResponse)?;
                            let value = decimal(number)?;
                            let end_time = entry
                                .get("end_time")
                                .and_then(Value::as_str)
                                .ok_or(MetaInsightsError::MalformedResponse)?;
                            let date = parse_meta_timestamp(end_time)?
                                .checked_sub_signed(chrono::Duration::days(1))
                                .ok_or(MetaInsightsError::MalformedResponse)?
                                .date_naive();
                            if date < start_date || date > end_date_inclusive {
                                partial = true;
                                continue;
                            }
                            let by_date = returned_dates.entry(metric_name).or_default();
                            if !by_date.insert(date) {
                                partial = true;
                                continue;
                            }
                            let key = (metric_name, definition, unit);
                            let sum = values.get(&key).copied().unwrap_or(Decimal::ZERO);
                            values.insert(key, checked_add(sum, value)?);
                        }
                    }
                    MetaInsightsKind::InstagramAccount => {
                        let number = metric_data
                            .get("total_value")
                            .and_then(|total| total.get("value"))
                            .and_then(Value::as_number)
                            .ok_or(MetaInsightsError::MalformedResponse)?;
                        values.insert((metric_name, definition, unit), decimal(number)?);
                    }
                }
            }

            let next = payload
                .pointer("/paging/cursors/after")
                .and_then(Value::as_str)
                .filter(|cursor| !cursor.is_empty())
                .map(str::to_string);
            let Some(next) = next else {
                break;
            };
            if !seen_cursors.insert(next.clone()) {
                partial = true;
                break;
            }
            if page + 1 == MAX_PAGES {
                partial = true;
                break;
            }
            after = Some(next);
        }

        let mut observations = Vec::new();
        let expected_dates = end_date_inclusive
            .signed_duration_since(start_date)
            .num_days()
            + 1;
        for (name, definition, unit) in metric_specs {
            let value = values.get(&(*name, *definition, *unit)).copied();
            if value.is_none() && rows_received > 0 {
                partial = true;
            }
            let coverage = match (kind, value) {
                (MetaInsightsKind::InstagramAccount, Some(_)) => MetricCoverage::Complete,
                (_, Some(_)) => {
                    let dates = returned_dates
                        .get(name)
                        .map(HashSet::len)
                        .unwrap_or_default();
                    if dates as i64 == expected_dates && !partial {
                        MetricCoverage::Complete
                    } else {
                        MetricCoverage::Partial
                    }
                }
                _ if rows_received == 0 => MetricCoverage::Unavailable,
                _ => MetricCoverage::Partial,
            };
            if let Some(value) = value {
                observations.push(SourceMetricObservation {
                    tenant_id: tenant_id.to_string(),
                    source: source.to_string(),
                    account_id: config.account_id.clone(),
                    scope: "merchant_account".to_string(),
                    metric: name.to_string(),
                    definition: definition.to_string(),
                    unit: unit.to_string(),
                    value,
                    denominator: None,
                    sample_size: None,
                    window_started_on: start_date,
                    window_ended_on: end_date_inclusive
                        .checked_add_days(Days::new(1))
                        .ok_or(MetaInsightsError::InvalidRequest)?,
                    window_timezone: "UTC".to_string(),
                    ingested_at: Utc::now(),
                    coverage,
                });
            } else if coverage == MetricCoverage::Unavailable {
                observations.push(SourceMetricObservation {
                    tenant_id: tenant_id.to_string(),
                    source: source.to_string(),
                    account_id: config.account_id.clone(),
                    scope: "merchant_account".to_string(),
                    metric: name.to_string(),
                    definition: definition.to_string(),
                    unit: unit.to_string(),
                    value: Decimal::ZERO,
                    denominator: None,
                    sample_size: None,
                    window_started_on: start_date,
                    window_ended_on: end_date_inclusive
                        .checked_add_days(Days::new(1))
                        .ok_or(MetaInsightsError::InvalidRequest)?,
                    window_timezone: "UTC".to_string(),
                    ingested_at: Utc::now(),
                    coverage,
                });
            }
        }
        let coverage = if observations
            .iter()
            .all(|observation| observation.coverage == MetricCoverage::Unavailable)
        {
            MetricCoverage::Unavailable
        } else if partial
            || observations
                .iter()
                .any(|observation| observation.coverage != MetricCoverage::Complete)
        {
            MetricCoverage::Partial
        } else {
            MetricCoverage::Complete
        };
        Ok(MetaInsightsBatch {
            source,
            account_id: config.account_id.clone(),
            window_started_on: start_date,
            window_ended_on: end_date_inclusive
                .checked_add_days(Days::new(1))
                .ok_or(MetaInsightsError::InvalidRequest)?,
            window_timezone: "UTC",
            ingested_at: Utc::now(),
            coverage,
            rows_received,
            observations,
        })
    }
}

fn checked_add(left: Decimal, right: Decimal) -> Result<Decimal, MetaInsightsError> {
    left.checked_add(right)
        .ok_or(MetaInsightsError::MalformedResponse)
}

fn appsecret_proof(access_token: &str, app_secret: &str) -> Result<String, MetaInsightsError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .map_err(|_| MetaInsightsError::InvalidRequest)?;
    mac.update(access_token.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn parse_meta_timestamp(value: &str) -> Result<DateTime<chrono::FixedOffset>, MetaInsightsError> {
    DateTime::parse_from_rfc3339(value)
        .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%z"))
        .map_err(|_| MetaInsightsError::MalformedResponse)
}

fn decimal(number: &serde_json::Number) -> Result<Decimal, MetaInsightsError> {
    let value =
        Decimal::from_str(&number.to_string()).map_err(|_| MetaInsightsError::MalformedResponse)?;
    if value < Decimal::ZERO {
        return Err(MetaInsightsError::MalformedResponse);
    }
    Ok(value)
}

fn ensure_success(status: u16) -> Result<(), MetaInsightsError> {
    match status {
        200..=299 => Ok(()),
        401 => Err(MetaInsightsError::CredentialsRejected),
        403 => Err(MetaInsightsError::PermissionDenied),
        429 => Err(MetaInsightsError::RateLimited),
        500..=599 => Err(MetaInsightsError::ProviderUnavailable),
        _ => Err(MetaInsightsError::RequestFailed),
    }
}

async fn read_json_bounded(mut response: reqwest::Response) -> Result<Value, MetaInsightsError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(MetaInsightsError::MalformedResponse);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| MetaInsightsError::RequestFailed)?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(MetaInsightsError::MalformedResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| MetaInsightsError::MalformedResponse)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config(channel: &str) -> MetaRuntimeConfig {
        MetaRuntimeConfig {
            tenant_id: "tenant-a".to_string(),
            channel: channel.to_string(),
            account_id: "1234567890".to_string(),
            webhook_id: "webhook-id".to_string(),
            app_secret: "app-secret".to_string(),
            verify_token: "verify-token".to_string(),
            access_token: "access-token".to_string(),
        }
    }

    #[tokio::test]
    async fn page_insights_aggregate_fixed_metrics_and_bound_date_window() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v26.0/1234567890/insights"))
            .and(header("authorization", "Bearer access-token"))
            .and(query_param(
                "metric",
                "page_post_engagements,page_media_view",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"name":"page_post_engagements","values":[
                        {"value":2,"end_time":"2026-10-02T00:00:00+0000"},
                        {"value":3,"end_time":"2026-10-03T00:00:00+0000"}
                    ]},
                    {"name":"page_media_view","values":[
                        {"value":10,"end_time":"2026-10-02T00:00:00+0000"},
                        {"value":11,"end_time":"2026-10-03T00:00:00+0000"}
                    ]}
                ]
            })))
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let batch = adapter
            .collect(
                MetaInsightsKind::FacebookPage,
                &config("messenger"),
                "tenant-a",
                NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 10, 2).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(batch.coverage, MetricCoverage::Complete);
        assert_eq!(batch.observations[0].value, Decimal::from(5));
        assert_eq!(batch.observations[1].value, Decimal::from(21));
    }

    #[tokio::test]
    async fn instagram_uses_total_values_and_empty_provider_data_is_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v26.0/1234567890/insights"))
            .and(query_param("metric_type", "total_value"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"name":"total_interactions","total_value":{"value":7}},
                    {"name":"reach","total_value":{"value":9}}
                ]
            })))
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let batch = adapter
            .collect(
                MetaInsightsKind::InstagramAccount,
                &config("instagram"),
                "tenant-a",
                date,
                date,
            )
            .await
            .unwrap();
        assert_eq!(batch.coverage, MetricCoverage::Complete);
        assert_eq!(batch.observations[0].value, Decimal::from(7));
        assert_eq!(batch.observations[1].value, Decimal::from(9));

        let empty = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":[]})))
            .mount(&empty)
            .await;
        let empty_adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", empty.uri())).unwrap(),
        )
        .await
        .unwrap();
        let unavailable = empty_adapter
            .collect(
                MetaInsightsKind::InstagramAccount,
                &config("instagram"),
                "tenant-a",
                date,
                date,
            )
            .await
            .unwrap();
        assert_eq!(unavailable.coverage, MetricCoverage::Unavailable);
        assert!(unavailable
            .observations
            .iter()
            .all(|observation| observation.coverage == MetricCoverage::Unavailable));
    }

    #[tokio::test]
    async fn page_insights_follow_bounded_pagination_without_losing_coverage() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v26.0/1234567890/insights"))
            .and(query_param("after", "cursor-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"name":"page_post_engagements","values":[{"value":3,"end_time":"2026-10-03T00:00:00+0000"}]},
                    {"name":"page_media_view","values":[{"value":13,"end_time":"2026-10-03T00:00:00+0000"}]}
                ]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v26.0/1234567890/insights"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"name":"page_post_engagements","values":[{"value":2,"end_time":"2026-10-02T00:00:00+0000"}]},
                    {"name":"page_media_view","values":[{"value":10,"end_time":"2026-10-02T00:00:00+0000"}]}
                ],
                "paging":{"cursors":{"after":"cursor-1"}}
            })))
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let batch = adapter
            .collect(
                MetaInsightsKind::FacebookPage,
                &config("messenger"),
                "tenant-a",
                NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 10, 2).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(batch.coverage, MetricCoverage::Complete);
        assert_eq!(batch.observations[0].value, Decimal::from(5));
        assert_eq!(batch.observations[1].value, Decimal::from(23));
    }

    #[tokio::test]
    async fn oversized_bodies_and_decimal_overflow_fail_closed() {
        assert_eq!(
            checked_add(Decimal::MAX, Decimal::ONE),
            Err(MetaInsightsError::MalformedResponse)
        );

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("x".repeat(MAX_RESPONSE_BYTES + 1)),
            )
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        assert!(matches!(
            adapter
                .collect(
                    MetaInsightsKind::FacebookPage,
                    &config("messenger"),
                    "tenant-a",
                    date,
                    date,
                )
                .await,
            Err(MetaInsightsError::MalformedResponse)
        ));
    }

    #[tokio::test]
    async fn wrong_tenant_permissions_and_invalid_version_fail_closed() {
        assert!(matches!(
            MetaInsightsAdapter::new("https://evil.example/v26.0"),
            Err(MetaInsightsError::InvalidRequest)
        ));
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        assert!(matches!(
            adapter
                .collect(
                    MetaInsightsKind::InstagramAccount,
                    &config("instagram"),
                    "tenant-b",
                    date,
                    date,
                )
                .await,
            Err(MetaInsightsError::InvalidRequest)
        ));
        assert!(matches!(
            adapter
                .collect(
                    MetaInsightsKind::InstagramAccount,
                    &config("instagram"),
                    "tenant-a",
                    date,
                    date,
                )
                .await,
            Err(MetaInsightsError::PermissionDenied)
        ));
    }

    #[tokio::test]
    async fn rate_limit_is_reported_without_retrying_the_provider_request() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;
        let adapter = MetaInsightsAdapter::with_api_base(
            Url::parse(&format!("{}/v26.0/", server.uri())).unwrap(),
        )
        .await
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();

        assert!(matches!(
            adapter
                .collect(
                    MetaInsightsKind::FacebookPage,
                    &config("messenger"),
                    "tenant-a",
                    date,
                    date,
                )
                .await,
            Err(MetaInsightsError::RateLimited)
        ));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
