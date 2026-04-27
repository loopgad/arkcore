//! ArkCore CLI 模块
//!
//! 命令行接口和配置管理
//!
//! # 子模块
//!
//! - [`args`] - Clap 命令结构定义
//! - [`config`] - 配置文件管理

pub mod args;
pub mod config;

pub use args::{Cli, Commands};
pub use config::{load_config, save_config, set_api_key, show_config, Config, LlmConfig};
