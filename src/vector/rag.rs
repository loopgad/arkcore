//! ArkCore RAG Pipeline 模块
//!
//! Retrieval-Augmented Generation 流水线实现

use crate::vector::store::{Document, ScoredDocument, VectorStore};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// RAG 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// 检索返回的最大文档数
    pub top_k: usize,
    /// 最小相似度分数阈值（0-1）
    pub min_score: f32,
    /// 上下文窗口大小（字符数）
    pub context_window: usize,
    /// 是否包含元数据到提示
    pub include_metadata: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            min_score: 0.0,
            context_window: 4000,
            include_metadata: false,
        }
    }
}

/// RAG 检索结果
#[derive(Debug, Clone)]
pub struct RagRetrievalResult {
    /// 检索到的文档
    pub documents: Vec<ScoredDocument>,
    /// 聚合的上下文文本
    pub context: String,
    /// 使用的配置
    pub config: RagConfig,
}

/// RAG Pipeline - 检索增强生成流水线
///
/// 将文档检索与上下文组装结合，为 LLM 提供知识库上下文
#[derive(Clone)]
pub struct RagPipeline {
    store: Arc<VectorStore>,
    config: RagConfig,
}

impl RagPipeline {
    /// 创建新的 RAG Pipeline
    pub fn new(store: Arc<VectorStore>) -> Self {
        Self {
            store,
            config: RagConfig::default(),
        }
    }

    /// 创建带配置的 RAG Pipeline
    pub fn with_config(store: Arc<VectorStore>, config: RagConfig) -> Self {
        Self { store, config }
    }

    /// 检索相关文档
    pub async fn retrieve(&self, query: &str) -> Result<RagRetrievalResult> {
        let documents = self.store.search(query, self.config.top_k).await?;

        // 过滤低于最小分数的文档
        let filtered_docs: Vec<ScoredDocument> = documents
            .into_iter()
            .filter(|doc| doc.score >= self.config.min_score)
            .collect();

        // 构建上下文
        let context = self.build_context(&filtered_docs);

        Ok(RagRetrievalResult {
            documents: filtered_docs,
            context,
            config: self.config.clone(),
        })
    }

    /// 构建上下文字符串
    fn build_context(&self, documents: &[ScoredDocument]) -> String {
        let mut context = String::new();
        context.push_str("=== 知识库上下文 ===\n\n");

        for (i, doc) in documents.iter().enumerate() {
            let truncated_content = Self::truncate_text(
                &doc.document.content,
                self.config.context_window / documents.len().max(1),
            );

            context.push_str(&format!("[文档 {}] (相似度: {:.4})\n", i + 1, doc.score));

            if self.config.include_metadata {
                context.push_str(&format!("元数据: {}\n", doc.document.metadata));
            }

            context.push_str(&format!("内容: {}\n\n", truncated_content));
        }

        context.push_str("=== 上下文结束 ===");
        context
    }

    /// 截断文本到指定长度
    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            // 在单词边界处截断
            let truncated = &text[..max_len];
            if let Some(last_space) = truncated.rfind(' ') {
                format!("{}...", &truncated[..last_space])
            } else {
                format!("{}...", truncated)
            }
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: RagConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn config(&self) -> &RagConfig {
        &self.config
    }

    /// 生成提示词（将检索结果格式化为 LLM 提示）
    pub fn generate_prompt(&self, query: &str, retrieval: &RagRetrievalResult) -> String {
        format!(
            r#"基于以下知识库上下文回答问题。如果上下文中没有相关信息，请说明不知道。

问题: {}

{}

请根据上下文回答问题。"#,
            query, retrieval.context
        )
    }
}

/// 添加文档到 RAG Pipeline 的知识库
pub struct RagIngestor {
    store: Arc<VectorStore>,
}

impl RagIngestor {
    /// 创建新的 ingestor
    pub fn new(store: Arc<VectorStore>) -> Self {
        Self { store }
    }

    /// 添加单个文档
    pub async fn add_document(&self, content: &str, metadata: serde_json::Value) -> Result<()> {
        self.store.insert(content, metadata).await
    }

