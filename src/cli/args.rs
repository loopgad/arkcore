//! CLI 命令行参数解析
//!
//! 使用 clap derive 定义命令行接口

#[cfg(test)]
use clap::CommandFactory;
use clap::{Parser, Subcommand};

/// ArkCore 命令行工具
///
/// Local-First OS Agent Engine
#[derive(Parser)]
#[command(name = "arkcore")]
#[command(about = "ArkCore - Local-First OS Agent Engine")]
pub struct Cli {
    /// 启用详细输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

/// ArkCore 子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 启动交互式 REPL 终端
    Repl {
        /// 初始化脚本路径
        #[arg(short, long)]
        init: Option<std::path::PathBuf>,
    },
    /// 启动后台守护进程
    Daemon {
        /// 服务端口
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// 配置管理
    Config {
        /// 设置 API Key
        #[arg(short = 'k', long = "api-key")]
        set_api_key: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_repl_command_parsing() {
        let args = Cli::try_parse_from(["arkcore", "repl"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(matches!(cli.command, Commands::Repl { init: None }));
    }

    #[test]
    fn test_cli_repl_with_init_script() {
        let args = Cli::try_parse_from(["arkcore", "repl", "--init", "/path/to/script.rs"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Repl { init } => {
                assert!(init.is_some());
                assert_eq!(
                    init.unwrap(),
                    std::path::PathBuf::from("/path/to/script.rs")
                );
            }
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_daemon_command_parsing() {
        let args = Cli::try_parse_from(["arkcore", "daemon"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Daemon { port } => {
                assert_eq!(port, 8080); // default port
            }
            _ => panic!("Expected Daemon command"),
        }
    }

    #[test]
    fn test_cli_daemon_with_custom_port() {
        let args = Cli::try_parse_from(["arkcore", "daemon", "-p", "9090"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Daemon { port } => {
                assert_eq!(port, 9090);
            }
            _ => panic!("Expected Daemon command"),
        }
    }

    #[test]
    fn test_cli_daemon_with_long_port_flag() {
        let args = Cli::try_parse_from(["arkcore", "daemon", "--port", "3000"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Daemon { port } => {
                assert_eq!(port, 3000);
            }
            _ => panic!("Expected Daemon command"),
        }
    }

    #[test]
    fn test_cli_config_command_parsing() {
        let args = Cli::try_parse_from(["arkcore", "config"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Config { set_api_key } => {
                assert!(set_api_key.is_none());
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_config_with_api_key_short() {
        let args = Cli::try_parse_from(["arkcore", "config", "-k", "sk-test-key-123"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Config { set_api_key } => {
                assert!(set_api_key.is_some());
                assert_eq!(set_api_key.unwrap(), "sk-test-key-123");
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_config_with_api_key_long() {
        let args = Cli::try_parse_from(["arkcore", "config", "--api-key", "sk-another-key-456"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        match cli.command {
            Commands::Config { set_api_key } => {
                assert!(set_api_key.is_some());
                assert_eq!(set_api_key.unwrap(), "sk-another-key-456");
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_verbose_flag() {
        let args = Cli::try_parse_from(["arkcore", "-v", "repl"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_verbose_long_flag() {
        let args = Cli::try_parse_from(["arkcore", "--verbose", "daemon"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_no_verbose_flag() {
        let args = Cli::try_parse_from(["arkcore", "repl"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_repl_verbose_combined() {
        let args = Cli::try_parse_from(["arkcore", "-v", "repl", "--init", "/test/init.rs"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(cli.verbose);
        match cli.command {
            Commands::Repl { init } => {
                assert!(init.is_some());
            }
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_daemon_verbose_with_port() {
        let args = Cli::try_parse_from(["arkcore", "--verbose", "daemon", "-p", "8888"]);
        assert!(args.is_ok());
        let cli = args.unwrap();
        assert!(cli.verbose);
        match cli.command {
            Commands::Daemon { port } => {
                assert_eq!(port, 8888);
            }
            _ => panic!("Expected Daemon command"),
        }
    }

    #[test]
    fn test_cli_help_output() {
        let mut cmd = Cli::command();
        // Verify help text contains expected content
        let help_text = cmd.render_help().to_string();
        assert!(help_text.contains("ArkCore"));
        assert!(help_text.contains("Local-First OS Agent Engine"));
        assert!(help_text.contains("repl"));
        assert!(help_text.contains("daemon"));
        assert!(help_text.contains("config"));
    }
}
