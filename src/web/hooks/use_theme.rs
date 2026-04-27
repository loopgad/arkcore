//! 主题 Hook
//!
//! 管理主题状态和应用。

use super::super::theme::{Theme, ThemeProvider};

/// 主题 Hook 状态
#[derive(Debug, Clone)]
pub struct ThemeState {
    /// 当前主题
    pub current: Theme,
    /// 系统是否偏好暗色
    pub system_prefers_dark: bool,
}

/// 主题 Hook
///
/// # 示例
///
/// ```rust,ignore
/// let theme = UseTheme::new(Theme::Dark);
///
/// // 切换主题
/// theme.toggle();
///
/// // 应用主题到 DOM
/// theme.apply();
/// ```
#[derive(Debug, Clone)]
pub struct UseTheme {
    state: ThemeState,
}

impl ThemeProvider for UseTheme {
    fn get_theme(&self) -> Theme {
        self.state.current
    }

    fn set_theme(&mut self, theme: Theme) {
        self.state.current = theme;
    }

    fn toggle_theme(&mut self) {
        self.state.current = self.state.current.toggle();
    }
}

impl UseTheme {
    /// 创建新的主题 Hook
    pub fn new(initial: Theme) -> Self {
        Self {
            state: ThemeState {
                current: initial,
                system_prefers_dark: false, // 默认为 false，实际应该检测系统偏好
            },
        }
    }

    /// 检测系统主题偏好
    pub fn detect_system_preference(&mut self) {
        // 在 Web 环境中检测系统偏好
        // 这里只是一个占位实现
        #[cfg(target_arch = "wasm32")]
        {
            // Web 环境检测
            // use web_sys::window;
            // let prefers_dark = window()
            //     .and_then(|w| w.match_media("(prefers-color-scheme: dark)"))
            //     .ok()
            //     .and_then(|m| m.map(|media| media.matches()));
            // self.state.system_prefers_dark = prefers_dark.unwrap_or(false);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state.system_prefers_dark = false;
        }
    }

    /// 获取解析后的主题（考虑系统偏好）
    pub fn resolved_theme(&self) -> Theme {
        self.state.current.resolve(self.state.system_prefers_dark)
    }

    /// 获取状态引用
    pub fn state(&self) -> &ThemeState {
        &self.state
    }

    /// 获取状态可变引用
    pub fn state_mut(&mut self) -> &mut ThemeState {
        &mut self.state
    }

    /// 获取主题的 CSS 类名
    pub fn css_class(&self) -> &'static str {
        if self.resolved_theme().is_dark() {
            "theme-dark"
        } else {
            "theme-light"
        }
    }

    /// 应用主题到 DOM（Web 环境）
    #[cfg(target_arch = "wasm32")]
    pub fn apply(&self) {
        use web_sys::window;
        use super::super::theme::{dark_values, light_values};

        let window = match window() {
            Some(w) => w,
            None => return,
        };

        let document = match window.document() {
            Some(d) => d,
            None => return,
        };

        let css = match self.resolved_theme() {
            Theme::Dark => dark_values::get_css(),
            Theme::Light | Theme::System => light_values::get_css(),
        };

        // 设置或更新样式元素
        let style_id = "arkcore-theme-styles";
        if let Some(style) = document.get_element_by_id(style_id) {
            if let Some(html_style) = style.dyn_ref::<web_sys::HtmlStyleElement>() {
                let _ = html_style.set_text_content(Some(css));
            }
        } else {
            // 创建新样式元素
            let style = match document.create_element("style") {
                Ok(s) => s,
                Err(_) => return,
            };
            let _ = style.set_id(style_id);
            if let Some(html_style) = style.dyn_ref::<web_sys::HtmlStyleElement>() {
                let _ = html_style.set_text_content(Some(css));
            }
            if let Some(head) = document.head() {
                let _ = head.append_child(&style);
            }
        }

        // 设置 data-theme 属性
        let html = match document.document_element() {
            Some(el) => el,
            None => return,
        };
        let theme_name = self.resolved_theme().name();
        let _ = html.set_attribute("data-theme", theme_name);
    }

    /// 非 Web 环境的占位实现
    #[cfg(not(target_arch = "wasm32"))]
    pub fn apply(&self) {
        // 非 Web 环境不需要应用主题
    }
}

impl Default for UseTheme {
    fn default() -> Self {
        Self::new(Theme::System)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_theme() {
        let theme = UseTheme::new(Theme::Dark);
        assert_eq!(theme.get_theme(), Theme::Dark);
    }

    #[test]
    fn test_toggle_theme() {
        let mut theme = UseTheme::new(Theme::Dark);
        theme.toggle_theme();
        assert_eq!(theme.get_theme(), Theme::Light);
    }

    #[test]
    fn test_resolved_theme() {
        let mut theme = UseTheme::new(Theme::System);
        theme.state.system_prefers_dark = true;
        assert_eq!(theme.resolved_theme(), Theme::Dark);
    }
}
