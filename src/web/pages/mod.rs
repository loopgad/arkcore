//! 页面模块
//!
//! # 页面
//!
//! - [`Dashboard`] - 主仪表盘页面
//! - [`Settings`] - 设置页面

pub mod dashboard;
pub mod settings;

pub use dashboard::{create_demo_dashboard, Dashboard, DashboardProps};
pub use settings::{Settings, SettingsProps};
