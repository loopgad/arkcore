//! 零信任安全检测与输出截断模块
//!
//! 实现六层安全检测机制，确保命令执行安全可控

/// 危险参数列表 - find/-exec/-delete 等高危操作（预转换为小写）
const DANGEROUS_ARGS: &[&str] = &[
    "-exec",
    "-execdir",
    "-ok",
    "-okdir",
    "-delete",
    "--to-command",
    "--replace",
    "-i",
    "--in-place",
    "-rf",
    "-r",
    "--no-preserve-root",
    "--one-file-system",
];

/// 危险环境变量列表
const DANGEROUS_ENV_VARS: &[&str] = &[
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "BASH_ENV",
    "ENV",
    "CDPATH",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "DYLD_FRAMEWORK_PATH",
    "DYLD_VERSIONED_LIBRARY_PATH",
    "DYLD_VERSIONED_FRAMEWORK_PATH",
    "DYLD_IMAGE_SUFFIX",
    "DYLD_INSERT_LIBRARIES_DYLIB",
    "DYLD_FORCE_FLAT_NAMESPACE",
    "DYLD_PRINT_OPTS",
    "DYLD_PRINT_ENV",
    "PATH",
    "HOME",
    "USER",
    "SHELL",
    "TERM",
    "LD_DEBUG",
    "LD_PROFILE",
];

/// 危险 shell 内置命令
const DANGEROUS_BUILTINS: &[&str] = &[
    "eval", "exec", "source", "alias", "export", "declare", "typeset", "local", "readonly",
];

/// 最大输出大小 (1KB)
const MAX_OUTPUT_SIZE: usize = 1000;

/// 安全检测结果
#[derive(Debug, Clone)]
pub struct SecurityCheckResult {
    pub passed: bool,
    pub violations: Vec<String>,
}

impl SecurityCheckResult {
    pub fn safe() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
        }
    }

    pub fn unsafe_result(violations: Vec<String>) -> Self {
        Self {
            passed: false,
            violations,
        }
    }
}

/// 第一层检测: 危险参数检测
fn check_dangerous_args(input: &str) -> Option<String> {
    // 预转换为小写（单次分配），避免循环内重复分配
    let input_lower = input.to_lowercase();
    for arg in DANGEROUS_ARGS {
        if input_lower.contains(arg) {
            return Some(format!("危险参数: {}", arg));
        }
    }
    None
}

/// 第二层检测: 空字节剥离验证
fn check_null_bytes(input: &str) -> Option<String> {
    if input.contains('\0') {
        return Some("检测到空字节注入".to_string());
    }
    None
}

/// 剥离字符串中的空字节
#[allow(dead_code)]
pub fn strip_null_bytes(input: &str) -> String {
    input.replace('\0', "")
}

/// 检查是否包含危险 shell 操作符
#[allow(dead_code)]
pub fn contains_shell_operators(input: &str) -> bool {
    check_shell_operators(input).is_some()
}

/// 第三层检测: Shell 操作符检测
fn check_shell_operators(input: &str) -> Option<String> {
    let dangerous_chars = ['|', ';', '&', '$', '`', '<', '>'];
    for c in dangerous_chars {
        if input.contains(c) {
            // 允许在引号内或特定上下文中的情况
            if !is_safe_in_context(input, c) {
                return Some(format!("检测到危险 shell 操作符: {}", c));
            }
        }
    }
    None
}

/// 检查字符在特定上下文中是否安全
fn is_safe_in_context(input: &str, ch: char) -> bool {
    // 检查是否在引号内
    let in_single_quote = input.chars().filter(|&c| c == '\'').count() >= 2;
    let in_double_quote = input.chars().filter(|&c| c == '"').count() >= 2;

    // 简单的启发式: 如果操作符在引号对之间，认为可能是安全的字符串
    match ch {
        '|' | ';' | '&' => {
            // 这些在命令上下文中通常是危险的
            if ch == '|' && input.contains("||") {
                return false; // 或运算
            }
            if ch == ';' && !in_single_quote && !in_double_quote {
                return false; // 分号命令链
            }
        }
        '$' if !in_double_quote && !in_single_quote => {
            // $ 在引号外可能是变量替换
            return false;
        }
        '`' => {
            return false; // 反引号命令替换始终危险
        }
        '<' | '>' if !in_single_quote && !in_double_quote => {
            // IO 重定向在引号外是危险的
            return false;
        }
        _ => {}
    }
    false
}

/// 第四层检测: 路径遍历检测
fn check_path_traversal(input: &str) -> Option<String> {
    if input.contains("..") {
        return Some("检测到路径遍历模式 (..)".to_string());
    }
    // 检测 symlink 相关的危险路径
    if input.contains("/proc/") || input.contains("/sys/") {
        return Some("检测到危险系统路径".to_string());
    }
    None
}

