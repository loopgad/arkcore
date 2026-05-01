//! ArkCore 性能基准测试
//!
//! T8.1: 基准测试框架
//! T8.2: 内存基准
//! T8.3: 延迟基准
//! T8.4: 并发基准

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;

// ============================================================================
// T8.1: 基准测试框架
// ============================================================================

/// 基准测试组 - 框架验证
fn bench_framework(c: &mut Criterion) {
    c.bench_function("framework_init", |b| {
        b.iter(|| {
            let _ = arkcore::services::cache::CacheManager::new(
                arkcore::services::cache::CacheConfig::default(),
            );
        });
    });

    c.bench_function("object_pool_init", |b| {
        b.iter(|| {
            let _ = arkcore::services::object_pool::ObjectPoolManager::new(
                arkcore::services::object_pool::ObjectPoolConfig::default(),
            );
        });
    });
}

// ============================================================================
// T8.2: 内存基准测试
// ============================================================================

mod memory_bench {
    use super::*;
    use arkcore::services::cache::{CacheConfig, CacheManager, LlmCacheKey};
    use arkcore::services::object_pool::{ObjectPoolConfig, ObjectPoolManager};
    use moka::sync::Cache;

    /// 缓存插入基准
    pub fn bench_cache_insert(c: &mut Criterion) {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);

        c.bench_function("cache_llm_insert", |b| {
            b.iter(|| {
                let key = LlmCacheKey::new("gpt-4", "Hello world prompt");
                manager.set_llm_response(key, "Response text".to_string(), "gpt-4".to_string());
            });
        });

