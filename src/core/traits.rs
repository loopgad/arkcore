//! ArkCore 核心 Trait 定义
//!
//! 定义所有核心接口抽象，支持泛型约束和异步操作。
//!
//! # 设计原则
//!
//! - `Send + Sync` 约束确保线程安全
//! - 使用关联类型实现泛型抽象
//! - `async` 方法支持异步实现
//! - 错误处理使用 `Result` 类型

use std::time::Duration;

// ============================================================================
// 共享数据类型
// ============================================================================

/// LLM 消息角色
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    /// 系统消息
    System,
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
}

/// LLM 消息
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// 消息角色
    pub role: MessageRole,
    /// 消息内容
    pub content: String,
}

/// 流式响应事件 (SSE 格式)
#[derive(Debug, Clone, PartialEq)]
pub struct StreamEvent {
    /// 事件内容
    pub content: String,
    /// 是否完成
    pub done: bool,
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
    /// 认证错误
    AuthError(String),
    /// Rate Limit 错误
    RateLimitError { retry_after: Option<u64> },
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NetworkError(msg) => write!(f, "NetworkError: {}", msg),
            LlmError::Timeout(d) => write!(f, "Timeout after {:?}", d),
            LlmError::ApiError { code, message } => write!(f, "ApiError {}: {}", code, message),
            LlmError::ConfigError(msg) => write!(f, "ConfigError: {}", msg),
            LlmError::ParseError(msg) => write!(f, "ParseError: {}", msg),
            LlmError::AuthError(msg) => write!(f, "AuthError: {}", msg),
            LlmError::RateLimitError { retry_after } => {
                write!(f, "RateLimitError")?;
                if let Some(after) = retry_after {
                    write!(f, " (retry after {}s)", after)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LlmError {}

/// 执行结果
#[derive(Debug)]
pub struct ExecutionResult {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 是否超时
    pub timed_out: bool,
    /// 安全违规列表
    pub security_violations: Vec<String>,
}

impl ExecutionResult {
    /// 判断执行是否成功
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out && self.security_violations.is_empty()
    }
}

/// 安全检查结果
#[derive(Debug)]
pub struct SecurityCheckResult {
    /// 是否通过检查
    pub passed: bool,
    /// 违规信息列表
    pub violations: Vec<String>,
}

// ============================================================================
// Storage Trait - 记忆存储接口
// ============================================================================

/// 技能数据结构
///
/// 用于存储和检索的技能元数据。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    /// 唯一标识符
    pub id: String,
    /// 技能名称
    pub skill_name: String,
    /// 技能分类
    pub category: String,
    /// 技能摘要
    pub summary: String,
    /// 技能详情（可选）
    pub details: Option<String>,
    /// 关键词（逗号分隔）
    pub keywords: String,
    /// 重要性权重
    pub importance: f64,
    /// 成功次数
    pub success_count: i32,
    /// 访问次数
    pub access_count: i32,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 更新时间（ISO 8601）
    pub updated_at: String,
}

/// 存储接口 Trait
///
/// 定义记忆存储的基本操作，支持技能数据的持久化和检索。
///
/// # 泛型约束
///
/// - `E` - 错误类型，默认使用 `anyhow::Error`
///
/// # 实现示例
///
/// ```rust,ignore
/// #[async_trait]
/// impl Storage for SkillMemory {
///     type Error = anyhow::Error;
///
///     async fn store_skill(&self, skill: &Skill) -> Result<(), Self::Error> { ... }
///     async fn search(&self, query: &str, limit: usize) -> Result<Vec<Skill>, Self::Error> { ... }
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait Storage: Send + Sync {
    /// 错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 存储技能到记忆
    async fn store_skill(&self, skill: &Skill) -> Result<(), Self::Error>;

    /// 使用全文搜索查询技能
    ///
    /// # 参数
    /// - `query` - 搜索查询字符串
    /// - `limit` - 返回结果数量上限
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Skill>, Self::Error>;

    /// 根据 ID 获取单个技能
    async fn get_skill(&self, skill_id: &str) -> Result<Option<Skill>, Self::Error>;

    /// 删除技能
    async fn delete_skill(&self, skill_id: &str) -> Result<(), Self::Error>;

    /// 更新技能访问计数
    async fn increment_access(&self, skill_id: &str) -> Result<(), Self::Error>;

    /// 更新技能成功计数
    async fn increment_success(&self, skill_id: &str) -> Result<(), Self::Error>;

    /// 获取所有技能（按访问次数排序）
    async fn get_all_skills(&self, limit: usize) -> Result<Vec<Skill>, Self::Error>;
}

// ============================================================================
// Executor Trait - 命令执行接口
// ============================================================================

/// 命令执行器 Trait
///
/// 定义安全命令执行接口，支持超时控制和安全检测。
///
/// # 泛型约束
///
/// - `E` - 错误类型，默认使用 `anyhow::Error`
///
/// # 安全特性
///
/// - 六层安全检测
/// - 超时控制
/// - 输出截断
///
/// # 实现示例
///
/// ```rust,ignore
/// #[async_trait]
/// impl Executor for Sandbox {
///     type Error = anyhow::Error;
///
///     async fn execute(&self, cmd: &str) -> Result<ExecutionResult, Self::Error> { ... }
///     async fn execute_raw(&self, cmd: &str) -> Result<ExecutionResult, Self::Error> { ... }
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait Executor: Send + Sync {
    /// 错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 执行命令（带输出截断）
    ///
    /// 先进行安全检测，通过后执行命令。
    /// 输出会被截断以防止资源耗尽。
    async fn execute(&self, cmd: &str) -> Result<ExecutionResult, Self::Error>;

    /// 执行命令并返回原始输出（不截断）
    ///
    /// 适用于需要完整输出的场景。
    async fn execute_raw(&self, cmd: &str) -> Result<ExecutionResult, Self::Error>;

    /// 执行安全检测
    ///
    /// # 参数
    /// - `cmd` - 待检测的命令字符串
    ///
    /// # 返回
    /// - [`SecurityCheckResult`] - 安全检查结果
    fn security_check(&self, cmd: &str) -> SecurityCheckResult;
}

// ============================================================================
// LLMProvider Trait - LLM 提供者接口
// ============================================================================

/// LLM 提供者 Trait
///
/// 定义 LLM 客户端接口，支持流式响应。
///
/// # 泛型约束
///
/// - `E` - 错误类型，实现 [`LlmError`]
///
/// # 流式响应
///
/// 该接口设计支持 SSE 流式响应，返回 [`StreamEvent`] 列表。
///
/// # 实现示例
///
/// ```rust,ignore
/// impl LLMProvider for OpenAIProvider {
///     type Error = LlmError;
///
///     fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> { ... }
///     fn model_name(&self) -> &str { ... }
/// }
/// ```
pub trait LLMProvider: Send + Sync {
    /// 错误类型
    type Error: From<LlmError> + std::error::Error + Send + Sync + 'static;

