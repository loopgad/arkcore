//! 页面模块
//!
//! # 页面
//!
//! - [`Dashboard`] - 主仪表盘页面
//! - [`Settings`] - 设置页面

pub mod dashboard;
pub mod settings;

pub use dashboard::{Dashboard, DashboardProps, create_demo_dashboard};
pub use settings::{Settings, SettingsProps};
