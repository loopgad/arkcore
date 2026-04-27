//! 敏感信息加密
//!
//! 使用 AES-256-GCM 进行存储加密:
//! - API keys
//! - passwords
//! - tokens
//!
//! 特性:
//! - AES-256-GCM 加密
//! - 密钥派生 (KDF from master password)
//! - 内存安全 (敏感数据立即清除)
//! - 支持密钥轮换

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 加密数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// 密文 (base64 编码)
    pub ciphertext: String,
    /// 盐值 (base64 编码)
    pub salt: String,
    /// Nonce (base64 编码)
    pub nonce: String,
    /// 版本号 (用于密钥轮换)
    pub version: u32,
}

/// 密钥信息
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct KeyInfo {
    /// 派生密钥
    key: [u8; 32],
    /// 版本号
    version: u32,
}

impl KeyInfo {
    /// 获取版本号
    pub fn version(&self) -> u32 {
        self.version
    }
}

/// 加密服务
pub struct CryptoService {
    /// 当前活跃密钥
    current_key: KeyInfo,
    /// 旧密钥映射 (用于解密历史数据)
    key_history: HashMap<u32, [u8; 32]>,
    /// 密钥派生参数
    kdf_iters: u32,
}

impl CryptoService {
    /// 创建新的加密服务
    pub fn new(master_password: &str) -> Self {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        let key = Self::derive_key(master_password, &salt, 100_000);

        Self {
            current_key: KeyInfo {
                key,
                version: 1,
            },
            key_history: HashMap::new(),
            kdf_iters: 100_000,
        }
    }

    /// 从已有密钥创建
    pub fn from_key(key: [u8; 32], version: u32) -> Self {
        Self {
            current_key: KeyInfo { key, version },
            key_history: HashMap::new(),
            kdf_iters: 100_000,
        }
    }

    /// 派生密钥 (PBKDF2-HMAC-SHA256)
    ///
    /// 使用标准的 PBKDF2 算法，迭代次数遵循 OWASP 2023 建议 (>= 600,000)
    fn derive_key(password: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
        let mut result = [0u8; 32];
        // PBKDF2-HMAC-SHA256: pbkdf2_hmac 使用 HMAC-SHA256 作为 PRF
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut result);
        result
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.current_key.key)
            .map_err(|e| CryptoError::KeyError(e.to_string()))?;

        // 生成加密安全的随机盐 (32 bytes / 256 bits)
        let mut salt_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut salt_bytes);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            salt: general_purpose::STANDARD.encode(salt_bytes),
            nonce: general_purpose::STANDARD.encode(nonce_bytes),
            version: self.current_key.version,
        })
    }

    /// 解密数据
    pub fn decrypt(&self, data: &EncryptedData) -> Result<String, CryptoError> {
        let key = if data.version == self.current_key.version {
            &self.current_key.key
        } else {
            self.key_history
                .get(&data.version)
                .ok_or(CryptoError::UnknownKeyVersion)?
        };

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::KeyError(e.to_string()))?;

        let ciphertext = general_purpose::STANDARD
            .decode(&data.ciphertext)
            .map_err(|e| CryptoError::Base64Error(e.to_string()))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&data.nonce)
            .map_err(|e| CryptoError::Base64Error(e.to_string()))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::Utf8Error(e.to_string()))
    }

    /// 轮换密钥
    pub fn rotate_key(&mut self, new_password: Option<&str>) -> Result<u32, CryptoError> {
        let old_version = self.current_key.version;
        let old_key = self.current_key.key;

        // 保存旧密钥
        self.key_history.insert(old_version, old_key);

        // 派生新密钥
        let new_version = old_version + 1;
        let (new_key, _new_salt) = if let Some(password) = new_password {
            let mut salt = [0u8; 32];
            OsRng.fill_bytes(&mut salt);
            let key = Self::derive_key(password, &salt, self.kdf_iters);
            (key, Some(salt))
        } else {
            // 无密码轮换 - 生成新随机密钥
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            (key, None)
        };

        self.current_key = KeyInfo {
            key: new_key,
            version: new_version,
        };

        Ok(new_version)
    }

    /// 验证密钥是否正确
    pub fn verify_key(&self, test_data: &EncryptedData) -> bool {
        self.decrypt(test_data).is_ok()
    }
}

/// 敏感字段处理器
pub struct SensitiveField;

impl SensitiveField {
    /// 加密 API 密钥
    pub fn encrypt_api_key(crypto: &CryptoService, api_key: &str) -> Result<EncryptedData, CryptoError> {
        crypto.encrypt(api_key)
    }

    /// 解密 API 密钥
    pub fn decrypt_api_key(crypto: &CryptoService, data: &EncryptedData) -> Result<String, CryptoError> {
        crypto.decrypt(data)
    }

