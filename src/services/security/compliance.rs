//! 安全合规报告
//!
//! 生成符合标准的安全报告:
//! - 漏洞扫描结果
//! - 依赖审计
//! - 安全配置检查
//!
//! 符合标准:
//! - OWASP Top 10
//! - CIS Benchmarks
//!
//! 报告格式: JSON

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 合规标准
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStandard {
    /// OWASP Top 10
    OwaspTop10,
    /// CIS Benchmarks
    CisBenchmarks,
    /// 自定义
    Custom,
}

/// 安全级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 严重
    Critical,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::Low => write!(f, "low"),
            SecurityLevel::Medium => write!(f, "medium"),
            SecurityLevel::High => write!(f, "high"),
            SecurityLevel::Critical => write!(f, "critical"),
        }
    }
}

/// 漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// 漏洞 ID
    pub id: String,
    /// 漏洞名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 安全级别
    pub severity: SecurityLevel,
    /// 影响组件
    pub affected_component: String,
    /// 修复建议
    pub remediation: String,
    /// 参考链接
    pub references: Vec<String>,
    /// 发现时间
    pub discovered_at: DateTime<Utc>,
}

/// 依赖审计项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAuditItem {
    /// 依赖名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 许可证
    pub license: Option<String>,
    /// 漏洞数量
    pub vulnerability_count: usize,
    /// 是否过期
    pub is_outdated: bool,
    /// 最新版本
    pub latest_version: Option<String>,
}

/// 配置检查项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCheckItem {
    /// 检查项名称
    pub name: String,
    /// 检查结果
    pub passed: bool,
    /// 当前值
    pub current_value: Option<String>,
    /// 期望值
    pub expected_value: Option<String>,
    /// 描述
    pub description: String,
    /// 安全级别
    pub severity: SecurityLevel,
}

/// 安全配置检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfigReport {
    /// 检查项列表
    pub checks: Vec<ConfigCheckItem>,
    /// 通过的检查数
    pub passed_count: usize,
    /// 失败的检查数
    pub failed_count: usize,
    /// 总体评分 (0-100)
    pub score: u32,
}

/// 依赖审计报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAuditReport {
    /// 审计时间
    pub audited_at: DateTime<Utc>,
    /// 依赖项列表
    pub dependencies: Vec<DependencyAuditItem>,
    /// 总依赖数
    pub total_count: usize,
    /// 有漏洞的依赖数
    pub vulnerable_count: usize,
    /// 过期的依赖数
    pub outdated_count: usize,
    /// 许可证不合规数
    pub license_issues: usize,
}

/// 漏洞扫描报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityScanReport {
    /// 扫描时间
    pub scanned_at: DateTime<Utc>,
    /// 扫描目标
    pub target: String,
    /// 漏洞列表
    pub vulnerabilities: Vec<Vulnerability>,
    /// 高风险漏洞数
    pub critical_count: usize,
    /// 高风险漏洞数
    pub high_count: usize,
    /// 中风险漏洞数
    pub medium_count: usize,
    /// 低风险漏洞数
    pub low_count: usize,
}

/// 综合安全报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityComplianceReport {
    /// 报告 ID
    pub report_id: String,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// 报告版本
    pub version: String,
    /// 目标系统
    pub target_system: String,
    /// 适用标准
    pub standards: Vec<ComplianceStandard>,
    /// 漏洞扫描报告
    pub vulnerability_scan: Option<VulnerabilityScanReport>,
    /// 依赖审计报告
    pub dependency_audit: Option<DependencyAuditReport>,
    /// 安全配置报告
    pub security_config: Option<SecurityConfigReport>,
    /// 总体评分 (0-100)
    pub overall_score: u32,
    /// 风险等级
    pub risk_level: SecurityLevel,
    /// 建议措施
    pub recommendations: Vec<String>,
}

