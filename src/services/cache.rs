//! 缓存层模块
//!
//! 提供多级缓存支持：LLM响应缓存、搜索结果缓存、配置缓存

use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::time::Duration;
use tokio::sync::RwLock;

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// LLM 响应缓存 TTL
    pub llm_ttl: Duration,
    /// FTS5 搜索结果缓存 TTL
    pub search_ttl: Duration,
    /// 配置缓存 TTL
    pub config_ttl: Duration,
    /// 最大容量
    pub max_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            llm_ttl: Duration::from_secs(300),      // 5 分钟
            search_ttl: Duration::from_secs(60),    // 1 分钟
            config_ttl: Duration::from_secs(3600),  // 1 小时
            max_capacity: 10_000,
        }
    }
}

impl CacheConfig {
    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            llm_ttl: Duration::from_secs(300),
            search_ttl: Duration::from_secs(60),
            config_ttl: Duration::from_secs(3600),
            max_capacity: 10_000,
        }
    }

    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            llm_ttl: Duration::from_secs(60),
            search_ttl: Duration::from_secs(30),
            config_ttl: Duration::from_secs(300),
            max_capacity: 1_000,
        }
    }
}

/// LLM 响应缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LlmCacheKey {
    pub model: String,
    pub prompt_hash: u64,
}

impl LlmCacheKey {
    /// 从模型名和提示词创建缓存键
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            prompt_hash: simple_hash(prompt),
        }
    }
}

/// FTS5 搜索缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SearchCacheKey {
    pub query: String,
    pub limit: usize,
}

impl SearchCacheKey {
    /// 从查询和限制创建缓存键
    pub fn new(query: &str, limit: usize) -> Self {
        Self {
            query: query.to_string(),
            limit,
        }
    }
}

/// 配置缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ConfigCacheKey {
    /// 应用配置
    App,
    /// 数据库配置
    Database,
    /// LLM 配置
    Llm,
    /// 安全配置
    Security,
    /// 平台配置
    Platform,
    /// 自定义键
    Custom(String),
}

impl ConfigCacheKey {
    /// 创建自定义配置键
    pub fn custom(key: &str) -> Self {
        Self::Custom(key.to_string())
    }
}

/// LLM 响应缓存（使用默认 hasher）
pub type LlmResponseCache = Cache<LlmCacheKey, LlmCacheEntry>;

/// LLM 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheEntry {
    pub response: String,
    pub model: String,
    pub created_at: std::time::SystemTime,
}

/// FTS5 搜索结果缓存（使用默认 hasher）
pub type SearchResultCache = Cache<SearchCacheKey, SearchCacheEntry>;

/// 搜索结果缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCacheEntry {
    pub results: Vec<SearchResult>,
    pub created_at: std::time::SystemTime,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
}

/// 配置缓存（使用默认 hasher）
pub type ConfigCache = Cache<ConfigCacheKey, ConfigCacheEntry>;

/// 配置缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCacheEntry {
    pub value: String,
    pub created_at: std::time::SystemTime,
}

