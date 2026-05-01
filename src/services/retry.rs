//! 重试机制服务
//!
//! 提供可靠的重试逻辑，包括 Fixed Delay、Exponential Backoff with Jitter
//! 支持可重试错误判断和最大重试次数配置

use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// 重试策略类型
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    /// 固定延迟重试
    Fixed,
    /// 指数退避重试（带抖动）
    ExponentialBackoff,
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟时间
    pub initial_delay: Duration,
    /// 最大延迟时间
    pub max_delay: Duration,
    /// 重试策略
    pub strategy: RetryStrategy,
    /// 基础乘数（用于指数退避）
    pub multiplier: f64,
    /// 抖动因子（0.0 - 1.0）
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            strategy: RetryStrategy::ExponentialBackoff,
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    /// 创建固定延迟重试配置
    pub fn fixed_delay(delay: Duration, max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay: delay,
            max_delay: delay,
            strategy: RetryStrategy::Fixed,
            multiplier: 1.0,
            jitter: 0.0,
        }
    }

    /// 创建指数退避重试配置
    pub fn exponential_backoff(
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: u32,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            strategy: RetryStrategy::ExponentialBackoff,
            multiplier: 2.0,
            jitter: 0.1,
        }
    }

    /// 计算第 n 次重试的延迟时间
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self.strategy {
            RetryStrategy::Fixed => self.initial_delay,
            RetryStrategy::ExponentialBackoff => {
                let exp_delay = self.initial_delay.as_millis() as f64
                    * self.multiplier.powi(attempt as i32 - 1);
                let delay_ms = exp_delay.min(self.max_delay.as_millis() as f64);

                // 应用抖动
                if self.jitter > 0.0 {
                    let jitter_range = delay_ms * self.jitter;
                    let jitter = rand_jitter(jitter_range as u64);
                    let final_delay = delay_ms as u64 + jitter;
                    Duration::from_millis(final_delay.min(self.max_delay.as_millis() as u64))
                } else {
                    Duration::from_millis(delay_ms as u64)
                }
            }
        }
    }
}

/// 生成随机抖动值
fn rand_jitter(max_jitter: u64) -> u64 {
    rand::thread_rng().gen_range(0..=max_jitter)
}

/// 可重试错误特征
pub trait Retryable: std::fmt::Debug + Send + Sync {
    /// 判断错误是否可重试
    fn is_retryable(&self) -> bool;

    /// 获取错误码（用于日志记录）
    fn error_code(&self) -> Option<String>;
}

/// 重试结果
#[derive(Debug)]
pub enum RetryResult<T> {
    /// 成功
    Success(T),
    /// 重试耗尽后失败
    RetriesExhausted {
        attempts: u32,
        last_error: Box<dyn std::fmt::Debug + Send + Sync>,
    },
    /// 不支持的操作
    NotRetryable,
}

/// 重试封装器
pub struct Retry<F, T, E> {
    config: RetryConfig,
    operation: F,
    _phantom: std::marker::PhantomData<(T, E)>,
}

impl<F, T, E> Retry<F, T, E> {
    /// 创建新的重试封装器
    pub fn new(config: RetryConfig, operation: F) -> Self {
        Self {
            config,
            operation,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T, E, Fut> Retry<F, T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Retryable + 'static,
{
    /// 执行重试逻辑
    #[allow(unused_assignments)]
    pub async fn execute(mut self) -> RetryResult<T> {
        let mut last_err: Option<Box<dyn std::fmt::Debug + Send + Sync>> = None;
        let mut attempt = 0u32;

        loop {
            attempt += 1;
            debug!(
                attempt = attempt,
                max_attempts = self.config.max_attempts,
                "执行重试操作"
            );

            match (self.operation)().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(attempts = attempt, "重试成功");
                    }
                    return RetryResult::Success(result);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        warn!(
                            error = ?e,
                            error_code = ?e.error_code(),
                            "错误不可重试，终止重试"
                        );
                        last_err = Some(Box::new(e));
                        break;
                    }

                    last_err = Some(Box::new(e));

                    if attempt >= self.config.max_attempts {
                        warn!(attempts = attempt, "重试次数耗尽");
                        break;
                    }

                    let delay = self.config.calculate_delay(attempt);
                    info!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        last_error = ?last_err,
                        "等待重试"
                    );
                    sleep(delay).await;
                }
            }
        }

        RetryResult::RetriesExhausted {
            attempts: attempt,
            last_error: last_err.unwrap_or_else(|| Box::new("Unknown error")),
        }
    }
}

/// 简化的重试宏
#[macro_export]
macro_rules! retry {
    ($operation:expr, $config:expr) => {{
        let retry = $crate::services::retry::Retry::new($config, $operation);
        retry.execute().await
    }};
}

/// 网络错误重试判断
#[derive(Debug)]
pub struct NetworkError {
    pub message: String,
    pub is_timeout: bool,
    pub is_connection_refused: bool,
}

impl NetworkError {
    pub fn timeout(message: String) -> Self {
        Self {
            message,
            is_timeout: true,
            is_connection_refused: false,
        }
    }

    pub fn connection_refused(message: String) -> Self {
        Self {
            message,
            is_timeout: false,
            is_connection_refused: true,
        }
    }
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NetworkError: {}", self.message)
    }
}

impl Retryable for NetworkError {
    fn is_retryable(&self) -> bool {
        // 超时和连接拒绝通常是可重试的
        self.is_timeout || self.is_connection_refused
    }

    fn error_code(&self) -> Option<String> {
        if self.is_timeout {
            Some("NETWORK_TIMEOUT".to_string())
        } else if self.is_connection_refused {
            Some("CONNECTION_REFUSED".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_delay_config() {
        let config = RetryConfig::fixed_delay(Duration::from_millis(100), 3);
        assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_exponential_backoff_config() {
        let config = RetryConfig::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            5,
        );

        // 第一次重试延迟 = 100ms + jitter (0-10ms, 因为 jitter=0.1)
        // 延迟应该在 100ms 到 110ms 之间
        let delay = config.calculate_delay(1);
        let delay_ms = delay.as_millis() as u64;
        assert!(
            (100..=110).contains(&delay_ms),
            "Expected delay between 100-110ms, got {}ms",
            delay_ms
        );

        // 第二次重试延迟 = 200ms + jitter (0-20ms)
        let delay2 = config.calculate_delay(2);
        let delay2_ms = delay2.as_millis() as u64;
        assert!(
            (200..=220).contains(&delay2_ms),
            "Expected delay between 200-220ms, got {}ms",
            delay2_ms
        );
    }

    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::default();
        let retry = Retry::new(config, || async move { Ok::<_, NetworkError>(42) });
        let result = retry.execute().await;

        match result {
            RetryResult::Success(value) => assert_eq!(value, 42),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_retry_not_retryable() {
        let config = RetryConfig::default();
        let retry = Retry::new(config, || async move {
            Err(NetworkError::connection_refused(
                "Connection refused".to_string(),
            ))
        });
        let result: RetryResult<()> = retry.execute().await;

        match result {
            RetryResult::RetriesExhausted { attempts, .. } => {
                assert_eq!(attempts, 3); // 默认 max_attempts
            }
            _ => panic!("Expected retries exhausted"),
        }
    }

    #[tokio::test]
    async fn test_network_error_retryable() {
        let err = NetworkError::timeout("Connection timed out".to_string());
        assert!(err.is_retryable());

        let err = NetworkError::connection_refused("Connection refused".to_string());
        assert!(err.is_retryable());
    }
}
