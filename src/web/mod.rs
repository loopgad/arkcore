//! ArkCore Web UI 层
//!
//! 提供 Dioxus Web UI 支持，包括：
//! - 实时 Agent 状态展示
//! - WebSocket 实时通信
//! - 主题系统（暗/亮主题）
//! - 动画效果
//!
//! # 模块结构
//!
//! - [`theme`] - 主题系统
//! - [`components`] - UI 组件
//! - [`pages`] - 页面
//! - [`hooks`] - 自定义 Hooks
//! - [`styles`] - 样式

use serde::{Deserialize, Serialize};

pub mod theme;
pub mod components;
pub mod pages;
pub mod hooks;
pub mod styles;

// Re-export commonly used types
pub use theme::{Theme, ThemeProvider};
pub use components::{
    AgentCard, StatusBar, CommandInput, OutputPanel,
    ThemeToggle, MetricsDisplay, ConnectionIndicator,
};
pub use hooks::{use_websocket, use_theme, use_agent_state};

// Web UI 配置
pub mod config {
    use serde::{Deserialize, Serialize};

    /// Web UI 配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebUiConfig {
        /// WebSocket 服务器地址
        pub ws_url: String,
        /// 默认主题
        pub default_theme: super::theme::Theme,
        /// 启用动画
        pub animations_enabled: bool,
        /// 自动重连
        pub auto_reconnect: bool,
        /// 重连间隔（毫秒）
        pub reconnect_interval_ms: u64,
    }

    impl Default for WebUiConfig {
        fn default() -> Self {
            Self {
                ws_url: "ws://127.0.0.1:8080/ws".to_string(),
                default_theme: super::theme::Theme::Dark,
                animations_enabled: true,
                auto_reconnect: true,
                reconnect_interval_ms: 3000,
            }
        }
    }
}

/// Agent 状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// 空闲
    Idle,
    /// 运行中
    Running,
    /// 思考中
    Thinking,
    /// 等待输入
    Waiting,
    /// 错误
    Error,
    /// 已连接
    Connected,
    /// 已断开
    Disconnected,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Thinking => write!(f, "thinking"),
            AgentStatus::Waiting => write!(f, "waiting"),
            AgentStatus::Error => write!(f, "error"),
            AgentStatus::Connected => write!(f, "connected"),
            AgentStatus::Disconnected => write!(f, "disconnected"),
        }
    }
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// 当前状态
    pub status: AgentStatus,
    /// 当前任务描述
    pub current_task: Option<String>,
    /// 开始时间
    pub start_time: Option<i64>,
    /// CPU 使用率
    pub cpu_usage: f32,
    /// 内存使用率
    pub memory_usage: f32,
}

impl AgentInfo {
    /// 创建新的 Agent 信息
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            status: AgentStatus::Idle,
            current_task: None,
            start_time: None,
            cpu_usage: 0.0,
            memory_usage: 0.0,
        }
    }

    /// 更新状态
    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.status = status;
        self
    }

    /// 更新任务
    pub fn with_task(mut self, task: impl Into<String>) -> Self {
        self.current_task = Some(task.into());
        self
    }
}

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Agent 状态更新
    #[serde(rename = "agent_status")]
    AgentStatus(AgentInfo),
    /// 系统指标更新
    #[serde(rename = "metrics")]
    Metrics(MetricsData),
    /// 连接状态更新
    #[serde(rename = "connection")]
    Connection(ConnectionStatus),
    /// 命令输出
    #[serde(rename = "output")]
    Output(CommandOutput),
    /// 错误消息
    #[serde(rename = "error")]
    Error(String),
}

/// 系统指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// CPU 使用率
    pub cpu: f32,
    /// 内存使用率
    pub memory: f32,
    /// 磁盘使用率
    pub disk: f32,
    /// 网络延迟（毫秒）
    pub latency_ms: u64,
}

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
}

/// 命令输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// 输出内容
    pub content: String,
    /// 输出类型
    pub output_type: OutputType,
    /// 时间戳
    pub timestamp: i64,
}

/// 输出类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Stdout,
    Stderr,
    System,
    Result,
}
