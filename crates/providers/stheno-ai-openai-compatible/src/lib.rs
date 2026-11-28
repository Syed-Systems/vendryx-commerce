//! OpenAI-compatible low-cost chat completion provider (DeepSeek, Qwen-compatible endpoints).

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use stheno_ai::{
    AiChatMessage, AiChatRequest, AiChatResponse, AiToolCall, AiUsage, ChatCompletionProvider,
};
use stheno_types::{Error, Result};

const MAX_REQUEST_BYTES: usize = 256 * 1024;
const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_OUTPUT_TOKENS: u32 = 4096;

/// Runtime credentials and endpoint for an OpenAI-compatible provider.
///
/// This type intentionally omits `Debug` so API keys cannot leak through diagnostic logging.
#[derive(Clone)]
pub struct OpenAiCompatibleConfig {
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
}

/// Stateless chat completion adapter with bounded request and response bodies.
pub struct OpenAiCompatibleProvider {
    config: OpenAiCompatibleConfig,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    /// Creates the provider with a ten-second transport timeout and TLS validation enabled.
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self> {
        validate_config(&config, false)?;
        Self::build(config)
    }

    /// Creates a provider for the server's fixed loopback Ollama endpoint only.
    pub fn new_local(config: OpenAiCompatibleConfig) -> Result<Self> {
        if config.endpoint != "http://127.0.0.1:11434/v1/chat/completions" {
            return Err(invalid(
                "Local AI must use the configured loopback Ollama endpoint.",
            ));
        }
        validate_config(&config, true)?;
        Self::build(config)
    }

    fn build(config: OpenAiCompatibleConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(provider_error)?;
        Ok(Self { config, client })
    }

    /// Uses a supplied loopback client for deterministic contract tests.
    #[cfg(test)]
    fn with_client(config: OpenAiCompatibleConfig, client: reqwest::Client) -> Result<Self> {
        validate_config(&config, true)?;
        Ok(Self { config, client })
    }
}

#[async_trait]
impl ChatCompletionProvider for OpenAiCompatibleProvider {
    async fn complete(&self, request: AiChatRequest) -> Result<AiChatResponse> {
        validate_request(&request, &self.config)?;
        let body = serde_json::to_vec(&json!({
            "model": request.model,
            "messages": request.messages.iter().map(serialize_message).collect::<Result<Vec<_>>>()?,
            "tools": request.tools.into_iter().map(|tool| json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters
                }
            })).collect::<Vec<_>>(),
            "tool_choice": "auto",
            "parallel_tool_calls": false,
            "max_tokens": request.max_output_tokens,
            "temperature": 0.2
        }))
        .map_err(provider_error)?;
        if body.len() > MAX_REQUEST_BYTES {
            return Err(invalid(
                "AI provider request exceeds the configured size limit.",
            ));
        }

        let response = self
            .client
            .post(&self.config.endpoint)
            .bearer_auth(&self.config.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(provider_error)?;
        let status = response.status();
        if !status.is_success() {
            // Do not include provider response bodies: they may echo prompts or credentials.
            return Err(Error::ProviderFailed {
                provider: "openai_compatible".to_string(),
                source: Box::new(std::io::Error::other(format!(
                    "provider returned HTTP {}",
                    status.as_u16()
                ))),
            });
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(invalid(
                "AI provider response exceeds the configured size limit.",
            ));
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(provider_error)? {
            if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
                return Err(invalid(
                    "AI provider response exceeds the configured size limit.",
                ));
            }
            bytes.extend_from_slice(&chunk);
        }

        let envelope: CompletionEnvelope =
            serde_json::from_slice(&bytes).map_err(provider_error)?;
        let choice = match envelope.choices.as_slice() {
            [choice] => choice,
            _ => return Err(invalid("AI provider returned an invalid choice count.")),
        };
        let tool_calls = choice
            .message
            .tool_calls
            .iter()
            .map(|call| {
                let arguments = serde_json::from_str(&call.function.arguments)
                    .unwrap_or_else(|_| Value::String(call.function.arguments.clone()));
                AiToolCall {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    arguments,
                }
            })
            .collect();
        let refusal = choice
            .message
            .refusal
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                (choice.finish_reason == "content_filter").then(|| "Provider refusal".to_string())
            });

        Ok(AiChatResponse {
            message: AiChatMessage {
                role: "assistant".to_string(),
                content: choice.message.content.clone(),
                tool_calls,
                tool_call_id: None,
            },
            refusal,
            usage: AiUsage {
                prompt_tokens: envelope.usage.prompt_tokens,
                completion_tokens: envelope.usage.completion_tokens,
            },
        })
    }
}

