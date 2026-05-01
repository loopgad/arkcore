//! LLM 模块
//!
//! LLM 接口定义和实现
//!
//! 支持多种 LLM Provider：OpenAI、Anthropic、Mock 等

pub mod provider;
pub mod router;

#[cfg(test)]
mod tests;

pub use provider::mock::MockLlmProvider;
pub use provider::LlmConfig;
pub use provider::{AnthropicProvider, OpenAiProvider};

// Re-export from core traits for convenience
pub use crate::core::traits::LLMProvider as LlmProviderTrait;
pub use crate::core::traits::{Message, MessageRole, StreamEvent};
pub use provider::LlmError;

// Re-export router types
pub use router::{ModelProvider, ModelRouter, RouterConfig, RoutingRule, TaskType};
