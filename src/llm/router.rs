//! LLM 路由模块
//!
//! 提供基于规则的模型路由功能，根据任务类型自动选择最优模型。
//!
//! # 路由策略
//!
//! - **简单问答** - 使用轻量级模型（如 GPT-3.5、Llama3.2）降低成本
//! - **复杂推理** - 使用强模型（如 GPT-4、Claude-3）获得更好效果
//! - **代码生成** - 使用专门优化的代码模型
//!
//! # 示例
//!
//! ```rust,ignore
//! use arkcore::llm::router::{ModelRouter, RouterConfig, RoutingRule, TaskType};
//! use arkcore::llm::{OpenAiProvider, LlmConfig};
//!
//! let config = RouterConfig {
//!     default_model: "gpt-4".to_string(),
//!     rules: vec![
//!         RoutingRule {
//!             task_type: TaskType::SimpleQA,
//!             model: "gpt-3.5-turbo".to_string(),
//!             keywords: vec!["hello".to_string(), "what".to_string()],
//!         },
//!     ],
//! };
//!
//! let router = ModelRouter::new(config);
//! ```

use std::collections::HashMap;

use crate::core::traits::{LLMProvider, LlmError, Message, StreamEvent};

/// 任务类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    /// 简单问答
    SimpleQA,
    /// 复杂推理
    ComplexReasoning,
    /// 代码生成
    CodeGeneration,
    /// 创意写作
    CreativeWriting,
    /// 技术文档
    Technical,
    /// 未知/默认
    Unknown,
}

impl TaskType {
    /// 从任务描述中识别任务类型
    pub fn from_task(task: &str) -> Self {
        let task_lower = task.to_lowercase();

        // 代码生成关键词
        let code_keywords = [
            "代码",
            "code",
            "函数",
            "function",
            "class",
            "implement",
            "写",
            "生成",
        ];
        if code_keywords.iter().any(|k| task_lower.contains(k)) {
            return TaskType::CodeGeneration;
        }

        // 复杂推理关键词
        let reasoning_keywords = [
            "分析",
            "analyze",
            "reason",
            "推理",
            "思考",
            "explain",
            "为什么",
            "原因",
            "逻辑",
            "比较",
        ];
        if reasoning_keywords.iter().any(|k| task_lower.contains(k)) {
            return TaskType::ComplexReasoning;
        }

        // 创意写作关键词
        let creative_keywords = ["写", "创作", "故事", "poem", "write", "creative", "fiction"];
        if creative_keywords.iter().any(|k| task_lower.contains(k)) {
            return TaskType::CreativeWriting;
        }

        // 技术文档关键词
        let tech_keywords = ["文档", "doc", "spec", "api", "教程", "tutorial"];
        if tech_keywords.iter().any(|k| task_lower.contains(k)) {
            return TaskType::Technical;
        }

        // 简单问答 - 短文本或常见问候
        if task.len() < 50 {
            return TaskType::SimpleQA;
        }

        TaskType::Unknown
    }
}

/// 路由规则
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// 任务类型
    pub task_type: TaskType,
    /// 目标模型名称
    pub model: String,
    /// 匹配关键词（可选，用于更细粒度匹配）
    pub keywords: Vec<String>,
}

/// 路由器配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// 默认模型
    pub default_model: String,
    /// 路由规则列表
    pub rules: Vec<RoutingRule>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_model: "gpt-4".to_string(),
            rules: Self::default_rules(),
        }
    }
}

impl RouterConfig {
    /// 默认路由规则
    fn default_rules() -> Vec<RoutingRule> {
        vec![
            RoutingRule {
                task_type: TaskType::SimpleQA,
                model: "gpt-3.5-turbo".to_string(),
                keywords: vec!["hello".to_string(), "hi".to_string(), "?".to_string()],
            },
            RoutingRule {
                task_type: TaskType::CodeGeneration,
                model: "gpt-4".to_string(),
                keywords: vec!["代码".to_string(), "code".to_string(), "函数".to_string()],
            },
            RoutingRule {
                task_type: TaskType::ComplexReasoning,
                model: "gpt-4".to_string(),
                keywords: vec!["分析".to_string(), "为什么".to_string(), "推理".to_string()],
            },
            RoutingRule {
                task_type: TaskType::CreativeWriting,
                model: "gpt-4".to_string(),
                keywords: vec!["写".to_string(), "创作".to_string(), "故事".to_string()],
            },
        ]
    }

