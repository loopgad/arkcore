//! 主题系统
//!
//! 提供暗/亮主题支持，使用 CSS 变量实现主题切换。

use serde::{Deserialize, Serialize};

/// 主题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// 暗色主题
    Dark,
    /// 亮色主题
    Light,
    /// 系统默认
    #[default]
    System,
}

impl Theme {
    /// 获取当前应使用的实际主题
    pub fn resolve(&self, system_prefers_dark: bool) -> Theme {
        match self {
            Theme::System => {
                if system_prefers_dark {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            _ => *self,
        }
    }

    /// 切换到另一个主题
    pub fn toggle(self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
            Theme::System => Theme::System,
        }
    }

    /// 获取主题名称
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "system",
        }
    }

    /// 是否为暗色主题
    pub fn is_dark(&self) -> bool {
        matches!(self, Theme::Dark)
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 主题 Provider Trait
pub trait ThemeProvider {
    /// 获取当前主题
    fn get_theme(&self) -> Theme;
    /// 设置主题
    fn set_theme(&mut self, theme: Theme);
    /// 切换主题
    fn toggle_theme(&mut self);
}

/// CSS 变量名称
pub mod css_vars {
    /// 主色调
    pub const PRIMARY: &str = "--color-primary";
    /// 次要色
    pub const SECONDARY: &str = "--color-secondary";
    /// 强调色
    pub const ACCENT: &str = "--color-accent";
    /// 背景色
    pub const BACKGROUND: &str = "--color-background";
    /// 表面色
    pub const SURFACE: &str = "--color-surface";
    /// 文本主色
    pub const TEXT_PRIMARY: &str = "--color-text-primary";
    /// 文本次要色
    pub const TEXT_SECONDARY: &str = "--color-text-secondary";
    /// 边框色
    pub const BORDER: &str = "--color-border";
    /// 成功色
    pub const SUCCESS: &str = "--color-success";
    /// 警告色
    pub const WARNING: &str = "--color-warning";
    /// 错误色
    pub const ERROR: &str = "--color-error";
    /// 信息色
    pub const INFO: &str = "--color-info";

    /// 字体家族
    pub const FONT_FAMILY: &str = "--font-family";
    /// 标题字体大小
    pub const FONT_SIZE_H1: &str = "--font-size-h1";
    pub const FONT_SIZE_H2: &str = "--font-size-h2";
    pub const FONT_SIZE_H3: &str = "--font-size-h3";
    pub const FONT_SIZE_BODY: &str = "--font-size-body";
    pub const FONT_SIZE_SMALL: &str = "--font-size-small";
    pub const FONT_SIZE_CAPTION: &str = "--font-size-caption";

    /// 圆角
    pub const BORDER_RADIUS: &str = "--border-radius";
    pub const BORDER_RADIUS_SM: &str = "--border-radius-sm";
    pub const BORDER_RADIUS_LG: &str = "--border-radius-lg";

    /// 阴影
    pub const SHADOW: &str = "--shadow";
    pub const SHADOW_LG: &str = "--shadow-lg";

    /// 过渡时间
    pub const TRANSITION: &str = "--transition";
    pub const TRANSITION_FAST: &str = "--transition-fast";
    pub const TRANSITION_SLOW: &str = "--transition-slow";

    /// 间距
    pub const SPACING: &str = "--spacing";
    pub const SPACING_SM: &str = "--spacing-sm";
    pub const SPACING_LG: &str = "--spacing-lg";
    pub const SPACING_XL: &str = "--spacing-xl";
}

/// 暗色主题 CSS 变量值
pub mod dark_values {

    /// 获取暗色主题的 CSS 变量声明
    pub fn get_css() -> &'static str {
        r#"
        :root {
            /* 颜色 - 暗色主题 */
            --color-primary: #6366f1;
            --color-secondary: #8b5cf6;
            --color-accent: #f472b6;
            --color-background: #0f0f0f;
            --color-surface: #1a1a1a;
            --color-surface-elevated: #242424;
            --color-text-primary: #f5f5f5;
            --color-text-secondary: #a3a3a3;
            --color-border: #333333;
            --color-success: #22c55e;
            --color-warning: #f59e0b;
            --color-error: #ef4444;
            --color-info: #3b82f6;

            /* 字体 */
            --font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-size-h1: 2rem;
            --font-size-h2: 1.5rem;
            --font-size-h3: 1.25rem;
            --font-size-body: 1rem;
            --font-size-small: 0.875rem;
            --font-size-caption: 0.75rem;

            /* 圆角 */
            --border-radius: 8px;
            --border-radius-sm: 4px;
            --border-radius-lg: 12px;

            /* 阴影 */
            --shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
            --shadow-lg: 0 10px 25px rgba(0, 0, 0, 0.6);

            /* 过渡 */
            --transition: 0.2s ease;
            --transition-fast: 0.1s ease;
            --transition-slow: 0.4s ease;

            /* 间距 */
            --spacing-xs: 0.25rem;
            --spacing-sm: 0.5rem;
            --spacing: 1rem;
            --spacing-lg: 1.5rem;
            --spacing-xl: 2rem;
        }
        "#
    }
}

/// 亮色主题 CSS 变量值
pub mod light_values {
    /// 获取亮色主题的 CSS 变量声明
    pub fn get_css() -> &'static str {
        r#"
        :root {
            /* 颜色 - 亮色主题 */
            --color-primary: #6366f1;
            --color-secondary: #8b5cf6;
            --color-accent: #ec4899;
            --color-background: #ffffff;
            --color-surface: #f5f5f5;
            --color-surface-elevated: #ffffff;
            --color-text-primary: #171717;
            --color-text-secondary: #525252;
            --color-border: #e5e5e5;
            --color-success: #16a34a;
            --color-warning: #d97706;
            --color-error: #dc2626;
            --color-info: #2563eb;

            /* 字体 */
            --font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-size-h1: 2rem;
            --font-size-h2: 1.5rem;
            --font-size-h3: 1.25rem;
            --font-size-body: 1rem;
            --font-size-small: 0.875rem;
            --font-size-caption: 0.75rem;

            /* 圆角 */
            --border-radius: 8px;
            --border-radius-sm: 4px;
            --border-radius-lg: 12px;

            /* 阴影 */
            --shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            --shadow-lg: 0 10px 25px rgba(0, 0, 0, 0.15);

            /* 过渡 */
            --transition: 0.2s ease;
            --transition-fast: 0.1s ease;
            --transition-slow: 0.4s ease;

            /* 间距 */
            --spacing-xs: 0.25rem;
            --spacing-sm: 0.5rem;
            --spacing: 1rem;
            --spacing-lg: 1.5rem;
            --spacing-xl: 2rem;
        }
        "#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolve() {
        assert_eq!(Theme::Dark.resolve(false), Theme::Dark);
        assert_eq!(Theme::Light.resolve(true), Theme::Light);
        assert_eq!(Theme::System.resolve(true), Theme::Dark);
        assert_eq!(Theme::System.resolve(false), Theme::Light);
    }

    #[test]
    fn test_theme_toggle() {
        assert_eq!(Theme::Dark.toggle(), Theme::Light);
        assert_eq!(Theme::Light.toggle(), Theme::Dark);
        assert_eq!(Theme::System.toggle(), Theme::System);
    }
}
