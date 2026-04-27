//! 代码审计漏洞模式定义
//!
//! 定义常见的安全漏洞模式，用于静态代码分析。

use super::audit::Severity;

/// 漏洞模式 trait
pub trait VulnerabilityPattern: Send + Sync {
    /// 获取漏洞名称
    fn name(&self) -> &'static str;
    /// 获取漏洞描述
    fn description(&self) -> &'static str;
    /// 获取匹配模式 (正则表达式)
    fn pattern(&self) -> &'static str;
    /// 获取严重性级别
    fn severity(&self) -> Severity;
    /// 获取 OWASP 分类
    fn owasp_category(&self) -> &'static str;
    /// 获取 CIS 规则 ID
    fn cis_rule(&self) -> &'static str;
    /// 检查行是否匹配此模式
    fn matches(&self, line: &str) -> bool;
}

/// 漏洞模式列表
pub const VULNERABILITY_PATTERNS: &[&dyn VulnerabilityPattern] = &[
    &SqlInjectionPattern,
    &CommandInjectionPattern,
    &PathTraversalPattern,
    &XssPattern,
    &HardcodedCredentialPattern,
    &WeakCryptoPattern,
    &UnsafeDeserializationPattern,
    &BufferOverflowPattern,
    &RaceConditionPattern,
    &InsecureRandomPattern,
    &UnvalidatedRedirectPattern,
    &MissingAuthCheckPattern,
    &InformationDisclosurePattern,
    &WeakHashPattern,
    &CertificateValidationPattern,
];

/// SQL 注入模式
struct SqlInjectionPattern;
impl VulnerabilityPattern for SqlInjectionPattern {
    fn name(&self) -> &'static str {
        "SQL Injection"
    }

    fn description(&self) -> &'static str {
        "Potential SQL injection vulnerability - user input concatenated into SQL query"
    }

    fn pattern(&self) -> &'static str {
        r#"format!\s*\(\s*".*\{"# // format!("SELECT * FROM users WHERE id = {}", user_input)
    }

    fn severity(&self) -> Severity {
        Severity::Critical
    }

    fn owasp_category(&self) -> &'static str {
        "A03" // Injection
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        // 简化的 SQL 注入检测
        line.contains("format!(\"SELECT")
            || line.contains("format!(\"INSERT")
            || line.contains("format!(\"UPDATE")
            || line.contains("format!(\"DELETE")
            || line.contains("format!(\"DROP")
    }
}

/// 命令注入模式
struct CommandInjectionPattern;
impl VulnerabilityPattern for CommandInjectionPattern {
    fn name(&self) -> &'static str {
        "Command Injection"
    }

    fn description(&self) -> &'static str {
        "Potential command injection vulnerability - shell command built from untrusted input"
    }

    fn pattern(&self) -> &'static str {
        r#"std::process::Command.*\+.*input"#
    }

    fn severity(&self) -> Severity {
        Severity::Critical
    }

    fn owasp_category(&self) -> &'static str {
        "A03" // Injection
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("Command::new(") && line.contains('+')
    }
}

/// 路径遍历模式
struct PathTraversalPattern;
impl VulnerabilityPattern for PathTraversalPattern {
    fn name(&self) -> &'static str {
        "Path Traversal"
    }

    fn description(&self) -> &'static str {
        "Potential path traversal vulnerability - user input used in file path without validation"
    }

    fn pattern(&self) -> &'static str {
        r#"\.join\(.*user"# // Path::new(base).join(user_input)
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A01" // Broken Access Control
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("Path::new(")
            || line.contains("PathBuf::from(")
            || line.contains(".join(")
    }
}

/// XSS 模式 (用于 Web 相关代码)
struct XssPattern;
impl VulnerabilityPattern for XssPattern {
    fn name(&self) -> &'static str {
        "Cross-Site Scripting (XSS)"
    }

    fn description(&self) -> &'static str {
        "Potential XSS vulnerability - unescaped user input in HTML output"
    }

    fn pattern(&self) -> &'static str {
        r#"\.into_html\(.*user_input"#
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A03" // Injection
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        // 简化检测：直接返回 HTML 内容
        (line.contains("Html::from(") || line.contains(".into_html()"))
            && (line.contains("user") || line.contains("input") || line.contains("request"))
    }
}

