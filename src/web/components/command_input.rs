//! 命令输入框组件
//!
//! 支持语法高亮和自动补全的命令输入框。

/// 命令输入框属性
#[derive(Debug, Clone)]
pub struct CommandInputProps {
    /// 是否启用
    pub enabled: bool,
    /// 占位符文本
    pub placeholder: &'static str,
    /// 历史记录
    pub history: Vec<String>,
    /// 自动补全选项
    pub completions: Vec<String>,
}

/// 语法高亮标记
#[derive(Debug, Clone)]
pub enum SyntaxToken {
    /// 命令关键字
    Command(String),
    /// 选项
    Option(String),
    /// 字符串
    String(String),
    /// 注释
    Comment(String),
    /// 普通文本
    Normal(String),
}

/// 对命令进行语法高亮
pub fn highlight_syntax(input: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        // 检查是否以引号开头
        if remaining.starts_with('"') {
            if let Some(end) = remaining[1..].find('"') {
                tokens.push(SyntaxToken::String(remaining[..=end + 1].to_string()));
                remaining = &remaining[end + 2..];
                continue;
            }
        }

        // 检查注释
        if remaining.starts_with('#') || remaining.starts_with("//") {
            tokens.push(SyntaxToken::Comment(remaining.to_string()));
            break;
        }

        // 检查选项 (以 - 开头)
        if remaining.starts_with('-') {
            if let Some(end) = remaining.find(|c: char| c.is_whitespace()) {
                tokens.push(SyntaxToken::Option(remaining[..end].to_string()));
                remaining = &remaining[end..];
                continue;
            } else {
                tokens.push(SyntaxToken::Option(remaining.to_string()));
                break;
            }
        }

        // 找到下一个特殊字符
        let special_pos = remaining
            .find(|c: char| ['"', '-', '#', '/'].contains(&c))
            .unwrap_or(remaining.len());

        if special_pos == 0 {
            tokens.push(SyntaxToken::Normal(remaining[..1].to_string()));
            remaining = &remaining[1..];
        } else {
            tokens.push(SyntaxToken::Normal(remaining[..special_pos].to_string()));
            remaining = &remaining[special_pos..];
        }
    }

    tokens
}

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 命令输入框组件
#[derive(Debug, Clone)]
pub struct CommandInput;

impl CommandInput {
    /// 渲染命令输入框
    pub fn render(props: &CommandInputProps) -> String {
        let enabled_attr = if props.enabled { "" } else { "disabled" };
        let placeholder = props.placeholder;

        // 生成高亮的输入预览（示例）
        let _highlighted_preview = highlight_syntax("");

        format!(
            r#"
            <div class="command-input-container">
                <div class="command-input-wrapper">
                    <span class="command-prompt">❯</span>
                    <div class="command-input-content">
                        <input
                            type="text"
                            class="command-input {}"
                            placeholder="{}"
                            {}
                            autocomplete="off"
                            spellcheck="false"
                        />
                        <div class="syntax-preview"></div>
                    </div>
                    <div class="command-actions">
                        <button class="btn-clear" title="Clear">✕</button>
                        <button class="btn-submit" title="Execute">▶</button>
                    </div>
                </div>
                <div class="command-hints">
                    <span class="hint-item">Tab: 补全</span>
                    <span class="hint-item">↑↓: 历史</span>
                    <span class="hint-item">Ctrl+C: 取消</span>
                </div>
            </div>
            "#,
            if props.enabled {
                ""
            } else {
                "command-input-disabled"
            },
            placeholder,
            enabled_attr
        )
    }

    /// 获取语法高亮的 HTML
    pub fn render_highlighted(tokens: &[SyntaxToken]) -> String {
        let mut html = String::new();
        for token in tokens {
            match token {
                SyntaxToken::Command(s) => {
                    html.push_str(&format!(
                        r#"<span class="syntax-command">{}</span>"#,
                        escape_html(s)
                    ));
                }
                SyntaxToken::Option(s) => {
                    html.push_str(&format!(
                        r#"<span class="syntax-option">{}</span>"#,
                        escape_html(s)
                    ));
                }
                SyntaxToken::String(s) => {
                    html.push_str(&format!(
                        r#"<span class="syntax-string">{}</span>"#,
                        escape_html(s)
                    ));
                }
                SyntaxToken::Comment(s) => {
                    html.push_str(&format!(
                        r#"<span class="syntax-comment">{}</span>"#,
                        escape_html(s)
                    ));
                }
                SyntaxToken::Normal(s) => {
                    html.push_str(&format!(
                        r#"<span class="syntax-normal">{}</span>"#,
                        escape_html(s)
                    ));
                }
            }
        }
        html
    }
}

/// 获取命令输入框的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .command-input-container {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }
    .command-input-wrapper {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        padding: var(--spacing-sm) var(--spacing);
        transition: border-color var(--transition), box-shadow var(--transition);
    }
    .command-input-wrapper:focus-within {
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
    }
    .command-prompt {
        color: var(--color-primary);
        font-size: 1.25rem;
        font-weight: 700;
    }
    .command-input-content {
        flex: 1;
        position: relative;
    }
    .command-input {
        background: transparent;
        border: none;
        outline: none;
        color: var(--color-text-primary);
        font-family: "JetBrains Mono", "Fira Code", monospace;
        font-size: var(--font-size-body);
        width: 100%;
    }
    .command-input::placeholder {
        color: var(--color-text-secondary);
    }
    .command-input-disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    .syntax-preview {
        position: absolute;
        top: 0;
        left: 0;
        pointer-events: none;
        color: transparent;
        font-family: inherit;
        font-size: inherit;
        white-space: pre;
    }
    .command-actions {
        display: flex;
        gap: var(--spacing-xs);
    }
    .btn-clear, .btn-submit {
        background: transparent;
        border: none;
        cursor: pointer;
        padding: var(--spacing-xs);
        border-radius: var(--border-radius-sm);
        transition: background-color var(--transition);
    }
    .btn-clear {
        color: var(--color-text-secondary);
    }
    .btn-clear:hover {
        background-color: rgba(239, 68, 68, 0.1);
        color: var(--color-error);
    }
    .btn-submit {
        color: var(--color-success);
    }
    .btn-submit:hover {
        background-color: rgba(34, 197, 94, 0.1);
    }
    .command-hints {
        display: flex;
        gap: var(--spacing);
        padding-left: var(--spacing-lg);
    }
    .hint-item {
        font-size: var(--font-size-caption);
        color: var(--color-text-secondary);
    }
    /* 语法高亮样式 */
    .syntax-command {
        color: var(--color-primary);
        font-weight: 600;
    }
    .syntax-option {
        color: var(--color-accent);
    }
    .syntax-string {
        color: var(--color-success);
    }
    .syntax-comment {
        color: var(--color-text-secondary);
        font-style: italic;
    }
    .syntax-normal {
        color: var(--color-text-primary);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_syntax() {
        let tokens = highlight_syntax("ls -la # list files");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }
}
