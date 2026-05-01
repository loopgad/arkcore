//! 仪表盘页面
//!
//! 主页面，显示 Agent 状态、指标和输出。

use super::super::components::{
    AgentCard, CommandInput, CommandInputProps, ConnectionIndicator, ConnectionIndicatorProps,
    MetricsDisplay, OutputPanel, OutputPanelProps, StatusBar, ThemeToggle, ThemeToggleProps,
};
use super::super::{
    AgentInfo, AgentStatus, CommandOutput, ConnectionStatus, MetricsData, OutputType,
};

/// 仪表盘页面属性
#[derive(Debug, Clone)]
pub struct DashboardProps {
    /// 页面标题
    pub title: &'static str,
    /// 连接的 Agent 列表
    pub agents: Vec<AgentInfo>,
    /// 系统指标
    pub metrics: Option<MetricsData>,
    /// 连接状态
    pub connection_status: ConnectionStatus,
    /// 命令输出
    pub outputs: Vec<CommandOutput>,
    /// 当前主题
    pub theme: &'static str,
    /// 系统运行时间（秒）
    pub uptime_seconds: u64,
}

impl Default for DashboardProps {
    fn default() -> Self {
        Self {
            title: "ArkCore Dashboard",
            agents: Vec::new(),
            metrics: None,
            connection_status: ConnectionStatus::Disconnected,
            outputs: Vec::new(),
            theme: "dark",
            uptime_seconds: 0,
        }
    }
}

/// 创建演示用的仪表盘
pub fn create_demo_dashboard() -> DashboardProps {
    use chrono::Utc;

    let agents = vec![
        AgentInfo {
            id: "agent-1".into(),
            name: "Main Agent".into(),
            status: AgentStatus::Running,
            current_task: Some("Processing user request...".into()),
            start_time: Some(Utc::now().timestamp() - 300),
            cpu_usage: 45.5,
            memory_usage: 62.3,
        },
        AgentInfo {
            id: "agent-2".into(),
            name: "Worker Agent".into(),
            status: AgentStatus::Idle,
            current_task: None,
            start_time: None,
            cpu_usage: 10.0,
            memory_usage: 25.0,
        },
        AgentInfo {
            id: "agent-3".into(),
            name: "Monitor Agent".into(),
            status: AgentStatus::Thinking,
            current_task: Some("Analyzing system metrics".into()),
            start_time: Some(Utc::now().timestamp() - 600),
            cpu_usage: 30.0,
            memory_usage: 40.0,
        },
    ];

    let metrics = Some(MetricsData {
        cpu: 45.5,
        memory: 62.3,
        disk: 38.0,
        latency_ms: 25,
    });

    let outputs = vec![
        CommandOutput {
            content: "ArkCore v0.1.0 started".into(),
            output_type: OutputType::System,
            timestamp: Utc::now().timestamp() - 3600,
        },
        CommandOutput {
            content: "Connected to 3 agents".into(),
            output_type: OutputType::System,
            timestamp: Utc::now().timestamp() - 3500,
        },
        CommandOutput {
            content: "Processing: ls -la /home".into(),
            output_type: OutputType::Stdout,
            timestamp: Utc::now().timestamp() - 300,
        },
        CommandOutput {
            content: "drwxr-xr-x  20 user user 4096 Apr 25 10:00 /home".into(),
            output_type: OutputType::Result,
            timestamp: Utc::now().timestamp() - 299,
        },
    ];

    DashboardProps {
        agents,
        metrics,
        connection_status: ConnectionStatus::Connected,
        outputs,
        uptime_seconds: 3665,
        ..Default::default()
    }
}

/// 仪表盘页面
pub struct Dashboard;

