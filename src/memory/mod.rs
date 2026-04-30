//! ArkCore Memory 模块 - SQLite FTS5 记忆引擎
//!
//! 基于 FTS5 的技能记忆存储与检索系统 (sqlx 异步版本)

use sqlx::{SqlitePool, FromRow, sqlite::SqlitePoolOptions};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 技能数据结构
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    /// 唯一标识符
    pub id: String,
    /// 技能名称
    pub skill_name: String,
    /// 技能分类
    pub category: String,
    /// 技能摘要
    pub summary: String,
    /// 技能详情（可选）
    pub details: Option<String>,
    /// 关键词（逗号分隔）
    pub keywords: String,
    /// 重要性权重
    pub importance: f64,
    /// 成功次数
    pub success_count: i32,
    /// 访问次数
    pub access_count: i32,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 更新时间（ISO 8601）
    pub updated_at: String,
}

/// SQLite FTS5 记忆引擎 (sqlx 异步版本)
#[derive(Clone)]
pub struct SkillMemory {
    pool: SqlitePool,
}

impl SkillMemory {
    /// 创建新的记忆引擎实例（内存数据库）
    pub async fn new() -> Result<Self> {
        // sqlx 不直接支持内存数据库，我们使用文件数据库 + 快速删除
        // 或者使用 :memory: 连接字符串（需要 runtime-tokio-native-tls）
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await?;

        let memory = Self { pool };
        memory.init_schema().await?;
        Ok(memory)
    }

    /// 创建新的记忆引擎实例（文件数据库）
    pub async fn from_file(path: &str) -> Result<Self> {
        // 验证路径安全，防止路径遍历攻击
        Self::validate_path(path)?;

        let database_url = format!("sqlite:{}?mode=rwc", path);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        let memory = Self { pool };
        memory.init_schema().await?;
        Ok(memory)
    }

