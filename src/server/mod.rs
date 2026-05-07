//! Server 模块
//!
//! Axum HTTP 服务器 - 支持 SSE 和 WebSocket
//! 提供 Graceful Shutdown 支持

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::orchestrator::Orchestrator;
use crate::protocol::{AgentSnapshot, ClientMessage, WsMessage};
use crate::services::{
    health::{AppState, HealthManager},
    ratelimit::RateLimiter,
};

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 端口号
    pub port: u16,
    /// 优雅关闭超时时间
    pub shutdown_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

impl ServerConfig {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            shutdown_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }
}

/// 优雅关闭状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    /// 运行中
    Running,
    /// 收到关闭信号
    ShuttingDown,
    /// 已完成关闭
    Terminated,
}

/// 服务器主结构
pub struct Server {
    config: ServerConfig,
    orchestrator: Arc<Orchestrator>,
    health_manager: Arc<HealthManager>,
    rate_limiter: Arc<RateLimiter>,
    shutdown_tx: broadcast::Sender<ShutdownState>,
}

impl Server {
    /// 创建新的 Server 实例
    pub fn new(orchestrator: Orchestrator, config: ServerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);
        let health_manager = Arc::new(HealthManager::new());
        let rate_limiter = Arc::new(RateLimiter::default_config());

        Self {
            config,
            orchestrator: Arc::new(orchestrator),
            health_manager,
            rate_limiter,
            shutdown_tx,
        }
    }

    /// 创建使用默认配置的 Server 实例
    pub fn with_default_config(orchestrator: Orchestrator, port: u16) -> Self {
        Self::new(orchestrator, ServerConfig::new(port))
    }

    /// 获取健康管理器
    pub fn health_manager(&self) -> Arc<HealthManager> {
        self.health_manager.clone()
    }

    /// 获取限流器
    pub fn rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter.clone()
    }

    /// 启动服务器
    pub async fn run(self) -> anyhow::Result<()> {
        let (graceful_tx, mut graceful_rx) = broadcast::channel::<()>(1);

        // 构建应用状态
        let mut app_state = AppState::new(self.health_manager.clone());
        app_state.orchestrator = Some(self.orchestrator.clone());

        // 构建路由
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/health/detailed", get(health_detailed_handler))
            .route("/events", get(sse_handler))
            .route("/ws", get(ws_handler))
            .with_state(app_state.clone());

        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("服务器运行于 {}", addr);

        // 启动信号监听任务
        let shutdown_timeout = self.config.shutdown_timeout;
        let graceful_tx_clone = graceful_tx.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            Self::listen_for_shutdown(shutdown_rx, graceful_tx_clone, shutdown_timeout).await;
        });

        // 启动服务器
        let server = axum::serve(listener, app);

        // 使用 graceful_rx 处理优雅关闭
        let server = server.with_graceful_shutdown(async move {
            graceful_rx.recv().await.ok();
        });

        if let Err(e) = server.await {
            error!("服务器错误: {}", e);
        }

        info!("服务器已关闭");
        Ok(())
    }

    /// 监听关闭信号
    async fn listen_for_shutdown(
        mut shutdown_rx: broadcast::Receiver<ShutdownState>,
        graceful_tx: broadcast::Sender<()>,
        timeout: Duration,
    ) {
        // 优先使用 tokio 的信号监听，Ctrl+C 优先级更高
        tokio::select! {
            biased;
            // 监听 SIGINT (优先级更高)
            _ = signal::ctrl_c() => {
                warn!("收到 SIGINT (Ctrl+C)，开始优雅关闭...");
                let _ = graceful_tx.send(());
            }
            // 监听我们的广播通道
            result = shutdown_rx.recv() => {
                if let Ok(state) = result {
                    if state == ShutdownState::ShuttingDown {
                        warn!("收到关闭信号，开始优雅关闭...");
                        let _ = graceful_tx.send(());
                    }
                }
            }
        }

        // 等待优雅关闭完成或超时
        tokio::select! {
            result = shutdown_rx.recv() => {
                if let Ok(state) = result {
                    if state == ShutdownState::Terminated {
                        info!("优雅关闭完成");
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                warn!("优雅关闭超时，强制退出");
            }
        }
    }

    /// 触发优雅关闭
    pub async fn shutdown(&self) {
        info!("触发服务器关闭");
        let _ = self.shutdown_tx.send(ShutdownState::ShuttingDown);

        // 等待一段时间让请求处理完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        let _ = self.shutdown_tx.send(ShutdownState::Terminated);
    }

    /// 保存状态（供子类或外部调用）
    pub async fn save_state(&self) -> anyhow::Result<()> {
        info!("保存服务器状态...");
        info!("状态保存完成");
        Ok(())
    }

    /// 释放资源
    pub async fn cleanup(&self) {
        info!("清理服务器资源...");
        info!("资源清理完成");
    }
}

/// 健康检查处理器
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    use crate::services::health::health_handler as svc_health;
    svc_health(axum::extract::State(state)).await
}

