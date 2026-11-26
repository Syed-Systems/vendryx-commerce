//! Cache purge operations for the Cloudflare zone API.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.cloudflare.com/client/v4";
const MAX_ITEMS_PER_REQUEST: usize = 100;

/// A configured or disabled cache purge client.
///
/// It is disabled unless both environment variables are present. The bearer token is kept
/// private and is never included in errors or debug output.
pub struct CloudflareCdn {
    client: Client,
    zone_id: Option<String>,
    token: Option<String>,
    api_url: String,
    timeout: Duration,
}

impl CloudflareCdn {
    /// Reads optional API credentials without making startup depend on them.
    pub fn from_env() -> Self {
        Self::new(
            std::env::var("CLOUDFLARE_ZONE_ID").ok(),
            std::env::var("CLOUDFLARE_API_TOKEN").ok(),
            API_URL.to_string(),
            Duration::from_secs(10),
        )
    }

    fn new(
        zone_id: Option<String>,
        token: Option<String>,
        api_url: String,
        timeout: Duration,
    ) -> Self {
        let configured = zone_id.filter(|id| valid_zone_id(id));
        Self {
            client: Client::new(),
            zone_id: configured,
            token: token.filter(|value| !value.is_empty()),
            api_url,
            timeout,
        }
    }

    /// Purges cached responses carrying any of the supplied tags.
    pub async fn purge_tags(&self, tags: &[String]) -> Result<(), PurgeError> {
        let (Some(zone_id), Some(token)) = (&self.zone_id, &self.token) else {
            return Ok(());
        };
        if !tags.iter().all(|tag| is_valid_tag(tag)) {
            return Err(PurgeError::InvalidTarget);
        }
        for chunk in tags.chunks(MAX_ITEMS_PER_REQUEST) {
            self.send(zone_id, token, &PurgeBody::Tags { tags: chunk })
                .await?;
        }
        Ok(())
    }

    /// Purges cached responses served for any of the supplied hostnames.
    pub async fn purge_hostnames(&self, hostnames: &[String]) -> Result<(), PurgeError> {
        let (Some(zone_id), Some(token)) = (&self.zone_id, &self.token) else {
            return Ok(());
        };
        if !hostnames.iter().all(|hostname| is_valid_hostname(hostname)) {
            return Err(PurgeError::InvalidTarget);
        }
        for chunk in hostnames.chunks(MAX_ITEMS_PER_REQUEST) {
            self.send(zone_id, token, &PurgeBody::Hosts { hosts: chunk })
                .await?;
        }
        Ok(())
    }

    async fn send(
        &self,
        zone_id: &str,
        token: &str,
        body: &PurgeBody<'_>,
    ) -> Result<(), PurgeError> {
        let endpoint = format!("{}/zones/{zone_id}/purge_cache", self.api_url);
        let request = self
            .client
            .post(endpoint)
            .bearer_auth(token)
            .json(body)
            .send();
        let response = tokio::time::timeout(self.timeout, request)
            .await
            .map_err(|_| PurgeError::Timeout)?
            .map_err(|_| PurgeError::Transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(PurgeError::RejectedStatus(status.as_u16()));
        }

        let response = tokio::time::timeout(self.timeout, response.json::<ApiResponse>())
            .await
            .map_err(|_| PurgeError::Timeout)?
            .map_err(|_| PurgeError::InvalidResponse)?;
        if response.success {
            Ok(())
        } else {
            Err(PurgeError::Rejected)
        }
    }
}

/// Errors intentionally omit API response bodies and request credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeError {
    InvalidTarget,
    Timeout,
    Transport,
    RejectedStatus(u16),
    Rejected,
    InvalidResponse,
}

