//! Provider-neutral bounded chat contracts used by the social commerce flow.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use stheno_types::Result;

/// One role-tagged message sent to or returned by a chat completion provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<AiToolCall>,
    pub tool_call_id: Option<String>,
}

/// A single bounded provider request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatRequest {
    pub model: String,
    pub messages: Vec<AiChatMessage>,
    pub tools: Vec<AiToolDefinition>,
    pub max_output_tokens: u32,
}

/// A normalized completion, refusal, or typed tool-call response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub message: AiChatMessage,
    pub refusal: Option<String>,
    pub usage: AiUsage,
}

/// A normalized function call; arguments remain JSON and are validated by the tool boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// JSON schema description exposed to a model as an available function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Provider-reported token usage, treated as untrusted accounting input.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Async boundary implemented by an LLM provider crate.
#[async_trait]
pub trait ChatCompletionProvider: Send + Sync {
    /// Completes one request under the provider's timeout and response-size limits.
    async fn complete(&self, request: AiChatRequest) -> Result<AiChatResponse>;
}