fn serialize_message(message: &AiChatMessage) -> Result<Value> {
    let mut serialized = json!({ "role": message.role });
    if let Some(content) = &message.content {
        serialized["content"] = Value::String(content.clone());
    }
    if !message.tool_calls.is_empty() {
        serialized["tool_calls"] = Value::Array(
            message
                .tool_calls
                .iter()
                .map(|call| {
                    Ok(json!({
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": call.name,
                            "arguments": serde_json::to_string(&call.arguments).map_err(provider_error)?
                        }
                    }))
                })
                .collect::<Result<Vec<_>>>()?,
        );
    }
    if let Some(id) = &message.tool_call_id {
        serialized["tool_call_id"] = Value::String(id.clone());
    }
    Ok(serialized)
}

fn validate_config(config: &OpenAiCompatibleConfig, allow_loopback: bool) -> Result<()> {
    let endpoint =
        reqwest::Url::parse(&config.endpoint).map_err(|_| invalid("Invalid AI endpoint URL."))?;
    let loopback = allow_loopback
        && endpoint.scheme() == "http"
        && matches!(endpoint.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    let approved_host = matches!(
        endpoint.host_str(),
        Some("api.deepseek.com" | "dashscope.aliyuncs.com")
    );
    if (!loopback && (!approved_host || endpoint.scheme() != "https"))
        || !endpoint.path().ends_with("/chat/completions")
        || endpoint.host_str().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(invalid(
            "AI endpoint must use HTTPS, except for a loopback test endpoint.",
        ));
    }
    if config.api_key.trim().is_empty() || config.api_key.len() > 4096 {
        return Err(invalid("AI API key is missing or too long."));
    }
    if config.model.trim().is_empty() || config.model.len() > 128 {
        return Err(invalid("AI model name is missing or too long."));
    }
    Ok(())
}

fn validate_request(request: &AiChatRequest, config: &OpenAiCompatibleConfig) -> Result<()> {
    if request.model != config.model {
        return Err(invalid(
            "AI request model does not match the configured model.",
        ));
    }
    if request.messages.is_empty() || request.messages.len() > 24 {
        return Err(invalid("AI message count is outside the supported range."));
    }
    if !(1..=MAX_OUTPUT_TOKENS).contains(&request.max_output_tokens) {
        return Err(invalid(
            "AI output token budget is outside the supported range.",
        ));
    }
    if request.tools.len() > 8 {
        return Err(invalid("AI tool count exceeds the supported limit."));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_provider_invalid_request",
        message: message.to_string(),
        field: None,
    }
}

fn provider_error(error: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::ProviderFailed {
        provider: "openai_compatible".to_string(),
        source: Box::new(error),
    }
}

#[derive(Deserialize)]
struct CompletionEnvelope {
    choices: Vec<CompletionChoice>,
    #[serde(default)]
    usage: CompletionUsage,
}

#[derive(Deserialize)]
struct CompletionChoice {
    message: CompletionMessage,
    #[serde(default)]
    finish_reason: String,
}

#[derive(Deserialize)]
struct CompletionMessage {
    content: Option<String>,
    #[serde(default)]
    refusal: Option<String>,
    #[serde(default)]
    tool_calls: Vec<CompletionToolCall>,
}

#[derive(Deserialize)]
struct CompletionToolCall {
    id: String,
    function: CompletionFunction,
}

#[derive(Deserialize)]
struct CompletionFunction {
    name: String,
    arguments: String,
}

#[derive(Default, Deserialize)]
struct CompletionUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use stheno_ai::{AiChatMessage, AiToolDefinition};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn request() -> AiChatRequest {
        AiChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![AiChatMessage {
                role: "user".to_string(),
                content: Some("Is this in stock?".to_string()),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            tools: vec![AiToolDefinition {
                name: "search_catalog".to_string(),
                description: "Search published catalog".to_string(),
                parameters: json!({"type":"object","properties":{}}),
            }],
            max_output_tokens: 256,
        }
    }

