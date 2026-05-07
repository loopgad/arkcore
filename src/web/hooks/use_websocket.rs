//! WebSocket Hook
//!
//! 管理 WebSocket 连接和消息处理。

use super::super::{ConnectionStatus, WsMessage};

/// WebSocket 状态
#[derive(Debug, Clone)]
pub struct WebSocketState {
    /// 连接状态
    pub status: ConnectionStatus,
    /// 最后收到消息的时间戳
    pub last_message_at: Option<i64>,
    /// 错误消息
    pub error: Option<String>,
    /// 重连尝试次数
    pub reconnect_attempts: u32,
}

/// WebSocket Hook
///
/// # 示例
///
/// ```rust,ignore
/// let ws = UseWebSocket::new("ws://127.0.0.1:8080/ws");
///
/// // 在组件中使用
/// match ws.state.status {
///     ConnectionStatus::Connected => "🟢",
///     ConnectionStatus::Disconnected => "🔴",
///     ConnectionStatus::Reconnecting => "🟡",
/// }
/// ```
pub struct UseWebSocket {
    /// WebSocket URL
    url: String,
    /// 当前状态
    state: WebSocketState,
    /// 消息回调 - 使用稳定类型避免 Clone/Debug 约束
    #[allow(clippy::type_complexity)]
    message_handler: Option<Box<dyn Fn(WsMessage) + Send + Sync>>,
}

impl UseWebSocket {
    /// 创建新的 WebSocket Hook
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            state: WebSocketState {
                status: ConnectionStatus::Disconnected,
                last_message_at: None,
                error: None,
                reconnect_attempts: 0,
            },
            message_handler: None,
        }
    }

    /// 设置消息处理回调
    pub fn on_message<F>(mut self, handler: F) -> Self
    where
        F: Fn(WsMessage) + Send + Sync + 'static,
    {
        self.message_handler = Some(Box::new(handler));
        self
    }

    /// 获取当前状态
    pub fn state(&self) -> &WebSocketState {
        &self.state
    }

    /// 获取连接状态
    pub fn is_connected(&self) -> bool {
        self.state.status == ConnectionStatus::Connected
    }

    /// 获取 WebSocket URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 连接到 WebSocket 服务器
    pub fn connect(&mut self) {
        self.state.status = ConnectionStatus::Reconnecting;
        self.state.error = None;
        // 实际连接逻辑会在 WASM 环境中实现
        // 这里只是更新状态
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.state.status = ConnectionStatus::Disconnected;
        self.state.reconnect_attempts = 0;
    }

    /// 模拟连接成功（用于测试）
    pub fn simulate_connect(&mut self) {
        self.state.status = ConnectionStatus::Connected;
        self.state.last_message_at = Some(chrono::Utc::now().timestamp());
        self.state.error = None;
    }

    /// 模拟断开连接（用于测试）
    pub fn simulate_disconnect(&mut self) {
        self.state.status = ConnectionStatus::Disconnected;
    }

    /// 模拟重连中（用于测试）
    pub fn simulate_reconnecting(&mut self) {
        self.state.status = ConnectionStatus::Reconnecting;
        self.state.reconnect_attempts += 1;
    }

    /// 模拟收到消息（用于测试）
    pub fn simulate_message(&mut self, msg: WsMessage) {
        self.state.last_message_at = Some(chrono::Utc::now().timestamp());
        if let Some(ref handler) = self.message_handler {
            handler(msg);
        }
    }

    /// 处理收到的消息
    pub fn handle_message(&mut self, msg: WsMessage) {
        self.state.last_message_at = Some(chrono::Utc::now().timestamp());
        if let Some(ref handler) = self.message_handler {
            handler(msg);
        }
    }
}

impl Default for UseWebSocket {
    fn default() -> Self {
        // WebSocket URL should be configured via environment variable or config
        // Use ARKCORE_WS_URL environment variable if set, otherwise use localhost
        let url = std::env::var("ARKCORE_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string());
        Self::new(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_websocket() {
        let ws = UseWebSocket::new("ws://example.com/ws");
        assert_eq!(ws.url(), "ws://example.com/ws");
        assert!(!ws.is_connected());
    }

    #[test]
    fn test_simulate_connect() {
        let mut ws = UseWebSocket::default();
        ws.simulate_connect();
        assert!(ws.is_connected());
    }
}
