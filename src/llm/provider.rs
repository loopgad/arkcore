//! LLM Provider 模块
//!
//! 提供 LLM Provider 的 trait 定义和多种实现

pub mod mock;

use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub use crate::core::traits::{Message, MessageRole, StreamEvent, LLMProvider as LlmProviderTrait};
// Re-export LlmError from core traits to avoid duplicate definitions
pub use crate::core::traits::LlmError;

/// LLM 配置
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// API Key
    pub api_key: Option<String>,
    /// 模型名称
    pub model: String,
    /// API Base URL
    pub base_url: String,
    /// 请求超时（秒）
    pub timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "gpt-4".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout: Duration::from_secs(60),
            max_retries: 3,
        }
    }
}

impl LlmConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ARKCORE_LLM_API_KEY").ok(),
            model: std::env::var("ARKCORE_LLM_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
            base_url: std::env::var("ARKCORE_LLM_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            timeout: Duration::from_secs(60),
            max_retries: 3,
        }
    }

    /// Ollama 配置（简化创建）
    pub fn ollama(model: &str) -> Self {
        Self {
            api_key: None,
            model: model.to_string(),
            base_url: std::env::var("ARKCORE_OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            timeout: Duration::from_secs(120),
            max_retries: 3,
        }
    }
}

/// OpenAI Provider (简化实现)
pub struct OpenAiProvider {
    config: LlmConfig,
}

impl OpenAiProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

impl LlmProviderTrait for OpenAiProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        Ok(vec![StreamEvent {
            content: format!("OpenAI response to: {}", messages.last().map(|m| m.content.as_str()).unwrap_or("")),
            done: true,
        }])
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

/// Anthropic Provider (简化实现)
pub struct AnthropicProvider {
    config: LlmConfig,
}

impl AnthropicProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

impl LlmProviderTrait for AnthropicProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        Ok(vec![StreamEvent {
            content: format!("Anthropic response to: {}", messages.last().map(|m| m.content.as_str()).unwrap_or("")),
            done: true,
        }])
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

// ============================================================================
// Ollama Provider - 真实 API 调用实现
// ============================================================================

/// Ollama API 请求
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API 响应
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessageContent,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageContent {
    content: String,
}

/// Ollama Provider - 支持本地 LLM 推理
///
/// 通过 Ollama API 与本地运行的 LLM 进行交互。
///
/// # 示例
///
/// ```rust,ignore
/// use arkcore::llm::provider::{LlmConfig, OllamaProvider};
///
/// let config = LlmConfig::ollama("llama3.2");
/// let provider = OllamaProvider::new(config);
/// ```
pub struct OllamaProvider {
    config: LlmConfig,
    client: Client,
}

impl OllamaProvider {
    /// 创建新的 Ollama Provider
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// 检查 Ollama 服务是否可用
    pub async fn health_check(&self) -> Result<bool, LlmError> {
        let url = format!("{}/api/tags", self.config.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl LlmProviderTrait for OllamaProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        // 同步版本：使用 block_on
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create runtime: {}", e)))?;

        runtime.block_on(self.stream_chat_async(messages))
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

impl OllamaProvider {
    /// 异步流式聊天实现
    async fn stream_chat_async(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, LlmError> {
        let url = format!("{}/api/chat", self.config.base_url);

        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                }.to_string(),
                content: m.content.clone(),
            })
            .collect();

        let request = OllamaRequest {
            model: self.config.model.clone(),
            messages: ollama_messages,
            stream: true,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(LlmError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Ollama API error: {}", response.status()),
            });
        }

        // 解析 SSE 流
        let mut stream = response.bytes_stream();
        let mut events = Vec::new();
        let mut buffer = String::new();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        buffer.push_str(&text);
                        // 处理 SSE 事件（每行以 data: 开头）
                        for line in buffer.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                let data_trimmed = data.trim();
                                if data_trimmed == "[DONE]" {
                                    events.push(StreamEvent {
                                        content: String::new(),
                                        done: true,
                                    });
                                    return Ok(events);
                                }
                                // 尝试解析响应
                                let resp: Result<OllamaResponse, _> = serde_json::from_str(data_trimmed);
                                if let Ok(resp) = resp {
                                    events.push(StreamEvent {
                                        content: resp.message.content,
                                        done: resp.done,
                                    });
                                    if resp.done {
                                        return Ok(events);
                                    }
                                }
                            }
                        }
                        buffer.clear();
                    }
                }
                Err(e) => {
                    return Err(LlmError::NetworkError(format!("Stream error: {}", e)));
                }
            }
        }

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_llm_config_ollama() {
        let config = LlmConfig::ollama("llama3.2");
        assert_eq!(config.model, "llama3.2");
        assert_eq!(config.base_url, "http://localhost:11434");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_openai_provider() {
        let config = LlmConfig::default();
        let provider = OpenAiProvider::new(config);
        assert_eq!(provider.model_name(), "gpt-4");
    }

    #[test]
    fn test_anthropic_provider() {
        let config = LlmConfig {
            model: "claude-3-sonnet".to_string(),
            ..Default::default()
        };
        let provider = AnthropicProvider::new(config);
        assert_eq!(provider.model_name(), "claude-3-sonnet");
    }

    #[test]
    fn test_ollama_provider() {
        let config = LlmConfig::ollama("llama3.2");
        let provider = OllamaProvider::new(config);
        assert_eq!(provider.model_name(), "llama3.2");
    }

    #[test]
    fn test_llm_error_display() {
        let err = LlmError::ConfigError("test".to_string());
        assert!(format!("{}", err).contains("ConfigError"));
    }
}
