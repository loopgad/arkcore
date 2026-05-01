//! 输出面板组件
//!
//! 显示命令执行结果的输出面板。

use super::super::{CommandOutput, OutputType};

/// 输出面板属性
#[derive(Debug, Clone)]
pub struct OutputPanelProps {
    /// 输出列表
    pub outputs: Vec<CommandOutput>,
    /// 最大行数
    pub max_lines: usize,
    /// 是否自动滚动到底部
    pub auto_scroll: bool,
}

/// 获取输出类型对应的类名
fn output_class(output_type: OutputType) -> &'static str {
    match output_type {
        OutputType::Stdout => "output-stdout",
        OutputType::Stderr => "output-stderr",
        OutputType::System => "output-system",
        OutputType::Result => "output-result",
    }
}

/// 获取输出类型对应的前缀
fn output_prefix(output_type: OutputType) -> &'static str {
    match output_type {
        OutputType::Stdout => "",
        OutputType::Stderr => "⚠",
        OutputType::System => "ℹ",
        OutputType::Result => "✓",
    }
}

/// 输出面板组件
#[derive(Debug, Clone)]
pub struct OutputPanel;

impl OutputPanel {
    /// 渲染输出面板
    pub fn render(props: &OutputPanelProps) -> String {
        let outputs_html: Vec<String> = props
            .outputs
            .iter()
            .rev()
            .take(props.max_lines)
            .map(|output| {
                let class = output_class(output.output_type);
                let prefix = output_prefix(output.output_type);
                let escaped_content = escape_html(&output.content);
                let timestamp = format_timestamp(output.timestamp);

                format!(
                    r#"<div class="output-line {}">
                        <span class="output-prefix">{}</span>
                        <span class="output-content">{}</span>
                        <span class="output-time">{}</span>
                    </div>"#,
                    class, prefix, escaped_content, timestamp
                )
            })
            .collect();

        let outputs_str = outputs_html.join("");
        let scroll_class = if props.auto_scroll { "auto-scroll" } else { "" };

        format!(
            r#"
            <div class="output-panel">
                <div class="output-header">
                    <span class="output-title">Output</span>
                    <div class="output-actions">
                        <button class="btn-clear-output" title="Clear">Clear</button>
                        <button class="btn-copy-output" title="Copy">Copy</button>
                    </div>
                </div>
                <div class="output-content {}">
                    {}
                </div>
            </div>
            "#,
            scroll_class, outputs_str
        )
    }
}

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 格式化时间戳
fn format_timestamp(timestamp: i64) -> String {
    // 简单的秒级时间戳格式化
    let total_secs = timestamp;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// 获取输出面板的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .output-panel {
        display: flex;
        flex-direction: column;
        background-color: var(--color-background);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        overflow: hidden;
        height: 100%;
        min-height: 200px;
    }
    .output-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: var(--spacing-sm) var(--spacing);
        background-color: var(--color-surface);
        border-bottom: 1px solid var(--color-border);
    }
    .output-title {
        font-weight: 600;
        color: var(--color-text-primary);
    }
    .output-actions {
        display: flex;
        gap: var(--spacing-sm);
    }
    .btn-clear-output, .btn-copy-output {
        background: transparent;
        border: 1px solid var(--color-border);
        color: var(--color-text-secondary);
        padding: var(--spacing-xs) var(--spacing-sm);
        border-radius: var(--border-radius-sm);
        cursor: pointer;
        font-size: var(--font-size-caption);
        transition: all var(--transition);
    }
    .btn-clear-output:hover {
        border-color: var(--color-error);
        color: var(--color-error);
    }
    .btn-copy-output:hover {
        border-color: var(--color-primary);
        color: var(--color-primary);
    }
    .output-content {
        flex: 1;
        overflow-y: auto;
        padding: var(--spacing);
        font-family: "JetBrains Mono", "Fira Code", monospace;
        font-size: var(--font-size-small);
        line-height: 1.6;
    }
    .auto-scroll {
        scroll-behavior: smooth;
    }
    .output-line {
        display: flex;
        gap: var(--spacing-sm);
        padding: var(--spacing-xs) 0;
        animation: slideIn var(--transition-fast) ease forwards;
    }
    .output-prefix {
        color: var(--color-text-secondary);
        min-width: 20px;
    }
    .output-content {
        flex: 1;
        white-space: pre-wrap;
        word-break: break-all;
        color: var(--color-text-primary);
    }
    .output-time {
        color: var(--color-text-secondary);
        font-size: var(--font-size-caption);
        margin-left: auto;
    }
    /* 输出类型样式 */
    .output-stderr .output-content {
        color: var(--color-error);
    }
    .output-stderr .output-prefix {
        color: var(--color-warning);
    }
    .output-system .output-content {
        color: var(--color-info);
    }
    .output-system .output-prefix {
        color: var(--color-info);
    }
    .output-result .output-content {
        color: var(--color-success);
    }
    .output-result .output-prefix {
        color: var(--color-success);
    }
    /* 自定义滚动条 */
    .output-content::-webkit-scrollbar {
        width: 8px;
    }
    .output-content::-webkit-scrollbar-track {
        background: var(--color-surface);
    }
    .output-content::-webkit-scrollbar-thumb {
        background: var(--color-border);
        border-radius: 4px;
    }
    .output-content::-webkit-scrollbar-thumb:hover {
        background: var(--color-text-secondary);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "00:00");
        assert_eq!(format_timestamp(65), "01:05");
        assert_eq!(format_timestamp(3665), "01:01:05");
    }
}