        c.bench_function("cache_llm_insert_1k", |b| {
            let config = CacheConfig::default();
            let manager = CacheManager::new(config);

            b.iter(|| {
                for i in 0..1000 {
                    let key = LlmCacheKey::new("gpt-4", &format!("Prompt {} with some content", i));
                    manager.set_llm_response(key, format!("Response {}", i), "gpt-4".to_string());
                }
            });
        });
    }

    /// 缓存读取基准
    pub fn bench_cache_read(c: &mut Criterion) {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);

        // 预热缓存
        for i in 0..1000 {
            let key = LlmCacheKey::new("gpt-4", &format!("Prompt {} with some content", i));
            manager.set_llm_response(key, format!("Response {}", i), "gpt-4".to_string());
        }

        c.bench_function("cache_llm_read_hit", |b| {
            b.iter(|| {
                let key = LlmCacheKey::new("gpt-4", "Prompt 500 with some content");
                black_box(manager.get_llm_response(&key));
            });
        });

        c.bench_function("cache_llm_read_miss", |b| {
            b.iter(|| {
                let key = LlmCacheKey::new("gpt-4", "Non-existent prompt");
                black_box(manager.get_llm_response(&key));
            });
        });

        c.bench_function("cache_llm_read_1k_ops", |b| {
            b.iter(|| {
                for i in 0..1000 {
                    let key = LlmCacheKey::new("gpt-4", &format!("Prompt {} with some content", i));
                    black_box(manager.get_llm_response(&key));
                }
            });
        });
    }

    /// 缓存批量操作基准
    pub fn bench_cache_batch(c: &mut Criterion) {
        c.bench_function("cache_batch_insert_1k", |b| {
            b.iter(|| {
                let config = CacheConfig::default();
                let manager = CacheManager::new(config);
                for i in 0..1000 {
                    let key = LlmCacheKey::new("gpt-4", &format!("Batch prompt {}", i));
                    manager.set_llm_response(
                        key,
                        format!("Batch response {}", i),
                        "gpt-4".to_string(),
                    );
                }
            });
        });

        c.bench_function("cache_batch_read_1k", |b| {
            let config = CacheConfig::default();
            let manager = CacheManager::new(config);
            for i in 0..1000 {
                let key = LlmCacheKey::new("gpt-4", &format!("Batch prompt {}", i));
                manager.set_llm_response(key, format!("Batch response {}", i), "gpt-4".to_string());
            }

            b.iter(|| {
                for i in 0..1000 {
                    let key = LlmCacheKey::new("gpt-4", &format!("Batch prompt {}", i));
                    black_box(manager.get_llm_response(&key));
                }
            });
        });
    }

    /// Moka Cache 直接基准
    pub fn bench_moka_cache(c: &mut Criterion) {
        let cache: Cache<String, String> = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build();

        c.bench_function("moka_cache_insert", |b| {
            b.iter(|| {
                cache.insert(
                    black_box(format!("key_{}", 1)),
                    black_box("value".to_string()),
                );
            });
        });

        c.bench_function("moka_cache_insert_1k", |b| {
            b.iter(|| {
                let cache: Cache<String, String> = Cache::builder().max_capacity(10_000).build();
                for i in 0..1000 {
                    cache.insert(format!("key_{}", i), format!("value_{}", i));
                }
            });
        });

        c.bench_function("moka_cache_read_hit", |b| {
            let cache: Cache<String, String> = Cache::builder().max_capacity(10_000).build();
            cache.insert("fixed_key".to_string(), "fixed_value".to_string());

            b.iter(|| {
                black_box(cache.get(&"fixed_key".to_string()));
            });
        });
    }

    /// 对象池基准
    pub fn bench_object_pool(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();

        c.bench_function("object_pool_acquire_release", |b| {
            let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

            b.iter(|| {
                rt.block_on(async {
                    let buffer = manager.string_pool().acquire().await;
                    let mut buf = buffer;
                    buf.as_str_mut().push_str("test content");
                    manager.string_pool().release(buf).await;
                });
            });
        });

        c.bench_function("object_pool_acquire_release_1k", |b| {
            let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

            b.iter(|| {
                rt.block_on(async {
                    for i in 0..1000 {
                        let buffer = manager.string_pool().acquire().await;
                        let mut buf = buffer;
                        buf.as_str_mut().push_str(&format!("content {}", i));
                        manager.string_pool().release(buf).await;
                    }
                });
            });
        });

        c.bench_function("object_pool_byte_buffer", |b| {
            let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

            b.iter(|| {
                rt.block_on(async {
                    let buffer = manager.byte_pool().acquire().await;
                    let mut buf = buffer;
                    buf.as_vec_mut().extend_from_slice(b"test bytes content");
                    manager.byte_pool().release(buf).await;
                });
            });
        });
    }

    /// 字符串分配 vs 池化对比
    pub fn bench_string_allocation(c: &mut Criterion) {
        c.bench_function("string_direct_allocation", |b| {
            b.iter(|| {
                let mut s = String::new();
                for i in 0..1000 {
                    s.push_str(&format!("content {} ", i));
                }
                black_box(s);
            });
        });

        c.bench_function("string_pooled_allocation", |b| {
            let rt = Runtime::new().unwrap();
            let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

            b.iter(|| {
                rt.block_on(async {
                    let buffer = manager.string_pool().acquire().await;
                    let mut buf = buffer;
                    for i in 0..1000 {
                        buf.as_str_mut().push_str(&format!("content {} ", i));
                    }
                    let result = buf.as_str().to_string();
                    manager.string_pool().release(buf).await;
                    black_box(result);
                });
            });
        });
    }
}

// ============================================================================
// T8.3: 延迟基准测试
// ============================================================================

mod latency_bench {
    use super::*;
    use arkcore::services::cache::{CacheConfig, CacheManager, LlmCacheKey};
    use arkcore::services::object_pool::{ObjectPoolConfig, ObjectPoolManager};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// 缓存操作延迟
    pub fn bench_cache_latency(c: &mut Criterion) {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);

