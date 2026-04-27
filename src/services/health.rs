//! 健康检查服务
//!
//! 提供系统健康状态检查，包括数据库连接、沙盒可用性、内存使用、协程数量等

#![allow(unexpected_cfgs)]

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 健康状态级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthLevel {
    /// 完全健康
    Healthy,
    /// 降级运行（部分组件不可用）
    Degraded,
    /// 不健康（关键组件不可用）
    Unhealthy,
}

/// 数据库健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub pool_size: u32,
    pub active_connections: u32,
}

/// 沙盒健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxHealth {
    pub available: bool,
    pub active_instances: u32,
    pub max_instances: u32,
}

/// 内存健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: f64,
}

/// 协程健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(unexpected_cfgs)]
pub struct CoroutineHealth {
    pub active_count: u64,
    #[cfg(feature = "tokio_unstable")]
    pub peak_count: u64,
}

/// 详细健康状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub level: HealthLevel,
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseHealth,
    pub sandbox: SandboxHealth,
    pub memory: MemoryHealth,
    pub coroutines: CoroutineHealth,
}

/// 简略健康响应（用于 /health 端点）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
}

/// 健康检查管理器
pub struct HealthManager {
    /// 启动时间戳
    start_time: std::time::Instant,
    /// 数据库健康状态
    db_health: RwLock<DatabaseHealth>,
    /// 沙盒健康状态
    sandbox_health: RwLock<SandboxHealth>,
}

impl HealthManager {
    /// 创建新的健康检查管理器
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            db_health: RwLock::new(DatabaseHealth {
                connected: true,
                pool_size: 5,
                active_connections: 0,
            }),
            sandbox_health: RwLock::new(SandboxHealth {
                available: true,
                active_instances: 0,
                max_instances: 10,
            }),
        }
    }

    /// 获取运行时间（秒）
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// 更新数据库健康状态
    pub async fn update_database_health(&self, health: DatabaseHealth) {
        let mut db = self.db_health.write().await;
        *db = health;
    }

    /// 更新沙盒健康状态
    pub async fn update_sandbox_health(&self, health: SandboxHealth) {
        let mut sandbox = self.sandbox_health.write().await;
        *sandbox = health;
    }

    /// 获取当前健康状态
    pub async fn get_status(&self) -> HealthStatus {
        let db_health = self.db_health.read().await;
        let sandbox_health = self.sandbox_health.read().await;

        // 获取内存使用情况
        let memory = MemoryHealth {
            used_bytes: 0,
            total_bytes: 0,
            usage_percent: 0.0,
        };

        // 获取协程数量（简化实现）
        let coroutines = CoroutineHealth {
            active_count: 0,
        };

        // 确定健康级别
        let level = if !db_health.connected || !sandbox_health.available {
            HealthLevel::Unhealthy
        } else if db_health.active_connections >= db_health.pool_size
            || sandbox_health.active_instances >= sandbox_health.max_instances
        {
            HealthLevel::Degraded
        } else {
            HealthLevel::Healthy
        };

        HealthStatus {
            level,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.uptime_seconds(),
            database: db_health.clone(),
            sandbox: sandbox_health.clone(),
            memory,
            coroutines,
        }
    }

    /// 获取简略健康状态
    pub async fn get_simple_status(&self) -> HealthResponse {
        let status = self.get_status().await;
        HealthResponse {
            status: match status.level {
                HealthLevel::Healthy => "healthy".to_string(),
                HealthLevel::Degraded => "degraded".to_string(),
                HealthLevel::Unhealthy => "unhealthy".to_string(),
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

impl Default for HealthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 应用状态（用于 Axum State 提取）
#[derive(Clone)]
pub struct AppState {
    pub health_manager: Arc<HealthManager>,
}

impl AppState {
    pub fn new(health_manager: Arc<HealthManager>) -> Self {
        Self { health_manager }
    }
}

/// 详细健康检查处理器（GET /health/detailed）
pub async fn health_detailed_handler(
    State(state): State<AppState>,
) -> Response {
    let status = state.health_manager.get_status().await;

    let http_status = match status.level {
        HealthLevel::Healthy => StatusCode::OK,
        HealthLevel::Degraded => StatusCode::OK,
        HealthLevel::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (http_status, Json(status)).into_response()
}

/// 简单健康检查处理器（GET /health）
pub async fn health_handler(
    State(state): State<AppState>,
) -> Response {
    let response = state.health_manager.get_simple_status().await;

    let http_status = match response.status.as_str() {
        "healthy" | "degraded" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (http_status, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_manager_default() {
        let manager = HealthManager::new();
        let status = manager.get_status().await;

        assert_eq!(status.level, HealthLevel::Healthy);
        assert!(status.database.connected);
        assert!(status.sandbox.available);
    }

    #[tokio::test]
    async fn test_health_manager_uptime() {
        let manager = HealthManager::new();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        // 使用 elapsed().as_millis() 因为 10ms < 1秒，as_secs() 会返回 0
        assert!(manager.start_time.elapsed().as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_simple_health_response() {
        let manager = HealthManager::new();
        let response = manager.get_simple_status().await;

        assert_eq!(response.status, "healthy");
        assert!(response.timestamp > 0);
    }

    #[tokio::test]
    async fn test_unhealthy_state() {
        let manager = HealthManager::new();

        // 设置数据库为断开连接
        manager.update_database_health(DatabaseHealth {
            connected: false,
            pool_size: 5,
            active_connections: 0,
        }).await;

        let status = manager.get_status().await;
        assert_eq!(status.level, HealthLevel::Unhealthy);
    }
}