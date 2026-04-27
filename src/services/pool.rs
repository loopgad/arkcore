//! 连接池调优模块
//!
//! 提供 sqlx 连接池配置和监控

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tokio::sync::RwLock;

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 空闲超时
    pub idle_timeout: Duration,
    /// 最大生命周期
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

impl PoolConfig {
    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }

    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            max_connections: 5,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }

    /// 创建测试环境配置
    pub fn test() -> Self {
        Self {
            max_connections: 2,
            min_connections: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(600),
        }
    }

    /// 应用配置到 SqlitePoolOptions
    pub fn apply_to_sqlite(&self, options: SqlitePoolOptions) -> SqlitePoolOptions {
        options
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(Some(self.idle_timeout))
            .max_lifetime(Some(self.max_lifetime))
    }
}

/// 连接池状态监控
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// 活跃连接数
    pub active_connections: u32,
    /// 空闲连接数
    pub idle_connections: u32,
    /// 等待连接的任务数
    pub waiting_tasks: u32,
}

/// SQLite 连接池管理器
pub struct SqlitePoolManager {
    pool: SqlitePool,
    config: PoolConfig,
}

impl SqlitePoolManager {
    /// 创建新的 SQLite 连接池管理器
    pub async fn new(database_url: &str, config: PoolConfig) -> Result<Self, sqlx::Error> {
        let pool = config
            .apply_to_sqlite(SqlitePoolOptions::new())
            .connect(database_url)
            .await?;

        Ok(Self { pool, config })
    }

    /// 创建内存数据库连接池
    pub async fn memory(config: PoolConfig) -> Result<Self, sqlx::Error> {
        let pool = config
            .apply_to_sqlite(SqlitePoolOptions::new())
            .connect(":memory:")
            .await?;

        Ok(Self { pool, config })
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// 获取当前配置
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// 获取连接池状态
    pub async fn stats(&self) -> PoolStats {
        // sqlx Pool 不直接暴露状态，使用简单估算
        // 实际应用中可使用 metrics 或监控来追踪
        PoolStats {
            active_connections: 0,
            idle_connections: 0,
            waiting_tasks: 0,
        }
    }

    /// 检查连接是否健康
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    /// 关闭连接池
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// 通用的连接池指标收集器
pub struct PoolMetrics {
    /// 活跃连接数
    active: RwLock<u32>,
    /// 空闲连接数
    idle: RwLock<u32>,
    /// 等待连接数
    waiting: RwLock<u32>,
    /// 获取连接的总次数
    acquire_total: RwLock<u64>,
    /// 获取连接失败的总次数
    acquire_errors: RwLock<u64>,
}

impl PoolMetrics {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            active: RwLock::new(0),
            idle: RwLock::new(0),
            waiting: RwLock::new(0),
            acquire_total: RwLock::new(0),
            acquire_errors: RwLock::new(0),
        }
    }

    /// 更新指标
    pub async fn update(&self, stats: PoolStats) {
        *self.active.write().await = stats.active_connections;
        *self.idle.write().await = stats.idle_connections;
        *self.waiting.write().await = stats.waiting_tasks;
    }

    /// 记录获取连接
    pub async fn record_acquire(&self, success: bool) {
        *self.acquire_total.write().await += 1;
        if !success {
            *self.acquire_errors.write().await += 1;
        }
    }

    /// 获取当前指标快照
    pub async fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            active: *self.active.read().await,
            idle: *self.idle.read().await,
            waiting: *self.waiting.read().await,
            acquire_total: *self.acquire_total.read().await,
            acquire_errors: *self.acquire_errors.read().await,
        }
    }
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标快照
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub active: u32,
    pub idle: u32,
    pub waiting: u32,
    pub acquire_total: u64,
    pub acquire_errors: u64,
}

impl MetricsSnapshot {
    /// 计算命中率（获取成功比例）
    pub fn hit_rate(&self) -> f64 {
        if self.acquire_total == 0 {
            1.0
        } else {
            (self.acquire_total - self.acquire_errors) as f64 / self.acquire_total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_pool_config_production() {
        let config = PoolConfig::production();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_pool_config_test() {
        let config = PoolConfig::test();
        assert_eq!(config.max_connections, 2);
        assert_eq!(config.min_connections, 1);
    }

    #[tokio::test]
    async fn test_memory_pool() {
        let config = PoolConfig::test();
        let manager = SqlitePoolManager::memory(config).await;
        assert!(manager.is_ok());

        if let Ok(m) = manager {
            assert!(m.is_healthy().await);
            m.close().await;
        }
    }

    #[tokio::test]
    async fn test_pool_metrics() {
        let metrics = PoolMetrics::new();

        metrics
            .update(PoolStats {
                active_connections: 5,
                idle_connections: 3,
                waiting_tasks: 0,
            })
            .await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.active, 5);
        assert_eq!(snapshot.idle, 3);
    }
}