        c.bench_function("latency_cache_insert_single", |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    let key = LlmCacheKey::new("gpt-4", "Single prompt");
                    manager.set_llm_response(key, "Response".to_string(), "gpt-4".to_string());
                }
                start.elapsed()
            });
        });

        c.bench_function("latency_cache_read_single", |b| {
            let key = LlmCacheKey::new("gpt-4", "Single prompt");
            manager.set_llm_response(key.clone(), "Response".to_string(), "gpt-4".to_string());

            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    black_box(manager.get_llm_response(&key));
                }
                start.elapsed()
            });
        });
    }

    /// 锁竞争延迟
    pub fn bench_lock_latency(c: &mut Criterion) {
        let lock = Arc::new(RwLock::new(0u64));
        let rt = Runtime::new().unwrap();

        c.bench_function("latency_rwlock_write", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let mut guard = lock.write().await;
                    *guard += 1;
                });
            });
        });

        c.bench_function("latency_rwlock_read", |b| {
            let lock_clone = lock.clone();
            rt.block_on(async {
                let mut guard = lock_clone.write().await;
                *guard = 42;
            });

            b.iter(|| {
                rt.block_on(async {
                    let guard = lock.read().await;
                    black_box(*guard);
                });
            });
        });
    }

    /// 异步任务延迟
    pub fn bench_async_task_latency(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();

        c.bench_function("latency_spawn_task", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let handle = tokio::spawn(async { black_box(42u64) });
                    handle.await.unwrap();
                });
            });
        });

        c.bench_function("latency_spawn_1k_tasks", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let handles: Vec<_> = (0..1000)
                        .map(|i| tokio::spawn(async move { black_box(i) }))
                        .collect();

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            });
        });

        c.bench_function("latency_join_all", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let results: Vec<_> = (0..1000).map(|i| async move { black_box(i) }).collect();
                    futures_util::future::join_all(results).await;
                });
            });
        });
    }

    /// 缓冲区操作延迟
    pub fn bench_buffer_latency(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let manager = ObjectPoolManager::new(ObjectPoolConfig::default());

        c.bench_function("latency_pool_acquire", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let _buffer = manager.string_pool().acquire().await;
                });
            });
        });

        c.bench_function("latency_pool_acquire_release", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let buffer = manager.string_pool().acquire().await;
                    manager.string_pool().release(buffer).await;
                });
            });
        });
    }
}

// ============================================================================
// T8.4: 并发基准测试
// ============================================================================

mod concurrency_bench {
    use super::*;
    use arkcore::services::cache::{CacheConfig, CacheManager, LlmCacheKey};
    use arkcore::services::object_pool::{ObjectPoolConfig, ObjectPoolManager};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// 并发缓存访问
    pub fn bench_concurrent_cache(c: &mut Criterion) {
        let manager = Arc::new(CacheManager::new(CacheConfig::default()));
        let num_threads = 4;
        let ops_per_thread = 250;

        c.bench_function("concurrent_cache_insert_1k", |b| {
            b.iter(|| {
                std::thread::scope(|s| {
                    for _t in 0..num_threads {
                        let manager = manager.clone();
                        s.spawn(move || {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                for i in 0..ops_per_thread {
                                    let key = LlmCacheKey::new(
                                        "gpt-4",
                                        &format!("Thread {} Prompt {} with content", _t, i),
                                    );
                                    manager.set_llm_response(
                                        key,
                                        format!("Response from thread {} op {}", _t, i),
                                        "gpt-4".to_string(),
                                    );
                                }
                            });
                        });
                    }
                });
            });
        });

        c.bench_function("concurrent_cache_read_1k", |b| {
            // 预热缓存
            for t in 0..num_threads {
                for i in 0..ops_per_thread {
                    let key = LlmCacheKey::new(
                        "gpt-4",
                        &format!("Thread {} Prompt {} with content", t, i),
                    );
                    manager.set_llm_response(
                        key,
                        format!("Response from thread {} op {}", t, i),
                        "gpt-4".to_string(),
                    );
                }
            }

            let manager = manager.clone();
            b.iter(|| {
                std::thread::scope(|s| {
                    for t in 0..num_threads {
                        let manager = manager.clone();
                        s.spawn(move || {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                for i in 0..ops_per_thread {
                                    let key = LlmCacheKey::new(
                                        "gpt-4",
                                        &format!("Thread {} Prompt {} with content", t, i),
                                    );
                                    black_box(manager.get_llm_response(&key));
                                }
                            });
                        });
                    }
                });
            });
        });
    }

    /// 并发对象池
    pub fn bench_concurrent_pool(c: &mut Criterion) {
        let manager = Arc::new(ObjectPoolManager::new(ObjectPoolConfig::default()));
        let num_threads = 4;
        let ops_per_thread = 250;

        c.bench_function("concurrent_pool_acquire_release_1k", |b| {
            b.iter(|| {
                std::thread::scope(|s| {
                    for _t in 0..num_threads {
                        let manager = manager.clone();
                        s.spawn(move || {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                for _ in 0..ops_per_thread {
                                    let buffer = manager.string_pool().acquire().await;
                                    let mut buf = buffer;
                                    buf.as_str_mut().push_str("concurrent content");
                                    manager.string_pool().release(buf).await;
                                }
                            });
                        });
                    }
                });
            });
        });
    }

    /// 并发锁竞争
    pub fn bench_concurrent_lock(c: &mut Criterion) {
        let lock = Arc::new(RwLock::new(0u64));
        let num_threads = 4;
        let ops_per_thread = 250;

        c.bench_function("concurrent_lock_contention", |b| {
            b.iter(|| {
                std::thread::scope(|s| {
                    for _t in 0..num_threads {
                        let lock = lock.clone();
                        s.spawn(move || {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                for _ in 0..ops_per_thread {
                                    let mut guard = lock.write().await;
                                    *guard += 1;
                                }
                            });
                        });
                    }
                });
            });
        });
    }

    /// 并发任务处理
    pub fn bench_concurrent_tasks(c: &mut Criterion) {
        c.bench_function("concurrent_spawn_1k_tasks", |b| {
            b.iter(|| {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let handles: Vec<_> = (0..1000)
                        .map(|i| {
                            tokio::spawn(async move {
                                // 模拟一些工作
                                let mut sum = 0u64;
                                for j in 0..100 {
                                    sum += (i + j) as u64;
                                }
                                sum
                            })
                        })
                        .collect();

                    for handle in handles {
                        black_box(handle.await.unwrap());
                    }
                });
            });
        });

        c.bench_function("concurrent_task_broadcast", |b| {
            b.iter(|| {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);

                    // Spawn 接收者
                    let handle = tokio::spawn(async move {
                        let mut count = 0;
                        while (rx.recv()).await.is_some() {
                            count += 1;
                        }
                        count
                    });

                    // 发送 1000 条消息
                    for i in 0..1000 {
                        tx.send(i).await.unwrap();
                    }
                    drop(tx);

                    black_box(handle.await.unwrap());
                });
            });
        });
    }

    /// 生产者-消费者模式
    pub fn bench_producer_consumer(c: &mut Criterion) {
        c.bench_function("producer_consumer_1k", |b| {
            let rt = Runtime::new().unwrap();
            b.iter(|| {
                rt.block_on(async {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
                    let tx_clone = tx.clone();

                    // 生产者
                    let producer = tokio::spawn(async move {
                        for i in 0..1000 {
                            tx_clone.send(i).await.unwrap();
                        }
                    });

                    // 消费者
                    let consumer = tokio::spawn(async move {
                        let mut sum = 0i64;
                        while let Some(v) = rx.recv().await {
                            sum += v;
                        }
                        sum
                    });

                    producer.await.unwrap();
                    drop(tx);
                    black_box(rt.block_on(consumer).unwrap());
                });
            });
        });
    }
}

