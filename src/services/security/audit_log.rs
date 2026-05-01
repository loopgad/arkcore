//! 防篡改审计日志模块
//!
//! 使用 HMAC-SHA256 签名确保审计日志的完整性和不可篡改性。
//!
//! # 特性
//!
//! - HMAC-SHA256 签名机制
//! - 追加式日志写入 (Append-Only)
//! - 链式校验 (每个条目包含前一条的哈希)
//! - 完整日志验证
//!
//! # 安全保证
//!
//! 1. **完整性**: 每个日志条目都带有 HMAC 签名
//! 2. **不可篡改**: 任何修改都会导致签名验证失败
//! 3. **顺序保证**: 链式哈希确保日志顺序不可修改
//! 4. **追加模式**: 只允许追加，不允许修改或删除

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

/// 日志头魔数
const LOG_MAGIC: &[u8; 8] = b"ARKLOG01";
/// 日志版本
const LOG_VERSION: u8 = 1;

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// 序列号
    pub seq: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 事件类型
    pub event_type: String,
    /// 详细信息
    pub details: Value,
    /// 前一条目哈希
    pub prev_hash: [u8; 32],
    /// HMAC 签名
    pub signature: [u8; 32],
}

/// 已验证的日志条目
#[derive(Debug, Clone)]
pub struct VerifiedEntry {
    /// 原始条目
    pub entry: AuditEntry,
    /// 验证结果
    pub valid: bool,
    /// 验证错误信息
    pub error: Option<String>,
}

/// 防篡改追加式日志
pub struct AppendOnlyLog {
    file: File,
    hmac_key: [u8; 32],
    path: std::path::PathBuf,
}

impl AppendOnlyLog {
    /// 创建新的追加式日志
    ///
    /// # 参数
    ///
    /// * `path` - 日志文件路径
    /// * `key` - HMAC 密钥 (32 字节)
    ///
    /// # 返回
    ///
    /// 成功返回日志实例，失败返回错误
    pub fn new(path: &Path, key: [u8; 32]) -> Result<Self, AuditLogError> {
        let path = path.to_path_buf();

        // 检查文件是否存在
        let is_new = !path.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(false) // 只允许追加
            .open(&path)
            .map_err(|e| AuditLogError::IoError(format!("打开日志文件失败: {}", e)))?;

        let mut log = Self {
            file,
            hmac_key: key,
            path,
        };

        // 如果是新文件，写入头部
        if is_new {
            log.write_header()?;
        } else {
            // 如果是已存在的文件，验证头部格式
            log.read_header()?;
        }

        Ok(log)
    }

    /// 写入日志头部
    fn write_header(&mut self) -> Result<(), AuditLogError> {
        let mut writer = BufWriter::new(&mut self.file);

        // 写入魔数
        writer
            .write_all(LOG_MAGIC)
            .map_err(|e| AuditLogError::IoError(format!("写入魔数失败: {}", e)))?;

        // 写入版本
        writer
            .write_all(&[LOG_VERSION])
            .map_err(|e| AuditLogError::IoError(format!("写入版本失败: {}", e)))?;

        // 写入初始哈希 (全零)
        let init_hash = [0u8; 32];
        writer
            .write_all(&init_hash)
            .map_err(|e| AuditLogError::IoError(format!("写入初始哈希失败: {}", e)))?;

        writer
            .flush()
            .map_err(|e| AuditLogError::IoError(format!("刷新写入失败: {}", e)))?;

        Ok(())
    }

    /// 读取日志头部
    fn read_header(&mut self) -> Result<[u8; 32], AuditLogError> {
        use std::io::Seek;

        self.file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(|e| AuditLogError::IoError(format!(" seek失败: {}", e)))?;

        let mut reader = BufReader::new(&mut self.file);

        // 读取并验证魔数
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| AuditLogError::IoError(format!("读取魔数失败: {}", e)))?;

        if &magic != LOG_MAGIC {
            return Err(AuditLogError::InvalidFormat("无效的日志魔数".to_string()));
        }

        // 读取版本
        let mut version = [0u8; 1];
        reader
            .read_exact(&mut version)
            .map_err(|e| AuditLogError::IoError(format!("读取版本失败: {}", e)))?;

        if version[0] != LOG_VERSION {
            return Err(AuditLogError::InvalidFormat(format!(
                "不支持的日志版本: {}",
                version[0]
            )));
        }

        // 读取初始哈希
        let mut last_hash = [0u8; 32];
        reader
            .read_exact(&mut last_hash)
            .map_err(|e| AuditLogError::IoError(format!("读取初始哈希失败: {}", e)))?;

