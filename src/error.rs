//! 统一错误类型定义 - 使用 thiserror 2.0
//!
//! 错误层级设计：
//! - [`Error::Core`] - 核心错误，持有 anyhow::Error
//! - [`Error::Storage`] - 存储相关错误
//! - [`Error::Security`] - 安全相关错误
//! - [`Error::Network`] - 网络相关错误
//! - [`Error::Config`] - 配置相关错误
//! - [`Error::Platform`] - 平台相关错误

use thiserror::Error;

/// Boxed error type for chained errors
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// 统一错误枚举
#[derive(Error, Debug)]
pub enum Error {
    /// 核心错误 - 用于包装任意错误类型
    #[error("Core error: {0}")]
    Core(#[from] anyhow::Error),

    /// 存储相关错误
    #[error("Storage error: {context}")]
    Storage {
        context: String,
        #[source]
        source: Option<BoxedError>,
    },

    /// 安全相关错误
    #[error("Security error: {reason}")]
    Security {
        reason: String,
        violation: Option<String>,
    },

    /// 网络相关错误
    #[error("Network error: {operation}")]
    Network {
        operation: String,
        #[source]
        source: Option<BoxedError>,
    },

    /// 配置相关错误
    #[error("Config error: {0}")]
    Config(String),

    /// 平台相关错误
    #[error("Platform error: {0}")]
    Platform(String),
}

// ============ 错误上下文方法 ============

impl Error {
    /// 创建存储错误
    pub fn storage(context: impl Into<String>) -> Self {
        Self::Storage {
            context: context.into(),
            source: None,
        }
    }

    /// 带源错误创建存储错误
    pub fn storage_with_source(context: impl Into<String>, source: BoxedError) -> Self {
        Self::Storage {
            context: context.into(),
            source: Some(source),
        }
    }

    /// 创建安全错误
    pub fn security(reason: impl Into<String>) -> Self {
        Self::Security {
            reason: reason.into(),
            violation: None,
        }
    }

    /// 带违规详情创建安全错误
    pub fn security_with_violation(reason: impl Into<String>, violation: impl Into<String>) -> Self {
        Self::Security {
            reason: reason.into(),
            violation: Some(violation.into()),
        }
    }

    /// 创建网络错误
    pub fn network(operation: impl Into<String>) -> Self {
        Self::Network {
            operation: operation.into(),
            source: None,
        }
    }

    /// 带源错误创建网络错误
    pub fn network_with_source(operation: impl Into<String>, source: BoxedError) -> Self {
        Self::Network {
            operation: operation.into(),
            source: Some(source),
        }
    }

    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// 创建平台错误
    pub fn platform(msg: impl Into<String>) -> Self {
        Self::Platform(msg.into())
    }
}

// ============ From trait 实现 ============

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Core(anyhow::anyhow!("IO error: {}", err))
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Core(anyhow::anyhow!("Database error: {}", err))
    }
}