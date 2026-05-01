//! 主题切换组件
//!
//! 提供暗/亮主题切换功能。

use super::super::theme::Theme;

/// 主题切换属性
#[derive(Debug, Clone)]
pub struct ThemeToggleProps {
    /// 当前主题
    pub current_theme: Theme,
    /// 是否显示标签
    pub show_label: bool,
}

/// 主题切换组件
#[derive(Debug, Clone)]
pub struct ThemeToggle;

impl ThemeToggle {
    /// 渲染主题切换按钮
    pub fn render(props: &ThemeToggleProps) -> String {
        let (icon, label) = match props.current_theme {
            Theme::Dark => ("🌙", "Dark"),
            Theme::Light => ("☀️", "Light"),
            Theme::System => ("💻", "System"),
        };

        let label_html = if props.show_label {
            format!(r#"<span class="theme-label">{}</span>"#, label)
        } else {
            String::new()
        };

        format!(
            r#"
            <button class="theme-toggle" title="Toggle theme ({})">
                <span class="theme-icon">{}</span>
                {}
            </button>
            "#,
            label, icon, label_html
        )
    }
}

/// 获取主题切换的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .theme-toggle {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        padding: var(--spacing-sm) var(--spacing);
        cursor: pointer;
        color: var(--color-text-primary);
        font-size: var(--font-size-small);
        transition: all var(--transition);
    }
    .theme-toggle:hover {
        border-color: var(--color-primary);
        background-color: var(--color-surface-elevated);
    }
    .theme-toggle:active {
        transform: scale(0.98);
    }
    .theme-icon {
        font-size: 1.25rem;
        transition: transform var(--transition);
    }
    .theme-toggle:hover .theme-icon {
        transform: rotate(15deg);
    }
    .theme-label {
        font-weight: 500;
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_toggle_render() {
        let props = ThemeToggleProps {
            current_theme: Theme::Dark,
            show_label: true,
        };
        let html = ThemeToggle::render(&props);
        assert!(html.contains("🌙"));
        assert!(html.contains("Dark"));
    }
}