        Ok(last_hash)
    }

    /// 计算条目的 HMAC 签名
    fn compute_signature(&self, entry: &AuditEntry) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC 可以从任意长度密钥创建");

        // 包含序列号
        mac.update(&entry.seq.to_le_bytes());

        // 包含时间戳
        mac.update(entry.timestamp.to_rfc3339().as_bytes());

        // 包含事件类型
        mac.update(entry.event_type.as_bytes());

        // 包含详情
        let details_json = serde_json::to_string(&entry.details).unwrap_or_default();
        mac.update(details_json.as_bytes());

        // 包含前一条哈希
        mac.update(&entry.prev_hash);

        let result = mac.finalize();
        let mut signature = [0u8; 32];
        signature.copy_from_slice(&result.into_bytes());
        signature
    }

    /// 计算条目的内容哈希 (不含签名)
    fn compute_content_hash(entry: &AuditEntry) -> [u8; 32] {
        use sha2::Digest;

        let mut hasher = Sha256::new();

        hasher.update(entry.seq.to_le_bytes());
        hasher.update(entry.timestamp.to_rfc3339().as_bytes());
        hasher.update(entry.event_type.as_bytes());

        let details_json = serde_json::to_string(&entry.details).unwrap_or_default();
        hasher.update(details_json.as_bytes());

        hasher.update(entry.prev_hash);

        hasher.finalize().into()
    }

    /// 获取最后一条记录的序列号和哈希
    fn get_last_entry_info(&mut self) -> Result<(u64, [u8; 32]), AuditLogError> {
        use std::io::Seek;

        // 头部占 41 字节 (8 魔数 + 1 版本 + 32 初始哈希)
        let header_size: u64 = 41;

        // 获取文件大小
        let file_size = self
            .file
            .metadata()
            .map_err(|e| AuditLogError::IoError(format!("获取文件元数据失败: {}", e)))?
            .len();

        if file_size <= header_size {
            return Ok((0, [0u8; 32]));
        }

        // 定位到文件末尾
        self.file
            .seek(std::io::SeekFrom::End(0))
            .map_err(|e| AuditLogError::IoError(format!(" seek失败: {}", e)))?;

        // 读取最后一条记录
        // 日志条目格式: 4字节长度 + 内容 (JSON)
        // 我们需要找到最后一条记录

        let mut reader = BufReader::new(&mut self.file);

        // 从头部之后开始搜索
        reader
            .seek(std::io::SeekFrom::Start(header_size))
            .map_err(|e| AuditLogError::IoError(format!(" seek失败: {}", e)))?;

        let mut last_seq = 0u64;
        let mut last_hash = [0u8; 32];

        loop {
            // 读取长度前缀 (4字节)
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(AuditLogError::IoError(format!("读取长度失败: {}", e))),
            }

            let entry_len = u32::from_le_bytes(len_buf) as usize;

            // 读取内容
            let mut entry_buf = vec![0u8; entry_len];
            reader
                .read_exact(&mut entry_buf)
                .map_err(|e| AuditLogError::IoError(format!("读取条目失败: {}", e)))?;

            // 解析条目
            if let Ok(entry) = serde_json::from_slice::<AuditEntry>(&entry_buf) {
                last_seq = entry.seq;
                last_hash = Self::compute_content_hash(&entry);
            }
        }

        Ok((last_seq, last_hash))
    }

    /// 追加新条目
    ///
    /// # 参数
    ///
    /// * `entry` - 审计条目
    ///
    /// # 返回
    ///
    /// 成功返回 (), 失败返回错误
    pub fn append(&mut self, entry: &AuditEntry) -> Result<(), AuditLogError> {
        // 获取最后一条信息
        let (last_seq, last_hash) = self.get_last_entry_info()?;

        // 验证序列号连续性
        if entry.seq != last_seq + 1 {
            return Err(AuditLogError::SequenceMismatch {
                expected: last_seq + 1,
                actual: entry.seq,
            });
        }

        // 验证前一条哈希
        if entry.prev_hash != last_hash {
            return Err(AuditLogError::HashMismatch {
                expected: last_hash,
                actual: entry.prev_hash,
            });
        }

        // 计算并设置签名
        let signature = self.compute_signature(entry);
        let mut signed_entry = entry.clone();
        signed_entry.signature = signature;

        // 序列化
        let json = serde_json::to_string(&signed_entry)
            .map_err(|e| AuditLogError::SerializationError(e.to_string()))?;

        // 写入长度前缀 + 内容
        let mut writer = BufWriter::new(&mut self.file);

        let len = json.len() as u32;
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| AuditLogError::IoError(format!("写入长度失败: {}", e)))?;

        writer
            .write_all(json.as_bytes())
            .map_err(|e| AuditLogError::IoError(format!("写入内容失败: {}", e)))?;

        writer
            .flush()
            .map_err(|e| AuditLogError::IoError(format!("刷新失败: {}", e)))?;

        Ok(())
    }

    /// 创建并追加新条目
    ///
    /// # 参数
    ///
    /// * `event_type` - 事件类型
    /// * `details` - 详细信息
    ///
    /// # 返回
    ///
    /// 成功返回创建的条目，失败返回错误
    pub fn create_and_append(
        &mut self,
        event_type: impl Into<String>,
        details: Value,
    ) -> Result<AuditEntry, AuditLogError> {
        // 获取最后一条信息
        let (last_seq, last_hash) = self.get_last_entry_info()?;

        let entry = AuditEntry {
            seq: last_seq + 1,
            timestamp: Utc::now(),
            event_type: event_type.into(),
            details,
            prev_hash: last_hash,
            signature: [0u8; 32], // 临时值，append 时会被替换
        };

        self.append(&entry)?;

        // 返回带有正确签名的条目
        Ok(AuditEntry {
            signature: self.compute_signature(&entry),
            ..entry
        })
    }

    /// 验证所有日志条目
    ///
    /// # 返回
    ///
    /// 返回所有条目的验证结果
    pub fn verify(&self) -> Result<Vec<VerifiedEntry>, AuditLogError> {
        let path = self.path.clone();
        let key = self.hmac_key;

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| AuditLogError::IoError(format!("打开日志文件失败: {}", e)))?;

        let mut reader = BufReader::new(file);

        // 读取并验证头部
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| AuditLogError::IoError(format!("读取魔数失败: {}", e)))?;

        if &magic != LOG_MAGIC {
            return Err(AuditLogError::InvalidFormat("无效的日志魔数".to_string()));
        }

        let mut version = [0u8; 1];
        reader
            .read_exact(&mut version)
            .map_err(|e| AuditLogError::IoError(format!("读取版本失败: {}", e)))?;

        if version[0] != LOG_VERSION {
            return Err(AuditLogError::InvalidFormat(format!(
                "不支持的日志版本: {}",
                version[0]
            )));
        }

        let mut prev_hash = [0u8; 32];
        reader
            .read_exact(&mut prev_hash)
            .map_err(|e| AuditLogError::IoError(format!("读取初始哈希失败: {}", e)))?;

        let mut results = Vec::new();
        let mut expected_seq = 1u64;

        loop {
            // 读取长度前缀
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(AuditLogError::IoError(format!("读取长度失败: {}", e))),
            }

            let entry_len = u32::from_le_bytes(len_buf) as usize;

            // 读取内容
            let mut entry_buf = vec![0u8; entry_len];
            reader
                .read_exact(&mut entry_buf)
                .map_err(|e| AuditLogError::IoError(format!("读取条目失败: {}", e)))?;

            // 解析条目
            let entry: AuditEntry = match serde_json::from_slice(&entry_buf) {
                Ok(e) => e,
                Err(e) => {
                    results.push(VerifiedEntry {
                        entry: AuditEntry {
                            seq: expected_seq,
                            timestamp: Utc::now(),
                            event_type: "parse_error".to_string(),
                            details: Value::Null,
                            prev_hash: [0u8; 32],
                            signature: [0u8; 32],
                        },
                        valid: false,
                        error: Some(format!("JSON 解析失败: {}", e)),
                    });
                    expected_seq += 1;
                    continue;
                }
            };

            // 验证序列号
            if entry.seq != expected_seq {
                results.push(VerifiedEntry {
                    entry: entry.clone(),
                    valid: false,
                    error: Some(format!(
                        "序列号不连续: 期望 {}, 实际 {}",
                        expected_seq, entry.seq
                    )),
                });
                expected_seq = entry.seq + 1;
                continue;
            }

            // 验证前一条哈希
            if entry.prev_hash != prev_hash {
                results.push(VerifiedEntry {
                    entry: entry.clone(),
                    valid: false,
                    error: Some("前一条哈希不匹配".to_string()),
                });
                expected_seq += 1;
                continue;
            }

            // 重新计算签名并验证
            let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC 可以从任意长度密钥创建");

            mac.update(&entry.seq.to_le_bytes());
            mac.update(entry.timestamp.to_rfc3339().as_bytes());
            mac.update(entry.event_type.as_bytes());

            let details_json = serde_json::to_string(&entry.details).unwrap_or_default();
            mac.update(details_json.as_bytes());

            mac.update(&entry.prev_hash);

            let expected_signature = mac.finalize();
            let mut expected_sig = [0u8; 32];
            expected_sig.copy_from_slice(&expected_signature.into_bytes());

            if entry.signature != expected_sig {
                results.push(VerifiedEntry {
                    entry: entry.clone(),
                    valid: false,
                    error: Some("HMAC 签名验证失败".to_string()),
                });
                expected_seq += 1;
                continue;
            }

            // 更新 prev_hash
            let mut hasher = Sha256::new();
            hasher.update(entry.seq.to_le_bytes());
            hasher.update(entry.timestamp.to_rfc3339().as_bytes());
            hasher.update(entry.event_type.as_bytes());
            hasher.update(details_json.as_bytes());
            hasher.update(entry.prev_hash);
            prev_hash = hasher.finalize().into();

            results.push(VerifiedEntry {
                entry,
                valid: true,
                error: None,
            });

            expected_seq += 1;
        }

        Ok(results)
    }

    /// 获取日志路径
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// 审计日志错误类型
#[derive(Debug)]
pub enum AuditLogError {
    /// IO 错误
    IoError(String),
    /// 序列化错误
    SerializationError(String),
    /// 无效格式
    InvalidFormat(String),
    /// 序列号不匹配
    SequenceMismatch { expected: u64, actual: u64 },
    /// 哈希不匹配
    HashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
}