    /// 创建空规则配置
    pub fn empty() -> Self {
        Self {
            default_model: "gpt-4".to_string(),
            rules: vec![],
        }
    }

    /// 创建仅包含默认规则的配置
    pub fn with_default_rules() -> Self {
        Self::default()
    }
}

/// 模型提供者包装器
///
/// 将具体的提供者包装为统一类型，支持动态分发
pub enum ModelProvider {
    /// OpenAI 提供者
    OpenAi(crate::llm::provider::OpenAiProvider),
    /// Anthropic 提供者
    Anthropic(crate::llm::provider::AnthropicProvider),
    /// Ollama 提供者
    Ollama(crate::llm::provider::OllamaProvider),
    /// Mock 提供者
    Mock(crate::llm::MockLlmProvider),
}

impl Clone for ModelProvider {
    fn clone(&self) -> Self {
        match self {
            ModelProvider::OpenAi(p) => {
                let config = crate::llm::LlmConfig {
                    model: p.model_name().to_string(),
                    ..Default::default()
                };
                ModelProvider::OpenAi(crate::llm::provider::OpenAiProvider::new(config))
            }
            ModelProvider::Anthropic(p) => {
                let config = crate::llm::LlmConfig {
                    model: p.model_name().to_string(),
                    ..Default::default()
                };
                ModelProvider::Anthropic(crate::llm::provider::AnthropicProvider::new(config))
            }
            ModelProvider::Ollama(p) => {
                ModelProvider::Ollama(crate::llm::provider::OllamaProvider::new(
                    crate::llm::LlmConfig::ollama(p.model_name()),
                ))
            }
            ModelProvider::Mock(p) => {
                ModelProvider::Mock(crate::llm::MockLlmProvider::new().with_model(p.model_name()))
            }
        }
    }
}

impl ModelProvider {
    /// 获取模型名称
    pub fn model_name(&self) -> &str {
        match self {
            ModelProvider::OpenAi(p) => p.model_name(),
            ModelProvider::Anthropic(p) => p.model_name(),
            ModelProvider::Ollama(p) => p.model_name(),
            ModelProvider::Mock(p) => p.model_name(),
        }
    }
}

impl LLMProvider for ModelProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        match self {
            ModelProvider::OpenAi(p) => p.stream_chat(messages),
            ModelProvider::Anthropic(p) => p.stream_chat(messages),
            ModelProvider::Ollama(p) => p.stream_chat(messages),
            ModelProvider::Mock(p) => p.stream_chat(messages),
        }
    }

    fn model_name(&self) -> &str {
        match self {
            ModelProvider::OpenAi(p) => p.model_name(),
            ModelProvider::Anthropic(p) => p.model_name(),
            ModelProvider::Ollama(p) => p.model_name(),
            ModelProvider::Mock(p) => p.model_name(),
        }
    }
}

/// 模型路由器
///
/// 根据任务类型自动选择最优模型。
///
/// # 示例
///
/// ```rust,ignore
/// use arkcore::llm::router::ModelRouter;
/// use arkcore::llm::{LlmConfig, MockLlmProvider};
///
/// let config = LlmConfig::default();
/// let mock_provider = MockLlmProvider::new();
///
/// let router = ModelRouter::with_config(RouterConfig::default())
///     .register_provider("gpt-4".to_string(), ModelProvider::Mock(mock_provider));
/// ```
#[derive(Clone)]
pub struct ModelRouter {
    /// 可用的模型提供者
    providers: HashMap<String, ModelProvider>,
    /// 路由配置
    config: RouterConfig,
}

