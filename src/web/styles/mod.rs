//! 样式模块
//!
//! 提供全局样式和 CSS 变量定义。

/// 全局样式
pub mod global {
    use super::super::components::styles as component_styles;
    use super::super::components::{
        agent_card, command_input, connection_indicator, metrics_display, output_panel, status_bar,
        theme_toggle,
    };
    use super::super::pages::{dashboard, settings};
    use super::super::theme::{dark_values, light_values};

    /// 获取完整的全局 CSS
    pub fn get_css(theme: super::super::theme::Theme) -> String {
        let theme_css = match theme {
            super::super::theme::Theme::Dark => dark_values::get_css(),
            super::super::theme::Theme::Light | super::super::theme::Theme::System => {
                light_values::get_css()
            }
        };

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            // 主题变量
            theme_css,
            // 组件样式
            component_styles::ANIMATIONS,
            component_styles::CARD_STYLE,
            component_styles::BUTTON_STYLE,
            component_styles::INPUT_STYLE,
            // 各组件样式
            status_bar::get_css(),
            agent_card::get_css(),
            command_input::get_css(),
            output_panel::get_css(),
            theme_toggle::get_css(),
            metrics_display::get_css(),
            connection_indicator::get_css(),
            // 页面样式
            dashboard::get_css(),
            settings::get_css(),
            // 全局重置样式
            get_reset_css(),
        )
    }

    /// 获取 CSS 重置样式
    fn get_reset_css() -> &'static str {
        r#"
        /* CSS Reset */
        *, *::before, *::after {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        html {
            font-size: 16px;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }
        body {
            font-family: var(--font-family);
            font-size: var(--font-size-body);
            line-height: 1.5;
            color: var(--color-text-primary);
            background-color: var(--color-background);
        }
        /* 链接样式 */
        a {
            color: var(--color-primary);
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
        /* 标题样式 */
        h1, h2, h3, h4, h5, h6 {
            font-weight: 600;
            line-height: 1.2;
        }
        h1 { font-size: var(--font-size-h1); }
        h2 { font-size: var(--font-size-h2); }
        h3 { font-size: var(--font-size-h3); }
        /* 按钮重置 */
        button {
            font-family: inherit;
            font-size: inherit;
            cursor: pointer;
        }
        /* 输入框重置 */
        input, textarea, select {
            font-family: inherit;
            font-size: inherit;
        }
        /* 图片 */
        img {
            max-width: 100%;
            height: auto;
        }
        /* 列表 */
        ul, ol {
            list-style: none;
        }
        /* 滚动条 */
        ::-webkit-scrollbar {
            width: 8px;
            height: 8px;
        }
        ::-webkit-scrollbar-track {
            background: var(--color-surface);
        }
        ::-webkit-scrollbar-thumb {
            background: var(--color-border);
            border-radius: 4px;
        }
        ::-webkit-scrollbar-thumb:hover {
            background: var(--color-text-secondary);
        }
        /* 选中文字颜色 */
        ::selection {
            background-color: var(--color-primary);
            color: white;
        }
        /* 焦点样式 */
        :focus-visible {
            outline: 2px solid var(--color-primary);
            outline-offset: 2px;
        }
        /* 禁用样式 */
        [disabled] {
            cursor: not-allowed;
            opacity: 0.6;
        }
        /* 隐藏元素 */
        [hidden] {
            display: none !important;
        }
        /* 浮动清理 */
        .clearfix::after {
            content: "";
            display: table;
            clear: both;
        }
        /* 屏幕阅读器专用 */
        .sr-only {
            position: absolute;
            width: 1px;
            height: 1px;
            padding: 0;
            margin: -1px;
            overflow: hidden;
            clip: rect(0, 0, 0, 0);
            white-space: nowrap;
            border: 0;
        }
        "#
    }
}

/// 动画类
pub mod animations {
    /// 淡入动画
    pub const FADE_IN: &str = r#"
    .animate-fade-in {
        animation: fadeIn var(--transition-slow) ease forwards;
    }
    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: translateY(-10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    "#;

    /// 滑入动画
    pub const SLIDE_IN: &str = r#"
    .animate-slide-in {
        animation: slideIn var(--transition-slow) ease forwards;
    }
    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(-20px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }
    "#;

    /// 脉冲动画
    pub const PULSE: &str = r#"
    .animate-pulse {
        animation: pulse 2s ease-in-out infinite;
    }
    @keyframes pulse {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }
    "#;

    /// 旋转动画
    pub const SPIN: &str = r#"
    .animate-spin {
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }
    "#;

    /// 缩放动画
    pub const SCALE: &str = r#"
    .animate-scale {
        animation: scale 0.2s ease forwards;
    }
    @keyframes scale {
        from {
            transform: scale(0.95);
            opacity: 0;
        }
        to {
            transform: scale(1);
            opacity: 1;
        }
    }
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_css_not_empty() {
        let css = global::get_css(super::super::theme::Theme::Dark);
        assert!(!css.is_empty());
        assert!(css.contains("--color-primary"));
    }

    #[test]
    fn test_light_theme_css() {
        let css = global::get_css(super::super::theme::Theme::Light);
        assert!(css.contains("--color-background"));
    }
}