impl std::fmt::Display for AuditLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLogError::IoError(msg) => write!(f, "IO 错误: {}", msg),
            AuditLogError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            AuditLogError::InvalidFormat(msg) => write!(f, "无效格式: {}", msg),
            AuditLogError::SequenceMismatch { expected, actual } => {
                write!(f, "序列号不匹配: 期望 {}, 实际 {}", expected, actual)
            }
            AuditLogError::HashMismatch { .. } => {
                write!(f, "哈希不匹配: 日志链断裂")
            }
        }
    }
}

impl std::error::Error for AuditLogError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_append() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key = [0u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key).unwrap();

        let entry = log
            .create_and_append("test_event", serde_json::json!({"key": "value"}))
            .unwrap();

        assert_eq!(entry.seq, 1);
        assert_eq!(entry.event_type, "test_event");
    }

    #[test]
    fn test_signature_verification() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key = [0u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key).unwrap();

        log.create_and_append("test_event", serde_json::json!({"key": "value"}))
            .unwrap();

        // 重新打开日志并验证
        let log2 = AppendOnlyLog::new(&log_path, key).unwrap();
        let results = log2.verify().unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].valid);
    }

    #[test]
    fn test_tamper_detection() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key = [0u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key).unwrap();

        log.create_and_append("test_event", serde_json::json!({"key": "value"}))
            .unwrap();

        // 篡改文件内容 - 写入任意内容会破坏格式
        std::fs::write(&log_path, b"tampered content").unwrap();

        // 重新打开应该失败，因为格式被破坏
        let result = AppendOnlyLog::new(&log_path, key);
        assert!(
            result.is_err(),
            "Expected error when opening corrupted log file"
        );
    }

    #[test]
    fn test_wrong_key_verification() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key1).unwrap();

        log.create_and_append("test_event", serde_json::json!({"key": "value"}))
            .unwrap();

        // 使用不同的密钥验证
        let log2 = AppendOnlyLog::new(&log_path, key2).unwrap();
        let results = log2.verify().unwrap();

        assert!(!results[0].valid);
        assert!(results[0].error.is_some());
    }

    #[test]
    fn test_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key = [0u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key).unwrap();

        for i in 0..5 {
            log.create_and_append(format!("event_{}", i), serde_json::json!({"index": i}))
                .unwrap();
        }

        let log2 = AppendOnlyLog::new(&log_path, key).unwrap();
        let results = log2.verify().unwrap();

        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.valid));
    }

    #[test]
    fn test_entry_sequential_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let key = [0u8; 32];

        let mut log = AppendOnlyLog::new(&log_path, key).unwrap();

        log.create_and_append("event_1", serde_json::json!({"seq": 1}))
            .unwrap();

        log.create_and_append("event_2", serde_json::json!({"seq": 2}))
            .unwrap();

        // 读取并验证链式哈希
        let log2 = AppendOnlyLog::new(&log_path, key).unwrap();
        let results = log2.verify().unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].valid);
        assert!(results[1].valid);

        // 验证序列号
        assert_eq!(results[0].entry.seq, 1);
        assert_eq!(results[1].entry.seq, 2);
    }
}
