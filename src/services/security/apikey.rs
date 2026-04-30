//! API 密钥管理
//!
//! 功能:
//! - API 密钥生成 (secure random)
//! - 密钥轮换机制
//! - 作用域限制
//! - 密钥存储加密
//! - API 密钥认证中间件

use aes_gcm::aead::OsRng;
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;

use super::crypto::{CryptoService, EncryptedData};
use super::rbac::RbacService;

/// API 密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    /// 密钥 ID
    pub key_id: String,
    /// 密钥名称
    pub name: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后使用时间
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 作用域
    pub scopes: Vec<String>,
    /// 是否启用
    pub enabled: bool,
    /// 密钥哈希
    key_hash: String,
}

/// API 密钥请求
#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// 密钥名称
    pub name: String,
    /// 作用域
    pub scopes: Vec<String>,
    /// 过期天数 (None = 永不过期)
    pub expires_in_days: Option<u32>,
    /// 密钥前缀 (用于识别)
    pub prefix: Option<String>,
}

/// API 密钥响应 (创建时返回)
#[derive(Debug, Clone)]
pub struct ApiKeyResponse {
    /// 密钥 ID
    pub key_id: String,
    /// 完整 API 密钥 (仅创建时返回)
    pub api_key: String,
    /// 密钥名称
    pub name: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 密钥使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsage {
    /// 密钥 ID
    pub key_id: String,
    /// 使用时间
    pub used_at: chrono::DateTime<chrono::Utc>,
    /// 请求路径
    pub path: String,
    /// 请求方法
    pub method: String,
    /// 结果
    pub result: String,
    /// IP 地址
    pub ip_address: Option<String>,
}

/// API 密钥认证结果
#[derive(Debug, Clone)]
pub struct ApiKeyAuthResult {
    /// 是否认证成功
    pub success: bool,
    /// 密钥信息
    pub key_info: Option<ApiKeyInfo>,
    /// 错误信息
    pub error: Option<String>,
}

impl ApiKeyAuthResult {
    /// 认证成功
    pub fn success(key_info: ApiKeyInfo) -> Self {
        Self {
            success: true,
            key_info: Some(key_info),
            error: None,
        }
    }

    /// 认证失败
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            key_info: None,
            error: Some(error.into()),
        }
    }
}

/// API 密钥服务
pub struct ApiKeyService {
    /// 密钥存储 (key_id -> encrypted data)
    keys: RwLock<HashMap<String, EncryptedData>>,
    /// 密钥信息 (key_id -> info)
    key_infos: RwLock<HashMap<String, ApiKeyInfo>>,
    /// 使用记录
    usage_records: RwLock<Vec<ApiKeyUsage>>,
    /// 加密服务
    crypto: CryptoService,
    /// RBAC 服务
    #[allow(dead_code)]
    rbac: std::sync::Arc<RbacService>,
    /// 密钥前缀
    key_prefix: String,
}

