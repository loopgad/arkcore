//! 设置页面
//!
//! 应用设置页面。

use super::super::theme::Theme;

/// 设置页面属性
#[derive(Debug, Clone)]
pub struct SettingsProps {
    /// 当前主题
    pub theme: Theme,
    /// 启用动画
    pub animations_enabled: bool,
    /// 自动重连
    pub auto_reconnect: bool,
    /// 重连间隔（秒）
    pub reconnect_interval_secs: u64,
    /// WebSocket URL
    pub ws_url: String,
}

impl Default for SettingsProps {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            animations_enabled: true,
            auto_reconnect: true,
            reconnect_interval_secs: 3,
            ws_url: "ws://127.0.0.1:8080/ws".to_string(),
        }
    }
}

/// 设置页面
pub struct Settings;

impl Settings {
    /// 渲染设置页面
    pub fn render(props: &SettingsProps) -> String {
        let theme_selected = |t: Theme| -> &'static str {
            if props.theme == t {
                "selected"
            } else {
                ""
            }
        };

        let checkbox = |checked: bool| -> &'static str {
            if checked {
                "checked"
            } else {
                ""
            }
        };

        format!(
            r#"
            <div class="settings-page">
                <header class="settings-header">
                    <h1 class="settings-title">Settings</h1>
                </header>
                <main class="settings-content">
                    <section class="settings-section">
                        <h2 class="section-title">Appearance</h2>
                        <div class="setting-item">
                            <label class="setting-label">Theme</label>
                            <div class="theme-options">
                                <button class="theme-option {}" data-theme="light">
                                    ☀️ Light
                                </button>
                                <button class="theme-option {}" data-theme="dark">
                                    🌙 Dark
                                </button>
                                <button class="theme-option {}" data-theme="system">
                                    💻 System
                                </button>
                            </div>
                        </div>
                        <div class="setting-item">
                            <label class="setting-label">
                                <input type="checkbox" class="setting-checkbox" {} />
                                Enable animations
                            </label>
                            <p class="setting-description">
                                Smooth transitions and micro-interactions
                            </p>
                        </div>
                    </section>
                    <section class="settings-section">
                        <h2 class="section-title">Connection</h2>
                        <div class="setting-item">
                            <label class="setting-label">WebSocket URL</label>
                            <input
                                type="text"
                                class="setting-input"
                                value="{}"
                                placeholder="ws://127.0.0.1:8080/ws"
                            />
                        </div>
                        <div class="setting-item">
                            <label class="setting-label">
                                <input
                                    type="checkbox"
                                    class="setting-checkbox"
                                    {}
                                />
                                Auto-reconnect
                            </label>
                            <p class="setting-description">
                                Automatically reconnect when connection is lost
                            </p>
                        </div>
                        <div class="setting-item">
                            <label class="setting-label">
                                Reconnect interval (seconds)
                            </label>
                            <input
                                type="number"
                                class="setting-input setting-input-small"
                                value="{}"
                                min="1"
                                max="60"
                            />
                        </div>
                    </section>
                    <section class="settings-section">
                        <h2 class="section-title">About</h2>
                        <div class="about-info">
                            <p><strong>ArkCore</strong> v0.1.0</p>
                            <p>Local-First OS Agent Engine</p>
                            <p class="about-copyright">
                                Built with ❤️ using Rust + Dioxus
                            </p>
                        </div>
                    </section>
                </main>
                <footer class="settings-footer">
                    <button class="btn btn-primary">Save Changes</button>
                    <button class="btn btn-secondary">Reset to Defaults</button>
                </footer>
            </div>
            "#,
            theme_selected(Theme::Light),
            theme_selected(Theme::Dark),
            theme_selected(Theme::System),
            checkbox(props.animations_enabled),
            props.ws_url,
            checkbox(props.auto_reconnect),
            props.reconnect_interval_secs
        )
    }
}

/// 获取设置页面的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .settings-page {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background-color: var(--color-background);
        color: var(--color-text-primary);
    }
    .settings-header {
        padding: var(--spacing-lg) var(--spacing-xl);
        background-color: var(--color-surface);
        border-bottom: 1px solid var(--color-border);
    }
    .settings-title {
        margin: 0;
        font-size: var(--font-size-h1);
        font-weight: 700;
    }
    .settings-content {
        flex: 1;
        padding: var(--spacing-xl);
        max-width: 800px;
        margin: 0 auto;
        width: 100%;
    }
    .settings-section {
        margin-bottom: var(--spacing-xl);
        padding-bottom: var(--spacing-xl);
        border-bottom: 1px solid var(--color-border);
    }
    .settings-section:last-child {
        border-bottom: none;
    }
    .section-title {
        font-size: var(--font-size-h3);
        font-weight: 600;
        margin: 0 0 var(--spacing-lg) 0;
        color: var(--color-text-primary);
    }
    .setting-item {
        margin-bottom: var(--spacing-lg);
    }
    .setting-label {
        display: block;
        font-weight: 500;
        margin-bottom: var(--spacing-sm);
        color: var(--color-text-primary);
    }
    .setting-description {
        font-size: var(--font-size-small);
        color: var(--color-text-secondary);
        margin: var(--spacing-xs) 0 0 0;
    }
    .setting-input {
        width: 100%;
        padding: var(--spacing-sm) var(--spacing);
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius-sm);
        color: var(--color-text-primary);
        font-size: var(--font-size-body);
        transition: border-color var(--transition), box-shadow var(--transition);
    }
    .setting-input:focus {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
    }
    .setting-input-small {
        width: 100px;
    }
    .setting-checkbox {
        margin-right: var(--spacing-sm);
        accent-color: var(--color-primary);
    }
    .theme-options {
        display: flex;
        gap: var(--spacing-sm);
    }
    .theme-option {
        padding: var(--spacing-sm) var(--spacing-lg);
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        color: var(--color-text-primary);
        cursor: pointer;
        transition: all var(--transition);
    }
    .theme-option:hover {
        border-color: var(--color-primary);
    }
    .theme-option.selected {
        background-color: var(--color-primary);
        border-color: var(--color-primary);
        color: white;
    }
    .about-info {
        font-size: var(--font-size-body);
        line-height: 1.8;
    }
    .about-copyright {
        color: var(--color-text-secondary);
        font-size: var(--font-size-small);
    }
    .settings-footer {
        display: flex;
        gap: var(--spacing);
        padding: var(--spacing-lg) var(--spacing-xl);
        background-color: var(--color-surface);
        border-top: 1px solid var(--color-border);
    }
    .btn {
        padding: var(--spacing-sm) var(--spacing-lg);
        border-radius: var(--border-radius);
        font-size: var(--font-size-body);
        font-weight: 500;
        cursor: pointer;
        transition: all var(--transition);
    }
    .btn-primary {
        background-color: var(--color-primary);
        border: 1px solid var(--color-primary);
        color: white;
    }
    .btn-primary:hover {
        filter: brightness(1.1);
    }
    .btn-secondary {
        background-color: transparent;
        border: 1px solid var(--color-border);
        color: var(--color-text-primary);
    }
    .btn-secondary:hover {
        border-color: var(--color-text-secondary);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_settings() {
        let settings = SettingsProps::default();
        let html = Settings::render(&settings);
        assert!(html.contains("Settings"));
        assert!(html.contains("Theme"));
        assert!(html.contains("WebSocket"));
    }
}
