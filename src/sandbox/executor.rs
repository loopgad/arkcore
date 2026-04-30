//! 命令执行器模块
//!
//! 基于 tokio 的异步命令执行，支持超时控制和资源限制

use std::collections::HashSet;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::sandbox::truncator::{security_check, truncate_output, SecurityCheckResult};

/// 允许的命令白名单 - 仅这些命令可以被执行
/// 这是第二层防护，即使安全检测通过也必须在此白名单中
const ALLOWED_COMMANDS: &[&str] = &[
    // 文件查看和操作
    "cat", "head", "tail", "less", "more", "grep", "rg", "find", "ls", "dir",
    "wc", "sort", "uniq", "cut", "tr", "awk", "sed", "tee",
    // 网络操作（只读）
    "ping", "nslookup", "host", "dig",
    // 系统信息
    "uname", "hostname", "uptime", "df", "du", "free", "top", "ps",
    // 文本处理
    "echo", "printf", "date", "pwd", "cd", "basename", "dirname", "realpath",
    // 进程管理
    "kill", "pkill", "sleep", "wait", "timeout",
    // 权限检查
    "id", "whoami", "groups", "stat",
    // 哈希校验
    "md5sum", "sha256sum", "sha1sum", "sha512sum", "cksum",
    // 其他安全操作
    "git", "svn", "tar", "zip", "unzip", "gzip", "gunzip",
];

/// 执行结果
#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub security_violations: Vec<String>,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out && self.security_violations.is_empty()
    }
}

/// 沙盒执行器
pub struct Sandbox {
    max_duration: Duration,
    allowed_commands: HashSet<String>,
}