/// 硬编码凭证模式
struct HardcodedCredentialPattern;
impl VulnerabilityPattern for HardcodedCredentialPattern {
    fn name(&self) -> &'static str {
        "Hardcoded Credential"
    }

    fn description(&self) -> &'static str {
        "Hardcoded password or API key detected"
    }

    fn pattern(&self) -> &'static str {
        r#"password\s*=\s*".*"#
    }

    fn severity(&self) -> Severity {
        Severity::Critical
    }

    fn owasp_category(&self) -> &'static str {
        "A02" // Cryptographic Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-6.1"
    }

    fn matches(&self, line: &str) -> bool {
        let lower = line.to_lowercase();
        (lower.contains("password")
            || lower.contains("secret")
            || lower.contains("api_key")
            || lower.contains("apikey")
            || lower.contains("token"))
            && (lower.contains("= \"") || lower.contains("= '"))
    }
}

/// 弱加密模式
struct WeakCryptoPattern;
impl VulnerabilityPattern for WeakCryptoPattern {
    fn name(&self) -> &'static str {
        "Weak Cryptographic Algorithm"
    }

    fn description(&self) -> &'static str {
        "Use of deprecated or weak cryptographic algorithm"
    }

    fn pattern(&self) -> &'static str {
        r#"::md5\(|::sha1\("#
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A02" // Cryptographic Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-6.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("Md5::")
            || line.contains("Sha1::")
            || line.contains("::md5(")
            || line.contains("::sha1(")
    }
}

/// 不安全反序列化模式
struct UnsafeDeserializationPattern;
impl VulnerabilityPattern for UnsafeDeserializationPattern {
    fn name(&self) -> &'static str {
        "Insecure Deserialization"
    }

    fn description(&self) -> &'static str {
        "Use of unsafe deserialization which can lead to code execution"
    }

    fn pattern(&self) -> &'static str {
        r#"bincode::deserialize.*\("#
    }

    fn severity(&self) -> Severity {
        Severity::Critical
    }

    fn owasp_category(&self) -> &'static str {
        "A08" // Software and Data Integrity Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("bincode::deserialize")
            || line.contains("serde_json::from_str")
                && (line.contains("user") || line.contains("input"))
    }
}

/// 缓冲区溢出模式 (Rust 通常安全, 但 FFI 调用可能有问题)
struct BufferOverflowPattern;
impl VulnerabilityPattern for BufferOverflowPattern {
    fn name(&self) -> &'static str {
        "Potential Buffer Overflow (FFI)"
    }

    fn description(&self) -> &'static str {
        "Potential buffer overflow in FFI code"
    }

    fn pattern(&self) -> &'static str {
        r#"unsafe\s*\{.* libc::"#
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A04" // Insecure Design
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("unsafe {") && line.contains("libc::")
    }
}

/// 竞态条件模式
struct RaceConditionPattern;
impl VulnerabilityPattern for RaceConditionPattern {
    fn name(&self) -> &'static str {
        "Potential Race Condition"
    }

    fn description(&self) -> &'static str {
        "Potential race condition in concurrent code"
    }

    fn pattern(&self) -> &'static str {
        r#"static\s+mut"#
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A04" // Insecure Design
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-4.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("static mut")
    }
}

/// 不安全随机数模式
struct InsecureRandomPattern;
impl VulnerabilityPattern for InsecureRandomPattern {
    fn name(&self) -> &'static str {
        "Insecure Random Number Generator"
    }

    fn description(&self) -> &'static str {
        "Use of rand::thread_rng for security-sensitive operations"
    }

    fn pattern(&self) -> &'static str {
        r#"rand::thread_rng\(\)"#
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn owasp_category(&self) -> &'static str {
        "A02" // Cryptographic Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-6.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("thread_rng()")
    }
}

/// 未验证重定向模式
struct UnvalidatedRedirectPattern;
impl VulnerabilityPattern for UnvalidatedRedirectPattern {
    fn name(&self) -> &'static str {
        "Unvalidated Redirect"
    }

    fn description(&self) -> &'static str {
        "Potential open redirect vulnerability"
    }

    fn pattern(&self) -> &'static str {
        r#"redirect\(.*user"# // redirect(user_input)
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn owasp_category(&self) -> &'static str {
        "A01" // Broken Access Control
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-2.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("redirect(") && (line.contains("user") || line.contains("input"))
    }
}

