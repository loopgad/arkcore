//! 输入验证层
//!
//! 使用 validator crate 实现全面的输入验证:
//! - UserInput - 用户输入验证
//! - CommandSpec - 命令规范验证
//! - ApiRequest - API 请求验证
//!
//! 验证规则:
//! - 长度限制
//! - 字符白名单
//! - 危险模式检测

use serde::{Deserialize, Serialize};
use validator::Validate;

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否通过验证
    pub passed: bool,
    /// 验证错误信息
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// 创建通过验证的结果
    pub fn passed() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
        }
    }

    /// 创建失败验证的结果
    pub fn failed(errors: Vec<String>) -> Self {
        Self {
            passed: errors.is_empty(),
            errors,
        }
    }

    /// 添加错误
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.passed = false;
    }
}

/// 用户输入验证
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserInput {
    /// 用户名 (3-32 字符, 字母数字下划线)
    #[validate(length(min = 3, max = 32, message = "用户名长度必须在 3-32 之间"))]
    pub username: Option<String>,

    /// 邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,

    /// 密码 (最小 8 字符)
    #[validate(length(min = 8, message = "密码长度至少为 8 字符"))]
    pub password: Option<String>,

    /// 真实姓名
    #[validate(length(max = 100, message = "姓名长度不能超过 100"))]
    pub real_name: Option<String>,

    /// 描述/备注
    #[validate(length(max = 500, message = "描述长度不能超过 500"))]
    pub description: Option<String>,
}

/// 命令规范验证
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CommandSpec {
    /// 命令字符串
    #[validate(length(min = 1, max = 10000, message = "命令长度必须在 1-10000 之间"))]
    pub command: String,

    /// 命令类型
    pub command_type: Option<String>,

    /// 预期执行时间 (秒)
    #[validate(range(min = 0, max = 3600, message = "预期执行时间必须在 0-3600 秒之间"))]
    pub expected_duration: Option<u32>,

    /// 资源限制 (MB)
    #[validate(range(min = 1, max = 10240, message = "资源限制必须在 1-10240 MB 之间"))]
    pub memory_limit_mb: Option<u32>,
}

/// 验证命令类型
pub fn validate_command_type(command_type: &str) -> bool {
    let allowed_types = ["shell", "exec", "script", "builtin"];
    allowed_types.contains(&command_type)
}

/// API 请求验证
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApiRequest {
    /// API 版本
    #[validate(length(min = 1, max = 20, message = "API 版本长度必须在 1-20 之间"))]
    pub api_version: String,

    /// 请求路径
    #[validate(length(min = 1, max = 256, message = "路径长度必须在 1-256 之间"))]
    pub path: String,

    /// 请求方法
    pub method: String,

    /// 请求头 (JSON 字符串)
    #[validate(length(max = 10000, message = "请求头长度不能超过 10000"))]
    pub headers: Option<String>,

    /// 请求体 (JSON 字符串, 最大 1MB)
    #[validate(length(max = 1048576, message = "请求体长度不能超过 1MB"))]
    pub body: Option<String>,

    /// 客户端 IP
    pub client_ip: Option<String>,
}

/// 验证 API 路径
pub fn validate_api_path(path: &str) -> bool {
    // 路径必须以 / 开头
    if !path.starts_with('/') {
        return false;
    }

    // 禁止路径遍历
    if path.contains("..") || path.contains("//") {
        return false;
    }

    // 禁止控制字符
    for c in path.chars() {
        if c.is_control() {
            return false;
        }
    }

    true
}

/// 验证 HTTP 方法
pub fn validate_http_method(method: &str) -> bool {
    let allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
    allowed_methods.contains(&method)
}

/// 危险模式检测
pub struct DangerPatternDetector {
    /// SQL 注入模式
    sql_injection_patterns: Vec<String>,
    /// XSS 模式
    xss_patterns: Vec<String>,
    /// 命令注入模式
    command_injection_patterns: Vec<String>,
}

impl Default for DangerPatternDetector {
    fn default() -> Self {
        Self {
            sql_injection_patterns: vec![
                r"' OR '1'='1".to_string(),
                r"'; DROP TABLE".to_string(),
                r"UNION SELECT".to_string(),
                r"--".to_string(),
                r";".to_string(),
            ],
            xss_patterns: vec![
                r"<script>".to_string(),
                r"javascript:".to_string(),
                r"onerror=".to_string(),
                r"onload=".to_string(),
            ],
            command_injection_patterns: vec![
                r";\s*rm".to_string(),
                r"\|\s*cat".to_string(),
                r"`.*`".to_string(),
                r"\$\(.*\)".to_string(),
            ],
        }
    }
}

