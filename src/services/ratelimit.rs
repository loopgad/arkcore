//! 限流器服务
//!
//! 基于 Token Bucket 算法实现多维度限流：
//! - 全局限流
//! - Per-user 限流
//! - Per-endpoint 限流
//!
//! 返回 429 Too Many Requests

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 限流维度
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RateLimitDimension {
    /// 全局限流
    Global,
    /// 按用户限流
    User(String),
    /// 按端点限流
    Endpoint(String),
    /// 按用户和端点组合限流
    UserEndpoint(String, String),
}

impl RateLimitDimension {
    /// 从请求上下文中提取限流维度
    pub fn from_user_and_endpoint(user_id: Option<&str>, endpoint: &str) -> Self {
        match user_id {
            Some(uid) => Self::UserEndpoint(uid.to_string(), endpoint.to_string()),
            None => Self::Endpoint(endpoint.to_string()),
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// 全局每秒请求数
    pub global_rate: f64,
    /// 全局桶容量
    pub global_capacity: u64,
    /// 每用户每秒请求数
    pub per_user_rate: f64,
    /// 每用户桶容量
    pub per_user_capacity: u64,
    /// 每端点每秒请求数
    pub per_endpoint_rate: f64,
    /// 每端点桶容量
    pub per_endpoint_capacity: u64,
    /// HashMap 最大条目数 (防止 DoS)
    pub max_bucket_entries: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            global_rate: 100.0,
            global_capacity: 200,
            per_user_rate: 10.0,
            per_user_capacity: 20,
            per_endpoint_rate: 50.0,
            per_endpoint_capacity: 100,
            // 默认限制 10000 个条目，防止内存耗尽
            max_bucket_entries: 10000,
        }
    }
}

/// 带容量限制的 Bucket Map，使用 LRU 驱逐策略
struct BoundedBucketMap<K, V> {
    map: HashMap<K, V>,
    max_size: usize,
    access_order: Vec<K>,
}

impl<K: Eq + std::hash::Hash + Clone, V> BoundedBucketMap<K, V> {
    fn new(max_size: usize) -> Self {
        Self {
            map: HashMap::new(),
            max_size,
            access_order: Vec::new(),
        }
    }

    /// 获取或插入 bucket，必要时驱逐最旧的条目
    fn get_or_insert<F: FnOnce() -> V>(&mut self, key: K, filler: F) -> &mut V {
        // 首先检查是否已存在
        if self.map.contains_key(&key) {
            // 更新访问顺序，将此 key 移到末尾（最近使用）
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key.clone());
            // 返回现有值
            return self.map.get_mut(&key).unwrap();
        }

        // 如果达到容量限制，驱逐最旧的条目
        if self.map.len() >= self.max_size {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.map.remove(&oldest);
                self.access_order.remove(0);
            }
        }

        // 插入新条目
        let key_for_access_order = key.clone();
        self.map.insert(key.clone(), filler());
        self.access_order.push(key_for_access_order);

        // 返回新插入的值
        self.map.get_mut(&key).unwrap()
    }

    /// 获取当前条目数
    fn len(&self) -> usize {
        self.map.len()
    }
}

/// Token Bucket 状态
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    rate: f64,
    capacity: u64,
}

impl TokenBucket {
    /// 创建新的 Token Bucket
    fn new(rate: f64, capacity: u64) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Instant::now(),
            rate,
            capacity,
        }
    }

    /// 尝试获取 token
    fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// 补充 token
    fn refill(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        let new_tokens = elapsed * self.rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_update = Instant::now();
    }

    /// 获取当前 token 数量
    fn available_tokens(&self) -> f64 {
        // 直接计算而不克隆 - 避免不必要的内存分配
        let elapsed = self.last_update.elapsed().as_secs_f64();
        let new_tokens = elapsed * self.rate;
        (self.tokens + new_tokens).min(self.capacity as f64)
    }

    /// 获取距离下次可用 token 的时间
    fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds = tokens_needed / self.rate;
            Duration::from_secs_f64(seconds)
        }
    }
}

/// 限流结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// 是否允许请求
    pub allowed: bool,
    /// 剩余请求数
    pub remaining: u64,
    /// 桶容量
    pub capacity: u64,
    /// 重置时间（Unix 时间戳）
    pub reset_at: u64,
    /// 距离下次可用时间（秒）
    pub retry_after: Option<f64>,
}

