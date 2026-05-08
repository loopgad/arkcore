//! ArkCore 引擎入口
//!
//! Local-First OS Agent Engine
//!
//! 支持三种运行模式:
//! - `repl` - 交互式终端模式
//! - `daemon` - 后台守护进程模式
//! - `config` - 配置管理

use anyhow::Result;
use arkcore::cli::Cli;
use arkcore::memory::SkillMemory;
use arkcore::orchestrator::Orchestrator;
use arkcore::repl::Repl;
use arkcore::sandbox::Sandbox;
use arkcore::server::{Server, ServerConfig};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志级别：--verbose 使用 debug，否则使用 info
    let log_level = if cli.verbose {
        "arkcore=debug"
    } else {
        "arkcore=info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        arkcore::cli::Commands::Repl { init } => {
            tracing::info!("启动 ArkCore 交互式终端...");
            let mut repl = Repl::new()?;
            if let Some(script) = init {
                repl.run_script(&script).await?;
            }
            repl.run().await?;
        }
        arkcore::cli::Commands::Daemon { port } => {
            tracing::info!("启动 ArkCore 后台服务于 127.0.0.1:{}", port);

            // 初始化组件
            let memory = SkillMemory::new().await?;
            let sandbox = Sandbox::new()?;
            let orchestrator = Orchestrator::new(memory, sandbox);
            let server = Server::new(orchestrator, ServerConfig::new(port));

            server.run().await?;
        }
        arkcore::cli::Commands::Config { set_api_key } => {
            if let Some(api_key) = set_api_key {
                arkcore::cli::config::set_api_key(&api_key)?;
                println!("✅ API Key 已保存");
            } else {
                arkcore::cli::config::show_config()?;
            }
        }
    }

    Ok(())
}