// ============================================================================
// 基准测试组
// ============================================================================

fn bench_t8_1_framework(c: &mut Criterion) {
    c.benchmark_group("t8_1_framework")
        .throughput(Throughput::Elements(1));
    bench_framework(c);
}

fn bench_t8_2_memory(c: &mut Criterion) {
    c.benchmark_group("t8_2_memory")
        .throughput(Throughput::Elements(1));
    memory_bench::bench_cache_insert(c);
    memory_bench::bench_cache_read(c);
    memory_bench::bench_cache_batch(c);
    memory_bench::bench_moka_cache(c);
    memory_bench::bench_object_pool(c);
    memory_bench::bench_string_allocation(c);
}

fn bench_t8_3_latency(c: &mut Criterion) {
    c.benchmark_group("t8_3_latency")
        .throughput(Throughput::Elements(1));
    latency_bench::bench_cache_latency(c);
    latency_bench::bench_lock_latency(c);
    latency_bench::bench_async_task_latency(c);
    latency_bench::bench_buffer_latency(c);
}

fn bench_t8_4_concurrency(c: &mut Criterion) {
    c.benchmark_group("t8_4_concurrency")
        .throughput(Throughput::Elements(1));
    concurrency_bench::bench_concurrent_cache(c);
    concurrency_bench::bench_concurrent_pool(c);
    concurrency_bench::bench_concurrent_lock(c);
    concurrency_bench::bench_concurrent_tasks(c);
    concurrency_bench::bench_producer_consumer(c);
}

criterion_group!(
    benches,
    bench_t8_1_framework,
    bench_t8_2_memory,
    bench_t8_3_latency,
    bench_t8_4_concurrency
);
criterion_main!(benches);