    /// 验证路径安全，防止路径遍历攻击
    fn validate_path(path: &str) -> Result<()> {
        // 禁止空路径
        if path.is_empty() {
            anyhow::bail!("路径不能为空");
        }

        // 禁止绝对路径（安全考虑，只允许相对路径）
        if path.starts_with('/') || path.starts_with('\\') {
            anyhow::bail!("只允许相对路径，不允许绝对路径");
        }

        // 禁止路径遍历序列
        if path.contains("..") || path.contains("~") {
            anyhow::bail!("路径不允许包含 '..' 或 '~'");
        }

        // 禁止 null 字节
        if path.contains('\0') {
            anyhow::bail!("路径包含无效字符");
        }

        // 解析并规范化路径，检查是否产生遍历
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
                    std::path::Component::ParentDir => {
                        // 检测到 .. 说明有路径遍历
                        acc
                    }
                    _ => acc,
                }
            });

        // 如果规范化后的路径包含 ..，说明有遍历尝试
        if normalized.contains("..") {
            anyhow::bail!("检测到路径遍历尝试");
        }

        Ok(())
    }

    /// 初始化数据库 schema
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            -- 技能记忆主表
            CREATE TABLE IF NOT EXISTS skill_memories (
                id TEXT PRIMARY KEY,
                skill_name TEXT NOT NULL,
                category TEXT NOT NULL,
                summary TEXT NOT NULL,
                details TEXT,
                keywords TEXT NOT NULL DEFAULT '',
                importance REAL DEFAULT 1.0,
                success_count INTEGER DEFAULT 1,
                access_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS skill_memories_fts USING fts5(
                skill_name,
                summary,
                details,
                keywords,
                content='skill_memories',
                content_rowid='rowid',
                tokenize='porter unicode61'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 插入触发器
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS skill_memories_ai AFTER INSERT ON skill_memories BEGIN
                INSERT INTO skill_memories_fts(rowid, skill_name, summary, details, keywords)
                VALUES (NEW.rowid, NEW.skill_name, NEW.summary, NEW.details, NEW.keywords);
            END
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 删除触发器
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS skill_memories_ad AFTER DELETE ON skill_memories BEGIN
                INSERT INTO skill_memories_fts(skill_memories_fts, rowid, skill_name, summary, details, keywords)
                VALUES ('delete', OLD.rowid, OLD.skill_name, OLD.summary, OLD.details, OLD.keywords);
            END
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 更新触发器
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS skill_memories_au AFTER UPDATE ON skill_memories BEGIN
                INSERT INTO skill_memories_fts(skill_memories_fts, rowid, skill_name, summary, details, keywords)
                VALUES ('delete', OLD.rowid, OLD.skill_name, OLD.summary, OLD.details, OLD.keywords);
                INSERT INTO skill_memories_fts(rowid, skill_name, summary, details, keywords)
                VALUES (NEW.rowid, NEW.skill_name, NEW.summary, NEW.details, NEW.keywords);
            END
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 存储技能到记忆
    pub async fn store_skill(&self, skill: &Skill) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO skill_memories
            (id, skill_name, category, summary, details, keywords, importance, success_count, access_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&skill.id)
        .bind(&skill.skill_name)
        .bind(&skill.category)
        .bind(&skill.summary)
        .bind(&skill.details)
        .bind(&skill.keywords)
        .bind(skill.importance)
        .bind(skill.success_count)
        .bind(skill.access_count)
        .bind(&skill.created_at)
        .bind(&skill.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 清理 FTS5 查询字符串，防止查询注入
    ///
    /// FTS5 有自己的查询语言，包含 AND、OR、NOT 等操作符。
    /// 如果用户输入包含这些字符，可能会影响查询行为。
    /// 此函数移除 FTS5 特殊字符，只保留安全的文本内容。
    fn sanitize_fts_query(query: &str) -> String {
        // FTS5 特殊字符和操作符
        const FTS5_SPECIAL_CHARS: &[char] = &[
            '(', ')', '*', ':', '^', '-', '+', '~', '"',
            'A', 'N', 'O', // AND, NOT, OR 的首字母会被移除
        ];

        let mut result = String::with_capacity(query.len());
        for c in query.chars() {
            if FTS5_SPECIAL_CHARS.contains(&c) {
                // 跳过 FTS5 特殊字符
                // 将 AND, NOT, OR 替换为空格以保留单词分隔
                if c == 'A' || c == 'N' || c == 'O' {
                    result.push(' ');
                }
                continue;
            }
            result.push(c);
        }

        // 折叠多余空格
        let mut folded = String::with_capacity(result.len());
        let mut last_was_space = false;
        for c in result.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    folded.push(' ');
                    last_was_space = true;
                }
            } else {
                folded.push(c);
                last_was_space = false;
            }
        }

        folded.trim().to_string()
    }

    /// 使用 FTS5 进行全文搜索
    ///
    /// # Security
    ///
    /// 使用参数化查询防止 SQL 注入，并对 FTS5 特殊字符进行清理
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Skill>> {
        // 清理 FTS5 特殊字符，防止 FTS5 查询注入
        let sanitized = Self::sanitize_fts_query(query);
        let fts_query = format!("\"{}\"", sanitized.replace("\"", "\"\""));

        let skills = sqlx::query_as::<_, Skill>(
            r#"
            SELECT m.id, m.skill_name, m.category, m.summary, m.details,
                   m.keywords, m.importance, m.success_count, m.access_count,
                   m.created_at, m.updated_at
            FROM skill_memories m
            INNER JOIN skill_memories_fts fts ON m.rowid = fts.rowid
            WHERE skill_memories_fts MATCH ?1
            ORDER BY bm25(skill_memories_fts) DESC
            LIMIT ?2
            "#,
        )
        .bind(&fts_query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(skills)
    }

    /// 更新技能访问计数
    pub async fn increment_access(&self, skill_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE skill_memories SET access_count = access_count + 1 WHERE id = ?1",
        )
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新技能成功计数
    pub async fn increment_success(&self, skill_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE skill_memories SET success_count = success_count + 1 WHERE id = ?1",
        )
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据 ID 获取技能
    pub async fn get_skill(&self, skill_id: &str) -> Result<Option<Skill>> {
        let skill = sqlx::query_as::<_, Skill>(
            r#"
            SELECT id, skill_name, category, summary, details, keywords,
                   importance, success_count, access_count, created_at, updated_at
            FROM skill_memories
            WHERE id = ?1
            "#,
        )
        .bind(skill_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(skill)
    }

    /// 删除技能
    pub async fn delete_skill(&self, skill_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM skill_memories WHERE id = ?1",
        )
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取所有技能（按访问次数排序）
    pub async fn get_all_skills(&self, limit: usize) -> Result<Vec<Skill>> {
        let skills = sqlx::query_as::<_, Skill>(
            r#"
            SELECT id, skill_name, category, summary, details, keywords,
                   importance, success_count, access_count, created_at, updated_at
            FROM skill_memories
            ORDER BY access_count DESC, importance DESC
            LIMIT ?1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_search() -> Result<()> {
        let memory = SkillMemory::new().await?;

        let skill = Skill {
            id: "test-001".to_string(),
            skill_name: "Rust Programming".to_string(),
            category: "Programming".to_string(),
            summary: "Systems programming language".to_string(),
            details: Some("Fast, safe, concurrent".to_string()),
            keywords: "rust, systems, programming".to_string(),
            importance: 1.0,
            success_count: 0,
            access_count: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        memory.store_skill(&skill).await?;

        let results = memory.search("Rust", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill_name, "Rust Programming");

        Ok(())
    }

    #[tokio::test]
    async fn test_search_keywords() -> Result<()> {
        let memory = SkillMemory::new().await?;

        let skill = Skill {
            id: "test-002".to_string(),
            skill_name: "Web Development".to_string(),
            category: "Programming".to_string(),
            summary: "Build websites and web apps".to_string(),
            details: Some("HTML, CSS, JavaScript".to_string()),
            keywords: "web, frontend, javascript".to_string(),
            importance: 1.0,
            success_count: 0,
            access_count: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        memory.store_skill(&skill).await?;

        let results = memory.search("javascript", 10).await?;
        assert_eq!(results.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_access_count() -> Result<()> {
        let memory = SkillMemory::new().await?;

        let skill = Skill {
            id: "test-003".to_string(),
            skill_name: "Test Skill".to_string(),
            category: "Testing".to_string(),
            summary: "A test skill".to_string(),
            details: None,
            keywords: "test".to_string(),
            importance: 1.0,
            success_count: 0,
            access_count: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        memory.store_skill(&skill).await?;
        memory.increment_access("test-003").await?;
        memory.increment_access("test-003").await?;

        let retrieved = memory.get_skill("test-003").await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().access_count, 2);

        Ok(())
    }
}