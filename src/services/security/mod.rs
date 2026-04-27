//! ArkCore 安全服务模块
//!
//! 提供完整的安全加固实现:
//!
//! # 模块
//!
//! - [`validator`] - 输入验证层
//! - [`audit`] - 安全审计日志
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
//! - AES-256-GCM 加密
//! - 基于角色的访问控制
//! - Secure API 密钥管理
//! - OWASP Top 10 / CIS Benchmarks 合规报告

pub mod apikey;
pub mod audit;
pub mod compliance;
pub mod crypto;
pub mod rbac;
pub mod validator;

// 重新导出主要类型
pub use apikey::{ApiKeyAuthResult, ApiKeyService, ApiKeyMiddleware, CreateApiKeyRequest};
pub use audit::{AuditEvent, AuditEventType, AuditLogWriter, AuditLogReader, AuditResult};
pub use compliance::{ComplianceReporter, SecurityComplianceReport, SecurityLevel, ComplianceStandard};
pub use crypto::{CryptoService, EncryptedData, SensitiveField, PasswordHasher};
pub use rbac::{Permission, Role, RbacService, RbacMiddleware, UserRole, PermissionCheckResult};
pub use validator::{UserInput, CommandSpec, ApiRequest, ValidationResult, DangerPatternDetector, InputSanitizer};