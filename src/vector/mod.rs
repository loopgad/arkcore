//! ArkCore Vector 模块 - 向量搜索与 RAG Pipeline
//!
//! 提供本地知识库问答的基础能力
//!
//! # 模块
//!
//! - [`embedding`] - Embedding trait 和基础实现
//! - [`store`] - 基于 SQLite 的向量存储
//! - [`rag`] - RAG Pipeline 实现
//!
//! # 示例
//!
//! ```ignore
//! use arkcore::vector::{VectorStore, SimpleEmbedding, RagPipeline};
//! use std::sync::Arc;
//!
//! // 创建 store（使用默认 embedding）
//! let store = VectorStore::new().await?;
//!
//! // 添加文档
//! store.insert("Rust 是一种系统编程语言", serde_json::json!({})).await?;
//!
//! // 创建 RAG pipeline
//! let rag = RagPipeline::new(Arc::new(store));
//! let result = rag.retrieve("Rust 是什么？").await?;
//! ```

pub mod embedding;
pub mod rag;
pub mod store;

// 重新导出常用类型
pub use embedding::{Embedding, EmbeddingProvider, SimpleEmbedding, DEFAULT_EMBEDDING_DIM};
pub use rag::{RagConfig, RagIngestor, RagPipeline, RagRetrievalResult};
pub use store::{Document, ScoredDocument, VectorStore};