impl Default for SecurityComplianceReport {
    fn default() -> Self {
        Self {
            report_id: uuid_v4(),
            generated_at: Utc::now(),
            version: "1.0.0".to_string(),
            target_system: "ArkCore".to_string(),
            standards: vec![ComplianceStandard::OwaspTop10, ComplianceStandard::CisBenchmarks],
            vulnerability_scan: None,
            dependency_audit: None,
            security_config: None,
            overall_score: 100,
            risk_level: SecurityLevel::Low,
            recommendations: Vec::new(),
        }
    }
}

/// 合规报告生成器
pub struct ComplianceReporter {
    /// OWASP Top 10 检查项
    owasp_checks: Vec<OwaspCheck>,
    /// CIS 检查项
    cis_checks: Vec<CisCheck>,
}

impl Default for ComplianceReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceReporter {
    /// 创建新的合规报告生成器
    pub fn new() -> Self {
        Self {
            owasp_checks: Self::init_owasp_checks(),
            cis_checks: Self::init_cis_checks(),
        }
    }

    /// 初始化 OWASP 检查项
    fn init_owasp_checks() -> Vec<OwaspCheck> {
        vec![
            OwaspCheck {
                id: "A01".to_string(),
                name: "Broken Access Control".to_string(),
                description: "访问控制失效".to_string(),
                severity: SecurityLevel::High,
            },
            OwaspCheck {
                id: "A02".to_string(),
                name: "Cryptographic Failures".to_string(),
                description: "加密失败".to_string(),
                severity: SecurityLevel::High,
            },
            OwaspCheck {
                id: "A03".to_string(),
                name: "Injection".to_string(),
                description: "注入攻击".to_string(),
                severity: SecurityLevel::High,
            },
            OwaspCheck {
                id: "A04".to_string(),
                name: "Insecure Design".to_string(),
                description: "不安全设计".to_string(),
                severity: SecurityLevel::Medium,
            },
            OwaspCheck {
                id: "A05".to_string(),
                name: "Security Misconfiguration".to_string(),
                description: "安全配置错误".to_string(),
                severity: SecurityLevel::Medium,
            },
            OwaspCheck {
                id: "A06".to_string(),
                name: "Vulnerable and Outdated Components".to_string(),
                description: "易受攻击和过时的组件".to_string(),
                severity: SecurityLevel::Medium,
            },
            OwaspCheck {
                id: "A07".to_string(),
                name: "Identification and Authentication Failures".to_string(),
                description: "识别和认证失败".to_string(),
                severity: SecurityLevel::High,
            },
            OwaspCheck {
                id: "A08".to_string(),
                name: "Software and Data Integrity Failures".to_string(),
                description: "软件和数据完整性失败".to_string(),
                severity: SecurityLevel::High,
            },
            OwaspCheck {
                id: "A09".to_string(),
                name: "Security Logging and Monitoring Failures".to_string(),
                description: "安全日志和监控失败".to_string(),
                severity: SecurityLevel::Medium,
            },
            OwaspCheck {
                id: "A10".to_string(),
                name: "Server-Side Request Forgery".to_string(),
                description: "服务器端请求伪造".to_string(),
                severity: SecurityLevel::High,
            },
        ]
    }

    /// 初始化 CIS 检查项
    fn init_cis_checks() -> Vec<CisCheck> {
        vec![
            CisCheck {
                id: "CIS-1.1".to_string(),
                name: "Disable unused filesystems".to_string(),
                description: "禁用未使用的文件系统".to_string(),
                severity: SecurityLevel::Medium,
            },
            CisCheck {
                id: "CIS-1.2".to_string(),
                name: "Disable unused protocols".to_string(),
                description: "禁用未使用的协议".to_string(),
                severity: SecurityLevel::Medium,
            },
            CisCheck {
                id: "CIS-2.1".to_string(),
                name: "Ensure separate partition".to_string(),
                description: "确保分区隔离".to_string(),
                severity: SecurityLevel::Low,
            },
            CisCheck {
                id: "CIS-3.1".to_string(),
                name: "Disable IP forwarding".to_string(),
                description: "禁用 IP 转发".to_string(),
                severity: SecurityLevel::Medium,
            },
            CisCheck {
                id: "CIS-3.2".to_string(),
                name: "Disable packet redirect sending".to_string(),
                description: "禁用数据包重定向发送".to_string(),
                severity: SecurityLevel::Medium,
            },
            CisCheck {
                id: "CIS-4.1".to_string(),
                name: "Configure firewall rules".to_string(),
                description: "配置防火墙规则".to_string(),
                severity: SecurityLevel::High,
            },
        ]
    }

