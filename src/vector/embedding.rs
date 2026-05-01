//! ArkCore Vector Embedding 模块
//!
//! 提供文本嵌入的 trait 和基础实现

use std::sync::Arc;

/// Embedding 维度常量
pub const DEFAULT_EMBEDDING_DIM: usize = 384;

/// Embedding trait - 文本嵌入接口
///
/// 提供将文本转换为向量表示的能力
pub trait Embedding: Send + Sync {
    /// 获取嵌入向量维度
    fn dimension(&self) -> usize;

    /// 将文本嵌入为向量
    fn embed(&self, text: &str) -> Vec<f32>;

    /// 批量嵌入文本
    fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }
}

/// 简单的基于 hash 的 embedding 实现
///
/// 这是一个轻量级实现，适用于测试和不需要真正 ML 模型的场景。
/// 生成的向量具有确定性（相同文本产生相同向量）。
#[derive(Clone)]
pub struct SimpleEmbedding {
    dimension: usize,
}

impl SimpleEmbedding {
    /// 创建新的 SimpleEmbedding 实例
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// 创建默认维度的实例
    pub fn default() -> Self {
        Self::new(DEFAULT_EMBEDDING_DIM)
    }

    /// 使用 FNV hash 生成确定性向量
    fn hash_to_floats(&self, text: &str) -> Vec<f32> {
        use std::hash::{Hash, Hasher, FnvHasher};

        let mut hasher = FnvHasher::default();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut vector = Vec::with_capacity(self.dimension);
        let mut state = hash;

        for _ in 0..self.dimension {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let value = (state as f64) / (u64::MAX as f64);
            vector.push((value * 2.0 - 1.0) as f32);
        }

        // L2 归一化
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut vector {
                *v /= magnitude;
            }
        }

        vector
    }
}

impl Default for SimpleEmbedding {
    fn default() -> Self {
        Self::default()
    }
}

impl Embedding for SimpleEmbedding {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        self.hash_to_floats(text)
    }
}

/// 包装 Arc<dyn Embedding> 的便捷类型
pub type EmbeddingProvider = Arc<dyn Embedding>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_embedding() {
        let embedding = SimpleEmbedding::default();

        let vector1 = embedding.embed("hello world");
        let vector2 = embedding.embed("hello world");
        let vector3 = embedding.embed("different text");

        // 相同文本应产生相同向量
        assert_eq!(vector1, vector2);

        // 不同文本应产生不同向量
        assert_ne!(vector1, vector3);

        // 检查维度
        assert_eq!(vector1.len(), DEFAULT_EMBEDDING_DIM);

        // 检查是否归一化
        let magnitude: f32 = vector1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_dimension() {
        let embedding_384 = SimpleEmbedding::default();
        let embedding_128 = SimpleEmbedding::new(128);

        assert_eq!(embedding_384.dimension(), 384);
        assert_eq!(embedding_128.dimension(), 128);

        let vec_384 = embedding_384.embed("test");
        let vec_128 = embedding_128.embed("test");

        assert_eq!(vec_384.len(), 384);
        assert_eq!(vec_128.len(), 128);
    }

    #[test]
    fn test_batch_embedding() {
        let embedding = SimpleEmbedding::default();

        let texts = &["hello", "world", "test"];
        let results = embedding.embed_batch(texts);

        assert_eq!(results.len(), 3);
        for result in results {
            assert_eq!(result.len(), DEFAULT_EMBEDDING_DIM);
        }
    }
}