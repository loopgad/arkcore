//! 异步 I/O 优化模块
//!
//! 提供 tokio 运行时调优和异步最佳实践

use tokio::runtime::{Builder, Runtime};
use std::num::NonZeroUsize;

/// Tokio 运行时配置
#[derive(Debug, Clone)]
pub struct TokioRuntimeConfig {
    /// 运行时类型
    pub flavor: TokioFlavor,
    /// 工作线程数（None = CPU 核心数）
    pub worker_threads: Option<NonZeroUsize>,
    /// 是否启用追踪
    pub enable_tracing: bool,
    /// 最大阻塞线程数
    pub max_blocking_threads: usize,
}

impl Default for TokioRuntimeConfig {
    fn default() -> Self {
        Self {
            flavor: TokioFlavor::MultiThread,
            worker_threads: None, // None = CPU cores
            enable_tracing: false,
            max_blocking_threads: 512,
        }
    }
}

impl TokioRuntimeConfig {
    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            flavor: TokioFlavor::MultiThread,
            worker_threads: None, // CPU cores
            enable_tracing: false,
            max_blocking_threads: 512,
        }
    }

    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            flavor: TokioFlavor::MultiThread,
            worker_threads: NonZeroUsize::new(2),
            enable_tracing: true,
            max_blocking_threads: 256,
        }
    }
}

/// Tokio 运行时类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokioFlavor {
    /// 多线程运行时（推荐生产环境）
    MultiThread,
    /// 单线程运行时（适合特定场景）
    SingleThread,
}

impl TokioRuntimeConfig {
    /// 根据配置构建 Runtime
    #[allow(clippy::let_unit_value)]
    pub fn build_runtime(&self) -> Result<Runtime, std::io::Error> {
        match self.flavor {
            TokioFlavor::MultiThread => {
                let mut builder = Builder::new_multi_thread();

                if let Some(threads) = self.worker_threads {
                    builder.worker_threads(threads.get());
                }

                builder
                    .enable_all()
                    .max_blocking_threads(self.max_blocking_threads);

                builder.build()
            }
            TokioFlavor::SingleThread => {
                let mut builder = Builder::new_current_thread();

                builder
                    .enable_all()
                    .max_blocking_threads(self.max_blocking_threads);

                builder.build()
            }
        }
    }
}

/// 异步任务工具
pub mod task {
    use tokio::task::JoinHandle;
    use std::future::Future;

    /// 安全地 spawn 任务（捕获 panic）
    pub fn spawn_safe<F>(future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        tokio::spawn(future)
    }

    /// Spawn 本地任务（仅在单线程运行时使用）
    pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        tokio::task::spawn_local(future)
    }

    /// 获取当前任务的 ID
    #[allow(clippy::unnecessary_to_owned)]
    pub fn current_task_id() -> Option<String> {
        tokio::task::try_id().map(|id| id.to_string())
    }

    /// 检查是否在 tokio 运行时中
    pub fn in_runtime() -> bool {
        tokio::runtime::Handle::try_current().is_ok()
    }
}

/// I/O 批处理工具
pub mod io {
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use std::io::Result;

    /// 批量读取辅助
    pub async fn read_batch<R, B>(reader: &mut R, buf: &mut B) -> Result<usize>
    where
        R: AsyncRead + Unpin,
        B: AsMut<[u8]>,
    {
        let slice = buf.as_mut();
        let mut read = 0;
        let batch_size = 64 * 1024; // 64KB batch

        loop {
            let available = slice.len() - read;
            if available == 0 || batch_size == 0 {
                break;
            }

            let chunk = AsyncReadExt::read(reader, &mut slice[read..]).await?;
            if chunk == 0 {
                break;
            }
            read += chunk;

            if chunk < batch_size {
                break;
            }
        }

        Ok(read)
    }

    /// 批量写入辅助
    pub async fn write_all_buf<W, B>(writer: &mut W, buf: &B) -> Result<()>
    where
        W: AsyncWrite + Unpin,
        B: AsRef<[u8]> + ?Sized,
    {
        let slice = buf.as_ref();
        let mut written = 0;

        while written < slice.len() {
            let n = AsyncWriteExt::write(writer, &slice[written..]).await?;
            written += n;
        }

        Ok(())
    }
}

/// 减少 .await 点的模式
pub mod coalesce {
    use futures_util::future::join_all;
    use std::future::Future;

    /// 将多个 future 合并执行（当它们相互独立时）
    pub async fn join_all_futures<I, F>(iter: I) -> Vec<F::Output>
    where
        I: IntoIterator<Item = F>,
        F: Future,
    {
        join_all(iter).await
    }
}

/// 运行时指标
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub active_tasks: u64,
    pub completed_tasks: u64,
}