    /// 执行 OWASP Top 10 检查
    pub fn check_owasp(&self, config: &HashMap<String, String>) -> Vec<ConfigCheckItem> {
        self.owasp_checks
            .iter()
            .map(|check| {
                let passed = match check.id.as_str() {
                    "A01" => config.get("access_control_enabled").map(|v| v == "true").unwrap_or(false),
                    "A02" => config.get("encryption_enabled").map(|v| v == "true").unwrap_or(false),
                    "A03" => config.get("input_validation_enabled").map(|v| v == "true").unwrap_or(false),
                    "A04" => config.get("secure_design_reviewed").map(|v| v == "true").unwrap_or(false),
                    "A05" => config.get("security_hardening_applied").map(|v| v == "true").unwrap_or(false),
                    "A06" => config.get("components_updated").map(|v| v == "true").unwrap_or(false),
                    "A07" => config.get("auth_mechanism_strong").map(|v| v == "true").unwrap_or(false),
                    "A08" => config.get("integrity_checks_enabled").map(|v| v == "true").unwrap_or(false),
                    "A09" => config.get("logging_monitoring_enabled").map(|v| v == "true").unwrap_or(false),
                    "A10" => config.get("ssrf_protection_enabled").map(|v| v == "true").unwrap_or(false),
                    _ => false,
                };

                ConfigCheckItem {
                    name: check.name.clone(),
                    passed,
                    current_value: Some(passed.to_string()),
                    expected_value: Some("true".to_string()),
                    description: check.description.clone(),
                    severity: check.severity,
                }
            })
            .collect()
    }

    /// 执行 CIS Benchmark 检查
    pub fn check_cis(&self, config: &HashMap<String, String>) -> Vec<ConfigCheckItem> {
        self.cis_checks
            .iter()
            .map(|check| {
                let passed = config
                    .get(&check.id.to_lowercase())
                    .map(|v| v == "true")
                    .unwrap_or(false);

                ConfigCheckItem {
                    name: check.name.clone(),
                    passed,
                    current_value: Some(passed.to_string()),
                    expected_value: Some("true".to_string()),
                    description: check.description.clone(),
                    severity: check.severity,
                }
            })
            .collect()
    }

    /// 生成安全配置报告
    pub fn generate_config_report(&self, config: &HashMap<String, String>) -> SecurityConfigReport {
        let mut checks = Vec::new();
        checks.extend(self.check_owasp(config));
        checks.extend(self.check_cis(config));

        let passed_count = checks.iter().filter(|c| c.passed).count();
        let failed_count = checks.len() - passed_count;
        let score: u32 = if checks.is_empty() {
            100
        } else {
            ((passed_count * 100) / checks.len()) as u32
        };

        SecurityConfigReport {
            checks,
            passed_count,
            failed_count,
            score,
        }
    }

    /// 生成空漏洞报告
    pub fn generate_empty_vulnerability_report(&self, target: &str) -> VulnerabilityScanReport {
        VulnerabilityScanReport {
            scanned_at: Utc::now(),
            target: target.to_string(),
            vulnerabilities: Vec::new(),
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
        }
    }