impl ApiKeyService {
    /// 创建新的 API 密钥服务
    pub fn new(crypto: CryptoService, rbac: std::sync::Arc<RbacService>) -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            key_infos: RwLock::new(HashMap::new()),
            usage_records: RwLock::new(Vec::new()),
            crypto,
            rbac,
            key_prefix: "ak_".to_string(),
        }
    }

    /// 生成随机密钥
    fn generate_key(&self, prefix: &str) -> String {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);

        let key = general_purpose::URL_SAFE_NO_PAD.encode(bytes);

        format!("{}{}", prefix, key)
    }

    /// 计算密钥哈希
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// 创建 API 密钥
    pub fn create(&self, request: CreateApiKeyRequest) -> Result<ApiKeyResponse, ApiKeyError> {
        // 生成密钥
        let prefix = request.prefix.unwrap_or_else(|| self.key_prefix.clone());
        let api_key = self.generate_key(&prefix);
        let key_hash = self.hash_key(&api_key);

        // 生成密钥 ID
        let mut id_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut id_bytes);
        let key_id = format!("{:02x}{:02x}{:02x}{:02x}", id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]);

        let now = chrono::Utc::now();
        let expires_at = request.expires_in_days.map(|days| now + chrono::Duration::days(days as i64));

        // 加密密钥
        let encrypted_key = self.crypto.encrypt(&api_key).map_err(|e| ApiKeyError::CryptoError(e.to_string()))?;

        // 创建密钥信息
        let key_info = ApiKeyInfo {
            key_id: key_id.clone(),
            name: request.name.clone(),
            created_at: now,
            expires_at,
            last_used_at: None,
            scopes: request.scopes.clone(),
            enabled: true,
            key_hash,
        };

        // 存储
        {
            let mut keys = self.keys.write()
                .map_err(|_| ApiKeyError::Internal("Key storage lock poisoned".to_string()))?;
            keys.insert(key_id.clone(), encrypted_key);
        }

        {
            let mut infos = self.key_infos.write()
                .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
            infos.insert(key_id.clone(), key_info);
        }

        Ok(ApiKeyResponse {
            key_id,
            api_key,
            name: request.name,
            created_at: now,
            expires_at,
        })
    }

    /// 验证 API 密钥
    pub fn authenticate(&self, api_key: &str) -> ApiKeyAuthResult {
        // 计算哈希
        let key_hash = self.hash_key(api_key);

        // 查找密钥
        let infos = match self.key_infos.read() {
            Ok(infos) => infos,
            Err(_) => return ApiKeyAuthResult::failure("Key info lock poisoned"),
        };
        let key_info = match infos.values().find(|k| k.key_hash == key_hash) {
            Some(info) => info.clone(),
            None => return ApiKeyAuthResult::failure("无效的 API 密钥"),
        };

        // 检查是否启用
        if !key_info.enabled {
            return ApiKeyAuthResult::failure("API 密钥已被禁用");
        }

        // 检查是否过期
        if let Some(expires) = key_info.expires_at {
            if expires < chrono::Utc::now() {
                return ApiKeyAuthResult::failure("API 密钥已过期");
            }
        }

        ApiKeyAuthResult::success(key_info)
    }

    /// 验证作用域
    pub fn validate_scope(&self, key_info: &ApiKeyInfo, required_scope: &str) -> bool {
        // 空作用域表示无权限 (安全默认值)
        if key_info.scopes.is_empty() {
            return false;
        }

        key_info.scopes.iter().any(|s| s == required_scope || s == "*")
    }

    /// 记录使用
    pub fn record_usage(&self, key_id: &str, path: &str, method: &str, result: &str, ip: Option<&str>) -> Result<(), ApiKeyError> {
        let record = ApiKeyUsage {
            key_id: key_id.to_string(),
            used_at: chrono::Utc::now(),
            path: path.to_string(),
            method: method.to_string(),
            result: result.to_string(),
            ip_address: ip.map(String::from),
        };

        let mut records = self.usage_records.write()
            .map_err(|_| ApiKeyError::Internal("Usage records lock poisoned".to_string()))?;
        records.push(record);

        // 更新最后使用时间
        let mut infos = self.key_infos.write()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
        if let Some(info) = infos.get_mut(key_id) {
            info.last_used_at = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// 撤销密钥
    pub fn revoke(&self, key_id: &str) -> Result<(), ApiKeyError> {
        let mut keys = self.keys.write()
            .map_err(|_| ApiKeyError::Internal("Key storage lock poisoned".to_string()))?;
        let mut infos = self.key_infos.write()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;

        keys.remove(key_id);
        infos.remove(key_id);

        Ok(())
    }

    /// 禁用密钥
    pub fn disable(&self, key_id: &str) -> Result<(), ApiKeyError> {
        let mut infos = self.key_infos.write()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
        if let Some(info) = infos.get_mut(key_id) {
            info.enabled = false;
            Ok(())
        } else {
            Err(ApiKeyError::KeyNotFound)
        }
    }

    /// 启用密钥
    pub fn enable(&self, key_id: &str) -> Result<(), ApiKeyError> {
        let mut infos = self.key_infos.write()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
        if let Some(info) = infos.get_mut(key_id) {
            info.enabled = true;
            Ok(())
        } else {
            Err(ApiKeyError::KeyNotFound)
        }
    }

    /// 轮换密钥
    pub fn rotate(&self, key_id: &str) -> Result<ApiKeyResponse, ApiKeyError> {
        let infos = self.key_infos.read()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
        let old_info = infos.get(key_id).ok_or(ApiKeyError::KeyNotFound)?.clone();

        // 创建新密钥
        self.create(CreateApiKeyRequest {
            name: old_info.name,
            scopes: old_info.scopes,
            expires_in_days: old_info.expires_at.map(|e| {
                (e - chrono::Utc::now()).num_days() as u32
            }),
            prefix: Some("ak_rotated_".to_string()),
        })
    }

    /// 获取所有密钥信息 (不含实际密钥)
    pub fn list_keys(&self) -> Result<Vec<ApiKeyInfo>, ApiKeyError> {
        let infos = self.key_infos.read()
            .map_err(|_| ApiKeyError::Internal("Key info lock poisoned".to_string()))?;
        Ok(infos.values().cloned().collect())
    }

    /// 获取密钥使用记录
    pub fn get_usage_history(&self, key_id: &str, limit: usize) -> Result<Vec<ApiKeyUsage>, ApiKeyError> {
        let records = self.usage_records.read()
            .map_err(|_| ApiKeyError::Internal("Usage records lock poisoned".to_string()))?;
        Ok(records
            .iter()
            .filter(|r| r.key_id == key_id)
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
}

/// API 密钥错误类型
#[derive(Debug, Clone)]
pub enum ApiKeyError {
    /// 密钥不存在
    KeyNotFound,
    /// 密钥已过期
    KeyExpired,
    /// 密钥已禁用
    KeyDisabled,
    /// 加密错误
    CryptoError(String),
    /// 无效的作用域
    InvalidScope,
    /// 内部错误 (如锁中毒)
    Internal(String),
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::KeyNotFound => write!(f, "API 密钥不存在"),
            ApiKeyError::KeyExpired => write!(f, "API 密钥已过期"),
            ApiKeyError::KeyDisabled => write!(f, "API 密钥已被禁用"),
            ApiKeyError::CryptoError(msg) => write!(f, "加密错误: {}", msg),
            ApiKeyError::InvalidScope => write!(f, "无效的作用域"),
            ApiKeyError::Internal(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for ApiKeyError {}

/// API 密钥认证中间件
pub struct ApiKeyMiddleware {
    service: std::sync::Arc<ApiKeyService>,
}

impl ApiKeyMiddleware {
    /// 创建新的 API 密钥中间件
    pub fn new(service: std::sync::Arc<ApiKeyService>) -> Self {
        Self { service }
    }

    /// 认证请求
    pub fn authenticate_request(&self, api_key: &str, path: &str, method: &str) -> ApiKeyAuthResult {
        let result = self.service.authenticate(api_key);

        if !result.success {
            return result;
        }

        let key_info = result.key_info.as_ref().unwrap();

        // 验证作用域
        if !self.service.validate_scope(key_info, path) && !self.service.validate_scope(key_info, method) {
            return ApiKeyAuthResult::failure("无权访问此资源");
        }

        // 记录使用
        let _ = self.service.record_usage(&key_info.key_id, path, method, "success", None);

        result
    }
}

/// 从请求头提取 API 密钥
pub fn extract_api_key_from_header(header_value: &str) -> Option<String> {
    // 支持 "Bearer <key>" 或直接 "<key>"
    if header_value.starts_with("Bearer ") || header_value.starts_with("bearer ") {
        Some(header_value[7..].trim().to_string())
    } else {
        Some(header_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 生成随机测试密码 (避免硬编码)
    fn generate_test_password() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[idx] as char
            })
            .collect();
        format!("test_pwd_{}", chars)
    }

    fn create_test_crypto() -> CryptoService {
        CryptoService::new(&generate_test_password())
    }

    fn create_test_rbac() -> std::sync::Arc<RbacService> {
        std::sync::Arc::new(RbacService::new())
    }

    #[test]
    fn test_create_api_key() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec!["read".to_string()],
                expires_in_days: Some(30),
                prefix: Some("test_".to_string()),
            })
            .unwrap();

        assert!(response.api_key.starts_with("test_"));
        assert_eq!(response.name, "Test Key");
    }

    #[test]
    fn test_authenticate_valid_key() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec!["read".to_string()],
                expires_in_days: None,
                prefix: None,
            })
            .unwrap();

        let result = service.authenticate(&response.api_key);
        assert!(result.success);
        assert!(result.key_info.is_some());
    }

    #[test]
    fn test_authenticate_invalid_key() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let result = service.authenticate("invalid_key");
        assert!(!result.success);
    }

    #[test]
    fn test_scope_validation() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec!["read".to_string(), "write".to_string()],
                expires_in_days: None,
                prefix: None,
            })
            .unwrap();

        let result = service.authenticate(&response.api_key).key_info.unwrap();

        assert!(service.validate_scope(&result, "read"));
        assert!(service.validate_scope(&result, "write"));
        assert!(!service.validate_scope(&result, "admin"));
    }

    #[test]
    fn test_revoke_key() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec![],
                expires_in_days: None,
                prefix: None,
            })
            .unwrap();

        service.revoke(&response.key_id).unwrap();

        let result = service.authenticate(&response.api_key);
        assert!(!result.success);
    }

    #[test]
    fn test_disable_enable_key() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec![],
                expires_in_days: None,
                prefix: None,
            })
            .unwrap();

        service.disable(&response.key_id).unwrap();
        let result = service.authenticate(&response.api_key);
        assert!(!result.success);

        service.enable(&response.key_id).unwrap();
        let result = service.authenticate(&response.api_key);
        assert!(result.success);
    }

    #[test]
    fn test_extract_api_key_from_header() {
        assert_eq!(
            extract_api_key_from_header("Bearer my_api_key"),
            Some("my_api_key".to_string())
        );
        assert_eq!(
            extract_api_key_from_header("my_api_key"),
            Some("my_api_key".to_string())
        );
    }

    #[test]
    fn test_record_usage() {
        let crypto = create_test_crypto();
        let rbac = create_test_rbac();
        let service = ApiKeyService::new(crypto, rbac);

        let response = service
            .create(CreateApiKeyRequest {
                name: "Test Key".to_string(),
                scopes: vec![],
                expires_in_days: None,
                prefix: None,
            })
            .unwrap();

        service.record_usage(&response.key_id, "/api/test", "GET", "success", Some("127.0.0.1")).unwrap();

        let history = service.get_usage_history(&response.key_id, 10).unwrap();
        assert!(!history.is_empty());
        assert_eq!(history[0].path, "/api/test");
    }
}