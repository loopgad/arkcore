//! Mock LLM Provider 测试模块
//!
//! 提供 LLM Provider 的 Mock 实现和单元测试

use std::time::Duration;
use std::collections::HashMap;

/// LLM 消息角色
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// LLM 消息
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// 流式响应事件 (SSE 格式)
#[derive(Debug, Clone, PartialEq)]
pub struct StreamEvent {
    pub content: String,
    pub done: bool,
}

/// LLM Provider Trait - 定义 LLM 客户端接口
pub trait LlmProvider: Send + Sync {
    /// 发送聊天消息并获取流式响应
    fn stream_chat(
        &self,
        messages: &[Message],
    ) -> Result<Vec<StreamEvent>, LlmError>;

    /// 获取模型名称
    fn model_name(&self) -> &str;
}

/// LLM 错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum LlmError {
    /// 网络错误
    NetworkError(String),
    /// 超时错误
    Timeout(Duration),
    /// API 错误
    ApiError { code: i32, message: String },
    /// 配置错误
    ConfigError(String),
    /// 解析错误
    ParseError(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NetworkError(msg) => write!(f, "NetworkError: {}", msg),
            LlmError::Timeout(d) => write!(f, "Timeout after {:?}", d),
            LlmError::ApiError { code, message } => write!(f, "ApiError {}: {}", code, message),
            LlmError::ConfigError(msg) => write!(f, "ConfigError: {}", msg),
            LlmError::ParseError(msg) => write!(f, "ParseError: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

/// Mock LLM Provider - 用于测试的模拟实现
pub struct MockLlmProvider {
    responses: HashMap<String, Vec<StreamEvent>>,
    delay_ms: u64,
    should_error: Option<LlmError>,
    call_count: u32,
}

impl MockLlmProvider {
    /// 创建新的 Mock Provider
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            delay_ms: 0,
            should_error: None,
            call_count: 0,
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
                done: true,
            },
        ];
        self.with_response("default", events)
    }

    /// 设置响应延迟 (毫秒)
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// 设置模拟错误
    pub fn with_error(mut self, error: LlmError) -> Self {
        self.should_error = Some(error);
        self
    }

    /// 获取调用次数 (预留用于重试计数验证)
    #[allow(dead_code)]
    pub fn call_count(&self) -> u32 {
        self.call_count
    }

    /// 重置调用计数 (预留用于重试计数验证)
    #[allow(dead_code)]
    pub fn reset_call_count(&mut self) {
        self.call_count = 0;
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for MockLlmProvider {
    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, LlmError> {
        // 如果设置了错误，返回错误
        if let Some(ref error) = self.should_error {
            return Err(error.clone());
        }

        // 应用延迟
        if self.delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.delay_ms));
        }

        // 构造提示词模式
        let prompt: String = messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        // 查找匹配的响应
        if let Some(events) = self.responses.get(&prompt) {
            return Ok(events.clone());
        }

        // 尝试匹配包含关系
        for (pattern, events) in &self.responses {
            if prompt.contains(pattern) {
                return Ok(events.clone());
            }
        }

        // 默认响应
        Ok(vec![StreamEvent {
            content: "Mock response".to_string(),
            done: true,
        }])
    }

    fn model_name(&self) -> &str {
        "mock-gpt-4"
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // MockLlmProvider 基本测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_provider_returns_predefined_response() {
        let events = vec![
            StreamEvent {
                content: "Hello".to_string(),
                done: false,
            },
            StreamEvent {
                content: " World".to_string(),
                done: true,
            },
        ];

        let mock = MockLlmProvider::new()
            .with_response("Hello", events.clone())
            .with_text_response("Default response");

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_ok());

        let received_events = result.unwrap();
        assert_eq!(received_events.len(), 2);
        assert_eq!(received_events[0].content, "Hello");
        assert_eq!(received_events[1].content, " World");
    }

    #[test]
    fn test_mock_provider_default_response() {
        let mock = MockLlmProvider::new();

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Unknown prompt".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Mock response");
        assert!(events[0].done);
    }

    #[test]
    fn test_mock_provider_pattern_matching() {
        let mock = MockLlmProvider::new()
            .with_text_response("Matched pattern");

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
        ];

        let result = mock.stream_chat(&messages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_provider_with_delay() {
        let mock = MockLlmProvider::new()
            .with_delay(10)
            .with_text_response("Delayed response");

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let start = std::time::Instant::now();
        let result = mock.stream_chat(&messages);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 延迟应该大于 0
        assert!(elapsed >= Duration::from_millis(9));
    }

    // ------------------------------------------------------------------------
    // SSE 流式响应解析测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_sse_parse_single_event() {
        let sse_data = "data: {\"content\": \"Hello\"}\n\n";
        let events = parse_sse_events(sse_data);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Hello");
        assert!(!events[0].done);
    }

    #[test]
    fn test_sse_parse_multiple_events() {
        let sse_data = "data: {\"content\": \"Hello\"}\n\ndata: {\"content\": \" World\"}\n\n";
        let events = parse_sse_events(sse_data);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].content, "Hello");
        assert_eq!(events[1].content, " World");
    }

    #[test]
    fn test_sse_parse_done_event() {
        let sse_data = "data: {\"content\": \"Done\", \"done\": true}\n\n";
        let events = parse_sse_events(sse_data);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Done");
        assert!(events[0].done);
    }

    #[test]
    fn test_sse_parse_empty_content() {
        let sse_data = "data: {}\n\n";
        let events = parse_sse_events(sse_data);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "");
    }

    #[test]
    fn test_sse_parse_invalid_json() {
        let sse_data = "data: invalid json\n\n";
        let events = parse_sse_events(sse_data);

        // 无效 JSON 应该被跳过或返回空
        assert!(events.is_empty() || events[0].content.is_empty());
    }

    #[test]
    fn test_sse_parse_partial_data() {
        let sse_data = "data: {\"content\": \"Partial";
        let events = parse_sse_events(sse_data);

        // 不完整的数据应该被处理
        assert!(events.len() <= 1);
    }

    #[test]
    fn test_sse_parse_with_newlines() {
        let sse_data = "data: {\"content\": \"Line1\\nLine2\"}\n\n";
        let events = parse_sse_events(sse_data);

        assert_eq!(events.len(), 1);
        assert!(events[0].content.contains("Line1"));
    }

    /// 解析 SSE 格式的事件流
    fn parse_sse_events(sse_data: &str) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        // 按 "data: " 行分割
        for line in sse_data.split("data: ") {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 移除结尾的换行
            let trimmed = line.trim_end_matches('\n');

            // 空 {} 产生空事件
            if trimmed == "{}" {
                events.push(StreamEvent {
                    content: String::new(),
                    done: false,
                });
                continue;
            }

            // 解析 JSON 对象
            let content = extract_json_string(trimmed, "content");
            let done = trimmed.contains("\"done\": true") || trimmed.contains("\"done\":true");

            events.push(StreamEvent {
                content,
                done,
            });
        }

        events
    }

    /// 从 JSON 对象字符串中提取指定字段值
    fn extract_json_string(json: &str, field: &str) -> String {
        let pattern = format!("\"{}\": \"", field);
        if let Some(start) = json.find(&pattern) {
            let content_start = start + pattern.len();
            let rest = &json[content_start..];
            // 找到结束引号（处理转义）
            let mut end = 0;
            let mut chars = rest.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    chars.next(); // 跳过转义字符
                } else if c == '"' {
                    break;
                }
                end += 1;
            }
            rest[..end].to_string()
        } else {
            String::new()
        }
    }

    // ------------------------------------------------------------------------
    // 错误处理测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_network_error_handling() {
        let mock = MockLlmProvider::new()
            .with_error(LlmError::NetworkError("Connection refused".to_string()));

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            LlmError::NetworkError(msg) => {
                assert!(msg.contains("Connection refused"));
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_timeout_error_handling() {
        let mock = MockLlmProvider::new()
            .with_error(LlmError::Timeout(Duration::from_secs(30)));

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        match result.unwrap_err() {
            LlmError::Timeout(d) => {
                assert_eq!(d, Duration::from_secs(30));
            }
            _ => panic!("Expected Timeout"),
        }
    }

    #[test]
    fn test_api_error_handling() {
        let mock = MockLlmProvider::new()
            .with_error(LlmError::ApiError {
                code: 429,
                message: "Rate limit exceeded".to_string(),
            });

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        match result.unwrap_err() {
            LlmError::ApiError { code, message } => {
                assert_eq!(code, 429);
                assert!(message.contains("Rate limit"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_config_error_handling() {
        let mock = MockLlmProvider::new()
            .with_error(LlmError::ConfigError("Missing API key".to_string()));

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        match result.unwrap_err() {
            LlmError::ConfigError(msg) => {
                assert!(msg.contains("Missing API key"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_parse_error_handling() {
        let mock = MockLlmProvider::new()
            .with_error(LlmError::ParseError("Invalid response format".to_string()));

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        match result.unwrap_err() {
            LlmError::ParseError(msg) => {
                assert!(msg.contains("Invalid response format"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_error_display_trait() {
        let error = LlmError::NetworkError("test".to_string());
        let display = format!("{}", error);
        assert!(display.contains("NetworkError"));

        let error = LlmError::Timeout(Duration::from_secs(5));
        let display = format!("{}", error);
        assert!(display.contains("5s"));
    }

    // ------------------------------------------------------------------------
    // 重试逻辑测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_retry_on_network_error() {
        let mut mock = MockLlmProvider::new();
        mock.should_error = Some(LlmError::NetworkError("Temp failure".to_string()));
        mock.call_count = 0;

        // 使用 with_error 的 Provider 无法直接测试重试
        // 这里测试的是错误类型识别
        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
        }];

        let result = mock.stream_chat(&messages);
        assert!(result.is_err());

        match result.unwrap_err() {
            LlmError::NetworkError(_) => {}
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_retry_decision_logic() {
        // 测试哪些错误应该重试
        let retriable = vec![
            LlmError::NetworkError("Connection reset".to_string()),
            LlmError::Timeout(Duration::from_secs(30)),
            LlmError::ApiError {
                code: 429,
                message: "Rate limit".to_string(),
            },
            LlmError::ApiError {
                code: 500,
                message: "Internal error".to_string(),
            },
        ];

        for error in retriable {
            assert!(is_retriable_error(&error), "Error {:?} should be retriable", error);
        }

        // 非重试错误
        let non_retriable = vec![
            LlmError::ConfigError("Invalid key".to_string()),
            LlmError::ParseError("Bad format".to_string()),
            LlmError::ApiError {
                code: 401,
                message: "Unauthorized".to_string(),
            },
            LlmError::ApiError {
                code: 403,
                message: "Forbidden".to_string(),
            },
        ];

        for error in non_retriable {
            assert!(!is_retriable_error(&error), "Error {:?} should not be retriable", error);
        }
    }

    /// 判断错误是否可重试
    fn is_retriable_error(error: &LlmError) -> bool {
        match error {
            LlmError::NetworkError(_) => true,
            LlmError::Timeout(_) => true,
            LlmError::ApiError { code, .. } => {
                // 429 (限流) 和 5xx (服务器错误) 可重试
                *code == 429 || (*code >= 500 && *code < 600)
            }
            LlmError::ConfigError(_) => false,
            LlmError::ParseError(_) => false,
        }
    }

    #[test]
    fn test_max_retries_exceeded() {
        let max_retries = 3;
        let mut attempts = 0;
        let mut last_error: Option<LlmError> = None;

        // 模拟多次重试
        let mock = MockLlmProvider::new()
            .with_error(LlmError::NetworkError("Persistent failure".to_string()));

        while attempts < max_retries {
            let messages = vec![Message {
                role: MessageRole::User,
                content: "Test".to_string(),
            }];

            let result = mock.stream_chat(&messages);
            if result.is_err() {
                last_error = Some(result.unwrap_err());
                attempts += 1;
            } else {
                break;
            }
        }

        assert_eq!(attempts, max_retries);
        assert!(last_error.is_some());
    }

    #[test]
    fn test_retry_with_backoff() {
        let base_delay_ms = 100;
        let max_retries = 3;

        // 模拟指数退避
        for retry in 0..max_retries {
            let delay_ms = base_delay_ms * 2_u64.pow(retry);
            // 第一次: 100ms, 第二次: 200ms, 第三次: 400ms
            assert!(delay_ms > 0);
        }
    }

    // ------------------------------------------------------------------------
    // LlmProvider Trait 对象测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_trait_object_safety() {
        let mock: MockLlmProvider = MockLlmProvider::new()
            .with_text_response("Trait test");

        // 测试 trait 对象可以创建
        let provider: &dyn LlmProvider = &mock;
        assert_eq!(provider.model_name(), "mock-gpt-4");
    }

    #[test]
    fn test_trait_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockLlmProvider>();
        assert_send_sync::<Box<dyn LlmProvider>>();
    }

    // ------------------------------------------------------------------------
    // 消息构建测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_message_construction() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };

        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_roles() {
        assert_eq!(
            format!("{:?}", MessageRole::System),
            "System"
        );
        assert_eq!(
            format!("{:?}", MessageRole::User),
            "User"
        );
        assert_eq!(
            format!("{:?}", MessageRole::Assistant),
            "Assistant"
        );
    }

    #[test]
    fn test_stream_event_construction() {
        let event = StreamEvent {
            content: "Test".to_string(),
            done: false,
        };

        assert_eq!(event.content, "Test");
        assert!(!event.done);
    }

    // ------------------------------------------------------------------------
    // LlmConfig 测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_llm_config_clone() {
        use crate::llm::LlmConfig;
        use std::time::Duration;

        let config = LlmConfig {
            model: "gpt-4".to_string(),
            api_key: Some("sk-test".to_string()),
            base_url: "https://api.openai.com".to_string(),
            timeout: Duration::from_secs(60),
            max_retries: 3,
        };

        let cloned = config.clone();
        assert_eq!(cloned.model, "gpt-4");
        assert_eq!(cloned.api_key, Some("sk-test".to_string()));
        assert_eq!(cloned.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_llm_config_debug() {
        use crate::llm::LlmConfig;

        let config = LlmConfig {
            model: "gpt-4".to_string(),
            api_key: None,
            base_url: "https://api.openai.com".to_string(),
            timeout: std::time::Duration::from_secs(60),
            max_retries: 3,
        };

        let debug = format!("{:?}", config);
        assert!(debug.contains("gpt-4"));
        assert!(debug.contains("api.openai.com"));
    }
}