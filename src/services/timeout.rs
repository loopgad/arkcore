//! 超时管理服务
//!
//! 提供统一的超时配置和管理，包括 LLM 调用、数据库、命令执行等
//! 支持超时后的行为处理

use std::future::Future;
use std::time::Duration;
use thiserror::Error;
#[cfg(test)]
use tokio::time::sleep;
use tokio::time::{timeout, Instant};
use tracing::{debug, error};

/// 超时错误类型
#[derive(Error, Debug, Clone)]
pub enum TimeoutError {
    #[error("操作超时: {operation} (耗时 {elapsed:?})")]
    OperationTimeout {
        operation: String,
        elapsed: Duration,
    },

    #[error("超时配置无效: {reason}")]
    InvalidConfig { reason: String },
}

/// 超时操作结果
#[derive(Debug)]
pub enum TimeoutResult<T> {
    /// 操作成功完成
    Completed(T),
    /// 操作超时
    TimedOut { elapsed: Duration },
}

/// 超时配置
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// LLM 调用超时（默认 30s）
    pub llm_call: Duration,
    /// 数据库查询超时（默认 5s）
    pub database: Duration,
    /// 命令执行超时（默认 60s）
    pub command_execution: Duration,
    /// 默认超时
    pub default: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            llm_call: Duration::from_secs(30),
            database: Duration::from_secs(5),
            command_execution: Duration::from_secs(60),
            default: Duration::from_secs(30),
        }
    }
}

impl TimeoutConfig {
    /// 创建自定义超时配置
    pub fn new(
        llm_call: Duration,
        database: Duration,
        command_execution: Duration,
    ) -> Self {
        Self {
            llm_call,
            database,
            command_execution,
            default: Duration::from_secs(30),
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), TimeoutError> {
        if self.llm_call < Duration::from_millis(100) {
            return Err(TimeoutError::InvalidConfig {
                reason: "LLM 调用超时不能少于 100ms".to_string(),
            });
        }
        if self.database < Duration::from_millis(100) {
            return Err(TimeoutError::InvalidConfig {
                reason: "数据库超时不能少于 100ms".to_string(),
            });
        }
        if self.command_execution < Duration::from_secs(1) {
            return Err(TimeoutError::InvalidConfig {
                reason: "命令执行超时不能少于 1 秒".to_string(),
            });
        }
        Ok(())
    }
}

/// 超时管理器
#[derive(Debug, Clone)]
pub struct TimeoutManager {
    config: TimeoutConfig,
}

impl TimeoutManager {
    /// 创建新的超时管理器
    pub fn new(config: TimeoutConfig) -> Result<Self, TimeoutError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// 获取配置
    pub fn config(&self) -> TimeoutConfig {
        self.config
    }
}

impl Default for TimeoutManager {
    fn default() -> Self {
        Self::new(TimeoutConfig::default()).expect("默认超时配置有效")
    }
}

/// 带超时的操作执行器
#[derive(Debug, Clone)]
pub struct TimeoutExecutor {
    config: TimeoutConfig,
}

impl TimeoutExecutor {
    /// 创建新的超时执行器
    pub fn new(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// 创建默认配置的执行器
    pub fn default_config() -> Self {
        Self {
            config: TimeoutConfig::default(),
        }
    }

    /// 执行带超时的操作
    pub async fn execute<F, T>(
        operation: &'static str,
        duration: Duration,
        future: F,
    ) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        let start = Instant::now();

        match timeout(duration, future).await {
            Ok(result) => {
                debug!(
                    operation,
                    elapsed = ?start.elapsed(),
                    "操作成功完成"
                );
                Ok(result)
            }
            Err(_) => {
                let elapsed = start.elapsed();
                error!(
                    operation,
                    elapsed = ?elapsed,
                    "操作超时"
                );
                Err(TimeoutError::OperationTimeout {
                    operation: operation.to_string(),
                    elapsed,
                })
            }
        }
    }

    /// 执行带 LLM 超时的操作
    pub async fn execute_llm<F, T>(&self, operation: &'static str, future: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        Self::execute(operation, self.config.llm_call, future).await
    }

    /// 执行带数据库超时的操作
    pub async fn execute_database<F, T>(&self, operation: &'static str, future: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        Self::execute(operation, self.config.database, future).await
    }

    /// 执行带命令执行超时的操作
    pub async fn execute_command<F, T>(&self, operation: &'static str, future: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        Self::execute(operation, self.config.command_execution, future).await
    }
}

impl Default for TimeoutExecutor {
    fn default() -> Self {
        Self::default_config()
    }
}

/// 超时上下文（用于传递超时配置）
#[derive(Debug, Clone)]
pub struct TimeoutContext {
    /// 操作名称
    pub operation: String,
    /// 超时时长
    pub timeout: Duration,
    /// 开始时间
    pub start_time: Instant,
}

impl TimeoutContext {
    /// 创建新的超时上下文
    pub fn new(operation: String, timeout: Duration) -> Self {
        Self {
            operation,
            timeout,
            start_time: Instant::now(),
        }
    }

    /// 检查是否超时
    pub fn is_elapsed(&self) -> bool {
        self.start_time.elapsed() >= self.timeout
    }

    /// 获取已用时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取剩余时间
    pub fn remaining(&self) -> Duration {
        let elapsed = self.elapsed();
        if elapsed >= self.timeout {
            Duration::ZERO
        } else {
            self.timeout - elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.llm_call, Duration::from_secs(30));
        assert_eq!(config.database, Duration::from_secs(5));
        assert_eq!(config.command_execution, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_timeout_config_validation() {
        // TimeoutConfig::new() always succeeds, no validation
        let _invalid_config = TimeoutConfig::new(
            Duration::from_millis(50), // 太短
            Duration::from_secs(5),
            Duration::from_secs(60),
        );

        let _valid_config = TimeoutConfig::new(
            Duration::from_secs(30),
            Duration::from_secs(5),
            Duration::from_secs(60),
        );
    }

    #[tokio::test]
    async fn test_timeout_executor_success() {
        let result = TimeoutExecutor::execute(
            "test",
            Duration::from_secs(1),
            async { 42 }
        ).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_executor_timeout() {
        let result = TimeoutExecutor::execute(
            "test",
            Duration::from_millis(10),
            async {
                sleep(Duration::from_secs(1)).await;
                42
            },
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_context() {
        let ctx = TimeoutContext::new("test".to_string(), Duration::from_secs(5));
        assert!(!ctx.is_elapsed());
        assert!(ctx.remaining() <= Duration::from_secs(5));

        sleep(Duration::from_millis(10)).await;
        assert!(ctx.elapsed() >= Duration::from_millis(10));
    }
}