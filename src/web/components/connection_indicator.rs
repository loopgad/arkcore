//! 连接指示器组件
//!
//! 显示 WebSocket 连接状态。

use super::super::ConnectionStatus;

/// 连接指示器属性
#[derive(Debug, Clone)]
pub struct ConnectionIndicatorProps {
    /// 连接状态
    pub status: ConnectionStatus,
    /// 是否显示文字
    pub show_text: bool,
    /// 上次连接时间
    pub last_connected: Option<i64>,
}

/// 获取连接状态对应的图标
fn status_icon(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Connected => "🟢",
        ConnectionStatus::Disconnected => "🔴",
        ConnectionStatus::Reconnecting => "🟡",
    }
}

/// 获取连接状态对应的文字
fn status_text(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Disconnected => "Disconnected",
        ConnectionStatus::Reconnecting => "Reconnecting...",
    }
}

/// 连接指示器组件
#[derive(Debug, Clone)]
pub struct ConnectionIndicator;

impl ConnectionIndicator {
    /// 渲染连接指示器
    pub fn render(props: &ConnectionIndicatorProps) -> String {
        let icon = status_icon(props.status);
        let text = status_text(props.status);
        let status_class = match props.status {
            ConnectionStatus::Connected => "status-connected",
            ConnectionStatus::Disconnected => "status-disconnected",
            ConnectionStatus::Reconnecting => "status-reconnecting",
        };

        let text_html = if props.show_text {
            format!(
                r#"<span class="connection-text {}">{}</span>"#,
                status_class, text
            )
        } else {
            String::new()
        };

        let last_connected_html = if let Some(timestamp) = props.last_connected {
            let time_str = format_time_since(timestamp);
            format!(r#"<span class="last-connected">Last: {}</span>"#, time_str)
        } else {
            String::new()
        };

        format!(
            r#"
            <div class="connection-indicator {}">
                <span class="connection-icon {}">{}</span>
                {}
                {}
            </div>
            "#,
            status_class, status_class, icon, text_html, last_connected_html
        )
    }
}

/// 格式化时间差
fn format_time_since(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else {
        format!("{}h ago", diff / 3600)
    }
}

/// 获取连接指示器的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .connection-indicator {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        padding: var(--spacing-xs) var(--spacing-sm);
        border-radius: var(--border-radius-sm);
        font-size: var(--font-size-small);
        transition: all var(--transition);
    }
    .connection-icon {
        font-size: 0.75rem;
        transition: transform var(--transition);
    }
    .connection-indicator.status-connected {
        background-color: rgba(34, 197, 94, 0.1);
    }
    .connection-indicator.status-disconnected {
        background-color: rgba(239, 68, 68, 0.1);
    }
    .connection-indicator.status-reconnecting {
        background-color: rgba(245, 158, 11, 0.1);
    }
    .connection-indicator.status-reconnecting .connection-icon {
        animation: pulse 1s ease-in-out infinite;
    }
    .connection-text {
        font-weight: 500;
    }
    .connection-text.status-connected {
        color: var(--color-success);
    }
    .connection-text.status-disconnected {
        color: var(--color-error);
    }
    .connection-text.status-reconnecting {
        color: var(--color-warning);
    }
    .last-connected {
        color: var(--color-text-secondary);
        font-size: var(--font-size-caption);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_icon() {
        assert_eq!(status_icon(ConnectionStatus::Connected), "🟢");
        assert_eq!(status_icon(ConnectionStatus::Disconnected), "🔴");
        assert_eq!(status_icon(ConnectionStatus::Reconnecting), "🟡");
    }

    #[test]
    fn test_status_text() {
        assert_eq!(status_text(ConnectionStatus::Connected), "Connected");
        assert_eq!(status_text(ConnectionStatus::Disconnected), "Disconnected");
    }
}