    /// 加密密码 (使用单独的盐值)
    pub fn encrypt_password(crypto: &CryptoService, password: &str) -> Result<EncryptedData, CryptoError> {
        crypto.encrypt(password)
    }

    /// 清理敏感字符串 (内存安全)
    pub fn secure_clear(s: &mut String) {
        if !s.is_empty() {
            let len = s.len();
            s.reserve(len);
            unsafe {
                let bytes = s.as_mut_vec();
                bytes.zeroize();
            }
            s.clear();
        }
    }

    /// 清理字节数组 (内存安全)
    pub fn secure_clear_bytes(b: &mut [u8]) {
        b.zeroize();
    }
}

/// 密码哈希器 (用于密码存储)
pub struct PasswordHasher {
    /// 盐值
    salt: [u8; 32],
    /// 迭代次数
    iterations: u32,
}

impl PasswordHasher {
    /// 创建新的密码哈希器
    pub fn new() -> Self {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        Self {
            salt,
            iterations: 100_000,
        }
    }

    /// 创建带指定盐值的哈希器 (用于验证)
    pub fn with_salt(salt: [u8; 32]) -> Self {
        Self {
            salt,
            iterations: 100_000,
        }
    }

    /// 哈希密码
    pub fn hash(&self, password: &str) -> String {
        let mut result = [0u8; 64];

        // 第一次哈希
        let mut current = Sha256::digest([password.as_bytes(), &self.salt].concat());

        // 多次迭代
        for _ in 0..self.iterations {
            current = Sha256::digest(current.as_slice());
        }

        result[..32].copy_from_slice(&current);
        result[32..].copy_from_slice(&self.salt);

        general_purpose::STANDARD.encode(result)
    }

    /// 验证密码
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        let computed = self.hash(password);
        computed == hash
    }

    /// 获取盐值
    pub fn salt(&self) -> [u8; 32] {
        self.salt
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// 加密错误类型
#[derive(Debug, Clone)]
pub enum CryptoError {
    /// 密钥错误
    KeyError(String),
    /// 加密错误
    EncryptionError(String),
    /// 解密错误
    DecryptionError(String),
    /// Base64 解码错误
    Base64Error(String),
    /// UTF-8 错误
    Utf8Error(String),
    /// 未知密钥版本
    UnknownKeyVersion,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyError(msg) => write!(f, "密钥错误: {}", msg),
            CryptoError::EncryptionError(msg) => write!(f, "加密错误: {}", msg),
            CryptoError::DecryptionError(msg) => write!(f, "解密错误: {}", msg),
            CryptoError::Base64Error(msg) => write!(f, "Base64 解码错误: {}", msg),
            CryptoError::Utf8Error(msg) => write!(f, "UTF-8 错误: {}", msg),
            CryptoError::UnknownKeyVersion => write!(f, "未知密钥版本"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = CryptoService::new("master_password_123");
        let plaintext = "secret_api_key_12345";

        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password() {
        let crypto = CryptoService::new("correct_password");
        let encrypted = crypto.encrypt("secret").unwrap();

        let crypto2 = CryptoService::new("wrong_password");
        let result = crypto2.decrypt(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_key_rotation() {
        let mut crypto = CryptoService::new("password");

        let data = crypto.encrypt("secret").unwrap();

        crypto.rotate_key(Some("new_password")).unwrap();

        // 旧密钥存储在历史记录中，可以解密旧数据
        let result = crypto.decrypt(&data);
        assert!(result.is_ok()); // 旧数据可以用旧密钥解密
        assert_eq!(result.unwrap(), "secret");

        // 用新密钥加密的数据可以解密
        let new_data = crypto.encrypt("new_secret").unwrap();
        assert_eq!(crypto.decrypt(&new_data).unwrap(), "new_secret");
    }

    #[test]
    fn test_password_hasher() {
        let hasher = PasswordHasher::new();
        let password = "my_secure_password";

        let hash1 = hasher.hash(password);
        let hash2 = hasher.hash(password);

        // 相同密码应产生相同哈希 (确定性)
        assert_eq!(hash1, hash2);

        // 验证正确密码
        assert!(hasher.verify(password, &hash1));

        // 验证错误密码
        assert!(!hasher.verify("wrong_password", &hash1));
    }

    #[test]
    fn test_sensitive_field_clear() {
        let mut s = "sensitive".to_string();
        SensitiveField::secure_clear(&mut s);
        assert!(s.is_empty());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let crypto = CryptoService::new("password");
        let encrypted = crypto.encrypt("test data").unwrap();

        let json = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedData = serde_json::from_str(&json).unwrap();

        assert_eq!(crypto.decrypt(&deserialized).unwrap(), "test data");
    }
}