    /// 添加带 ID 的文档
    pub async fn add_document_with_id(
        &self,
        id: &str,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<()> {
        self.store.insert_with_id(id, content, metadata).await
    }

    /// 批量添加文档
    pub async fn add_documents(&self, documents: Vec<(&str, serde_json::Value)>) -> Result<()> {
        for (content, metadata) in documents {
            self.add_document(content, metadata).await?;
        }
        Ok(())
    }

    /// 获取文档总数
    pub async fn count(&self) -> Result<i64> {
        self.store.count().await
    }

    /// 清空知识库
    pub async fn clear(&self) -> Result<()> {
        self.store.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> Arc<VectorStore> {
        Arc::new(
            futures::executor::block_on(VectorStore::with_embedding(Arc::new(crate::vector::embedding::SimpleEmbedding::default()))).unwrap(),
        )
    }

    #[tokio::test]
    async fn test_rag_retrieve() -> Result<()> {
        let store = create_test_store();

        // 添加测试文档
        store
            .insert(
                "Rust 是一种系统编程语言，以安全性和并发性著称",
                serde_json::json!({"category": "programming", "language": "rust"}),
            )
            .await?;

        store
            .insert(
                "Python 是一种高级编程语言，广泛用于数据科学和机器学习",
                serde_json::json!({"category": "programming", "language": "python"}),
            )
            .await?;

        store
            .insert(
                "TypeScript 是 JavaScript 的超集，添加了类型系统",
                serde_json::json!({"category": "programming", "language": "typescript"}),
            )
            .await?;

        let pipeline = RagPipeline::new(store);
        let result = pipeline.retrieve("Rust 编程语言").await?;

        assert!(!result.documents.is_empty());
        assert!(!result.context.is_empty());
        assert!(result.context.contains("Rust"));

        Ok(())
    }

    #[tokio::test]
    async fn test_rag_config() -> Result<()> {
        let store = create_test_store();

        store
            .insert(
                "文档1 内容",
                serde_json::json!({"id": 1}),
            )
            .await?;

        store
            .insert(
                "文档2 内容",
                serde_json::json!({"id": 2}),
            )
            .await?;

        let config = RagConfig {
            top_k: 1,
            min_score: 0.0,
            context_window: 1000,
            include_metadata: true,
        };

        let pipeline = RagPipeline::with_config(store, config);
        let result = pipeline.retrieve("文档").await?;

        assert_eq!(result.documents.len(), 1);
        assert!(result.context.contains("元数据"));

        Ok(())
    }

    #[tokio::test]
    async fn test_rag_generate_prompt() -> Result<()> {
        let store = create_test_store();

        store
            .insert(
                "水的沸点是100摄氏度",
                serde_json::json!({"fact": "boiling point"}),
            )
            .await?;

        let pipeline = RagPipeline::new(store);
        let result = pipeline.retrieve("水的沸点").await?;
        let prompt = pipeline.generate_prompt("水的沸点是多少？", &result);

        assert!(prompt.contains("水的沸点是多少？"));
        assert!(prompt.contains("知识库上下文"));
        assert!(prompt.contains("100摄氏度"));

        Ok(())
    }

    #[tokio::test]
    async fn test_rag_ingestor() -> Result<()> {
        let store = create_test_store();
        let ingestor = RagIngestor::new(store.clone());

        ingestor
            .add_document("测试文档内容", serde_json::json!({"test": true}))
            .await?;

        let count = ingestor.count().await?;
        assert_eq!(count, 1);

        // 批量添加
        ingestor
            .add_documents(vec![
                ("文档1", serde_json::json!({"n": 1})),
                ("文档2", serde_json::json!({"n": 2})),
            ])
            .await?;

        let count = ingestor.count().await?;
        assert_eq!(count, 3);

        // 清空
        ingestor.clear().await?;
        let count = ingestor.count().await?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_truncate_text() -> Result<()> {
        let store = create_test_store();

        store
            .insert(
                "短文本",
                serde_json::json!({}),
            )
            .await?;

        let pipeline = RagPipeline::new(store);
        let result = pipeline.retrieve("短文本").await?;

        // 短文本不应被截断
        assert!(result.context.contains("短文本"));

        Ok(())
    }

    #[tokio::test]
    async fn test_min_score_filter() -> Result<()> {
        let store = create_test_store();

        store
            .insert(
                "完全不相关的内容 xyz",
                serde_json::json!({}),
            )
            .await?;

        let config = RagConfig {
            top_k: 5,
            min_score: 0.5, // 高阈值，只保留相似度 > 0.5 的
            context_window: 4000,
            include_metadata: false,
        };

        let pipeline = RagPipeline::with_config(store, config);
        let result = pipeline.retrieve("完全不相关的内容 xyz").await?;

        // 可能没有任何结果，因为 SimpleEmbedding 的相似度可能较低
        // 这个测试主要验证过滤逻辑正常工作
        assert!(result.documents.iter().all(|d| d.score >= 0.5));

        Ok(())
    }
}