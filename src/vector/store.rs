//! ArkCore VectorStore 模块
//!
//! 基于 SQLite 的向量存储和相似度搜索实现

use crate::vector::embedding::{Embedding, SimpleEmbedding};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{SqlitePool, Row, sqlite::SqlitePoolOptions};
use std::sync::Arc;

/// 文档数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 唯一标识符
    pub id: String,
    /// 文档内容
    pub content: String,
    /// 元数据（JSON 格式）
    pub metadata: Value,
    /// 创建时间（ISO 8601）
    pub created_at: String,
}

/// 带分数的搜索结果
#[derive(Debug, Clone)]
pub struct ScoredDocument {
    /// 文档
    pub document: Document,
    /// 相似度分数（余弦相似度）
    pub score: f32,
}

/// VectorStore - 基于 SQLite 的向量存储
///
/// 使用余弦相似度进行向量搜索
pub struct VectorStore {
    pool: SqlitePool,
    embedding: Arc<SimpleEmbedding>,
}

impl VectorStore {
    /// 创建新的 VectorStore 实例（内存数据库）
    pub async fn new() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await?;

        let embedding = Arc::new(SimpleEmbedding::default());
        let store = Self { pool, embedding };
        store.init_schema().await?;
        Ok(store)
    }

    /// 创建新的 VectorStore 实例（内存数据库，带自定义 embedding）
    pub async fn with_embedding(embedding: Arc<SimpleEmbedding>) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await?;

        let store = Self { pool, embedding };
        store.init_schema().await?;
        Ok(store)
    }

    /// 创建新的 VectorStore 实例（文件数据库）
    pub async fn from_file(path: &str) -> Result<Self> {
        Self::validate_path(path)?;

        let database_url = format!("sqlite:{}?mode=rwc", path);

        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .after_connect(|conn, _| {
                Box::pin(async move {
                    sqlx::query("PRAGMA journal_mode=WAL")
                        .execute(&mut *conn)
                        .await
                        .ok();
                    sqlx::query("PRAGMA synchronous=NORMAL")
                        .execute(&mut *conn)
                        .await
                        .ok();
                    Ok(())
                })
            })
            .connect(&database_url)
            .await?;

        let embedding = Arc::new(SimpleEmbedding::default());
        let store = Self { pool, embedding };
        store.init_schema().await?;
        Ok(store)
    }

    /// 验证路径安全
    fn validate_path(path: &str) -> Result<()> {
        if path.is_empty() {
            anyhow::bail!("路径不能为空");
        }
        if path.starts_with('/') || path.starts_with('\\') {
            anyhow::bail!("只允许相对路径，不允许绝对路径");
        }
        if path.contains("..") || path.contains("~") {
            anyhow::bail!("路径不允许包含 '..' 或 '~'");
        }
        if path.contains('\0') {
            anyhow::bail!("路径包含无效字符");
        }

        let normalized = std::path::Path::new(path)
            .components()
            .fold(String::new(), |acc, comp| {
                match comp {
                    std::path::Component::Normal(name) => {
                        if acc.is_empty() {
                            name.to_string_lossy().to_string()
                        } else {
                            format!("{}/{}", acc, name.to_string_lossy())
                        }
                    }
                    std::path::Component::ParentDir => acc,
                    _ => acc,
                }
            });

        if normalized.contains("..") {
            anyhow::bail!("检测到路径遍历尝试");
        }

        Ok(())
    }

    /// 初始化数据库 schema
    async fn init_schema(&self) -> Result<()> {
        let dimension = self.embedding.dimension();

        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{{}}',
                embedding BLOB NOT NULL,
                created_at TEXT NOT NULL
            )
            "#
        ))
        .execute(&self.pool)
        .await?;

        // 预计算向量范数
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vector_norms (
                id TEXT PRIMARY KEY,
                norm REAL NOT NULL,
                FOREIGN KEY (id) REFERENCES documents(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at)",
        )
        .execute(&self.pool)
        .await?;

        // 存储维度信息
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS vector_store_meta (key TEXT PRIMARY KEY, value TEXT)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO vector_store_meta (key, value) VALUES ('dimension', ?1)",
        )
        .bind(dimension.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 插入文档到向量存储
    pub async fn insert(&self, text: &str, metadata: Value) -> Result<()> {
        let id = uuid_v4();
        let embedding = self.embedding.embed(text);
        let created_at = chrono::Utc::now().to_rfc3339();

        let embedding_bytes = vector_to_bytes(&embedding);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        let metadata_json = serde_json::to_string(&metadata)?;
        sqlx::query(
            r#"
            INSERT INTO documents (id, content, metadata, embedding, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(text)
        .bind(&metadata_json)
        .bind(&embedding_bytes)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO vector_norms (id, norm) VALUES (?1, ?2)",
        )
        .bind(&id)
        .bind(norm)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 插入带 ID 的文档
    pub async fn insert_with_id(&self, id: &str, text: &str, metadata: Value) -> Result<()> {
        let embedding = self.embedding.embed(text);
        let created_at = chrono::Utc::now().to_rfc3339();

        let embedding_bytes = vector_to_bytes(&embedding);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        let metadata_json = serde_json::to_string(&metadata)?;
        sqlx::query(
            r#"
            INSERT INTO documents (id, content, metadata, embedding, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(id)
        .bind(text)
        .bind(&metadata_json)
        .bind(&embedding_bytes)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO vector_norms (id, norm) VALUES (?1, ?2)",
        )
        .bind(id)
        .bind(norm)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 搜索最相似的文档
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<ScoredDocument>> {
        let query_embedding = self.embedding.embed(query);
        let query_norm: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        // 查询所有文档
        let rows = sqlx::query("SELECT id, content, metadata, created_at FROM documents")
            .fetch_all(&self.pool)
            .await?;

        let mut scored: Vec<ScoredDocument> = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let content: String = row.get("content");
            let metadata_str: String = row.get("metadata");
            let created_at: String = row.get("created_at");
            let metadata: Value = serde_json::from_str(&metadata_str).unwrap_or(Value::Null);

            let embedding_bytes: Vec<u8> = sqlx::query_scalar(
                "SELECT embedding FROM documents WHERE id = ?1",
            )
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;

            let doc_embedding = bytes_to_vector(&embedding_bytes)?;
            let doc_norm: f32 = sqlx::query_scalar(
                "SELECT norm FROM vector_norms WHERE id = ?1",
            )
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;

            // 计算余弦相似度
            let dot_product: f32 = query_embedding
                .iter()
                .zip(doc_embedding.iter())
                .map(|(a, b)| a * b)
                .sum();

            let cosine_similarity = if query_norm > 0.0 && doc_norm > 0.0 {
                dot_product / (query_norm * doc_norm)
            } else {
                0.0
            };

            scored.push(ScoredDocument {
                document: Document {
                    id,
                    content,
                    metadata,
                    created_at,
                },
                score: cosine_similarity,
            });
        }

        // 按分数排序并取 top_k
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored)
    }

    /// 获取文档总数
    pub async fn count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// 获取所有文档
    pub async fn get_all(&self, limit: usize) -> Result<Vec<Document>> {
        let rows = sqlx::query("SELECT id, content, metadata, created_at FROM documents LIMIT ?1")
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let docs = rows
            .into_iter()
            .map(|row| {
                let metadata_str: String = row.get("metadata");
                Document {
                    id: row.get("id"),
                    content: row.get("content"),
                    metadata: serde_json::from_str(&metadata_str).unwrap_or(Value::Null),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(docs)
    }

    /// 根据 ID 获取文档
    pub async fn get(&self, id: &str) -> Result<Option<Document>> {
        let row = sqlx::query("SELECT id, content, metadata, created_at FROM documents WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let metadata_str: String = row.get("metadata");
                Ok(Some(Document {
                    id: row.get("id"),
                    content: row.get("content"),
                    metadata: serde_json::from_str(&metadata_str).unwrap_or(Value::Null),
                    created_at: row.get("created_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// 删除文档
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM vector_norms WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 清空所有文档
    pub async fn clear(&self) -> Result<()> {
        sqlx::query("DELETE FROM documents").execute(&self.pool).await?;
        sqlx::query("DELETE FROM vector_norms").execute(&self.pool).await?;
        Ok(())
    }
}

// 辅助函数：将 f32 向量转换为字节
fn vector_to_bytes(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for &v in vector {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

// 辅助函数：从字节转换为 f32 向量
fn bytes_to_vector(bytes: &[u8]) -> Result<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        anyhow::bail!("无效的向量字节长度");
    }

    let mut vector = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        vector.push(value);
    }
    Ok(vector)
}

// 生成 UUID v4
fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();

    let bytes = [
        bytes[0],
        bytes[1],
        bytes[2],
        (bytes[3] & 0x0f) | 0x40,
        (bytes[4] & 0x3f) | 0x80,
        bytes[5],
        bytes[6],
        bytes[7],
        (bytes[8] & 0x3f) | 0x80,
        bytes[9],
        bytes[10],
        bytes[11],
        (bytes[12] & 0x3f) | 0x80,
        bytes[13],
        bytes[14],
        bytes[15],
    ];

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_search() -> Result<()> {
        let store = VectorStore::new().await?;

        store
            .insert(
                "Rust is a systems programming language",
                serde_json::json!({"category": "programming"}),
            )
            .await?;

        store
            .insert(
                "Python is great for data science",
                serde_json::json!({"category": "data-science"}),
            )
            .await?;

        store
            .insert(
                "JavaScript is the language of the web",
                serde_json::json!({"category": "web"}),
            )
            .await?;

        let results = store.search("programming language", 2).await?;

        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);

        assert!(
            results[0].document.content.contains("programming")
                || results[0].document.content.contains("Rust")
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_search_ordering() -> Result<()> {
        let store = VectorStore::new().await?;

        store
            .insert("apple fruit", serde_json::json!({"type": "fruit"}))
            .await?;
        store
            .insert("banana fruit", serde_json::json!({"type": "fruit"}))
            .await?;
        store
            .insert("carrot vegetable", serde_json::json!({"type": "vegetable"}))
            .await?;

        let results = store.search("fruits", 3).await?;

        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_delete() -> Result<()> {
        let store = VectorStore::new().await?;

        store
            .insert("test document", serde_json::json!({"test": true}))
            .await?;

        let docs = store.get_all(10).await?;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].content, "test document");

        store.delete(&docs[0].id).await?;

        let count = store.count().await?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_clear() -> Result<()> {
        let store = VectorStore::new().await?;

        store.insert("doc1", serde_json::json!({})).await?;
        store.insert("doc2", serde_json::json!({})).await?;
        store.insert("doc3", serde_json::json!({})).await?;

        let count = store.count().await?;
        assert_eq!(count, 3);

        store.clear().await?;

        let count = store.count().await?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_with_id() -> Result<()> {
        let store = VectorStore::new().await?;

        store
            .insert_with_id(
                "custom-id-123",
                "document with custom id",
                serde_json::json!({"custom": true}),
            )
            .await?;

        let doc = store.get("custom-id-123").await?;
        assert!(doc.is_some());
        assert_eq!(doc.unwrap().content, "document with custom id");

        Ok(())
    }
}