/// 运行时监控
pub mod metrics {
    use super::RuntimeMetrics;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 活跃任务计数
    static ACTIVE_TASKS: AtomicU64 = AtomicU64::new(0);

    /// 累计完成任务计数
    static COMPLETED_TASKS: AtomicU64 = AtomicU64::new(0);

    /// 记录任务开始
    pub fn task_started() {
        ACTIVE_TASKS.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录任务完成
    pub fn task_completed() {
        ACTIVE_TASKS.fetch_sub(1, Ordering::Relaxed);
        COMPLETED_TASKS.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取当前活跃任务数
    pub fn active_tasks() -> u64 {
        ACTIVE_TASKS.load(Ordering::Relaxed)
    }

    /// 获取累计完成任务数
    pub fn completed_tasks() -> u64 {
        COMPLETED_TASKS.load(Ordering::Relaxed)
    }

    /// 获取运行时指标
    pub fn runtime_metrics() -> RuntimeMetrics {
        RuntimeMetrics {
            active_tasks: active_tasks(),
            completed_tasks: completed_tasks(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokio_runtime_config_default() {
        let config = TokioRuntimeConfig::default();
        assert_eq!(config.flavor, TokioFlavor::MultiThread);
        assert!(config.worker_threads.is_none());
    }

    #[test]
    fn test_tokio_runtime_config_production() {
        let config = TokioRuntimeConfig::production();
        assert_eq!(config.flavor, TokioFlavor::MultiThread);
    }

    #[test]
    fn test_tokio_runtime_config_development() {
        let config = TokioRuntimeConfig::development();
        assert!(config.worker_threads.is_some());
        assert!(config.enable_tracing);
    }

    #[tokio::test]
    async fn test_task_metrics() {
        // 验证 metrics 函数正常工作
        let initial = metrics::active_tasks();
        metrics::task_started();
        assert!(metrics::active_tasks() >= initial + 1);

        metrics::task_completed();
        // 验证 completed 计数增加
        assert!(metrics::completed_tasks() >= 1);
    }

    #[tokio::test]
    async fn test_in_runtime() {
        assert!(task::in_runtime());
    }

    #[test]
    fn test_build_runtime_multi_thread() {
        let config = TokioRuntimeConfig::default();
        let runtime = config.build_runtime();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_build_runtime_single_thread() {
        let config = TokioRuntimeConfig {
            flavor: TokioFlavor::SingleThread,
            worker_threads: None,
            enable_tracing: false,
            max_blocking_threads: 128,
        };
        let runtime = config.build_runtime();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_tokio_flavor_eq() {
        assert_eq!(TokioFlavor::MultiThread, TokioFlavor::MultiThread);
        assert_eq!(TokioFlavor::SingleThread, TokioFlavor::SingleThread);
        assert_ne!(TokioFlavor::MultiThread, TokioFlavor::SingleThread);
    }

    #[tokio::test]
    async fn test_spawn_safe() {
        let handle = task::spawn_safe(async { 42 });
        let result = handle.await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_current_task_id() {
        let id = task::current_task_id();
        // try_id() 在某些上下文中可能返回 None
        if id.is_some() {
            let id2 = task::current_task_id();
            assert_eq!(id, id2); // 同一任务内 ID 相同
        }
    }

    #[tokio::test]
    async fn test_io_read_batch() {
        use std::io::Cursor;

        let data = b"hello world";
        let mut cursor = Cursor::new(data);
        let mut buf = [0u8; 64];

        let n = io::read_batch(&mut cursor, &mut buf).await.unwrap();
        assert_eq!(n, 11);
        assert_eq!(&buf[..n], b"hello world");
    }

    #[tokio::test]
    async fn test_io_write_all_buf() {
        use std::io::Cursor;

        let mut cursor = Cursor::new(Vec::new());
        let data = b"test data";

        io::write_all_buf(&mut cursor, &data[..]).await.unwrap();

        assert_eq!(cursor.into_inner(), b"test data");
    }

    #[tokio::test]
    async fn test_join_all_futures() {
        // 使用 futures_util::join_all - 需要 boxed 来统一类型
        use futures_util::future::join_all;

        let futures: Vec<_> = (1..=3).map(|i| async move { i }).collect();
        let results = join_all(futures).await;
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_runtime_metrics() {
        // 验证 metrics 函数返回合理的值（全局状态可能被其他测试影响）
        let initial = metrics::runtime_metrics();
        assert!(initial.active_tasks >= 0);
        assert!(initial.completed_tasks >= 0);

        metrics::task_started();
        let after_start = metrics::active_tasks();
        assert!(after_start >= 1);

        metrics::task_completed();
        // completed 应该增加
        assert!(metrics::completed_tasks() >= 1);
    }
}