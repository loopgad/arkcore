//! 安全审计日志
//!
//! 审计事件类型:
//! - 认证 (Authentication)
//! - 授权 (Authorization)
//! - 命令执行 (Command Execution)
//! - 配置变更 (Configuration Change)
//! - 敏感操作 (Sensitive Operation)
//!
//! JSON 格式日志, 包含防篡改机制

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// 审计事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// 认证事件
    Authentication,
    /// 授权事件
    Authorization,
    /// 命令执行
    CommandExecution,
    /// 配置变更
    ConfigurationChange,
    /// 敏感操作
    SensitiveOperation,
    /// 系统事件
    SystemEvent,
    /// 网络事件
    NetworkEvent,
    /// 资源访问
    ResourceAccess,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::Authentication => write!(f, "authentication"),
            AuditEventType::Authorization => write!(f, "authorization"),
            AuditEventType::CommandExecution => write!(f, "command_execution"),
            AuditEventType::ConfigurationChange => write!(f, "configuration_change"),
            AuditEventType::SensitiveOperation => write!(f, "sensitive_operation"),
            AuditEventType::SystemEvent => write!(f, "system_event"),
            AuditEventType::NetworkEvent => write!(f, "network_event"),
            AuditEventType::ResourceAccess => write!(f, "resource_access"),
        }
    }
}

/// 审计结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 拒绝
    Denied,
    /// 错误
    Error,
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "success"),
            AuditResult::Failure => write!(f, "failure"),
            AuditResult::Denied => write!(f, "denied"),
            AuditResult::Error => write!(f, "error"),
        }
    }
}

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件 ID (UUID)
    pub event_id: String,
    /// 时间戳 (ISO 8601)
    pub timestamp: DateTime<Utc>,
    /// 事件类型
    pub event_type: AuditEventType,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 用户名
    pub username: Option<String>,
    /// 操作
    pub action: String,
    /// 资源
    pub resource: Option<String>,
    /// 结果
    pub result: AuditResult,
    /// 详情
    pub details: Option<HashMap<String, String>>,
    /// IP 地址
    pub ip_address: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 会话 ID
    pub session_id: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 前一个值 (用于变更追踪)
    pub previous_value: Option<String>,
    /// 新的值 (用于变更追踪)
    pub new_value: Option<String>,
    /// 校验和 (防篡改)
    pub checksum: String,
}

impl AuditEvent {
    /// 创建新的审计事件
    pub fn new(event_type: AuditEventType, action: impl Into<String>, result: AuditResult) -> Self {
        let event_id = uuid_v4();
        let timestamp = Utc::now();
        let action_str = action.into();
        let checksum = Self::calculate_checksum(&event_id, &timestamp, &action_str);

        Self {
            event_id,
            timestamp,
            event_type,
            user_id: None,
            username: None,
            action: action_str,
            resource: None,
            result,
            details: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            error_message: None,
            previous_value: None,
            new_value: None,
            checksum,
        }
    }

    /// 计算校验和
    fn calculate_checksum(event_id: &str, timestamp: &DateTime<Utc>, action: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(event_id.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(action.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证校验和
    pub fn verify_checksum(&self) -> bool {
        let expected = Self::calculate_checksum(&self.event_id, &self.timestamp, &self.action);
        self.checksum == expected
    }

    /// 设置用户信息
    pub fn with_user(mut self, user_id: impl Into<String>, username: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self.username = Some(username.into());
        self
    }

    /// 设置资源
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// 设置 IP 地址
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// 设置详情
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }

    /// 设置错误信息
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    /// 设置前一个值
    pub fn with_previous_value(mut self, value: Option<String>) -> Self {
        self.previous_value = value;
        self
    }

    /// 设置新的值
    pub fn with_new_value(mut self, value: String) -> Self {
        self.new_value = Some(value);
        self
    }
}

/// 生成 UUID v4 (使用 getrandom 实现)
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

/// 审计日志书写器
pub struct AuditLogWriter {
    /// 日志文件路径
    log_path: Option<String>,
    /// 内存缓冲
    buffer: Vec<AuditEvent>,
    /// 缓冲区大小
    buffer_size: usize,
}

impl Default for AuditLogWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogWriter {
    /// 创建新的审计日志书写器
    pub fn new() -> Self {
        Self {
            log_path: None,
            buffer: Vec::new(),
            buffer_size: 100,
        }
    }

    /// 设置日志文件路径
    pub fn with_log_path(mut self, path: impl Into<String>) -> Self {
        self.log_path = Some(path.into());
        self
    }

    /// 设置缓冲区大小
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// 记录审计事件
    pub fn log(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        // 验证校验和
        if !event.verify_checksum() {
            return Err(AuditError::ChecksumMismatch);
        }

        self.buffer.push(event);

        // 如果缓冲区满, 刷新到磁盘
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }

        Ok(())
    }

    /// 刷新缓冲区到磁盘
    pub fn flush(&mut self) -> Result<(), AuditError> {
        if let Some(ref path) = self.log_path {
            let json = serde_json::to_string_pretty(&self.buffer)
                .map_err(|e| AuditError::SerializationError(e.to_string()))?;

            std::fs::write(path, json).map_err(|e| AuditError::IoError(e.to_string()))?;
        }

        self.buffer.clear();
        Ok(())
    }

    /// 获取缓冲区中的事件数量
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }
}

