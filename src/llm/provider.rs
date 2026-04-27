//! LLM Provider 模块
//!
//! 提供 LLM Provider 的 trait 定义和多种实现

pub mod mock;

use std::time::Duration;

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
    fn test_llm_error_display() {
        let err = LlmError::ConfigError("test".to_string());
        assert!(format!("{}", err).contains("ConfigError"));
    }
}