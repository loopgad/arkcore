//! ArkCore 服务层
//!
//! 提供核心服务实现：编排器、沙盒、记忆存储等。
//!
//! # 模块
//!
//! - [`orchestrator`] - AI Agent 调度器
//! - [`sandbox`] - 零信任执行沙盒
//! - [`memory`] - SQLite FTS5 记忆引擎
//! - [`security`] - 安全服务 (审计、加密、RBAC、合规报告)

pub mod security;

// 重新导出根模块的服务（保持向后兼容）
pub use crate::memory::SkillMemory;
pub use crate::orchestrator::Orchestrator;
pub use crate::sandbox::Sandbox;

// 安全服务重导出
pub use security::audit::AuditEvent;
pub use security::crypto::CryptoService;
pub use security::rbac::RbacService;
pub use security::apikey::ApiKeyService;
pub use security::compliance::ComplianceReporter;

// 可靠性服务模块
pub mod health;
pub mod ratelimit;
pub mod retry;
pub mod timeout;

// 性能优化服务模块
pub mod pool;      // T5.1: 连接池调优
pub mod cache;     // T5.2: 缓存层 (moka)
pub mod object_pool; // T5.3: 对象池 (bbqueue)
pub mod async_rt;  // T5.4: 异步 I/O 优化

// 重新导出可靠性类型
pub use health::{
    AppState, HealthLevel, HealthManager, HealthResponse, HealthStatus,
};
pub use ratelimit::{
    RateLimitConfig, RateLimitDimension, RateLimitHeaders, RateLimitResult,
    RateLimitState, RateLimiter, RateLimiterStatus, TooManyRequestsResponse,
};
pub use retry::{
    NetworkError, Retry, RetryConfig, RetryResult, RetryStrategy, Retryable,
};
pub use timeout::{
    TimeoutConfig, TimeoutContext, TimeoutError, TimeoutExecutor, TimeoutManager,
    TimeoutResult,
};

// 重新导出性能优化类型
pub use pool::{
    PoolConfig, PoolMetrics, PoolStats, SqlitePoolManager,
};
pub use cache::{
    CacheConfig, CacheManager, CacheStats, ConfigCacheKey, LlmCacheEntry,
    LlmCacheKey, SearchCacheEntry, SearchCacheKey, SearchResult,
};
pub use object_pool::{
    ByteBufferPool, ObjectPoolConfig, ObjectPoolManager, ObjectPoolStats,
    PooledByteBuffer, PooledStringBuffer, StringBufferPool,
};
pub use async_rt::{
    TokioFlavor, TokioRuntimeConfig, RuntimeMetrics,
    task, io, coalesce, metrics,
};