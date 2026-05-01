//! Mock LLM Provider
//!
//! 用于测试的模拟 LLM Provider

use std::collections::HashMap;

#[cfg(test)]
use super::MessageRole;
use super::{LlmError, Message, StreamEvent};
use crate::core::traits::LLMProvider;

/// Mock LLM Provider - 用于测试的模拟实现
pub struct MockLlmProvider {
    responses: HashMap<String, Vec<StreamEvent>>,
    delay_ms: u64,
    should_error: Option<LlmError>,
    call_count: u32,
    model_name: String,
}

impl MockLlmProvider {
    /// 创建新的 Mock Provider
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            delay_ms: 0,
            should_error: None,
            call_count: 0,
            model_name: "mock-gpt-4".to_string(),
        }
    }

    /// 添加预定义的响应
    pub fn with_response(mut self, prompt_pattern: &str, events: Vec<StreamEvent>) -> Self {
        self.responses.insert(prompt_pattern.to_string(), events);
        self
    }

    /// 添加简单的文本响应
    pub fn with_text_response(self, text: &str) -> Self {
        let events = vec![
            StreamEvent {
                content: text.to_string(),
                done: false,
            },
            StreamEvent {
                content: String::new(),
                done: true,
            },
        ];
        self.with_response("default", events)
    }

    /// 设置响应延迟（毫秒）
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// 设置 Provider 返回错误
    pub fn with_error(mut self, error: LlmError) -> Self {
        self.should_error = Some(error);
        self
    }

    /// 设置模型名称
    pub fn with_model(mut self, model: &str) -> Self {
        self.model_name = model.to_string();
        self
    }

    /// 获取调用次数
    pub fn call_count(&self) -> u32 {
        self.call_count
    }

    /// 重置调用计数
    pub fn reset_call_count(&mut self) {
        self.call_count = 0;
    }

    /// 查找匹配的响应
    fn find_matching_response(&self, messages: &[Message]) -> Vec<StreamEvent> {
        let last_message = messages.last().map(|m| m.content.as_str()).unwrap_or("");

        // 尝试精确匹配
        if let Some(response) = self.responses.get(last_message) {
            return response.clone();
        }

        // 尝试模式匹配
        for (pattern, response) in &self.responses {
            if last_message.contains(pattern) {
                return response.clone();
            }
        }

        // 返回默认响应
        vec![StreamEvent {
            content: format!("Mock response to: {}", last_message),
            done: true,
        }]
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LLMProvider for MockLlmProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        // 返回错误如果配置了
        if let Some(ref error) = self.should_error {
            return Err(error.clone());
        }

        // 构建响应
        let response = self.find_matching_response(messages);

        Ok(response)
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn is_retriable_error(error: &crate::core::traits::LlmError) -> bool {
        match error {
            crate::core::traits::LlmError::NetworkError(_) => true,
            crate::core::traits::LlmError::Timeout(_) => true,
            crate::core::traits::LlmError::ApiError { code, .. } => {
                *code == 429 || (*code >= 500 && *code < 600)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
        }
    }

    #[test]
    fn test_mock_provider_new() {
        let provider = MockLlmProvider::new();
        assert_eq!(provider.call_count(), 0);
    }

    #[test]
    fn test_mock_provider_with_text_response() {
        let provider = MockLlmProvider::new().with_text_response("Hello from mock!");

        let messages = vec![create_test_message(MessageRole::User, "hi")];
        let result = provider.stream_chat(&messages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_provider_with_error() {
        let provider =
            MockLlmProvider::new().with_error(LlmError::ConfigError("Test error".to_string()));

        let messages = vec![create_test_message(MessageRole::User, "hi")];
        let result = provider.stream_chat(&messages);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_provider_model_name() {
        let provider = MockLlmProvider::new().with_model("test-model-v1");

        assert_eq!(provider.model_name(), "test-model-v1");
    }

    #[test]
    fn test_mock_provider_default_response() {
        let provider = MockLlmProvider::new();

        let messages = vec![create_test_message(MessageRole::User, "unknown prompt")];
        let result = provider.stream_chat(&messages);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert!(!events.is_empty());
        assert!(events.last().unwrap().done);
    }

    #[test]
    fn test_mock_provider_with_custom_response() {
        let custom_events = vec![
            StreamEvent {
                content: "Part 1".to_string(),
                done: false,
            },
            StreamEvent {
                content: "Part 2".to_string(),
                done: true,
            },
        ];

        let provider = MockLlmProvider::new().with_response("specific", custom_events);

        let messages = vec![create_test_message(MessageRole::User, "specific")];
        let result = provider.stream_chat(&messages);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].content, "Part 1");
        assert_eq!(events[1].content, "Part 2");
    }

    #[test]
    fn test_mock_provider_partial_match() {
        let custom_events = vec![StreamEvent {
            content: "Matched!".to_string(),
            done: true,
        }];

        let provider = MockLlmProvider::new().with_response("hello", custom_events);

        let messages = vec![create_test_message(MessageRole::User, "say hello to me")];
        let result = provider.stream_chat(&messages);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert!(events[0].content.contains("Matched"));
    }
}
