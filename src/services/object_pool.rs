//! 对象池模块
//!
//! 提供缓冲区池化，减少内存分配

use std::mem::ManuallyDrop;
use tokio::sync::RwLock;

/// 对象池配置
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// String 缓冲区初始容量
    pub string_buffer_capacity: usize,
    /// Vec<u8> 缓冲区初始容量
    pub byte_buffer_capacity: usize,
    /// 池化缓冲区数量
    pub pool_size: usize,
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        Self {
            string_buffer_capacity: 4096,
            byte_buffer_capacity: 8192,
            pool_size: 16,
        }
    }
}

impl ObjectPoolConfig {
    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            string_buffer_capacity: 4096,
            byte_buffer_capacity: 8192,
            pool_size: 32,
        }
    }

    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            string_buffer_capacity: 1024,
            byte_buffer_capacity: 2048,
            pool_size: 8,
        }
    }
}

/// 池化的 String 缓冲区
pub struct PooledStringBuffer {
    inner: ManuallyDrop<String>,
}

impl PooledStringBuffer {
    /// 创建新的池化缓冲区
    fn new(capacity: usize) -> Self {
        let inner = ManuallyDrop::new(String::with_capacity(capacity));
        Self { inner }
    }

    /// 获取内部 String 引用
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// 获取内部 String 可变引用
    pub fn as_str_mut(&mut self) -> &mut String {
        &mut self.inner
    }

    /// 清空缓冲区内容
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// 池化的字节缓冲区
pub struct PooledByteBuffer {
    inner: ManuallyDrop<Vec<u8>>,
}

impl PooledByteBuffer {
    /// 创建新的池化字节缓冲区
    fn new(capacity: usize) -> Self {
        let inner = ManuallyDrop::new(Vec::with_capacity(capacity));
        Self { inner }
    }

    /// 获取内部字节切片引用
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// 获取内部 Vec<u8> 可变引用
    pub fn as_vec_mut(&mut self) -> &mut Vec<u8> {
        &mut self.inner
    }

    /// 清空缓冲区内容
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// 对象池统计
#[derive(Debug, Clone)]
pub struct ObjectPoolStats {
    pub string_buffer_pool_size: usize,
    pub byte_buffer_pool_size: usize,
    pub string_buffer_allocated: usize,
    pub byte_buffer_allocated: usize,
}

/// String 缓冲区池
pub struct StringBufferPool {
    buffers: RwLock<Vec<PooledStringBuffer>>,
    capacity: usize,
    max_size: usize,
}

impl StringBufferPool {
    /// 创建新的 String 缓冲区池
    pub fn new(config: &ObjectPoolConfig) -> Self {
        Self {
            buffers: RwLock::new(Vec::with_capacity(config.pool_size)),
            capacity: config.string_buffer_capacity,
            max_size: config.pool_size,
        }
    }

    /// 获取缓冲区
    pub async fn acquire(&self) -> PooledStringBuffer {
        // 尝试从池中获取
        let buffer = {
            let mut buffers = self.buffers.write().await;
            buffers.pop()
        };

        match buffer {
            Some(b) => b,
            None => {
                // 池为空，创建新的
                PooledStringBuffer::new(self.capacity)
            }
        }
    }

    /// 释放缓冲区回池
    pub async fn release(&self, buffer: PooledStringBuffer) {
        // 重置缓冲区内容以便复用
        let mut b = buffer;
        b.as_str_mut().clear();
        let mut buffers = self.buffers.write().await;
        if buffers.len() < self.max_size {
            buffers.push(b);
        }
    }

    /// 获取当前池大小
    pub async fn pool_size(&self) -> usize {
        self.buffers.read().await.len()
    }
}

/// 字节缓冲区池
pub struct ByteBufferPool {
    buffers: RwLock<Vec<PooledByteBuffer>>,
    capacity: usize,
    max_size: usize,
}

impl ByteBufferPool {
    /// 创建新的字节缓冲区池
    pub fn new(config: &ObjectPoolConfig) -> Self {
        Self {
            buffers: RwLock::new(Vec::with_capacity(config.pool_size)),
            capacity: config.byte_buffer_capacity,
            max_size: config.pool_size,
        }
    }

    /// 获取缓冲区
    pub async fn acquire(&self) -> PooledByteBuffer {
        let buffer = {
            let mut buffers = self.buffers.write().await;
            buffers.pop()
        };

        match buffer {
            Some(b) => b,
            None => PooledByteBuffer::new(self.capacity),
        }
    }

    /// 释放缓冲区回池
    pub async fn release(&self, mut buffer: PooledByteBuffer) {
        // 重置缓冲区内容以便复用
        buffer.clear();
        let mut buffers = self.buffers.write().await;
        if buffers.len() < self.max_size {
            buffers.push(buffer);
        }
    }

    /// 获取当前池大小
    pub async fn pool_size(&self) -> usize {
        self.buffers.read().await.len()
    }
}

/// 全局对象池管理器
pub struct ObjectPoolManager {
    string_pool: StringBufferPool,
    byte_pool: ByteBufferPool,
    config: ObjectPoolConfig,
}

impl ObjectPoolManager {
    /// 创建新的对象池管理器
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            string_pool: StringBufferPool::new(&config),
            byte_pool: ByteBufferPool::new(&config),
            config,
        }
    }

    /// 获取 String 缓冲区池
    pub fn string_pool(&self) -> &StringBufferPool {
        &self.string_pool
    }

    /// 获取字节缓冲区池
    pub fn byte_pool(&self) -> &ByteBufferPool {
        &self.byte_pool
    }