    /// 发送聊天消息并获取流式响应
    ///
    /// # 参数
    /// - `messages` - 消息列表
    ///
    /// # 返回
    /// - 成功返回 [`StreamEvent`] 向量
    /// - 失败返回 [`LlmError`]
    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error>;

    /// 获取模型名称
    fn model_name(&self) -> &str;

    /// 判断错误是否可重试
    ///
    /// # 默认实现
    ///
    /// - 网络错误、超时、429/5xx 错误可重试
    /// - 配置错误、解析错误、401/403 错误不可重试
    fn is_retriable_error(error: &LlmError) -> bool {
        match error {
            LlmError::NetworkError(_) => true,
            LlmError::Timeout(_) => true,
            LlmError::ApiError { code, .. } => *code == 429 || (*code >= 500 && *code < 600),
            LlmError::RateLimitError { .. } => true,
            LlmError::ConfigError(_) => false,
            LlmError::ParseError(_) => false,
            LlmError::AuthError(_) => false,
        }
    }
}

// ============================================================================
// Authenticator Trait - 认证接口（预留）
// ============================================================================

/// 认证器 Trait（预留）
///
/// 定义认证接口，支持多种认证方式。
///
/// # 状态
///
/// 当前为预留接口，具体认证方式待实现。
///
/// # 计划支持
///
/// - API Key 认证
/// - OAuth 2.0
/// - JWT Token
/// - 自定义认证方案
#[allow(async_fn_in_trait)]
pub trait Authenticator: Send + Sync {
    /// 错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 验证凭据
    async fn authenticate(&self, credentials: &Credentials) -> Result<(), Self::Error>;

    /// 刷新认证令牌
    async fn refresh(&self) -> Result<(), Self::Error>;
}

/// 认证凭据
#[derive(Debug, Clone)]
pub struct Credentials {
    /// 认证类型
    pub auth_type: AuthType,
    /// 凭据数据
    pub data: Vec<u8>,
}

/// 认证类型
#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    /// API Key
    ApiKey,
    /// Bearer Token
    Bearer,
    /// OAuth
    OAuth,
    /// 自定义
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // === MessageRole tests ===

