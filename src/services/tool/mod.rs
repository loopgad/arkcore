//! 工具注册与调用系统
//!
//! 参考 LangChain 的 Tool Calling 机制设计的工具系统
//!
//! # 模块
//!
//! - [`trait`] - 工具接口定义
//! - [`registry`] - 工具注册表
//! - [`builtin`] - 内置工具实现

pub mod trait;
pub mod registry;
pub mod builtin;

// 重新导出常用类型
pub use trait::{Schema, Tool, ToolError, ToolResult};
pub use registry::{ToolInfo, ToolRegistry};
pub use builtin::{all_builtin_tools, Calculator, DateTime, FileRead, FileWrite, UtcDateTime};