/// 简单哈希函数（用于缓存键）
fn simple_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// 缓存管理器
pub struct CacheManager {
    #[allow(dead_code)]
    config: CacheConfig,
    llm_cache: LlmResponseCache,
    search_cache: SearchResultCache,
    config_cache: ConfigCache,
    /// 缓存命中率统计
    hits: RwLock<u64>,
    /// 缓存未命中统计
    misses: RwLock<u64>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        let llm_cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.llm_ttl)
            .build();

        let search_cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.search_ttl)
            .build();

        let config_cache = Cache::builder()
            .max_capacity(100) // 配置缓存较小
            .time_to_live(config.config_ttl)
            .build();

        Self {
            config,
            llm_cache,
            search_cache,
            config_cache,
            hits: RwLock::new(0),
            misses: RwLock::new(0),
        }
    }

    /// 获取 LLM 响应缓存
    pub fn llm_cache(&self) -> &LlmResponseCache {
        &self.llm_cache
    }

    /// 获取搜索结果缓存
    pub fn search_cache(&self) -> &SearchResultCache {
        &self.search_cache
    }

    /// 获取配置缓存
    pub fn config_cache(&self) -> &ConfigCache {
        &self.config_cache
    }

    /// 获取 LLM 响应
    pub fn get_llm_response(&self, key: &LlmCacheKey) -> Option<LlmCacheEntry> {
        let entry = self.llm_cache.get(key);
        if entry.is_some() {
            // 记录命中
        }
        entry
    }

    /// 设置 LLM 响应
    pub fn set_llm_response(&self, key: LlmCacheKey, response: String, model: String) {
        let entry = LlmCacheEntry {
            response,
            model,
            created_at: std::time::SystemTime::now(),
        };
        self.llm_cache.insert(key, entry);
    }

    /// 获取搜索结果
    pub fn get_search_results(&self, key: &SearchCacheKey) -> Option<SearchCacheEntry> {
        self.search_cache.get(key)
    }

    /// 设置搜索结果
    pub fn set_search_results(&self, key: SearchCacheKey, results: Vec<SearchResult>) {
        let entry = SearchCacheEntry {
            results,
            created_at: std::time::SystemTime::now(),
        };
        self.search_cache.insert(key, entry);
    }

    /// 获取配置
    pub fn get_config(&self, key: &ConfigCacheKey) -> Option<ConfigCacheEntry> {
        self.config_cache.get(key)
    }

    /// 设置配置
    pub fn set_config(&self, key: ConfigCacheKey, value: String) {
        let entry = ConfigCacheEntry {
            value,
            created_at: std::time::SystemTime::now(),
        };
        self.config_cache.insert(key, entry);
    }

    /// 使缓存失效
    pub fn invalidate_llm(&self) {
        self.llm_cache.invalidate_all();
    }

    /// 使搜索缓存失效
    pub fn invalidate_search(&self) {
        self.search_cache.invalidate_all();
    }

    /// 使配置缓存失效
    pub fn invalidate_config(&self) {
        self.config_cache.invalidate_all();
    }

    /// 获取缓存统计
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            llm_cache_size: self.llm_cache.entry_count(),
            search_cache_size: self.search_cache.entry_count(),
            config_cache_size: self.config_cache.entry_count(),
            hits: *self.hits.read().await,
            misses: *self.misses.read().await,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub llm_cache_size: u64,
    pub search_cache_size: u64,
    pub config_cache_size: u64,
    pub hits: u64,
    pub misses: u64,
}

impl CacheStats {
    /// 计算总命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// 创建全局缓存实例（单例）
pub fn create_global_cache() -> CacheManager {
    CacheManager::new(CacheConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.llm_ttl, Duration::from_secs(300));
        assert_eq!(config.search_ttl, Duration::from_secs(60));
        assert_eq!(config.config_ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_llm_cache_key() {
        let key1 = LlmCacheKey::new("gpt-4", "Hello");
        let key2 = LlmCacheKey::new("gpt-4", "Hello");
        let key3 = LlmCacheKey::new("gpt-4", "World");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_search_cache_key() {
        let key1 = SearchCacheKey::new("rust", 10);
        let key2 = SearchCacheKey::new("rust", 10);
        let key3 = SearchCacheKey::new("golang", 10);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let manager = CacheManager::new(CacheConfig::default());

        // 测试 LLM 缓存
        let key = LlmCacheKey::new("gpt-4", "Hello world");
        manager.set_llm_response(key.clone(), "Hi there!".to_string(), "gpt-4".to_string());
        let entry = manager.get_llm_response(&key);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().response, "Hi there!");

        // 测试搜索缓存
        let search_key = SearchCacheKey::new("rust async", 5);
        let results = vec![SearchResult {
            id: "1".to_string(),
            score: 0.9,
        }];
        manager.set_search_results(search_key.clone(), results.clone());
        let cached = manager.get_search_results(&search_key);
        assert!(cached.is_some());

        // 测试配置缓存
        let config_key = ConfigCacheKey::App;
        manager.set_config(config_key.clone(), "test_value".to_string());
        let cached_config = manager.get_config(&config_key);
        assert!(cached_config.is_some());
    }

    #[test]
    fn test_simple_hash() {
        let hash1 = simple_hash("test");
        let hash2 = simple_hash("test");
        let hash3 = simple_hash("other");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}