impl Dashboard {
    /// 渲染仪表盘页面
    pub fn render(props: &DashboardProps) -> String {
        // 渲染状态栏
        let status_bar = StatusBar::render(&super::super::components::StatusBarProps {
            connected: props.connection_status == ConnectionStatus::Connected,
            agent_count: props.agents.len(),
            uptime_seconds: props.uptime_seconds,
            theme: props.theme,
        });

        // 渲染 Agent 卡片
        let agents_html: Vec<String> = props
            .agents
            .iter()
            .map(|agent| {
                AgentCard::render(&super::super::components::AgentCardProps {
                    agent: agent.clone(),
                    selected: false,
                })
            })
            .collect();
        let agents_grid = agents_html.join("");

        // 渲染指标
        let metrics_html = if let Some(ref metrics) = props.metrics {
            MetricsDisplay::render(&super::super::components::MetricsDisplayProps {
                metrics: metrics.clone(),
                detailed: false,
            })
        } else {
            String::new()
        };

        // 渲染输出面板
        let output_panel = OutputPanel::render(&OutputPanelProps {
            outputs: props.outputs.clone(),
            max_lines: 50,
            auto_scroll: true,
        });

        // 渲染连接指示器
        let connection = ConnectionIndicator::render(&ConnectionIndicatorProps {
            status: props.connection_status,
            show_text: true,
            last_connected: None,
        });

        // 渲染命令输入框
        let command_input = CommandInput::render(&CommandInputProps {
            enabled: props.connection_status == ConnectionStatus::Connected,
            placeholder: "Enter command...",
            history: vec![],
            completions: vec![
                "ls".into(),
                "cd".into(),
                "pwd".into(),
                "mkdir".into(),
                "rm".into(),
            ],
        });

        // 渲染主题切换
        let theme_toggle = ThemeToggle::render(&ThemeToggleProps {
            current_theme: if props.theme == "dark" {
                super::super::theme::Theme::Dark
            } else {
                super::super::theme::Theme::Light
            },
            show_label: false,
        });

        format!(
            r#"
            <div class="dashboard">
                <header class="dashboard-header">
                    {status_bar}
                </header>
                <main class="dashboard-main">
                    <aside class="dashboard-sidebar">
                        <div class="sidebar-section">
                            <h3 class="section-title">Agents</h3>
                            <div class="agents-grid">
                                {agents_grid}
                            </div>
                        </div>
                        <div class="sidebar-section">
                            <h3 class="section-title">System</h3>
                            {metrics_html}
                        </div>
                        <div class="sidebar-section">
                            {connection}
                        </div>
                    </aside>
                    <div class="dashboard-content">
                        <div class="content-main">
                            {output_panel}
                        </div>
                        <div class="content-input">
                            {command_input}
                        </div>
                    </div>
                </main>
                <footer class="dashboard-footer">
                    <div class="footer-left">
                        <span>ArkCore Dashboard</span>
                    </div>
                    <div class="footer-right">
                        {theme_toggle}
                    </div>
                </footer>
            </div>
            "#
        )
    }
}

/// 获取仪表盘的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .dashboard {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background-color: var(--color-background);
        color: var(--color-text-primary);
    }
    .dashboard-header {
        flex-shrink: 0;
    }
    .dashboard-main {
        display: flex;
        flex: 1;
        overflow: hidden;
    }
    .dashboard-sidebar {
        width: 320px;
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        gap: var(--spacing);
        padding: var(--spacing);
        background-color: var(--color-surface);
        border-right: 1px solid var(--color-border);
        overflow-y: auto;
    }
    .sidebar-section {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }
    .section-title {
        font-size: var(--font-size-small);
        font-weight: 600;
        color: var(--color-text-secondary);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        margin: 0;
    }
    .agents-grid {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }
    .dashboard-content {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }
    .content-main {
        flex: 1;
        overflow: hidden;
        padding: var(--spacing);
    }
    .content-input {
        flex-shrink: 0;
        padding: var(--spacing);
        border-top: 1px solid var(--color-border);
    }
    .dashboard-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: var(--spacing-sm) var(--spacing);
        background-color: var(--color-surface);
        border-top: 1px solid var(--color-border);
        font-size: var(--font-size-small);
    }
    .footer-left {
        color: var(--color-text-secondary);
    }
    .footer-right {
        display: flex;
        gap: var(--spacing);
    }
    /* 响应式布局 */
    @media (max-width: 768px) {
        .dashboard-main {
            flex-direction: column;
        }
        .dashboard-sidebar {
            width: 100%;
            border-right: none;
            border-bottom: 1px solid var(--color-border);
        }
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_demo_dashboard() {
        let dashboard = create_demo_dashboard();
        assert!(!dashboard.agents.is_empty());
        assert!(dashboard.metrics.is_some());
    }

    #[test]
    fn test_render_dashboard() {
        let dashboard = create_demo_dashboard();
        let html = Dashboard::render(&dashboard);
        assert!(html.contains("ArkCore"));
        assert!(html.contains("Main Agent"));
    }
}