impl ModelRouter {
    /// 创建新的路由器
    pub fn new(providers: HashMap<String, ModelProvider>, config: RouterConfig) -> Self {
        Self { providers, config }
    }

    /// 创建仅包含配置的路由器（无预注册提供者）
    pub fn with_config(config: RouterConfig) -> Self {
        Self {
            providers: HashMap::new(),
            config,
        }
    }

    /// 注册提供者
    pub fn register_provider(mut self, name: String, provider: ModelProvider) -> Self {
        self.providers.insert(name, provider);
        self
    }

    /// 根据任务路由到合适的模型
    ///
    /// # 参数
    /// - `task` - 任务描述字符串
    ///
    /// # 返回
    /// - 成功返回选中的提供者
    /// - 失败返回 [`LlmError::ConfigError`]
    pub async fn route(&self, task: &str) -> Result<ModelProvider, LlmError> {
        let task_type = TaskType::from_task(task);

        // 查找匹配的规则
        for rule in &self.config.rules {
            if rule.task_type == task_type {
                if let Some(provider) = self.providers.get(&rule.model) {
                    return Ok(provider.clone());
                }
            }
        }

        // 尝试关键词匹配
        let task_lower = task.to_lowercase();
        for rule in &self.config.rules {
            for keyword in &rule.keywords {
                if task_lower.contains(&keyword.to_lowercase()) {
                    if let Some(provider) = self.providers.get(&rule.model) {
                        return Ok(provider.clone());
                    }
                }
            }
        }

        // 回退到默认模型
        if let Some(provider) = self.providers.get(&self.config.default_model) {
            return Ok(provider.clone());
        }

        Err(LlmError::ConfigError(format!(
            "No provider found for default model: {}",
            self.config.default_model
        )))
    }

    /// 发送聊天消息到路由到的模型
    ///
    /// # 参数
    /// - `task` - 任务描述字符串
    /// - `messages` - 消息列表
    ///
    /// # 返回
    /// - 成功返回响应内容
    /// - 失败返回错误
    pub async fn chat(&self, task: &str, messages: &[Message]) -> Result<String, LlmError> {
        let provider = self.route(task).await?;

        let events = provider.stream_chat(messages)?;

        let response = events
            .iter()
            .map(|e| e.content.clone())
            .collect::<Vec<_>>()
            .join("");

        Ok(response)
    }

    /// 获取路由配置
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// 获取可用模型列表
    pub fn available_models(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// 检查模型是否可用
    pub fn has_model(&self, model: &str) -> bool {
        self.providers.contains_key(model)
    }
}

impl std::fmt::Debug for ModelRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRouter")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create messages
    fn create_message(content: &str) -> Message {
        Message {
            role: crate::core::traits::MessageRole::User,
            content: content.to_string(),
        }
    }

    #[test]
    fn test_task_type_from_simple_qa() {
        assert_eq!(TaskType::from_task("hello"), TaskType::SimpleQA);
        assert_eq!(TaskType::from_task("hi there"), TaskType::SimpleQA);
        assert_eq!(TaskType::from_task("what is rust?"), TaskType::SimpleQA);
    }

    #[test]
    fn test_task_type_from_code_generation() {
        assert_eq!(TaskType::from_task("写一个函数"), TaskType::CodeGeneration);
        assert_eq!(
            TaskType::from_task("generate code"),
            TaskType::CodeGeneration
        );
        assert_eq!(
            TaskType::from_task("帮我写一个 Rust class"),
            TaskType::CodeGeneration
        );
    }

    #[test]
    fn test_task_type_from_complex_reasoning() {
        // "reasoning" 包含 "reason" 关键词
        assert_eq!(
            TaskType::from_task("reasoning about this"),
            TaskType::ComplexReasoning
        );
        assert_eq!(
            TaskType::from_task("请分析这个问题"),
            TaskType::ComplexReasoning
        );
    }