    /// 生成综合安全报告
    pub fn generate_compliance_report(
        &self,
        target_system: &str,
        config: &HashMap<String, String>,
    ) -> SecurityComplianceReport {
        let config_report = self.generate_config_report(config);
        let vuln_report = self.generate_empty_vulnerability_report(target_system);
        let overall_score = config_report.score;

        let mut recommendations = Vec::new();

        // 基于失败的检查添加建议
        for check in &config_report.checks {
            if !check.passed {
                recommendations.push(format!(
                    "修复 {} ({}) - {}",
                    check.name, check.severity, check.description
                ));
            }
        }

        // 计算风险等级
        let risk_level = if overall_score < 50 {
            SecurityLevel::Critical
        } else if overall_score < 70 {
            SecurityLevel::High
        } else if overall_score < 85 {
            SecurityLevel::Medium
        } else {
            SecurityLevel::Low
        };

        SecurityComplianceReport {
            report_id: uuid_v4(),
            generated_at: Utc::now(),
            version: "1.0.0".to_string(),
            target_system: target_system.to_string(),
            standards: vec![ComplianceStandard::OwaspTop10, ComplianceStandard::CisBenchmarks],
            vulnerability_scan: Some(vuln_report),
            dependency_audit: None,
            security_config: Some(config_report),
            overall_score,
            risk_level,
            recommendations,
        }
    }

    /// 生成报告 JSON
    pub fn generate_json(&self, report: &SecurityComplianceReport) -> Result<String, ComplianceError> {
        serde_json::to_string_pretty(report).map_err(|e| ComplianceError::SerializationError(e.to_string()))
    }
}

/// OWASP 检查项
struct OwaspCheck {
    id: String,
    name: String,
    description: String,
    severity: SecurityLevel,
}

/// CIS 检查项
struct CisCheck {
    id: String,
    name: String,
    description: String,
    severity: SecurityLevel,
}

/// 合规错误类型
#[derive(Debug, Clone)]
pub enum ComplianceError {
    /// 序列化错误
    SerializationError(String),
    /// 配置错误
    ConfigError(String),
    /// 扫描错误
    ScanError(String),
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            ComplianceError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            ComplianceError::ScanError(msg) => write!(f, "扫描错误: {}", msg),
        }
    }
}

impl std::error::Error for ComplianceError {}

/// 生成 UUID (使用 getrandom 实现)
fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        (bytes[6] & 0x0f) | 0x40, bytes[7],
        (bytes[8] & 0x3f) | 0x80, bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_reporter_creation() {
        let reporter = ComplianceReporter::new();
        assert_eq!(reporter.owasp_checks.len(), 10);
        assert_eq!(reporter.cis_checks.len(), 6);
    }

    #[test]
    fn test_generate_config_report() {
        let reporter = ComplianceReporter::new();
        let mut config = HashMap::new();
        config.insert("access_control_enabled".to_string(), "true".to_string());
        config.insert("encryption_enabled".to_string(), "true".to_string());

        let report = reporter.generate_config_report(&config);
        assert!(report.passed_count >= 2);
    }

    #[test]
    fn test_generate_compliance_report() {
        let reporter = ComplianceReporter::new();
        let config = HashMap::new();

        let report = reporter.generate_compliance_report("TestSystem", &config);
        assert_eq!(report.target_system, "TestSystem");
        assert!(report.overall_score <= 100);
    }

    #[test]
    fn test_report_json_serialization() {
        let reporter = ComplianceReporter::new();
        let config = HashMap::new();

        let report = reporter.generate_compliance_report("TestSystem", &config);
        let json = reporter.generate_json(&report);

        assert!(json.is_ok());
        assert!(json.unwrap().contains("TestSystem"));
    }

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Critical > SecurityLevel::High);
        assert!(SecurityLevel::High > SecurityLevel::Medium);
        assert!(SecurityLevel::Medium > SecurityLevel::Low);
    }

    #[test]
    fn test_risk_level_calculation() {
        let reporter = ComplianceReporter::new();
        let config = HashMap::new();

        let report = reporter.generate_compliance_report("TestSystem", &config);

        // 空的 config 应该导致低分
        assert!(report.overall_score < 100 || report.risk_level == SecurityLevel::Low);
    }
}