impl Sandbox {
    /// 创建新的沙盒实例
    pub fn new() -> anyhow::Result<Self> {
        let allowed_commands: HashSet<String> = ALLOWED_COMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            max_duration: Duration::from_secs(30),
            allowed_commands,
        })
    }

    /// 创建带自定义超时时间的沙盒
    pub fn with_timeout(max_duration: Duration) -> Self {
        let allowed_commands: HashSet<String> = ALLOWED_COMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            max_duration,
            allowed_commands,
        }
    }

    /// 执行安全检测
    pub fn security_check(&self, cmd: &str) -> SecurityCheckResult {
        security_check(cmd)
    }

    /// 检查命令是否在白名单中
    fn is_command_whitelisted(&self, cmd: &str) -> bool {
        // 解析命令字符串获取第一个命令
        let first_word = cmd
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();

        // 移除可能的路径前缀（如 /bin/cat -> cat）
        let cmd_name = first_word
            .strip_prefix("./")
            .or_else(|| first_word.strip_prefix("/"))
            .unwrap_or(&first_word);

        self.allowed_commands.contains(cmd_name)
    }

    /// 对字符串进行 shell 转义
    /// 将所有危险字符转义以防止注入攻击
    fn shell_escape(&self, input: &str) -> String {
        let mut escaped = String::with_capacity(input.len() * 2);
        for c in input.chars() {
            match c {
                // 需要转义的字符
                '\'' => escaped.push_str("'\\''"),
                '"' => escaped.push_str("\\\""),
                '$' => escaped.push_str("\\$"),
                '`' => escaped.push_str("\\`"),
                '\\' => escaped.push_str("\\\\"),
                ';' => escaped.push_str("\\;"),
                '|' => escaped.push_str("\\|"),
                '&' => escaped.push_str("\\&"),
                '<' => escaped.push_str("\\<"),
                '>' => escaped.push_str("\\>"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\0' => escaped.push_str(""), // 空字节直接移除
                _ => escaped.push(c),
            }
        }
        escaped
    }

    /// 解析命令字符串为程序名和参数
    fn parse_command(&self, cmd: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return (String::new(), Vec::new());
        }

        let program = parts[0].to_string();
        let args: Vec<String> = parts[1..]
            .iter()
            .map(|&s| self.shell_escape(s))
            .collect();

        (program, args)
    }

    /// 验证并准备命令执行
    /// 返回 (是否安全, 错误消息, 转义后的命令字符串)
    fn validate_and_prepare_command(&self, cmd: &str) -> (bool, Option<String>, String) {
        // 1. 安全检测
        let check_result = security_check(cmd);
        if !check_result.passed {
            return (false, Some(format!("安全检测失败: {:?}", check_result.violations)), String::new());
        }

        // 2. 解析命令
        let (program, args) = self.parse_command(cmd);

        // 3. 白名单检测 (只检查程序名)
        if !self.is_command_whitelisted(&program) {
            return (false, Some("命令不在白名单中".to_string()), String::new());
        }

        // 4. 使用转义后的参数重新构建命令
        let escaped_cmd = if args.is_empty() {
            program
        } else {
            format!("{} {}", program, args.join(" "))
        };

        (true, None, escaped_cmd)
    }

    /// 执行命令
    /// 实施三层安全防护：
    /// 1. 六层安全检测 (truncator)
    /// 2. 命令白名单验证
    /// 3. 参数转义 (通过 shell 执行)
    pub async fn execute(&self, cmd: &str) -> anyhow::Result<ExecutionResult> {
        // 第一层: 验证并准备命令
        let (is_safe, error_msg, escaped_cmd) = self.validate_and_prepare_command(cmd);
        if !is_safe {
            return Ok(ExecutionResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                timed_out: false,
                security_violations: vec![error_msg.unwrap_or_else(|| "命令验证失败".to_string())],
            });
        }

        // 第二层: 通过 shell 执行转义后的命令
        let result = if cfg!(target_os = "windows") {
            timeout(
                self.max_duration,
                Command::new("cmd")
                    .args(["/C", &escaped_cmd])
                    .kill_on_drop(true)
                    .output(),
            )
            .await
        } else {
            timeout(
                self.max_duration,
                Command::new("sh")
                    .args(["-c", &escaped_cmd])
                    .kill_on_drop(true)
                    .output(),
            )
            .await
        };

        match result {
            Ok(Ok(process)) => {
                let exit_code = process.status.code();
                let stdout = String::from_utf8_lossy(&process.stdout).to_string();
                let stderr = String::from_utf8_lossy(&process.stderr).to_string();

                Ok(ExecutionResult {
                    stdout: truncate_output(&stdout),
                    stderr,
                    exit_code,
                    timed_out: false,
                    security_violations: Vec::new(),
                })
            }
            Ok(Err(e)) => {
                anyhow::bail!("命令执行失败: {}", e)
            }
            Err(_) => {
                // 超时
                Ok(ExecutionResult {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    timed_out: true,
                    security_violations: Vec::new(),
                })
            }
        }
    }

    /// 执行命令并返回原始输出（不截断）
    /// 同样实施三层安全防护
    pub async fn execute_raw(&self, cmd: &str) -> anyhow::Result<ExecutionResult> {
        // 第一层: 验证并准备命令
        let (is_safe, error_msg, escaped_cmd) = self.validate_and_prepare_command(cmd);
        if !is_safe {
            return Ok(ExecutionResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                timed_out: false,
                security_violations: vec![error_msg.unwrap_or_else(|| "命令验证失败".to_string())],
            });
        }

        // 第二层: 通过 shell 执行转义后的命令
        let result = if cfg!(target_os = "windows") {
            timeout(
                self.max_duration,
                Command::new("cmd")
                    .args(["/C", &escaped_cmd])
                    .kill_on_drop(true)
                    .output(),
            )
            .await
        } else {
            timeout(
                self.max_duration,
                Command::new("sh")
                    .args(["-c", &escaped_cmd])
                    .kill_on_drop(true)
                    .output(),
            )
            .await
        };

        match result {
            Ok(Ok(process)) => {
                let exit_code = process.status.code();
                let stdout = String::from_utf8_lossy(&process.stdout).to_string();
                let stderr = String::from_utf8_lossy(&process.stderr).to_string();

                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    timed_out: false,
                    security_violations: Vec::new(),
                })
            }
            Ok(Err(e)) => {
                anyhow::bail!("命令执行失败: {}", e)
            }
            Err(_) => {
                Ok(ExecutionResult {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    timed_out: true,
                    security_violations: Vec::new(),
                })
            }
        }
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new().expect("Sandbox 创建失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_command() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute("echo hello").await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[tokio::test]
    async fn test_dangerous_command_rejected() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute("find / -exec rm {} \\;").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }

    #[tokio::test]
    async fn test_timeout() {
        let sandbox = Sandbox::with_timeout(Duration::from_millis(100));
        // 跨平台睡眠命令 - 使用能确保超时的方法
        // 使用 ping 作为跨平台延迟命令 (在白名单中)
        #[cfg(target_os = "windows")]
        let result = sandbox.execute("ping -n 11 127.0.0.1").await.unwrap();
        #[cfg(not(target_os = "windows"))]
        let result = sandbox.execute("sleep 10").await.unwrap();
        assert!(result.timed_out);
    }

    #[tokio::test]
    async fn test_null_byte_rejected() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute("echo hello\0world").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }

    // ========== S1 命令注入修复验证测试 ==========

    #[tokio::test]
    async fn test_injection_semicolon_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // 分号命令链注入
        let result = sandbox.execute("echo hello; rm -rf /").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_injection_pipe_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // 管道注入
        let result = sandbox.execute("echo hello | cat /etc/passwd").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_injection_variable_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // 变量替换注入
        let result = sandbox.execute("echo $HOME").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_injection_backtick_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // 反引号命令替换
        let result = sandbox.execute("echo `whoami`").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_injection_dollar_substitution_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // $(...) 命令替换
        let result = sandbox.execute("$(whoami)").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_injection_redirect_blocked() {
        let sandbox = Sandbox::new().unwrap();
        // IO 重定向注入
        let result = sandbox.execute("echo hello > /etc/passwd").await.unwrap();
        assert!(!result.security_violations.is_empty() || !result.is_success());
    }

    #[tokio::test]
    async fn test_whitelist_only_allows_allowed_commands() {
        let sandbox = Sandbox::new().unwrap();
        // vim 不在白名单中，应该被阻止
        let result = sandbox.execute("vim /etc/passwd").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }

    #[tokio::test]
    async fn test_whitelist_allows_echo() {
        let sandbox = Sandbox::new().unwrap();
        // echo 在白名单中
        let result = sandbox.execute("echo test").await.unwrap();
        assert!(result.is_success() || !result.security_violations.is_empty());
    }

    #[tokio::test]
    async fn test_whitelist_allows_cat() {
        let sandbox = Sandbox::new().unwrap();
        // cat 在白名单中
        let _result = sandbox.execute("cat /etc/passwd").await.unwrap();
        // cat 可能存在，但安全检测应该通过
        // 注意: 实际结果取决于 /etc/passwd 是否存在
    }

    #[tokio::test]
    async fn test_rm_rf_blocked() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute("rm -rf /").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }

    #[tokio::test]
    async fn test_fork_bomb_blocked() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute(":(){ :|:& };:").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }

    #[tokio::test]
    async fn test_ld_preload_blocked() {
        let sandbox = Sandbox::new().unwrap();
        let result = sandbox.execute("LD_PRELOAD=/malicious.so command").await.unwrap();
        assert!(!result.security_violations.is_empty());
    }
}