    /// 获取配置
    pub fn config(&self) -> &ObjectPoolConfig {
        &self.config
    }

    /// 获取统计信息
    pub async fn stats(&self) -> ObjectPoolStats {
        ObjectPoolStats {
            string_buffer_pool_size: self.string_pool.pool_size().await,
            byte_buffer_pool_size: self.byte_pool.pool_size().await,
            string_buffer_allocated: self.config.pool_size,
            byte_buffer_allocated: self.config.pool_size,
        }
    }
}

impl Default for ObjectPoolManager {
    fn default() -> Self {
        Self::new(ObjectPoolConfig::default())
    }
}

/// 创建全局对象池实例
pub fn create_global_object_pool() -> ObjectPoolManager {
    ObjectPoolManager::new(ObjectPoolConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_config_default() {
        let config = ObjectPoolConfig::default();
        assert_eq!(config.string_buffer_capacity, 4096);
        assert_eq!(config.byte_buffer_capacity, 8192);
        assert_eq!(config.pool_size, 16);
    }

    #[test]
    fn test_object_pool_config_production() {
        let config = ObjectPoolConfig::production();
        assert_eq!(config.pool_size, 32);
    }

    #[tokio::test]
    async fn test_string_buffer_pool() {
        let pool = StringBufferPool::new(&ObjectPoolConfig::default());

        // 初始池大小
        assert_eq!(pool.pool_size().await, 0);

        // 获取缓冲区
        let buffer = pool.acquire().await;
        assert!(buffer.capacity() >= 4096);

        // 释放缓冲区
        pool.release(buffer).await;

        // 池大小应该增加
        assert_eq!(pool.pool_size().await, 1);
    }

    #[tokio::test]
    async fn test_byte_buffer_pool() {
        let pool = ByteBufferPool::new(&ObjectPoolConfig::default());

        // 初始池大小
        assert_eq!(pool.pool_size().await, 0);

        // 获取缓冲区
        let buffer = pool.acquire().await;
        assert!(buffer.capacity() >= 8192);

        // 释放缓冲区
        pool.release(buffer).await;

        // 池大小应该增加
        assert_eq!(pool.pool_size().await, 1);
    }

    #[tokio::test]
    async fn test_object_pool_manager() {
        let manager = ObjectPoolManager::new(ObjectPoolConfig::default());
        let stats = manager.stats().await;

        assert_eq!(stats.string_buffer_pool_size, 0);
        assert_eq!(stats.byte_buffer_pool_size, 0);
    }

    #[test]
    fn test_object_pool_config_development() {
        let config = ObjectPoolConfig::development();
        assert_eq!(config.pool_size, 8);
        assert_eq!(config.string_buffer_capacity, 1024);
        assert_eq!(config.byte_buffer_capacity, 2048);
    }

    #[tokio::test]
    async fn test_pooled_string_buffer_methods() {
        let mut buffer = PooledStringBuffer::new(1024);

        // 测试 capacity
        assert!(buffer.capacity() >= 1024);

        // 测试 len 和 is_empty
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // 测试 as_str_mut 和 clear
        buffer.as_str_mut().push_str("test");
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_str(), "test");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_pooled_byte_buffer_methods() {
        let mut buffer = PooledByteBuffer::new(2048);

        // 测试 capacity
        assert!(buffer.capacity() >= 2048);

        // 测试 len 和 is_empty
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // 测试 as_vec_mut
        buffer.as_vec_mut().extend_from_slice(b"bytes");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), b"bytes");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_string_buffer_pool_exceed_max_size() {
        let config = ObjectPoolConfig {
            pool_size: 2,
            string_buffer_capacity: 4096,
            byte_buffer_capacity: 8192,
        };
        let pool = StringBufferPool::new(&config);

        // 获取两个 buffer 并释放，池应该达到 max_size
        let buf1 = pool.acquire().await;
        let buf2 = pool.acquire().await;
        pool.release(buf1).await;
        pool.release(buf2).await;
        assert_eq!(pool.pool_size().await, 2);

        // 再获取一个 buffer 释放后，池大小不应超过 max_size
        let buf3 = pool.acquire().await;
        pool.release(buf3).await;
        assert_eq!(pool.pool_size().await, 2); // 保持为 2
    }

    #[tokio::test]
    async fn test_byte_buffer_pool_exceed_max_size() {
        let config = ObjectPoolConfig {
            pool_size: 3,
            string_buffer_capacity: 4096,
            byte_buffer_capacity: 8192,
        };
        let pool = ByteBufferPool::new(&config);

        // 获取多个 buffer 而不立即释放
        let mut buffers = Vec::new();
        for _ in 0..5 {
            buffers.push(pool.acquire().await);
        }

        // 全部释放后，池大小不应超过 max_size
        for buf in buffers {
            pool.release(buf).await;
        }
        // 池应该保持在 max_size 或更小
        assert!(pool.pool_size().await <= 3);
    }

    #[tokio::test]
    async fn test_object_pool_manager_pools() {
        let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

        // 验证两个池都存在
        assert!(manager.string_pool() as *const _ != std::ptr::null());
        assert!(manager.byte_pool() as *const _ != std::ptr::null());
    }

    #[tokio::test]
    async fn test_create_global_object_pool() {
        let pool = create_global_object_pool();
        let stats = pool.stats().await;

        assert_eq!(stats.string_buffer_pool_size, 0);
        assert_eq!(stats.byte_buffer_pool_size, 0);
        assert_eq!(stats.string_buffer_allocated, 16);
        assert_eq!(stats.byte_buffer_allocated, 16);
    }
}