/// 缺失权限检查模式
struct MissingAuthCheckPattern;
impl VulnerabilityPattern for MissingAuthCheckPattern {
    fn name(&self) -> &'static str {
        "Missing Authorization Check"
    }

    fn description(&self) -> &'static str {
        "Function performing sensitive operation without authorization check"
    }

    fn pattern(&self) -> &'static str {
        r#"fn\s+admin_|fn\s+delete_|fn\s+update_|fn\s+create_"#
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn owasp_category(&self) -> &'static str {
        "A01" // Broken Access Control
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-5.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("async fn admin_")
            || line.contains("async fn delete_")
            || line.contains("async fn update_")
            || line.contains("async fn create_")
            || line.contains("pub async fn admin_")
            || line.contains("pub async fn delete_")
    }
}

/// 信息泄露模式
struct InformationDisclosurePattern;
impl VulnerabilityPattern for InformationDisclosurePattern {
    fn name(&self) -> &'static str {
        "Information Disclosure"
    }

    fn description(&self) -> &'static str {
        "Sensitive information being logged or exposed"
    }

    fn pattern(&self) -> &'static str {
        r#"eprintln!\(.*password|eprintln!\(.*secret"#
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn owasp_category(&self) -> &'static str {
        "A09" // Security Logging and Monitoring Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-4.1"
    }

    fn matches(&self, line: &str) -> bool {
        let lower = line.to_lowercase();
        (lower.contains("eprintln!")
            || lower.contains("println!")
            || lower.contains("log::"))
            && (lower.contains("password")
                || lower.contains("secret")
                || lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("credential"))
    }
}

/// 弱哈希模式
struct WeakHashPattern;
impl VulnerabilityPattern for WeakHashPattern {
    fn name(&self) -> &'static str {
        "Weak Hash Function"
    }

    fn description(&self) -> &'static str {
        "Use of weak hash function for security purposes"
    }

    fn pattern(&self) -> &'static str {
        r#"sha2::Sha1|sha2::Sha256"#
    }

    fn severity(&self) -> Severity {
        Severity::Low
    }

    fn owasp_category(&self) -> &'static str {
        "A02" // Cryptographic Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-6.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("sha2::Sha1")
    }
}

/// 证书验证禁用模式
struct CertificateValidationPattern;
impl VulnerabilityPattern for CertificateValidationPattern {
    fn name(&self) -> &'static str {
        "Disabled Certificate Validation"
    }

    fn description(&self) -> &'static str {
        "TLS certificate validation is disabled"
    }

    fn pattern(&self) -> &'static str {
        r#"danger_accept_invalid_certs\(true\)"#
    }

    fn severity(&self) -> Severity {
        Severity::Critical
    }

    fn owasp_category(&self) -> &'static str {
        "A02" // Cryptographic Failures
    }

    fn cis_rule(&self) -> &'static str {
        "CIS-6.1"
    }

    fn matches(&self, line: &str) -> bool {
        line.contains("danger_accept_invalid_certs(true)")
    }
}

/// 审计代码模式
#[derive(Clone)]
pub struct CodeAuditPattern {
    pub name: &'static str,
    pub description: &'static str,
    pub pattern: &'static str,
    pub severity: Severity,
    pub owasp_category: &'static str,
    pub cis_rule: &'static str,
}

impl VulnerabilityPattern for CodeAuditPattern {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn pattern(&self) -> &'static str {
        self.pattern
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn owasp_category(&self) -> &'static str {
        self.owasp_category
    }

    fn cis_rule(&self) -> &'static str {
        self.cis_rule
    }

    fn matches(&self, line: &str) -> bool {
        line.contains(self.pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let pattern = SqlInjectionPattern;
        assert!(pattern.matches(r#"let query = format!("SELECT * FROM users WHERE id = {}", user_id);"#));
        assert!(!pattern.matches(r#"let safe = format!("Hello {}!", name);"#));
    }

    #[test]
    fn test_hardcoded_credential_detection() {
        let pattern = HardcodedCredentialPattern;
        assert!(pattern.matches(r#"let password = "secret123";"#));
        assert!(pattern.matches(r#"api_key = "abc123";"#));
        assert!(!pattern.matches(r#"let password = std::env::var("PASSWORD")?;"#));
    }

    #[test]
    fn test_weak_crypto_detection() {
        let pattern = WeakCryptoPattern;
        assert!(pattern.matches("Md5::digest(&data)"));
        assert!(pattern.matches("Sha1::new()"));
        assert!(!pattern.matches("Sha256::new()"));
    }

    #[test]
    fn test_certificate_validation_detection() {
        let pattern = CertificateValidationPattern;
        assert!(pattern.matches("danger_accept_invalid_certs(true)"));
        assert!(!pattern.matches("danger_accept_invalid_certs(false)"));
    }
}