/// 第五层检测: 环境变量注入检测
fn check_env_injection(input: &str) -> Option<String> {
    for env in DANGEROUS_ENV_VARS {
        if input.contains(env) {
            return Some(format!("检测到危险环境变量: {}", env));
        }
    }
    // 检测变量赋值模式
    if input.contains("ENV=") || input.contains("BASH_ENV=") {
        return Some("检测到环境变量注入尝试".to_string());
    }
    None
}

/// 第六层检测: 危险内置命令检测
fn check_dangerous_builtins(input: &str) -> Option<String> {
    // 预转换为小写（单次分配），避免循环内重复分配
    let input_lower = input.to_lowercase();
    for builtin in DANGEROUS_BUILTINS {
        // 使用切片比较避免循环内 format! 分配
        if let Some(rest) = input_lower.strip_prefix(builtin) {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('=') {
                return Some(format!("检测到危险内置命令: {}", builtin));
            }
        }
    }
    None
}

/// 执行六层安全检测
pub fn security_check(input: &str) -> SecurityCheckResult {
    let mut violations = Vec::new();

    // 第一层: 危险参数
    if let Some(v) = check_dangerous_args(input) {
        violations.push(v);
    }

    // 第二层: 空字节
    if let Some(v) = check_null_bytes(input) {
        violations.push(v);
    }

    // 第三层: Shell 操作符
    if let Some(v) = check_shell_operators(input) {
        violations.push(v);
    }

    // 第四层: 路径遍历
    if let Some(v) = check_path_traversal(input) {
        violations.push(v);
    }

    // 第五层: 环境变量注入
    if let Some(v) = check_env_injection(input) {
        violations.push(v);
    }

    // 第六层: 危险内置命令
    if let Some(v) = check_dangerous_builtins(input) {
        violations.push(v);
    }

    if violations.is_empty() {
        SecurityCheckResult::safe()
    } else {
        SecurityCheckResult::unsafe_result(violations)
    }
}

/// 截断输出，保留头部和尾部
pub fn truncate_preserve_head_tail(output: &str, max_bytes: usize) -> String {
    let output_bytes = output.as_bytes();
    if output_bytes.len() <= max_bytes {
        return output.to_string();
    }

    let head_size = max_bytes / 2;

    // 安全地获取 UTF-8 边界
    let head_end = find_utf8_boundary(output, head_size);
    let tail_start = find_utf8_boundary_from_end(output, head_size);

    let truncated_middle = output_bytes.len() - max_bytes;

    format!(
        "{}...\n[truncated {} bytes]...\n{}",
        &output[..head_end],
        truncated_middle,
        &output[tail_start..]
    )
}

/// 找到 UTF-8 字符边界（从开头）
fn find_utf8_boundary(s: &str, max_bytes: usize) -> usize {
    let bytes = s.as_bytes();
    if bytes.len() <= max_bytes {
        return bytes.len();
    }

    // 回溯找到合法的 UTF-8 字符边界
    let mut end = max_bytes;
    while end > 0 && (bytes[end] & 0x80) == 0x80 && (bytes[end] & 0xC0) != 0xC0 {
        end -= 1;
    }
    end
}

/// 找到 UTF-8 字符边界（从末尾）
fn find_utf8_boundary_from_end(s: &str, max_bytes: usize) -> usize {
    let bytes = s.as_bytes();
    let total_len = bytes.len();
    let start = total_len.saturating_sub(max_bytes);

    // 前进到合法的 UTF-8 字符边界
    let mut begin = start;
    while begin < total_len && (bytes[begin] & 0x80) == 0x80 && (bytes[begin] & 0xC0) != 0xC0 {
        begin += 1;
    }
    begin
}

/// 默认截断（1MB）
pub fn truncate_output(output: &str) -> String {
    truncate_preserve_head_tail(output, MAX_OUTPUT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        let result = security_check("ls -la");
        assert!(result.passed);
    }

    #[test]
    fn test_dangerous_exec() {
        let result = security_check("find / -exec rm -rf {} \\;");
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.contains("危险参数")));
    }

    #[test]
    fn test_null_byte() {
        let result = security_check("echo hello\0world");
        assert!(!result.passed);
    }

    #[test]
    fn test_path_traversal() {
        let result = security_check("cat /etc/passwd../../../secret");
        assert!(!result.passed);
    }

    #[test]
    fn test_env_injection() {
        let result = security_check("LD_PRELOAD=/malicious.so command");
        assert!(!result.passed);
    }

    #[test]
    fn test_strip_null_bytes() {
        assert_eq!(strip_null_bytes("hello\0world"), "helloworld");
    }

    #[test]
    fn test_truncate_preserve_head_tail() {
        let long_output = "a".repeat(2000);
        let truncated = truncate_preserve_head_tail(&long_output, 100);
        assert!(truncated.contains("[truncated"));
        assert!(truncated.starts_with("aaa"));
        assert!(truncated.ends_with("aaa"));
    }
}