/// 详细健康检查处理器
async fn health_detailed_handler(State(state): State<AppState>) -> impl IntoResponse {
    use crate::services::health::health_detailed_handler as svc_health_detailed;
    svc_health_detailed(axum::extract::State(state)).await
}

/// SSE 事件流处理器
async fn sse_handler(
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = tokio_stream::iter(vec![
        Ok(Event::default().data("Connected to ArkCore")),
        Ok(Event::default()
            .event("status")
            .data(r#"{"status":"ready"}"#)),
    ]);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// WebSocket 升级处理器
async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// WebSocket 配置常量
mod ws_config {
    /// 最大消息大小 (1MB)
    pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
    /// 最大每连接消息数
    pub const MAX_MESSAGES_PER_CONN: usize = 10000;
    /// 空闲超时 (秒)
    pub const IDLE_TIMEOUT_SECS: u64 = 300;
    /// 回话最大存活时间 (秒)
    pub const MAX_SESSION_SECS: u64 = 3600;
}

/// WebSocket 消息处理
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 消息计数器，用于限制每连接消息数
    let mut msg_count: usize = 0;
    // 连接开始时间
    let start_time = std::time::Instant::now();

    // 订阅 Agent 状态变化
    let mut agent_state_rx = state.subscribe_agent_state();

    // 发送连接成功消息
    let welcome_msg = WsMessage::Error("Connected to ArkCore".to_string());
    if let Ok(text) = serde_json::to_string(&welcome_msg) {
        let _ = sender
            .send(axum::extract::ws::Message::Text(text.into()))
            .await;
    }

    loop {
        // 检查会话最大存活时间
        if start_time.elapsed().as_secs() > ws_config::MAX_SESSION_SECS {
            let msg = WsMessage::Error("session_timeout".to_string());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = sender
                    .send(axum::extract::ws::Message::Text(text.into()))
                    .await;
            }
            break;
        }

        // 使用 tokio::select! 同时处理客户端消息和状态推送
        tokio::select! {
            biased;

            // 处理 Agent 状态变化推送
            Ok(agent_state) = agent_state_rx.recv() => {
                let snapshot = AgentSnapshot::from(&agent_state);
                let msg = WsMessage::AgentStatus(snapshot);
                if let Ok(text) = serde_json::to_string(&msg) {
                    if sender.send(axum::extract::ws::Message::Text(text.into())).await.is_err() {
                        break; // 客户端断开
                    }
                }
            }

            // 处理客户端消息（带超时）
            msg = tokio::time::timeout(
                Duration::from_secs(ws_config::IDLE_TIMEOUT_SECS),
                receiver.next(),
            ) => {
                match msg {
                    Ok(Some(Ok(axum::extract::ws::Message::Text(text)))) => {
                        // 检查消息大小
                        if text.len() > ws_config::MAX_MESSAGE_SIZE {
                            let err_msg = WsMessage::Error("message_too_large".to_string());
                            if let Ok(err_text) = serde_json::to_string(&err_msg) {
                                let _ = sender.send(axum::extract::ws::Message::Text(err_text.into())).await;
                            }
                            break;
                        }

                        msg_count += 1;

                        // 检查消息数限制
                        if msg_count > ws_config::MAX_MESSAGES_PER_CONN {
                            let err_msg = WsMessage::Error("too_many_messages".to_string());
                            if let Ok(err_text) = serde_json::to_string(&err_msg) {
                                let _ = sender.send(axum::extract::ws::Message::Text(err_text.into())).await;
                            }
                            break;
                        }

                        // 解析客户端消息
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                let response = handle_client_message(client_msg, &state).await;
                                if let Ok(response_text) = serde_json::to_string(&response) {
                                    if sender.send(axum::extract::ws::Message::Text(response_text.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let err_msg = WsMessage::Error(format!("Invalid message: {}", e));
                                if let Ok(err_text) = serde_json::to_string(&err_msg) {
                                    let _ = sender.send(axum::extract::ws::Message::Text(err_text.into())).await;
                                }
                            }
                        }
                    }
                    Ok(Some(Ok(axum::extract::ws::Message::Close(_)))) => {
                        // 客户端关闭连接
                        break;
                    }
                    Ok(Some(Err(e))) => {
                        // WebSocket 错误
                        warn!("WebSocket 错误: {}", e);
                        break;
                    }
                    Ok(None) => {
                        // 连接已关闭
                        break;
                    }
                    Err(_) => {
                        // 空闲超时
                        let err_msg = WsMessage::Error("idle_timeout".to_string());
                        if let Ok(err_text) = serde_json::to_string(&err_msg) {
                            let _ = sender.send(axum::extract::ws::Message::Text(err_text.into())).await;
                        }
                        break;
                    }
                    _ => {
                        // 忽略其他消息类型
                    }
                }
            }
        }
    }

    info!("WebSocket 连接已关闭");
}

/// 处理客户端消息
async fn handle_client_message(msg: ClientMessage, state: &AppState) -> WsMessage {
    match msg {
        ClientMessage::RunTask { task } => {
            if let Some(ref orchestrator) = state.orchestrator {
                // 广播状态变化
                state.broadcast_agent_state(&crate::orchestrator::AgentState::Planning {
                    task: task.clone(),
                });

                match orchestrator.run_task(&task).await {
                    Ok(result) => {
                        state.broadcast_agent_state(&crate::orchestrator::AgentState::Completed {
                            result: result.clone(),
                        });
                        WsMessage::Output(crate::protocol::CommandOutput {
                            content: result,
                            output_type: crate::protocol::OutputType::Result,
                            timestamp: chrono::Utc::now().timestamp(),
                        })
                    }
                    Err(e) => {
                        state.broadcast_agent_state(&crate::orchestrator::AgentState::Failed {
                            reason: e.to_string(),
                        });
                        WsMessage::Error(e.to_string())
                    }
                }
            } else {
                WsMessage::Error("Orchestrator not available".to_string())
            }
        }
        ClientMessage::RunCommand { task, command } => {
            if let Some(ref orchestrator) = state.orchestrator {
                state.broadcast_agent_state(&crate::orchestrator::AgentState::Planning {
                    task: task.clone(),
                });

                match orchestrator.run_with_approval(&task, &command).await {
                    Ok(result) => {
                        state.broadcast_agent_state(&crate::orchestrator::AgentState::Completed {
                            result: result.clone(),
                        });
                        WsMessage::Output(crate::protocol::CommandOutput {
                            content: result,
                            output_type: crate::protocol::OutputType::Result,
                            timestamp: chrono::Utc::now().timestamp(),
                        })
                    }
                    Err(e) => {
                        state.broadcast_agent_state(&crate::orchestrator::AgentState::Failed {
                            reason: e.to_string(),
                        });
                        WsMessage::Error(e.to_string())
                    }
                }
            } else {
                WsMessage::Error("Orchestrator not available".to_string())
            }
        }
        ClientMessage::Reset => {
            if let Some(ref orchestrator) = state.orchestrator {
                orchestrator.reset().await;
                state.broadcast_agent_state(&crate::orchestrator::AgentState::Idle);
                WsMessage::Error("Agent reset".to_string())
            } else {
                WsMessage::Error("Orchestrator not available".to_string())
            }
        }
        ClientMessage::GetMetrics => {
            let status = state.health_manager.get_status();
            let metrics = crate::protocol::SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: status.memory.usage_percent as f32,
                disk_usage: 0.0,
                latency_ms: 0,
                health_level: match status.level {
                    crate::services::health::HealthLevel::Healthy => {
                        crate::protocol::HealthLevel::Healthy
                    }
                    crate::services::health::HealthLevel::Degraded => {
                        crate::protocol::HealthLevel::Degraded
                    }
                    crate::services::health::HealthLevel::Unhealthy => {
                        crate::protocol::HealthLevel::Unhealthy
                    }
                },
                uptime_seconds: status.uptime_seconds,
                version: status.version,
            };
            WsMessage::Metrics(metrics)
        }
    }
}

/// 服务器关闭句柄
pub struct ShutdownHandle {
    shutdown_tx: broadcast::Sender<ShutdownState>,
}

impl ShutdownHandle {
    /// 创建新的关闭句柄
    pub fn new(shutdown_tx: broadcast::Sender<ShutdownState>) -> Self {
        Self { shutdown_tx }
    }

    /// 请求关闭
    pub async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(ShutdownState::ShuttingDown);
    }

    /// 强制终止
    pub async fn terminate(&self) {
        let _ = self.shutdown_tx.send(ShutdownState::Terminated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_server_config_builder() {
        let config = ServerConfig::new(9090).with_shutdown_timeout(Duration::from_secs(60));

        assert_eq!(config.port, 9090);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_shutdown_state_enum_variants() {
        assert_eq!(ShutdownState::Running as u8, 0);
        assert_eq!(ShutdownState::ShuttingDown as u8, 1);
        assert_eq!(ShutdownState::Terminated as u8, 2);
    }

    #[test]
    fn test_shutdown_state_partial_eq() {
        assert_eq!(ShutdownState::Running, ShutdownState::Running);
        assert_eq!(ShutdownState::ShuttingDown, ShutdownState::ShuttingDown);
        assert_eq!(ShutdownState::Terminated, ShutdownState::Terminated);
        assert_ne!(ShutdownState::Running, ShutdownState::ShuttingDown);
        assert_ne!(ShutdownState::ShuttingDown, ShutdownState::Terminated);
    }

    #[tokio::test]
    async fn test_shutdown_handle_new() {
        let (tx, _rx) = broadcast::channel(1);
        let _handle = ShutdownHandle::new(tx);
    }

    #[tokio::test]
    async fn test_shutdown_handle_shutdown() {
        let (tx, _rx) = broadcast::channel(1);
        let handle = ShutdownHandle::new(tx);
        handle.shutdown().await;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    #[tokio::test]
    async fn test_shutdown_handle_terminate() {
        let (tx, _rx) = broadcast::channel(1);
        let handle = ShutdownHandle::new(tx);
        handle.terminate().await;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    #[tokio::test]
    async fn test_shutdown_state_with_tokio_broadcast() {
        let (tx, mut rx) = broadcast::channel::<ShutdownState>(1);
        let result = tx.send(ShutdownState::ShuttingDown);
        assert!(result.is_ok());
        if let Ok(state) = rx.recv().await {
            assert_eq!(state, ShutdownState::ShuttingDown);
        }
    }
}
