//! ArkCore 引擎核心库
//!
//! Local-First OS Agent Engine
//!
//! # 模块
//!
//! - [`core`] - 核心抽象层（traits, config, error）
//! - [`services`] - 服务实现（Orchestrator, Sandbox, Memory）
//! - [`platform`] - 平台适配（Windows/Unix 差异）
//! - [`web`] - Web UI（预留）
//! - [`cli`] - 命令行解析
//! - [`repl`] - 交互式解释器
//! - [`server`] - Axum HTTP 服务器
//! - [`llm`] - LLM 提供者
//! - [`security`] - 安全审计（依赖审计、代码审计、渗透测试、OWASP/CIS）

pub mod cli;
pub mod core;
pub mod error;
pub mod llm;
pub mod memory;
pub mod orchestrator;
pub mod platform;
pub mod repl;
pub mod sandbox;
pub mod security;
pub mod server;
pub mod services;
pub mod web;

pub use error::Error;

// 重新导出常用类型（保持向后兼容）
pub use core::{Config, Container};
pub use services::{Orchestrator, Sandbox, SkillMemory};