    #[test]
    fn test_message_role_variants() {
        assert!(matches!(MessageRole::System, MessageRole::System));
        assert!(matches!(MessageRole::User, MessageRole::User));
        assert!(matches!(MessageRole::Assistant, MessageRole::Assistant));
    }

    #[test]
    fn test_message_role_clone() {
        let role = MessageRole::User;
        let cloned = role.clone();
        assert_eq!(cloned, role);
    }

    #[test]
    fn test_message_role_debug() {
        let role = MessageRole::Assistant;
        let debug_str = format!("{:?}", role);
        assert!(debug_str.contains("Assistant"));
    }

    // === Message tests ===

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_clone() {
        let msg = Message {
            role: MessageRole::System,
            content: "System prompt".to_string(),
        };
        let cloned = msg.clone();
        assert_eq!(cloned.content, msg.content);
        assert_eq!(cloned.role, msg.role);
    }

    #[test]
    fn test_message_partial_eq() {
        let msg1 = Message {
            role: MessageRole::User,
            content: "test".to_string(),
        };
        let msg2 = Message {
            role: MessageRole::User,
            content: "test".to_string(),
        };
        let msg3 = Message {
            role: MessageRole::Assistant,
            content: "test".to_string(),
        };
        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    // === StreamEvent tests ===

    #[test]
    fn test_stream_event_creation() {
        let event = StreamEvent {
            content: "chunk".to_string(),
            done: false,
        };
        assert_eq!(event.content, "chunk");
        assert!(!event.done);
    }

    #[test]
    fn test_stream_event_done() {
        let event = StreamEvent {
            content: "final".to_string(),
            done: true,
        };
        assert!(event.done);
    }

    #[test]
    fn test_stream_event_clone() {
        let event = StreamEvent {
            content: "data".to_string(),
            done: false,
        };
        let cloned = event.clone();
        assert_eq!(cloned.content, event.content);
        assert_eq!(cloned.done, event.done);
    }

    // === LlmError tests ===

    #[test]
    fn test_llm_error_network_error() {
        let err = LlmError::NetworkError("connection failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("NetworkError"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_llm_error_timeout() {
        let err = LlmError::Timeout(Duration::from_secs(30));
        let display = format!("{}", err);
        assert!(display.contains("Timeout"));
    }

    #[test]
    fn test_llm_error_api_error() {
        let err = LlmError::ApiError {
            code: 429,
            message: "rate limited".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("ApiError"));
        assert!(display.contains("429"));
        assert!(display.contains("rate limited"));
    }

    #[test]
    fn test_llm_error_config_error() {
        let err = LlmError::ConfigError("invalid key".to_string());
        let display = format!("{}", err);
        assert!(display.contains("ConfigError"));
    }

    #[test]
    fn test_llm_error_parse_error() {
        let err = LlmError::ParseError("json malformed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("ParseError"));
    }

    #[test]
    fn test_llm_error_clone() {
        let err = LlmError::NetworkError("test".to_string());
        let cloned = err.clone();
        assert_eq!(format!("{}", cloned), format!("{}", err));
    }

    #[test]
    fn test_llm_error_partial_eq() {
        let err1 = LlmError::NetworkError("test".to_string());
        let err2 = LlmError::NetworkError("test".to_string());
        let err3 = LlmError::NetworkError("other".to_string());
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    // === LLMProvider is_retriable_error tests ===

    // Test struct that implements LLMProvider for testing the static method
    struct TestLlmProvider;

    impl LLMProvider for TestLlmProvider {
        type Error = LlmError;

        fn stream_chat(&self, _messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
            Ok(vec![])
        }

        fn model_name(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_is_retriable_error_network() {
        let err = LlmError::NetworkError("conn".to_string());
        assert!(<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_timeout() {
        let err = LlmError::Timeout(Duration::from_secs(5));
        assert!(<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_429() {
        let err = LlmError::ApiError { code: 429, message: "".to_string() };
        assert!(<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_500() {
        let err = LlmError::ApiError { code: 500, message: "".to_string() };
        assert!(<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_502() {
        let err = LlmError::ApiError { code: 502, message: "".to_string() };
        assert!(<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_401() {
        let err = LlmError::ApiError { code: 401, message: "".to_string() };
        assert!(!<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_403() {
        let err = LlmError::ApiError { code: 403, message: "".to_string() };
        assert!(!<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_config() {
        let err = LlmError::ConfigError("bad".to_string());
        assert!(!<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    #[test]
    fn test_is_retriable_error_parse() {
        let err = LlmError::ParseError("bad".to_string());
        assert!(!<TestLlmProvider as LLMProvider>::is_retriable_error(&err));
    }

    // === ExecutionResult tests ===

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult {
            stdout: "output".to_string(),
            stderr: "".to_string(),
            exit_code: Some(0),
            timed_out: false,
            security_violations: vec![],
        };
        assert!(result.is_success());
    }

    #[test]
    fn test_execution_result_failure_exit_code() {
        let result = ExecutionResult {
            stdout: "".to_string(),
            stderr: "error".to_string(),
            exit_code: Some(1),
            timed_out: false,
            security_violations: vec![],
        };
        assert!(!result.is_success());
    }

    #[test]
    fn test_execution_result_failure_timeout() {
        let result = ExecutionResult {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: None,
            timed_out: true,
            security_violations: vec![],
        };
        assert!(!result.is_success());
    }

    #[test]
    fn test_execution_result_failure_security() {
        let result = ExecutionResult {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: Some(0),
            timed_out: false,
            security_violations: vec!["dangerous".to_string()],
        };
        assert!(!result.is_success());
    }

    #[test]
    fn test_execution_result_debug() {
        let result = ExecutionResult {
            stdout: "out".to_string(),
            stderr: "err".to_string(),
            exit_code: Some(0),
            timed_out: false,
            security_violations: vec![],
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("stdout"));
        assert!(debug_str.contains("out"));
    }

    // === SecurityCheckResult tests ===

    #[test]
    fn test_security_check_result_passed() {
        let result = SecurityCheckResult {
            passed: true,
            violations: vec![],
        };
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_security_check_result_failed() {
        let result = SecurityCheckResult {
            passed: false,
            violations: vec!["cmd injection".to_string(), " dangerous".to_string()],
        };
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_security_check_result_debug() {
        let result = SecurityCheckResult {
            passed: true,
            violations: vec![],
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("passed"));
    }

    // === Skill tests ===

    #[test]
    fn test_skill_creation() {
        let skill = Skill {
            id: "skill-1".to_string(),
            skill_name: "TestSkill".to_string(),
            category: "testing".to_string(),
            summary: "A test skill".to_string(),
            details: Some("Detailed description".to_string()),
            keywords: "test,skill".to_string(),
            importance: 0.8,
            success_count: 10,
            access_count: 50,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        assert_eq!(skill.id, "skill-1");
        assert_eq!(skill.skill_name, "TestSkill");
        assert!(skill.details.is_some());
    }

    #[test]
    fn test_skill_serialization() {
        let skill = Skill {
            id: "test".to_string(),
            skill_name: "Test".to_string(),
            category: "cat".to_string(),
            summary: "sum".to_string(),
            details: None,
            keywords: "kw".to_string(),
            importance: 0.5,
            success_count: 0,
            access_count: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        // Test that Skill implements Serialize and Deserialize
        let json = serde_json::to_string(&skill);
        assert!(json.is_ok());
    }

    // === Credentials tests ===

    #[test]
    fn test_credentials_creation() {
        let cred = Credentials {
            auth_type: AuthType::ApiKey,
            data: vec![1, 2, 3],
        };
        assert!(matches!(cred.auth_type, AuthType::ApiKey));
        assert_eq!(cred.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_credentials_clone() {
        let cred = Credentials {
            auth_type: AuthType::Bearer,
            data: vec![1, 2, 3],
        };
        let cloned = cred.clone();
        assert_eq!(cloned.auth_type, cred.auth_type);
        assert_eq!(cloned.data, cred.data);
    }

    #[test]
    fn test_credentials_debug() {
        let cred = Credentials {
            auth_type: AuthType::ApiKey,
            data: vec![],
        };
        let debug_str = format!("{:?}", cred);
        assert!(debug_str.contains("Credentials"));
    }

    // === AuthType tests ===

    #[test]
    fn test_auth_type_api_key() {
        assert!(matches!(AuthType::ApiKey, AuthType::ApiKey));
    }

    #[test]
    fn test_auth_type_bearer() {
        assert!(matches!(AuthType::Bearer, AuthType::Bearer));
    }

    #[test]
    fn test_auth_type_oauth() {
        assert!(matches!(AuthType::OAuth, AuthType::OAuth));
    }

    #[test]
    fn test_auth_type_custom() {
        let custom = AuthType::Custom("jwt".to_string());
        match custom {
            AuthType::Custom(s) => assert_eq!(s, "jwt"),
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_auth_type_partial_eq() {
        assert_eq!(AuthType::ApiKey, AuthType::ApiKey);
        assert_eq!(AuthType::Bearer, AuthType::Bearer);
        assert_ne!(AuthType::ApiKey, AuthType::Bearer);
    }
}
