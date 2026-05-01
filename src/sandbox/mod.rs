//! ArkCore 零信任执行沙盒模块
//!
//! 六层安全检测架构:
//! 1. 危险参数检测 (-exec, -delete, -i 等)
//! 2. 空字节注入检测
//! 3. Shell 操作符检测 (|, ;, &, $, `)
//! 4. 路径遍历检测 (.., /proc/, /sys/)
//! 5. 环境变量注入检测 (LD_PRELOAD, DYLD_*)
//! 6. 危险内置命令检测 (eval, exec, source)
//!
//! 进程隔离架构:
//! - Linux: namespace 隔离 (PID, Network, Mount, UTS, IPC)
//! - Windows: Job Objects 隔离
//! - macOS: Sandbox
//!
//! # 示例
//!
//! ```rust,no_run
//! use arkcore::sandbox::Sandbox;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let sandbox = Sandbox::new()?;
//!
//!     // 安全命令
//!     let result = sandbox.execute("ls -la").await?;
//!     println!("{}", result.stdout);
//!
//!     Ok(())
//! }
//! ```

#[cfg(test)]
mod tests;

mod executor;
mod isolator;
mod truncator;

pub use executor::{ExecutionResult, Sandbox};
pub use isolator::{IsolationConfig, ProcessIsolator};
pub use truncator::{security_check, truncate_output, SecurityCheckResult};