impl std::fmt::Display for PurgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTarget => formatter.write_str("cache purge target is invalid"),
            Self::Timeout => formatter.write_str("cache purge request timed out"),
            Self::Transport => formatter.write_str("cache purge transport failed"),
            Self::RejectedStatus(status) => {
                write!(formatter, "cache purge was rejected with HTTP {status}")
            }
            Self::Rejected => formatter.write_str("cache purge was not accepted"),
            Self::InvalidResponse => formatter.write_str("cache purge response was invalid"),
        }
    }
}

impl std::error::Error for PurgeError {}

#[derive(Serialize)]
#[serde(untagged)]
enum PurgeBody<'a> {
    Tags { tags: &'a [String] },
    Hosts { hosts: &'a [String] },
}

#[derive(Deserialize)]
struct ApiResponse {
    success: bool,
}

fn valid_zone_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_valid_tag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value.contains(',')
        && value
            .bytes()
            .all(|byte| (0x21..=0x7e).contains(&byte))
}

fn is_valid_hostname(value: &str) -> bool {
    if value.is_empty() || value.len() > 253 || !value.is_ascii() {
        return false;
    }
    value.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

#[cfg(test)]
mod tests {
    use super::{CloudflareCdn, PurgeError};
    use std::time::Duration;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn configured(server: &MockServer, timeout: Duration) -> CloudflareCdn {
        CloudflareCdn::new(
            Some("0123456789abcdef0123456789abcdef".to_string()),
            Some("test-token-do-not-log".to_string()),
            server.uri(),
            timeout,
        )
    }

    #[tokio::test]
    async fn purges_tags_with_the_tag_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/zones/0123456789abcdef0123456789abcdef/purge_cache"))
            .and(header("authorization", "Bearer test-token-do-not-log"))
            .and(body_json(serde_json::json!({"tags": ["tenant:ten_1:product:prod_1"]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = configured(&server, Duration::from_secs(1));
        assert_eq!(
            client
                .purge_tags(&["tenant:ten_1:product:prod_1".to_string()])
                .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn purges_hostnames_with_the_host_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/zones/0123456789abcdef0123456789abcdef/purge_cache"))
            .and(body_json(serde_json::json!({"hosts": ["shop.example.test"]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        assert_eq!(
            configured(&server, Duration::from_secs(1))
                .purge_hostnames(&["shop.example.test".to_string()])
                .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn exposes_retryable_status_without_the_provider_body() {
        for status in [429, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(status).set_body_string("secret response body"),
                )
                .expect(1)
                .mount(&server)
                .await;

            let error = configured(&server, Duration::from_secs(1))
                .purge_tags(&["tenant:ten_1:listing".to_string()])
                .await
                .expect_err("non-success status must be retried by the caller");
            assert_eq!(error, PurgeError::RejectedStatus(status));
            assert!(!error.to_string().contains("secret response body"));
            assert!(!format!("{error:?}").contains("test-token-do-not-log"));
        }
    }

    #[tokio::test]
    async fn returns_timeout_without_exposing_request_secrets() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(200))
                    .set_body_json(serde_json::json!({"success": true})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let error = configured(&server, Duration::from_millis(20))
            .purge_tags(&["tenant:ten_1:listing".to_string()])
            .await
            .expect_err("request must honor the configured timeout");
        assert_eq!(error, PurgeError::Timeout);
        assert!(!format!("{error:?}").contains("test-token-do-not-log"));
    }

    #[tokio::test]
    async fn missing_credentials_make_purging_a_no_op() {
        let client = CloudflareCdn::new(None, None, "invalid".to_string(), Duration::ZERO);
        assert_eq!(client.purge_tags(&[]).await, Ok(()));
        assert_eq!(
            client
                .purge_hostnames(&["not a hostname".to_string()])
                .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn batches_more_than_one_hundred_items() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .expect(2)
            .mount(&server)
            .await;
        let tags: Vec<_> = (0..101)
            .map(|index| format!("tenant:ten_1:product:prod_{index}"))
            .collect();

        assert_eq!(
            configured(&server, Duration::from_secs(1))
                .purge_tags(&tags)
                .await,
            Ok(())
        );
    }
}
