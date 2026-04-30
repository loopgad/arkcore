//! 跨平台命令抽象
//!
//! 提供统一的命令接口，适配 Unix/Windows 命令差异

use std::collections::HashMap;
use std::path::Path;

/// 命令提供者 trait
pub trait CommandProvider {
    /// 将 Unix 命令转换为平台特定命令
    fn translate(command: &str) -> CommandTranslation;

    /// 获取 shell 名称
    fn shell_name() -> &'static str;

    /// 获取 shell 参数
    fn shell_args() -> &'static [&'static str];

    /// 获取命令分隔符
    fn command_separator() -> &'static str;

    /// 获取环境变量分隔符
    fn path_separator() -> char;
}

/// Windows 命令实现
pub struct WindowsCommands;

impl CommandProvider for WindowsCommands {
    fn translate(command: &str) -> CommandTranslation {
        // Windows 命令映射
        let windows_mappings: HashMap<&str, &str> = [
            ("ls", "dir"),
            ("ll", "dir /Q"),
            ("la", "dir /A"),
            ("rm", "del /F"),
            ("rmdir", "rmdir /S /Q"),
            ("cp", "copy"),
            ("mv", "move"),
            ("cat", "type"),
            ("pwd", "cd"),
            ("echo", "echo"),
            ("clear", "cls"),
            ("which", "where"),
            ("ps", "tasklist"),
            ("kill", "taskkill /F /PID"),
            ("mkdir", "mkdir"),
            ("touch", "copy NUL"),
            ("chmod", "attrib"),
            ("ln", "mklink"),
            ("grep", "findstr"),
            ("find", "dir /S"),
            ("awk", "gawk"),
            ("sed", "gawk -F"),
            ("sort", "sort /R"),
            ("head", "more /P"),
            ("tail", "powershell -Command "),
            ("diff", "fc"),
            ("top", "tasklist /V"),
            ("df", "wmic logicaldisk get size,freespace,caption"),
            ("du", "dir /S"),
            ("env", "set"),
            ("export", "set"),
            ("sleep", "timeout /t"),
            ("wc", "find /C /V \"\""),
        ]
        .iter()
        .cloned()
        .collect();

        let mut result = command.to_string();
        for (unix_cmd, win_cmd) in &windows_mappings {
            if command.starts_with(*unix_cmd) && command.len() >= unix_cmd.len() {
                result = command.replacen(*unix_cmd, win_cmd, 1);
                break;
            }
        }

        // 路径转换 (/ -> \)
        if result.contains('/') && !result.contains("wmic") && !result.contains("powershell") {
            result = result.replace('/', "\\");
            // 处理 - 开头的参数中的路径
            let parts: Vec<&str> = result.split_whitespace().collect();
            result = parts
                .iter()
                .map(|p| {
                    if p.starts_with('-') || p.starts_with('/') {
                        // 参数保持原样
                        p.to_string()
                    } else if p.contains('\\') || p.starts_with('\\') || p.chars().next().map(|c| c.is_alphabetic() && p.len() > 1 && p.chars().nth(1) == Some(':')).unwrap_or(false) {
                        // Windows 路径保持原样
                        p.to_string()
                    } else if p.contains('/') {
                        // Unix 路径转换为 Windows
                        p.replace('/', "\\")
                    } else {
                        p.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }

        let was_translated = command != result;
        CommandTranslation {
            original: command.to_string(),
            translated: result,
            was_translated,
        }
    }

    fn shell_name() -> &'static str {
        "cmd"
    }

    fn shell_args() -> &'static [&'static str] {
        &["/C"]
    }

    fn command_separator() -> &'static str {
        " & "
    }

    fn path_separator() -> char {
        ';'
    }
}

/// Unix 命令实现
pub struct UnixCommands;

impl CommandProvider for UnixCommands {
    fn translate(command: &str) -> CommandTranslation {
        CommandTranslation {
            original: command.to_string(),
            translated: command.to_string(),
            was_translated: false,
        }
    }

    fn shell_name() -> &'static str {
        "sh"
    }

    fn shell_args() -> &'static [&'static str] {
        &["-c"]
    }

    fn command_separator() -> &'static str {
        " && "
    }

    fn path_separator() -> char {
        ':'
    }
}

/// 命令翻译结果
#[derive(Debug, Clone)]
pub struct CommandTranslation {
    /// 原始命令
    pub original: String,
    /// 翻译后命令
    pub translated: String,
    /// 是否经过翻译
    pub was_translated: bool,
}

impl CommandTranslation {
    /// 检查是否需要翻译
    pub fn needs_translation(&self) -> bool {
        self.was_translated
    }
}

/// 用户自定义命令映射
#[derive(Debug, Clone)]
pub struct CustomCommandMappings {
    mappings: HashMap<String, String>,
}

impl CustomCommandMappings {
    /// 创建新的自定义映射
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// 添加映射
    pub fn add_mapping(&mut self, unix_cmd: &str, platform_cmd: &str) {
        self.mappings
            .insert(unix_cmd.to_string(), platform_cmd.to_string());
    }

    /// 应用自定义映射到命令
    pub fn apply(&self, command: &str) -> String {
        let mut result = command.to_string();
        for (unix_cmd, platform_cmd) in &self.mappings {
            if command.starts_with(unix_cmd) {
                result = command.replacen(unix_cmd, platform_cmd, 1);
                break;
            }
        }
        result
    }

    /// 从文件加载映射
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut mappings = Self::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((unix, win)) = line.split_once('=') {
                mappings.add_mapping(unix.trim(), win.trim());
            }
        }
        Ok(mappings)
    }
}