/// 审计错误类型
#[derive(Debug, Clone)]
pub enum AuditError {
    /// 校验和不匹配
    ChecksumMismatch,
    /// 序列化错误
    SerializationError(String),
    /// IO 错误
    IoError(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::ChecksumMismatch => write!(f, "审计事件校验和不匹配"),
            AuditError::SerializationError(msg) => write!(f, "审计事件序列化失败: {}", msg),
            AuditError::IoError(msg) => write!(f, "审计日志 IO 错误: {}", msg),
        }
    }
}

impl std::error::Error for AuditError {}

/// 审计日志读取器
pub struct AuditLogReader {
    log_path: String,
}

impl AuditLogReader {
    /// 创建新的审计日志读取器
    pub fn new(log_path: impl Into<String>) -> Self {
        Self {
            log_path: log_path.into(),
        }
    }

    /// 读取所有审计事件
    pub fn read_all(&self) -> Result<Vec<AuditEvent>, AuditError> {
        let content = std::fs::read_to_string(&self.log_path)
            .map_err(|e| AuditError::IoError(e.to_string()))?;

        let events: Vec<AuditEvent> = serde_json::from_str(&content)
            .map_err(|e| AuditError::SerializationError(e.to_string()))?;

        Ok(events)
    }

    /// 验证所有事件的校验和
    pub fn verify_all(&self) -> Result<Vec<(AuditEvent, bool)>, AuditError> {
        let events = self.read_all()?;
        Ok(events
            .into_iter()
            .map(|e| (e.clone(), e.verify_checksum()))
            .collect())
    }
}

/// 便捷函数: 记录认证事件
pub fn log_authentication(
    user_id: &str,
    username: &str,
    result: AuditResult,
    ip: Option<&str>,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::Authentication,
        if result == AuditResult::Success {
            "user_login"
        } else {
            "user_login_failed"
        },
        result,
    )
    .with_user(user_id, username)
    .with_ip(ip.unwrap_or("unknown"))
}

/// 便捷函数: 记录命令执行事件
pub fn log_command_execution(user_id: &str, command: &str, result: AuditResult) -> AuditEvent {
    let mut details = HashMap::new();
    details.insert("command".to_string(), command.to_string());

    AuditEvent::new(AuditEventType::CommandExecution, "execute_command", result)
        .with_user(user_id, "system")
        .with_resource(command)
        .with_details(details)
}

/// 便捷函数: 记录配置变更事件
pub fn log_configuration_change(
    user_id: &str,
    config_key: &str,
    old_value: Option<&str>,
    new_value: &str,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::ConfigurationChange,
        "update_configuration",
        AuditResult::Success,
    )
    .with_user(user_id, "system")
    .with_resource(config_key)
    .with_previous_value(old_value.map(String::from))
    .with_new_value(new_value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "user_login",
            AuditResult::Success,
        );

        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.action, "user_login");
        assert!(event.verify_checksum());
    }

    #[test]
    fn test_audit_event_with_user() {
        let event = AuditEvent::new(
            AuditEventType::Authorization,
            "access_resource",
            AuditResult::Success,
        )
        .with_user("user123", "testuser")
        .with_ip("192.168.1.1");

        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.username, Some("testuser".to_string()));
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_checksum_verification() {
        let event = AuditEvent::new(
            AuditEventType::SystemEvent,
            "system_startup",
            AuditResult::Success,
        );

        assert!(event.verify_checksum());

        // 篡改后校验失败
        let mut tampered = event.clone();
        tampered.action = "tampered_action".to_string();
        assert!(!tampered.verify_checksum());
    }

    #[test]
    fn test_log_authentication() {
        let event = log_authentication("123", "admin", AuditResult::Success, Some("127.0.0.1"));
        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.result, AuditResult::Success);
    }

    #[test]
    fn test_log_command_execution() {
        let event = log_command_execution("123", "ls -la", AuditResult::Success);
        assert_eq!(event.event_type, AuditEventType::CommandExecution);
        assert!(event.details.is_some());
    }

    #[test]
    fn test_audit_log_writer() {
        let mut writer = AuditLogWriter::new().with_buffer_size(10);
        let event = AuditEvent::new(
            AuditEventType::NetworkEvent,
            "connection_opened",
            AuditResult::Success,
        );

        writer.log(event).unwrap();
        assert_eq!(writer.buffered_count(), 1);
    }
}
