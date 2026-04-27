//! LLM 模块
//!
//! LLM 接口定义和实现
//!
//! 支持多种 LLM Provider：OpenAI、Anthropic、Mock 等

pub mod provider;

#[cfg(test)]
mod tests;

pub use provider::mock::MockLlmProvider;
pub use provider::{OpenAiProvider, AnthropicProvider};
pub use provider::LlmConfig;

// Re-export from core traits for convenience
pub use crate::core::traits::{Message, MessageRole, StreamEvent};
pub use crate::core::traits::LLMProvider as LlmProviderTrait;
pub use provider::LlmError;
