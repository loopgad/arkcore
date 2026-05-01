//! ArkCore 安全服务模块
//!
//! 提供完整的安全加固实现:
//!
//! # 模块
//!
//! - [`validator`] - 输入验证层
//! - [`audit`] - 安全审计日志
//! - [`audit_log`] - HMAC 防篡改审计日志
//! - [`crypto`] - 敏感信息加密
//! - [`rbac`] - RBAC 权限模型
//! - [`apikey`] - API 密钥管理
//! - [`compliance`] - 安全合规报告
//!
//! # 安全特性
//!
//! - 沙盒进程隔离
//! - 六层输入验证
//! - JSON 格式审计日志 (防篡改)
//! - HMAC-SHA256 签名防篡改机制
//! - AES-256-GCM 加密
//! - 基于角色的访问控制
//! - Secure API 密钥管理
//! - OWASP Top 10 / CIS Benchmarks 合规报告

pub mod apikey;
pub mod audit;
pub mod audit_log;
pub mod compliance;
pub mod crypto;
pub mod rbac;
pub mod validator;

// 重新导出主要类型
pub use apikey::{ApiKeyAuthResult, ApiKeyMiddleware, ApiKeyService, CreateApiKeyRequest};
pub use audit::{AuditEvent, AuditEventType, AuditLogReader, AuditLogWriter, AuditResult};
pub use audit_log::{AppendOnlyLog, AuditEntry, AuditLogError, VerifiedEntry};
pub use compliance::{
    ComplianceReporter, ComplianceStandard, SecurityComplianceReport, SecurityLevel,
};
pub use crypto::{CryptoService, EncryptedData, PasswordHasher, SensitiveField};
pub use rbac::{Permission, PermissionCheckResult, RbacMiddleware, RbacService, Role, UserRole};
pub use validator::{
    ApiRequest, CommandSpec, DangerPatternDetector, InputSanitizer, UserInput, ValidationResult,
};