    async fn provider(server: &MockServer) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::with_client(
            OpenAiCompatibleConfig {
                endpoint: format!("{}/chat/completions", server.uri()),
                model: "deepseek-chat".to_string(),
                api_key: "test-secret".to_string(),
            },
            reqwest::Client::new(),
        )
        .expect("valid loopback provider")
    }

    #[tokio::test]
    async fn parses_a_typed_tool_call_and_usage() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{"id":"call_1","type":"function","function":{"name":"search_catalog","arguments":"{\"query\":\"lamp\"}"}}]}}],
                "usage": {"prompt_tokens": 50, "completion_tokens": 7}
            })))
            .mount(&server)
            .await;
        let result = provider(&server)
            .await
            .complete(request())
            .await
            .expect("completion");
        assert_eq!(result.message.tool_calls[0].name, "search_catalog");
        assert_eq!(result.message.tool_calls[0].arguments["query"], "lamp");
        assert_eq!(result.usage.prompt_tokens, 50);
        let sent = server.received_requests().await.expect("request capture");
        let body: Value = serde_json::from_slice(&sent[0].body).expect("request JSON");
        assert_eq!(body["model"], "deepseek-chat");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["tools"][0]["type"], "function");
    }

    #[tokio::test]
    async fn provider_status_errors_do_not_echo_response_bodies_or_retry() {
        for status in [429, 500, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status).set_body_string("secret-echo"))
                .mount(&server)
                .await;
            let error = provider(&server)
                .await
                .complete(request())
                .await
                .unwrap_err();
            assert!(!error.to_string().contains("secret-echo"));
            assert_eq!(server.received_requests().await.expect("requests").len(), 1);
        }
    }

    #[tokio::test]
    async fn timeout_and_malformed_provider_response_fail_without_echo_or_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(50))
                    .set_body_json(json!({"choices":[]})),
            )
            .mount(&server)
            .await;
        let timeout_provider = OpenAiCompatibleProvider::with_client(
            OpenAiCompatibleConfig {
                endpoint: format!("{}/chat/completions", server.uri()),
                model: "deepseek-chat".to_string(),
                api_key: "test-secret".to_string(),
            },
            reqwest::Client::builder()
                .timeout(Duration::from_millis(5))
                .build()
                .expect("short timeout client"),
        )
        .expect("loopback provider");
        let error = timeout_provider.complete(request()).await.unwrap_err();
        assert!(!error.to_string().contains("test-secret"));
        assert_eq!(server.received_requests().await.expect("requests").len(), 1);

        let malformed = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("private provider echo"))
            .mount(&malformed)
            .await;
        let error = provider(&malformed)
            .await
            .complete(request())
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("private provider echo"));
    }

    #[tokio::test]
    async fn oversized_provider_response_is_rejected_without_echo_or_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("private-echo:{}", "x".repeat(MAX_RESPONSE_BYTES))),
            )
            .mount(&server)
            .await;

        let error = provider(&server)
            .await
            .complete(request())
            .await
            .expect_err("oversized response must fail closed");
        assert!(!error.to_string().contains("private-echo"));
        assert_eq!(server.received_requests().await.expect("requests").len(), 1);
    }

    #[tokio::test]
    async fn provider_refusal_is_normalized_without_exposing_internal_reason() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices":[{"finish_reason":"stop","message":{"role":"assistant","content":null,"refusal":"restricted request"}}]
            })))
            .mount(&server)
            .await;
        let result = provider(&server)
            .await
            .complete(request())
            .await
            .expect("normalized refusal");
        assert_eq!(result.refusal.as_deref(), Some("restricted request"));
        assert!(result.message.content.is_none());
    }

    #[test]
    fn rejects_insecure_remote_endpoint_and_unbounded_requests() {
        let remote_http = OpenAiCompatibleConfig {
            endpoint: "http://example.com/chat/completions".to_string(),
            model: "qwen".to_string(),
            api_key: "secret".to_string(),
        };
        assert!(OpenAiCompatibleProvider::new(remote_http).is_err());
        let mut request = request();
        request.messages.resize(25, request.messages[0].clone());
        assert!(validate_request(
            &request,
            &OpenAiCompatibleConfig {
                endpoint: "https://api.deepseek.com/v1/chat/completions".into(),
                model: "deepseek-chat".into(),
                api_key: "secret".into(),
            }
        )
        .is_err());
    }

    #[tokio::test]
    async fn configured_model_cannot_be_overridden_by_a_request() {
        let server = MockServer::start().await;
        let mut request = request();
        request.model = "expensive-unapproved-model".to_string();
        let error = provider(&server)
            .await
            .complete(request)
            .await
            .expect_err("request model must match configured model");
        assert!(!error.to_string().contains("test-secret"));
        assert!(server
            .received_requests()
            .await
            .expect("requests")
            .is_empty());
    }
}