/// 限流响应头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitHeaders {
    /// 最多请求数
    pub x_ratelimit_limit: u64,
    /// 剩余请求数
    pub x_ratelimit_remaining: u64,
    /// 重置时间（Unix 时间戳）
    pub x_ratelimit_reset: u64,
    /// 距离下次可用时间（秒）
    pub retry_after: Option<f64>,
}

impl RateLimitHeaders {
    pub fn from_result(result: &RateLimitResult, capacity: u64) -> Self {
        Self {
            x_ratelimit_limit: capacity,
            x_ratelimit_remaining: result.remaining,
            x_ratelimit_reset: result.reset_at,
            retry_after: result.retry_after,
        }
    }
}

/// 429 Too Many Requests 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooManyRequestsResponse {
    pub error: String,
    pub message: String,
    pub retry_after: f64,
}

impl IntoResponse for TooManyRequestsResponse {
    fn into_response(self) -> Response {
        let retry_after_secs = self.retry_after.ceil() as u64;
        let mut response = Json(self).into_response();
        let headers = response.headers_mut();

        headers.insert(
            "retry-after",
            retry_after_secs.to_string().parse().unwrap(),
        );
        headers.insert(
            "x-ratelimit-limit",
            "100".parse().unwrap(),
        );
        headers.insert(
            "x-ratelimit-remaining",
            "0".parse().unwrap(),
        );

        (StatusCode::TOO_MANY_REQUESTS, response).into_response()
    }
}

/// 限流器
pub struct RateLimiter {
    config: RateLimitConfig,
    /// 全局限流桶
    global_bucket: RwLock<TokenBucket>,
    /// 每用户限流桶 (有界)
    user_buckets: RwLock<BoundedBucketMap<String, TokenBucket>>,
    /// 每端点限流桶 (有界)
    endpoint_buckets: RwLock<BoundedBucketMap<String, TokenBucket>>,
    /// 每用户-端点组合限流桶 (有界)
    user_endpoint_buckets: RwLock<BoundedBucketMap<(String, String), TokenBucket>>,
}

impl RateLimiter {
    /// 创建新的限流器
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            global_bucket: RwLock::new(TokenBucket::new(
                config.global_rate,
                config.global_capacity,
            )),
            user_buckets: RwLock::new(BoundedBucketMap::new(config.max_bucket_entries)),
            endpoint_buckets: RwLock::new(BoundedBucketMap::new(config.max_bucket_entries)),
            user_endpoint_buckets: RwLock::new(BoundedBucketMap::new(config.max_bucket_entries)),
        }
    }

    /// 创建默认配置的限流器
    pub fn default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// 检查并获取限流结果
    pub async fn check(&self, dimension: &RateLimitDimension) -> RateLimitResult {
        let now = Instant::now();
        let reset_at = now + Duration::from_secs(60);
        let reset_at_ts = reset_at
            .duration_since(Instant::now())
            .as_secs()
            + std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

        match dimension {
            RateLimitDimension::Global => {
                let mut bucket = self.global_bucket.write().await;
                let allowed = bucket.try_acquire();
                let remaining = bucket.available_tokens() as u64;

                RateLimitResult {
                    allowed,
                    remaining,
                    capacity: self.config.global_capacity,
                    reset_at: reset_at_ts,
                    retry_after: if allowed {
                        None
                    } else {
                        Some(bucket.time_until_available().as_secs_f64())
                    },
                }
            }
            RateLimitDimension::User(user_id) => {
                let mut buckets = self.user_buckets.write().await;
                let bucket = buckets.get_or_insert(user_id.clone(), || {
                    TokenBucket::new(self.config.per_user_rate, self.config.per_user_capacity)
                });
                let allowed = bucket.try_acquire();
                let remaining = bucket.available_tokens() as u64;

                RateLimitResult {
                    allowed,
                    remaining,
                    capacity: self.config.per_user_capacity,
                    reset_at: reset_at_ts,
                    retry_after: if allowed {
                        None
                    } else {
                        Some(bucket.time_until_available().as_secs_f64())
                    },
                }
            }
            RateLimitDimension::Endpoint(endpoint) => {
                let mut buckets = self.endpoint_buckets.write().await;
                let bucket = buckets.get_or_insert(endpoint.clone(), || {
                    TokenBucket::new(
                        self.config.per_endpoint_rate,
                        self.config.per_endpoint_capacity,
                    )
                });
                let allowed = bucket.try_acquire();
                let remaining = bucket.available_tokens() as u64;

                RateLimitResult {
                    allowed,
                    remaining,
                    capacity: self.config.per_endpoint_capacity,
                    reset_at: reset_at_ts,
                    retry_after: if allowed {
                        None
                    } else {
                        Some(bucket.time_until_available().as_secs_f64())
                    },
                }
            }
            RateLimitDimension::UserEndpoint(user_id, endpoint) => {
                let mut buckets = self.user_endpoint_buckets.write().await;
                let key = (user_id.clone(), endpoint.clone());
                let bucket = buckets.get_or_insert(key, || {
                    TokenBucket::new(self.config.per_user_rate, self.config.per_user_capacity)
                });
                let allowed = bucket.try_acquire();
                let remaining = bucket.available_tokens() as u64;

                RateLimitResult {
                    allowed,
                    remaining,
                    capacity: self.config.per_user_capacity,
                    reset_at: reset_at_ts,
                    retry_after: if allowed {
                        None
                    } else {
                        Some(bucket.time_until_available().as_secs_f64())
                    },
                }
            }
        }
    }

    /// 快捷方法：检查全局限流
    pub async fn check_global(&self) -> RateLimitResult {
        self.check(&RateLimitDimension::Global).await
    }

    /// 快捷方法：检查用户限流
    pub async fn check_user(&self, user_id: &str) -> RateLimitResult {
        self.check(&RateLimitDimension::User(user_id.to_string())).await
    }

    /// 快捷方法：检查端点限流
    pub async fn check_endpoint(&self, endpoint: &str) -> RateLimitResult {
        self.check(&RateLimitDimension::Endpoint(endpoint.to_string())).await
    }

    /// 获取当前状态摘要
    pub async fn status(&self) -> RateLimiterStatus {
        let global = self.global_bucket.read().await.available_tokens() as u64;

        RateLimiterStatus {
            global_available: global,
            global_capacity: self.config.global_capacity,
            user_count: self.user_buckets.read().await.len(),
            endpoint_count: self.endpoint_buckets.read().await.len(),
        }
    }
}