impl DangerPatternDetector {
    /// 检测 SQL 注入
    pub fn detect_sql_injection(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        self.sql_injection_patterns
            .iter()
            .any(|pattern| input_lower.contains(&pattern.to_lowercase()))
    }

    /// 检测 XSS
    pub fn detect_xss(&self, input: &str) -> bool {
        self.xss_patterns.iter().any(|pattern| input.contains(pattern))
    }

    /// 检测命令注入
    pub fn detect_command_injection(&self, input: &str) -> bool {
        self.command_injection_patterns
            .iter()
            .any(|pattern| {
                input.contains(pattern)
            }) || input.contains("&&")
            || input.contains("||")
            // 检测常见命令注入模式（带空格）
            || input.contains("; ")
            || input.contains("| ")
            || input.contains("` ")
    }

    /// 综合检测
    pub fn detect_all(&self, input: &str) -> Vec<String> {
        let mut threats = Vec::new();

        if self.detect_sql_injection(input) {
            threats.push("SQL 注入风险".to_string());
        }

        if self.detect_xss(input) {
            threats.push("XSS 风险".to_string());
        }

        if self.detect_command_injection(input) {
            threats.push("命令注入风险".to_string());
        }

        threats
    }
}

/// 输入净化器
pub struct InputSanitizer;

impl InputSanitizer {
    /// 移除非打印字符
    pub fn strip_non_printable(input: &str) -> String {
        input.chars().map(|c| {
            if c.is_control() && !c.is_whitespace() {
                ' '
            } else {
                c
            }
        }).collect()
    }

    /// 移除非 ASCII 字符
    pub fn strip_non_ascii(input: &str) -> String {
        input.chars().filter(|c| c.is_ascii()).collect()
    }

    /// 规范化空白字符
    pub fn normalize_whitespace(input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_validation() {
        let input = UserInput {
            username: Some("test_user".to_string()),
            email: Some("test@example.com".to_string()),
            password: Some("password123".to_string()),
            real_name: Some("Test User".to_string()),
            description: None,
        };

        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_user_input_validation_fail() {
        let input = UserInput {
            username: Some("ab".to_string()), // 太短
            email: Some("invalid-email".to_string()),
            password: Some("short".to_string()), // 太短
            real_name: None,
            description: None,
        };

        assert!(input.validate().is_err());
    }

    #[test]
    fn test_command_spec_validation() {
        let cmd = CommandSpec {
            command: "ls -la".to_string(),
            command_type: Some("shell".to_string()),
            expected_duration: Some(30),
            memory_limit_mb: Some(256),
        };

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_api_request_validation() {
        let request = ApiRequest {
            api_version: "v1".to_string(),
            path: "/api/users".to_string(),
            method: "GET".to_string(),
            headers: None,
            body: None,
            client_ip: Some("127.0.0.1".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_danger_pattern_detector() {
        let detector = DangerPatternDetector::default();

        assert!(detector.detect_sql_injection("' OR '1'='1"));
        assert!(detector.detect_xss("<script>alert(1)</script>"));
        assert!(detector.detect_command_injection("; rm -rf /"));
    }

    #[test]
    fn test_input_sanitizer() {
        assert_eq!(
            InputSanitizer::strip_non_printable("hello\x00world"),
            "hello world"
        );
        assert_eq!(
            InputSanitizer::normalize_whitespace("hello   world"),
            "hello world"
        );
    }

    #[test]
    fn test_validate_command_type() {
        assert!(validate_command_type("shell"));
        assert!(validate_command_type("exec"));
        assert!(!validate_command_type("invalid"));
    }

    #[test]
    fn test_validate_api_path() {
        assert!(validate_api_path("/api/users"));
        assert!(!validate_api_path("api/users")); // 缺少前导 /
        assert!(!validate_api_path("/api//users")); // 双斜杠
        assert!(!validate_api_path("/api/../secret")); // 路径遍历
    }

    #[test]
    fn test_validate_http_method() {
        assert!(validate_http_method("GET"));
        assert!(validate_http_method("POST"));
        assert!(!validate_http_method("INVALID"));
    }
}