impl Default for CustomCommandMappings {
    fn default() -> Self {
        Self::new()
    }
}

/// 根据当前平台选择命令实现
#[cfg(windows)]
pub type CurrentPlatformCommands = WindowsCommands;

#[cfg(not(windows))]
pub type CurrentPlatformCommands = UnixCommands;

/// 翻译命令到当前平台
pub fn translate_command(command: &str) -> CommandTranslation {
    CurrentPlatformCommands::translate(command)
}

/// 获取 shell 名称
pub fn shell_name() -> &'static str {
    CurrentPlatformCommands::shell_name()
}

/// 获取 shell 参数
pub fn shell_args() -> &'static [&'static str] {
    CurrentPlatformCommands::shell_args()
}

/// 获取命令分隔符
pub fn command_separator() -> &'static str {
    CurrentPlatformCommands::command_separator()
}

/// 获取环境变量分隔符
pub fn path_separator() -> char {
    CurrentPlatformCommands::path_separator()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_ls_translation() {
        #[cfg(windows)]
        {
            let result = translate_command("ls -la");
            assert!(result.translated.contains("dir"));
            assert!(result.needs_translation());
        }
    }

    #[test]
    fn test_unix_command_passthrough() {
        #[cfg(not(windows))]
        {
            let result = translate_command("ls -la");
            assert_eq!(result.original, result.translated);
            assert!(!result.needs_translation());
        }
    }

    #[test]
    fn test_custom_mappings() {
        let mut custom = CustomCommandMappings::new();
        custom.add_mapping("mycmd", "mycmd.exe");
        let result = custom.apply("mycmd arg1 arg2");
        assert_eq!(result, "mycmd.exe arg1 arg2");
    }

    #[test]
    fn test_path_separator() {
        #[cfg(windows)]
        assert_eq!(path_separator(), ';');
        #[cfg(not(windows))]
        assert_eq!(path_separator(), ':');
    }

    // ========== 新增测试 ==========

    #[test]
    fn test_command_translation_struct() {
        let translation = CommandTranslation {
            original: "ls -la".to_string(),
            translated: "dir".to_string(),
            was_translated: true,
        };

        assert!(translation.needs_translation());
        assert_eq!(translation.original, "ls -la");
        assert_eq!(translation.translated, "dir");
    }

    #[test]
    fn test_custom_mappings_apply_no_match() {
        let custom = CustomCommandMappings::new();
        let result = custom.apply("unknown_cmd arg");
        // 没有匹配的映射，应该返回原命令
        assert_eq!(result, "unknown_cmd arg");
    }

    #[test]
    fn test_custom_mappings_multiple() {
        let mut custom = CustomCommandMappings::new();
        custom.add_mapping("cmd1", "cmd1.exe");
        custom.add_mapping("cmd2", "cmd2.exe");

        // 只应用第一个匹配的映射
        let result = custom.apply("cmd1 arg");
        assert_eq!(result, "cmd1.exe arg");

        let result2 = custom.apply("cmd2 arg");
        assert_eq!(result2, "cmd2.exe arg");
    }

    #[test]
    fn test_custom_mappings_replace_all_occurrences() {
        let mut custom = CustomCommandMappings::new();
        custom.add_mapping("test", "REPLACED");

        // replacen 只替换第一个
        let result = custom.apply("test test test");
        assert_eq!(result, "REPLACED test test");
    }

    #[test]
    fn test_windows_commands_translate_rm() {
        #[cfg(windows)]
        {
            let result = translate_command("rm file.txt");
            assert!(result.translated.contains("del"));
            assert!(result.needs_translation());
        }
    }

    #[test]
    fn test_windows_commands_translate_cat() {
        #[cfg(windows)]
        {
            let result = translate_command("cat file.txt");
            assert!(result.translated.contains("type"));
            assert!(result.needs_translation());
        }
    }

    #[test]
    fn test_windows_commands_translate_cp() {
        #[cfg(windows)]
        {
            let result = translate_command("cp src dst");
            assert!(result.translated.contains("copy"));
            assert!(result.needs_translation());
        }
    }

    #[test]
    fn test_unix_commands_passthrough_all() {
        #[cfg(not(windows))]
        {
            // 所有命令都应该直接通过
            let cmds = ["ls", "cat", "grep", "awk", "sed", "sort"];
            for cmd in cmds {
                let result = translate_command(cmd);
                assert!(!result.needs_translation());
                assert_eq!(result.original, result.translated);
            }
        }
    }

    #[test]
    fn test_shell_name_and_args() {
        #[cfg(windows)]
        {
            assert_eq!(shell_name(), "cmd");
            assert_eq!(shell_args(), &["/C"]);
            assert_eq!(command_separator(), " & ");
        }
        #[cfg(not(windows))]
        {
            assert_eq!(shell_name(), "sh");
            assert_eq!(shell_args(), &["-c"]);
            assert_eq!(command_separator(), " && ");
        }
    }

    #[test]
    fn test_custom_mappings_default() {
        let custom = CustomCommandMappings::default();
        // 默认应该没有映射
        let result = custom.apply("any_cmd");
        assert_eq!(result, "any_cmd");
    }

    #[test]
    fn test_custom_mappings_empty_key() {
        let mut custom = CustomCommandMappings::new();
        custom.add_mapping("", "replacement");
        // 空键会匹配任何以该键开头的命令
        // 这是当前实现的行为：空键意味着"以空字符串开头" = 匹配所有
        let result = custom.apply("test");
        assert_eq!(result, "replacementtest");
    }

    #[test]
    fn test_translate_command_function() {
        let result = translate_command("echo hello");
        assert!(!result.original.is_empty());
        assert!(!result.translated.is_empty());
    }
}