/// 限流器状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterStatus {
    pub global_available: u64,
    pub global_capacity: u64,
    pub user_count: usize,
    pub endpoint_count: usize,
}

/// 限流中间件状态
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
}

impl RateLimitState {
    pub fn new(limiter: RateLimiter) -> Self {
        Self {
            limiter: Arc::new(limiter),
        }
    }

    pub fn default_config() -> Self {
        Self::new(RateLimiter::default_config())
    }

    /// 检查限流并返回结果
    pub async fn check(&self, dimension: RateLimitDimension) -> RateLimitResult {
        self.limiter.check(&dimension).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket_acquire() {
        let mut bucket = TokenBucket::new(10.0, 10);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        // 使用近似比较处理浮点精度问题
        assert!((bucket.tokens - 8.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_token_bucket_exhausted() {
        let mut bucket = TokenBucket::new(1.0, 2);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire()); // 耗尽
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(100.0, 10); // 100 tokens/sec
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());

        // 等待补充
        tokio::time::sleep(Duration::from_millis(30)).await;
        bucket.refill();

        // 应该有足够的 token
        assert!(bucket.available_tokens() >= 8.0);
    }

    #[tokio::test]
    async fn test_rate_limiter_global() {
        let limiter = RateLimiter::default_config();
        let result = limiter.check_global().await;
        assert!(result.allowed);
        assert_eq!(result.capacity, 200);
    }

    #[tokio::test]
    async fn test_rate_limiter_user() {
        let limiter = RateLimiter::default_config();
        let result = limiter.check_user("user123").await;
        assert!(result.allowed);
        assert_eq!(result.capacity, 20);
    }

    #[tokio::test]
    async fn test_rate_limiter_endpoint() {
        let limiter = RateLimiter::default_config();
        let result = limiter.check_endpoint("/api/health").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_rate_limit_dimension_from_context() {
        let dim = RateLimitDimension::from_user_and_endpoint(Some("user1"), "/api/test");
        assert_eq!(
            dim,
            RateLimitDimension::UserEndpoint("user1".to_string(), "/api/test".to_string())
        );

        let dim = RateLimitDimension::from_user_and_endpoint(None, "/api/test");
        assert_eq!(
            dim,
            RateLimitDimension::Endpoint("/api/test".to_string())
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_status() {
        let limiter = RateLimiter::default_config();
        let status = limiter.status().await;

        assert_eq!(status.global_capacity, 200);
        assert!(status.global_available <= 200);
        assert_eq!(status.user_count, 0);
    }
}