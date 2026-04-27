//! ArkCore 安全模块
//!
//! 提供完整的安全功能套件:
//!
//! # 核心功能
//!
//! - [`audit`] - 安全审计工具 (T9.1-T9.4)
//!   - 依赖审计 (cargo audit)
//!   - 代码审计
//!   - 渗透测试
//!   - OWASP/CIS 合规报告生成
//!
//! # 架构
//!
//! ```text
//! security/
//! ├── audit.rs    - 审计核心 (T9.1-T9.4)
//! ├── patterns.rs - 漏洞模式定义
//! └── pentest.rs - 渗透测试模块
//! ```
//!
//! # 安全等级
//!
//! ArkCore 实现最高安全等级, 符合:
//!
//! - OWASP Top 10 2021
//! - CIS Benchmarks for Rust
//! - RustSEC Advisory Database
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use arkcore::security::audit::{SecurityAuditor, AuditConfig, AuditReport};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = AuditConfig::default();
//!     let auditor = SecurityAuditor::new(config);
//!
//!     // 运行完整审计
//!     let report = auditor.run_full_audit().await?;
//!
//!     // 输出报告
//!     println!("{}", report.summary());
//!     println!("{}", report.to_markdown());
//!
//!     // 检查是否有严重漏洞
//!     if report.has_critical_vulnerabilities() {
//!         eprintln!("严重漏洞发现, 审计失败!");
//!         std::process::exit(1);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod audit;
pub mod patterns;
pub mod pentest;

// Re-export 主要类型
pub use audit::{
    generate_cis_report, generate_owasp_report, AuditConfig, AuditReport, CodeVulnerability,
    DependencyVulnerability, OwaspComplianceItem, PentestResult, SecurityAuditor, Severity,
};
pub use pentest::PenetrationTester;
pub use patterns::{CodeAuditPattern, VulnerabilityPattern};
