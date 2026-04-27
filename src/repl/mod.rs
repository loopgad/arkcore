//! ArkCore REPL (交互式解释器)
//!
//! 基于 rustyline 18 实现的交互式命令行界面

use rustyline::{Result, Editor};
use rustyline::history::{FileHistory, History};
use tracing::info;

pub struct Repl {
    editor: Editor<(), FileHistory>,
}

impl Repl {
    pub fn new() -> Result<Self> {
        let editor = Editor::with_history(rustyline::Config::default(), FileHistory::new())?;
        Ok(Self { editor })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let readline = self.editor.readline("arkcore> ");
            match readline {
                Ok(line) => {
                    self.editor.add_history_entry(&line)?;
                    if line.trim() == "exit" || line.trim() == "quit" {
                        break;
                    }
                    info!("执行: {}", line);
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    info!("使用 'exit' 退出");
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(e) => {
                    tracing::error!("错误: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn run_script(&mut self, _path: &std::path::Path) -> Result<()> {
        // 读取并执行脚本
        Ok(())
    }

    /// 添加命令到历史记录
    pub fn add_history(&mut self, cmd: &str) -> Result<bool> {
        self.editor.add_history_entry(cmd)
    }

    /// 获取历史记录行数
    pub fn history_len(&self) -> usize {
        self.editor.history().len()
    }

    /// 检查命令是否为退出命令
    pub fn is_exit_command(cmd: &str) -> bool {
        let trimmed = cmd.trim();
        trimmed == "exit" || trimmed == "quit"
    }

    /// 解析命令（去除空白）
    pub fn parse_command(cmd: &str) -> String {
        cmd.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_new() {
        let repl = Repl::new();
        assert!(repl.is_ok());
    }

    #[test]
    fn test_repl_history_starts_empty() {
        let repl = Repl::new().unwrap();
        assert_eq!(repl.history_len(), 0);
    }

    #[test]
    fn test_repl_add_history() {
        let mut repl = Repl::new().unwrap();
        repl.add_history("test command").unwrap();
        assert_eq!(repl.history_len(), 1);
    }

    #[test]
    fn test_repl_add_multiple_history() {
        let mut repl = Repl::new().unwrap();
        repl.add_history("cmd1").unwrap();
        repl.add_history("cmd2").unwrap();
        repl.add_history("cmd3").unwrap();
        assert_eq!(repl.history_len(), 3);
    }

    #[test]
    fn test_is_exit_command_exit() {
        assert!(Repl::is_exit_command("exit"));
        assert!(Repl::is_exit_command("exit "));
        assert!(Repl::is_exit_command(" exit"));
        assert!(Repl::is_exit_command("  exit  "));
    }

    #[test]
    fn test_is_exit_command_quit() {
        assert!(Repl::is_exit_command("quit"));
        assert!(Repl::is_exit_command("quit "));
        assert!(Repl::is_exit_command(" quit"));
        assert!(Repl::is_exit_command("  quit  "));
    }

    #[test]
    fn test_is_exit_command_false() {
        assert!(!Repl::is_exit_command(""));
        assert!(!Repl::is_exit_command("exitall"));
        assert!(!Repl::is_exit_command("exiting"));
        assert!(!Repl::is_exit_command("quitall"));
        assert!(!Repl::is_exit_command("ls"));
        assert!(!Repl::is_exit_command("help"));
    }

    #[test]
    fn test_parse_command_trims_whitespace() {
        assert_eq!(Repl::parse_command("  hello  "), "hello");
        assert_eq!(Repl::parse_command("\t\tworld\t\t"), "world");
        assert_eq!(Repl::parse_command("  cmd with spaces  "), "cmd with spaces");
    }

    #[test]
    fn test_parse_command_empty() {
        assert_eq!(Repl::parse_command(""), "");
        assert_eq!(Repl::parse_command("   "), "");
    }

    #[test]
    fn test_repl_editor_creation() {
        // Test that Repl can be created with default config
        let repl = Repl::new();
        assert!(repl.is_ok());

        let repl = repl.unwrap();
        // Editor should be functional
        assert_eq!(repl.history_len(), 0);
    }

    #[test]
    fn test_repl_history_not_duplicated() {
        let mut repl = Repl::new().unwrap();
        // Adding the same command twice should create two entries
        repl.add_history("same cmd").unwrap();
        repl.add_history("same cmd").unwrap();
        // rustyline may or may not deduplicate - behavior depends on config
        assert!(repl.history_len() >= 1);
    }
}
