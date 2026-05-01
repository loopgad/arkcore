//! Agent 卡片组件
//!
//! 显示单个 Agent 的状态信息。

use super::super::{AgentInfo, AgentStatus};

/// Agent 卡片属性
#[derive(Debug, Clone)]
pub struct AgentCardProps {
    /// Agent 信息
    pub agent: AgentInfo,
    /// 是否选中
    pub selected: bool,
}

/// 获取状态对应的图标
fn status_icon(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Idle => "💤",
        AgentStatus::Running => "⚡",
        AgentStatus::Thinking => "🤔",
        AgentStatus::Waiting => "⏳",
        AgentStatus::Error => "❌",
        AgentStatus::Connected => "🔗",
        AgentStatus::Disconnected => "🔌",
    }
}

/// 获取状态对应的类名
fn status_class(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Idle => "agent-status-idle",
        AgentStatus::Running => "agent-status-running",
        AgentStatus::Thinking => "agent-status-thinking",
        AgentStatus::Waiting => "agent-status-waiting",
        AgentStatus::Error => "agent-status-error",
        AgentStatus::Connected => "agent-status-connected",
        AgentStatus::Disconnected => "agent-status-disconnected",
    }
}

/// Agent 卡片组件
#[derive(Debug, Clone)]
pub struct AgentCard;

impl AgentCard {
    /// 渲染 Agent 卡片
    pub fn render(props: &AgentCardProps) -> String {
        let status_icon = status_icon(props.agent.status);
        let status_class = status_class(props.agent.status);
        let selected_class = if props.selected {
            "agent-card-selected"
        } else {
            ""
        };

        format!(
            r#"
            <div class="agent-card {}">
                <div class="agent-card-header">
                    <span class="agent-avatar">🤖</span>
                    <div class="agent-info">
                        <span class="agent-name">{}</span>
                        <span class="agent-id">{}</span>
                    </div>
                    <span class="agent-status-icon {}">{}</span>
                </div>
                <div class="agent-card-body">
                    <div class="agent-task">
                        <span class="task-label">Task:</span>
                        <span class="task-value">{}</span>
                    </div>
                    <div class="agent-metrics">
                        <div class="metric">
                            <span class="metric-label">CPU</span>
                            <div class="metric-bar">
                                <div class="metric-fill" style="width: {}%"></div>
                            </div>
                            <span class="metric-value">{}%</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">MEM</span>
                            <div class="metric-bar">
                                <div class="metric-fill" style="width: {}%"></div>
                            </div>
                            <span class="metric-value">{}%</span>
                        </div>
                    </div>
                </div>
            </div>
            "#,
            selected_class,
            props.agent.name,
            props.agent.id,
            status_class,
            status_icon,
            props
                .agent
                .current_task
                .as_deref()
                .unwrap_or("No active task"),
            props.agent.cpu_usage as i32,
            props.agent.cpu_usage,
            props.agent.memory_usage as i32,
            props.agent.memory_usage
        )
    }
}

/// 获取 Agent 卡片的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .agent-card {
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        padding: var(--spacing);
        cursor: pointer;
        transition: all var(--transition);
        animation: fadeIn var(--transition-slow) ease forwards;
    }
    .agent-card:hover {
        border-color: var(--color-primary);
        box-shadow: var(--shadow);
    }
    .agent-card-selected {
        border-color: var(--color-primary);
        background-color: var(--color-surface-elevated);
        box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.3);
    }
    .agent-card-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        margin-bottom: var(--spacing);
    }
    .agent-avatar {
        font-size: 1.5rem;
    }
    .agent-info {
        flex: 1;
        display: flex;
        flex-direction: column;
    }
    .agent-name {
        font-weight: 600;
        color: var(--color-text-primary);
    }
    .agent-id {
        font-size: var(--font-size-caption);
        color: var(--color-text-secondary);
        font-family: monospace;
    }
    .agent-status-icon {
        font-size: 1.25rem;
    }
    .agent-status-running {
        animation: pulse 1.5s ease-in-out infinite;
    }
    .agent-status-thinking {
        animation: pulse 2s ease-in-out infinite;
    }
    .agent-card-body {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }
    .agent-task {
        font-size: var(--font-size-small);
        color: var(--color-text-secondary);
    }
    .task-label {
        font-weight: 500;
    }
    .task-value {
        color: var(--color-text-primary);
    }
    .agent-metrics {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }
    .metric {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
    }
    .metric-label {
        font-size: var(--font-size-caption);
        color: var(--color-text-secondary);
        width: 30px;
    }
    .metric-bar {
        flex: 1;
        height: 4px;
        background-color: var(--color-border);
        border-radius: 2px;
        overflow: hidden;
    }
    .metric-fill {
        height: 100%;
        background-color: var(--color-primary);
        transition: width var(--transition);
    }
    .metric-value {
        font-size: var(--font-size-caption);
        color: var(--color-text-secondary);
        width: 40px;
        text-align: right;
        font-family: monospace;
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_icon() {
        assert_eq!(status_icon(AgentStatus::Idle), "💤");
        assert_eq!(status_icon(AgentStatus::Running), "⚡");
        assert_eq!(status_icon(AgentStatus::Thinking), "🤔");
    }

    #[test]
    fn test_status_class() {
        assert_eq!(status_class(AgentStatus::Idle), "agent-status-idle");
        assert_eq!(status_class(AgentStatus::Running), "agent-status-running");
    }
}