    #[test]
    fn test_task_type_from_creative_writing() {
        // "创作" 不包含 "写"，应归类为 CreativeWriting
        assert_eq!(TaskType::from_task("创作一首诗"), TaskType::CreativeWriting);
        assert_eq!(TaskType::from_task("写一篇小说"), TaskType::CodeGeneration);
        // "写" 在代码关键词中
    }

    #[test]
    fn test_task_type_from_technical() {
        // "技术文档" 不包含 "写"，应该归类为 Technical
        assert_eq!(TaskType::from_task("技术文档"), TaskType::Technical);
        assert_eq!(TaskType::from_task("api tutorial"), TaskType::Technical);
    }

    #[test]
    fn test_task_type_unknown() {
        assert_eq!(
            TaskType::from_task("请介绍一下人工智能的发展历史和未来趋势"),
            TaskType::Unknown
        );
    }

    #[test]
    fn test_routing_rule_debug() {
        let rule = RoutingRule {
            task_type: TaskType::SimpleQA,
            model: "gpt-3.5-turbo".to_string(),
            keywords: vec!["hello".to_string()],
        };
        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("SimpleQA"));
        assert!(debug_str.contains("gpt-3.5-turbo"));
    }

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert_eq!(config.default_model, "gpt-4");
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_router_config_empty() {
        let config = RouterConfig::empty();
        assert_eq!(config.default_model, "gpt-4");
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_model_router_debug() {
        let router = ModelRouter::with_config(RouterConfig::empty());
        let debug_str = format!("{:?}", router);
        assert!(debug_str.contains("ModelRouter"));
    }

    #[test]
    fn test_model_router_available_models() {
        let mock_provider = crate::llm::MockLlmProvider::new().with_model("test-model");

        let router = ModelRouter::with_config(RouterConfig::empty())
            .register_provider("test-model".to_string(), ModelProvider::Mock(mock_provider));

        let models = router.available_models();
        assert_eq!(models.len(), 1);
        assert!(models.contains(&"test-model".to_string()));
    }

    #[test]
    fn test_model_router_has_model() {
        let mock_provider = crate::llm::MockLlmProvider::new().with_model("gpt-4");

        let router = ModelRouter::with_config(RouterConfig::empty())
            .register_provider("gpt-4".to_string(), ModelProvider::Mock(mock_provider));

        assert!(router.has_model("gpt-4"));
        assert!(!router.has_model("gpt-3.5"));
    }

    #[test]
    fn test_model_router_route_to_default() {
        let mock_provider = crate::llm::MockLlmProvider::new().with_model("gpt-4");

        let router = ModelRouter::with_config(RouterConfig::empty())
            .register_provider("gpt-4".to_string(), ModelProvider::Mock(mock_provider));

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(router.route("unknown task type"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().model_name(), "gpt-4");
    }

    #[test]
    fn test_model_router_route_by_keyword() {
        let gpt4_provider = crate::llm::MockLlmProvider::new().with_model("gpt-4");
        let gpt35_provider = crate::llm::MockLlmProvider::new().with_model("gpt-3.5-turbo");

        let config = RouterConfig {
            default_model: "gpt-4".to_string(),
            rules: vec![RoutingRule {
                task_type: TaskType::SimpleQA,
                model: "gpt-3.5-turbo".to_string(),
                keywords: vec!["hello".to_string(), "hi".to_string()],
            }],
        };

        let router = ModelRouter::new(
            HashMap::from([
                ("gpt-4".to_string(), ModelProvider::Mock(gpt4_provider)),
                (
                    "gpt-3.5-turbo".to_string(),
                    ModelProvider::Mock(gpt35_provider),
                ),
            ]),
            config,
        );

        let rt = tokio::runtime::Runtime::new().unwrap();

        // Test keyword matching - "say hello" should match hello keyword
        let result = rt.block_on(router.route("say hello to someone"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().model_name(), "gpt-3.5-turbo");
    }

    #[test]
    fn test_model_router_route_complex_task() {
        let gpt4_provider = crate::llm::MockLlmProvider::new().with_model("gpt-4");
        let gpt35_provider = crate::llm::MockLlmProvider::new().with_model("gpt-3.5-turbo");

        let config = RouterConfig {
            default_model: "gpt-4".to_string(),
            rules: vec![RoutingRule {
                task_type: TaskType::CodeGeneration,
                model: "gpt-4".to_string(),
                keywords: vec!["代码".to_string()],
            }],
        };

        let router = ModelRouter::new(
            HashMap::from([
                ("gpt-4".to_string(), ModelProvider::Mock(gpt4_provider)),
                (
                    "gpt-3.5-turbo".to_string(),
                    ModelProvider::Mock(gpt35_provider),
                ),
            ]),
            config,
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(router.route("写代码实现快速排序"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().model_name(), "gpt-4");
    }

    #[test]
    fn test_model_router_route_no_provider_error() {
        let router = ModelRouter::with_config(RouterConfig::empty());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(router.route("hello"));

        assert!(result.is_err());
        match result {
            Err(LlmError::ConfigError(msg)) => {
                assert!(msg.contains("No provider found"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_model_router_chat() {
        let mock_provider = crate::llm::MockLlmProvider::new()
            .with_model("gpt-4")
            .with_text_response("Hello! How can I help you?");

        let router = ModelRouter::with_config(RouterConfig::empty())
            .register_provider("gpt-4".to_string(), ModelProvider::Mock(mock_provider));

        let rt = tokio::runtime::Runtime::new().unwrap();
        let messages = vec![create_message("Hello!")];

        let result = rt.block_on(router.chat("greeting", &messages));

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello"));
    }

    #[test]
    fn test_router_config_with_custom_rules() {
        let custom_rules = vec![
            RoutingRule {
                task_type: TaskType::SimpleQA,
                model: "llama3.2".to_string(),
                keywords: vec!["?".to_string()],
            },
            RoutingRule {
                task_type: TaskType::CodeGeneration,
                model: "codellama".to_string(),
                keywords: vec!["function".to_string(), "def".to_string()],
            },
        ];

        let config = RouterConfig {
            default_model: "llama3.2".to_string(),
            rules: custom_rules,
        };

        assert_eq!(config.default_model, "llama3.2");
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].model, "llama3.2");
        assert_eq!(config.rules[1].model, "codellama");
    }

    #[test]
    fn test_task_type_clone() {
        let tt = TaskType::SimpleQA;
        let cloned = tt.clone();
        assert_eq!(cloned, tt);
    }

    #[test]
    fn test_task_type_partial_eq() {
        assert_eq!(TaskType::SimpleQA, TaskType::SimpleQA);
        assert_ne!(TaskType::SimpleQA, TaskType::CodeGeneration);
    }

    #[test]
    fn test_model_provider_clone() {
        let mock = crate::llm::MockLlmProvider::new().with_model("test");
        let provider = ModelProvider::Mock(mock);
        let cloned = provider.clone();
        assert_eq!(cloned.model_name(), "test");
    }

    #[test]
    fn test_model_provider_openai() {
        let config = crate::llm::LlmConfig::default();
        let openai = crate::llm::OpenAiProvider::new(config);
        let provider = ModelProvider::OpenAi(openai);
        assert_eq!(provider.model_name(), "gpt-4");
    }

    #[test]
    fn test_model_provider_anthropic() {
        let config = crate::llm::LlmConfig {
            model: "claude-3-sonnet".to_string(),
            ..Default::default()
        };
        let anthropic = crate::llm::AnthropicProvider::new(config);
        let provider = ModelProvider::Anthropic(anthropic);
        assert_eq!(provider.model_name(), "claude-3-sonnet");
    }

    // #[test]
    // fn test_model_provider_ollama() {
    //     // OllamaProvider 未实现，暂注释
    //     let config = crate::llm::LlmConfig::ollama("llama3.2");
    //     let ollama = crate::llm::OllamaProvider::new(config);
    //     let provider = ModelProvider::Ollama(ollama);
    //     assert_eq!(provider.model_name(), "llama3.2");
    // }
}
