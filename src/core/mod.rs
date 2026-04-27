//! ArkCore 核心抽象层
//!
//! 提供核心 Trait 接口抽象，用于解耦具体实现。
//!
//! # 核心 Trait
//!
//! - [`Storage`] - 记忆存储接口
//! - [`Executor`] - 命令执行接口
//! - [`LLMProvider`] - LLM 提供者接口
//! - [`Authenticator`] - 认证接口（预留）
//!
//! # 模块
//!
//! - [`traits`] - 核心 Trait 定义
//! - [`config`] - 统一配置系统
//! - [`container`] - 依赖注入容器

pub mod config;
pub mod container;
pub mod traits;

// Re-export Container from container module
pub use container::Container;

// Re-export Error from crate root
pub use crate::error::Error;

pub use config::{
    AppConfig, Config, ConfigLoader, DatabaseConfig, Environment, LlmConfig, PlatformConfig,
    SecurityConfig,
};
pub use traits::{
    Authenticator, Executor, ExecutionResult, LLMProvider, LlmError, Message, MessageRole,
    SecurityCheckResult, Storage, StreamEvent,
};
