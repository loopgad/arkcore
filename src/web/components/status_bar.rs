//! 状态栏组件
//!
//! 显示系统状态，包括连接状态、Agent 数量、运行时间等。

/// 状态栏属性
#[derive(Debug, Clone)]
pub struct StatusBarProps {
    /// 连接状态
    pub connected: bool,
    /// Agent 数量
    pub agent_count: usize,
    /// 系统运行时间（秒）
    pub uptime_seconds: u64,
    /// 当前主题
    pub theme: &'static str,
}

/// 格式化运行时间
pub fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// 状态栏组件
///
/// # 示例
///
/// ```rust,ignore
/// StatusBar {
///     connected: true,
///     agent_count: 3,
///     uptime_seconds: 3665,
///     theme: "dark",
/// }
/// ```
#[derive(Debug, Clone)]
pub struct StatusBar;

impl StatusBar {
    /// 渲染状态栏
    pub fn render(props: &StatusBarProps) -> String {
        let connection_status = if props.connected {
            "connected"
        } else {
            "disconnected"
        };
        let connection_class = if props.connected {
            "status-connected"
        } else {
            "status-disconnected"
        };

        format!(
            r#"
            <div class="status-bar">
                <div class="status-bar-left">
                    <span class="status-logo">🦀 ArkCore</span>
                    <span class="status-version">v0.1.0</span>
                </div>
                <div class="status-bar-center">
                    <span class="status-item">
                        <span class="status-label">Agents:</span>
                        <span class="status-value">{}</span>
                    </span>
                    <span class="status-item">
                        <span class="status-label">Uptime:</span>
                        <span class="status-value">{}</span>
                    </span>
                    <span class="status-item">
                        <span class="status-label">Theme:</span>
                        <span class="status-value">{}</span>
                    </span>
                </div>
                <div class="status-bar-right">
                    <span class="{}">{}</span>
                </div>
            </div>
            "#,
            props.agent_count,
            format_uptime(props.uptime_seconds),
            props.theme,
            connection_class,
            connection_status
        )
    }
}

/// 获取状态栏的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .status-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: var(--spacing-sm) var(--spacing);
        background-color: var(--color-surface);
        border-bottom: 1px solid var(--color-border);
        font-size: var(--font-size-small);
        height: 40px;
        box-sizing: border-box;
    }
    .status-bar-left, .status-bar-center, .status-bar-right {
        display: flex;
        align-items: center;
        gap: var(--spacing);
    }
    .status-logo {
        font-weight: 700;
        color: var(--color-primary);
    }
    .status-version {
        color: var(--color-text-secondary);
    }
    .status-item {
        display: flex;
        gap: var(--spacing-xs);
    }
    .status-label {
        color: var(--color-text-secondary);
    }
    .status-value {
        color: var(--color-text-primary);
        font-family: monospace;
    }
    .status-connected, .status-disconnected {
        padding: var(--spacing-xs) var(--spacing-sm);
        border-radius: var(--border-radius-sm);
        font-weight: 500;
        text-transform: uppercase;
        font-size: var(--font-size-caption);
    }
    .status-connected {
        background-color: rgba(34, 197, 94, 0.2);
        color: var(--color-success);
    }
    .status-disconnected {
        background-color: rgba(239, 68, 68, 0.2);
        color: var(--color-error);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "00:00");
        assert_eq!(format_uptime(59), "00:59");
        assert_eq!(format_uptime(60), "01:00");
        assert_eq!(format_uptime(3665), "01:01:05");
        assert_eq!(format_uptime(7325), "02:02:05");
    }
}
