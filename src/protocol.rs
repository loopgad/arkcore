//! 共享协议模块
//!
//! 定义前后端共用的消息类型，确保类型安全通信

use serde::{Deserialize, Serialize};

use crate::orchestrator::AgentState;

/// WebSocket 消息协议 - 前后端共用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsMessage {
    /// Agent 状态更新
    AgentStatus(AgentSnapshot),
    /// 系统指标更新
    Metrics(SystemMetrics),
    /// 命令输出
    Output(CommandOutput),
    /// 错误消息
    Error(String),
}

/// 客户端发送的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    /// 执行任务
    RunTask { task: String },
    /// 执行命令（带审批）
    RunCommand { task: String, command: String },
    /// 重置 Agent
    Reset,
    /// 请求系统指标
    GetMetrics,
}

/// Agent 快照 - 用于 WebSocket 传输
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    /// Agent ID
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// 当前状态标签
    pub state: AgentStateLabel,
    /// 当前任务描述
    pub current_task: Option<String>,
    /// 步骤进度
    pub step_progress: Option<StepProgress>,
}

/// Agent 状态标签 - 简化版，用于前端显示
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStateLabel {
    Idle,
    Planning,
    Executing,
    AwaitingApproval,
    Reflecting,
    Completed,
    Failed,
}

/// 步骤进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepProgress {
    /// 当前步骤
    pub current: usize,
    /// 总步骤数
    pub total: usize,
}

/// 系统指标 - 统一后端 HealthStatus 和前端 MetricsData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU 使用率
    pub cpu_usage: f32,
    /// 内存使用率
    pub memory_usage: f32,
    /// 磁盘使用率
    pub disk_usage: f32,
    /// 网络延迟（毫秒）
    pub latency_ms: u64,
    /// 健康级别
    pub health_level: HealthLevel,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 版本号
    pub version: String,
}

/// 健康级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
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

impl From<&AgentState> for AgentSnapshot {
    fn from(state: &AgentState) -> Self {
        match state {
            AgentState::Idle => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Idle,
                current_task: None,
                step_progress: None,
            },
            AgentState::Planning { task } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Planning,
                current_task: Some(task.clone()),
                step_progress: None,
            },
            AgentState::Executing { step, total } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Executing,
                current_task: None,
                step_progress: Some(StepProgress {
                    current: *step,
                    total: *total,
                }),
            },
            AgentState::AwaitingApproval {
                command,
                risk_level,
            } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::AwaitingApproval,
                current_task: Some(format!("{} [{}]", command, risk_level)),
                step_progress: None,
            },
            AgentState::Reflecting { assessment } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Reflecting,
                current_task: Some(assessment.clone()),
                step_progress: None,
            },
            AgentState::Completed { result } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Completed,
                current_task: Some(result.clone()),
                step_progress: None,
            },
            AgentState::Failed { reason } => AgentSnapshot {
                id: "agent-1".to_string(),
                name: "Main Agent".to_string(),
                state: AgentStateLabel::Failed,
                current_task: Some(reason.clone()),
                step_progress: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Error("test error".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains(r#""data":"test error""#));
    }

    #[test]
    fn test_agent_state_to_snapshot() {
        let state = AgentState::Planning {
            task: "test".to_string(),
        };
        let snapshot = AgentSnapshot::from(&state);
        assert_eq!(snapshot.state, AgentStateLabel::Planning);
        assert_eq!(snapshot.current_task, Some("test".to_string()));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"run_task","data":{"task":"test"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::RunTask { .. }));
    }
}
