//! Web UI 组件
//!
//! # 主要组件
//!
//! - [`StatusBar`] - 状态栏
//! - [`AgentCard`] - Agent 卡片
//! - [`CommandInput`] - 命令输入框
//! - [`OutputPanel`] - 输出面板
//! - [`ThemeToggle`] - 主题切换
//! - [`MetricsDisplay`] - 指标显示
//! - [`ConnectionIndicator`] - 连接指示器

pub mod agent_card;
pub mod command_input;
pub mod connection_indicator;
pub mod metrics_display;
pub mod output_panel;
pub mod status_bar;
pub mod theme_toggle;

pub use agent_card::{AgentCard, AgentCardProps};
pub use command_input::{highlight_syntax, CommandInput, CommandInputProps, SyntaxToken};
pub use connection_indicator::{ConnectionIndicator, ConnectionIndicatorProps};
pub use metrics_display::{MetricsDisplay, MetricsDisplayProps};
pub use output_panel::{OutputPanel, OutputPanelProps};
pub use status_bar::{StatusBar, StatusBarProps};
pub use theme_toggle::{ThemeToggle, ThemeToggleProps};

/// 组件通用的 CSS 样式
pub mod styles {
    /// 卡片样式
    pub const CARD_STYLE: &str = r#"
        .card {
            background-color: var(--color-surface);
            border: 1px solid var(--color-border);
            border-radius: var(--border-radius);
            padding: var(--spacing);
            transition: box-shadow var(--transition), transform var(--transition);
        }
        .card:hover {
            box-shadow: var(--shadow-lg);
            transform: translateY(-2px);
        }
    "#;

    /// 按钮样式
    pub const BUTTON_STYLE: &str = r#"
        .btn {
            background-color: var(--color-primary);
            color: white;
            border: none;
            border-radius: var(--border-radius-sm);
            padding: var(--spacing-sm) var(--spacing);
            cursor: pointer;
            font-size: var(--font-size-body);
            transition: background-color var(--transition), transform var(--transition-fast);
        }
        .btn:hover {
            filter: brightness(1.1);
        }
        .btn:active {
            transform: scale(0.98);
        }
        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
    "#;

    /// 输入框样式
    pub const INPUT_STYLE: &str = r#"
        .input {
            background-color: var(--color-surface);
            border: 1px solid var(--color-border);
            border-radius: var(--border-radius-sm);
            padding: var(--spacing-sm) var(--spacing);
            color: var(--color-text-primary);
            font-size: var(--font-size-body);
            width: 100%;
            transition: border-color var(--transition), box-shadow var(--transition);
        }
        .input:focus {
            outline: none;
            border-color: var(--color-primary);
            box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
        }
        .input::placeholder {
            color: var(--color-text-secondary);
        }
    "#;

    /// 动画关键帧
    pub const ANIMATIONS: &str = r#"
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(-10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        @keyframes slideIn {
            from { opacity: 0; transform: translateX(-20px); }
            to { opacity: 1; transform: translateX(0); }
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
        }
        .animate-fade-in {
            animation: fadeIn var(--transition-slow) ease forwards;
        }
        .animate-slide-in {
            animation: slideIn var(--transition-slow) ease forwards;
        }
        .animate-pulse {
            animation: pulse 2s ease-in-out infinite;
        }
        .animate-spin {
            animation: spin 1s linear infinite;
        }
    "#;
}
