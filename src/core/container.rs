//! 依赖注入容器
//!
//! 手写的轻量级 DI 容器，支持 Singleton 和 Transient 生命周期管理。
//!
//! # 特性
//!
//! - **Singleton**: 单例模式，整个容器生命周期内只创建一个实例
//! - **Transient**: 瞬态模式，每次 resolve 都创建新实例
//! - **类型安全**: 使用 `TypeId` 和 `Any` 实现泛型注册，无需反射

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// 服务生命周期类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// 单例模式 - 整个容器生命周期内只创建一个实例
    Singleton,
    /// 瞬态模式 - 每次 resolve 都创建新实例
    Transient,
}

/// 服务注册条目
struct ServiceEntry {
    factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    lifecycle: Lifecycle,
}

impl std::fmt::Debug for ServiceEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceEntry")
            .field("lifecycle", &self.lifecycle)
            .finish()
    }
}

/// 依赖注入容器
///
/// 使用 `TypeId` 作为键存储服务，支持 Singleton 和 Transient 生命周期。
///
/// # 示例
///
/// ```rust
/// use arkcore::core::container::{Container, Lifecycle};
///
/// let mut container = Container::new();
///
/// // 注册单例
/// container.register_singleton::<String, _>(|| "singleton".to_string());
///
/// // 注册瞬态
/// container.register::<i32, _>(|| 100);
/// ```
#[derive(Debug, Default)]
pub struct Container {
    services: HashMap<TypeId, ServiceEntry>,
    singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Container {
    /// 创建一个新的空容器
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            singletons: HashMap::new(),
        }
    }

    /// 注册一个瞬态服务
    ///
    /// # 类型参数
    ///
    /// - `T`: 服务类型，必须实现 `Send + Sync`
    /// - `F`: 工厂函数类型，必须返回 `T`
    pub fn register<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync> =
            Arc::new(move || Arc::new(factory()));
        self.services.insert(
            type_id,
            ServiceEntry {
                factory,
                lifecycle: Lifecycle::Transient,
            },
        );
    }

    /// 注册一个单例服务
    ///
    /// # 类型参数
    ///
    /// - `T`: 服务类型，必须实现 `Send + Sync`
    /// - `F`: 工厂函数类型，必须返回 `T`
    pub fn register_singleton<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync> =
            Arc::new(move || Arc::new(factory()));
        self.services.insert(
            type_id,
            ServiceEntry {
                factory,
                lifecycle: Lifecycle::Singleton,
            },
        );
    }

    /// 获取单例服务实例
    ///
    /// 首次调用时创建实例并缓存，后续调用返回缓存的实例。
    ///
    /// # 类型参数
    ///
    /// - `T`: 服务类型，必须实现 `Send + Sync`
    ///
    /// # 返回
    ///
    /// - `Some(Arc<T>)`: 服务已注册
    /// - `None`: 服务未注册或类型不匹配
    ///
    /// # 示例
    ///
    /// ```rust
    /// use arkcore::core::container::Container;
    ///
    /// let mut container = Container::new();
    /// container.register_singleton::<String, _>(|| "hello".to_string());
    ///
    /// let value = container.resolve::<String>();
    /// assert_eq!(&*value.unwrap(), "hello");
    /// ```
    pub fn resolve<T: Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // 单例模式：检查缓存
        if let Some(instance) = self.singletons.get(&type_id) {
            return instance.clone().downcast().ok();
        }

        // 查找服务注册条目
        let entry = self.services.get(&type_id)?;

        if entry.lifecycle != Lifecycle::Singleton {
            return None;
        }

        // 首次创建单例
        let arc = (entry.factory)();
        let result = arc.clone().downcast::<T>().ok();
        if result.is_some() {
            self.singletons.insert(type_id, arc);
        }
        result
    }

    /// 获取瞬态服务的新实例
    ///
    /// 每次调用都会通过 factory 创建新实例。
    ///
    /// # 类型参数
    ///
    /// - `T`: 服务类型，必须实现 `Send + Sync`
    ///
    /// # 返回
    ///
    /// - `Some(Arc<T>)`: 服务已注册，新创建的实例
    /// - `None`: 服务未注册或类型不匹配
    ///
    /// # 示例
    ///
    /// ```rust
    /// use arkcore::core::container::Container;
    /// use std::sync::Arc;
    ///
    /// let mut container = Container::new();
    /// container.register::<u32, _>(|| 42u32);
    ///
    /// let first = container.resolve_transient::<u32>();
    /// let second = container.resolve_transient::<u32>();
    /// assert!(first.is_some());
    /// assert!(second.is_some());
    /// // 瞬态模式每次返回不同的实例
    /// assert_ne!(Arc::as_ptr(&first.unwrap()) as *const u32, Arc::as_ptr(&second.unwrap()) as *const u32);
    /// ```
    pub fn resolve_transient<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // 查找服务注册条目
        let entry = self.services.get(&type_id)?;

        if entry.lifecycle != Lifecycle::Transient {
            return None;
        }

        // 创建新实例
        let arc = (entry.factory)();
        Arc::downcast::<T>(arc).ok()
    }

    /// 检查指定类型的服务是否已注册
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.services.contains_key(&TypeId::of::<T>())
    }

    /// 获取服务生命周期类型
    pub fn get_lifecycle<T: 'static>(&self) -> Option<Lifecycle> {
        self.services.get(&TypeId::of::<T>()).map(|e| e.lifecycle)
    }

    /// 清除所有已注册的服务和缓存的实例
    pub fn clear(&mut self) {
        self.services.clear();
        self.singletons.clear();
    }
}

// === 错误类型 ===

/// 容器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// 服务未注册
    #[error("服务 {0} 未注册")]
    ServiceNotFound(String),

    /// 服务类型不匹配
    #[error("服务类型不匹配")]
    TypeMismatch,

    /// 生命周期错误（例如尝试对瞬态服务调用 resolve）
    #[error("不支持的生命周期操作")]
    LifecycleError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_new() {
        let container = Container::new();
        assert!(container.get_lifecycle::<String>().is_none());
    }

    #[test]
    fn test_register_and_resolve_singleton() {
        let mut container = Container::new();
        container.register_singleton::<String, _>(|| "hello".to_string());

        assert!(container.is_registered::<String>());
        assert_eq!(container.get_lifecycle::<String>(), Some(Lifecycle::Singleton));

        let value = container.resolve::<String>();
        assert!(value.is_some());
        assert_eq!(&*value.unwrap(), "hello");
    }

    #[test]
    fn test_singleton_returns_same_instance() {
        let mut container = Container::new();
        container.register_singleton::<Vec<u32>, _>(|| vec![1, 2, 3]);

        let first = container.resolve::<Vec<u32>>();
        let second = container.resolve::<Vec<u32>>();

        assert!(first.is_some());
        assert!(second.is_some());
        // 同一个实例
        assert_eq!(Arc::as_ptr(&first.unwrap()), Arc::as_ptr(&second.unwrap()));
    }

    #[test]
    fn test_register_transient() {
        let mut container = Container::new();
        container.register::<u32, _>(|| 42u32);

        assert!(container.is_registered::<u32>());
        assert_eq!(container.get_lifecycle::<u32>(), Some(Lifecycle::Transient));
    }

    #[test]
    fn test_resolve_transient_creates_new_instance() {
        let mut container = Container::new();
        container.register::<u32, _>(|| 42u32);

        // 瞬态服务每次 resolve_transient 创建新实例
        let first = container.resolve_transient::<u32>();
        let second = container.resolve_transient::<u32>();

        assert!(first.is_some());
        assert!(second.is_some());
        assert_eq!(first.as_ref().unwrap(), second.as_ref().unwrap());
        // 但它们是不同的实例
        assert_ne!(Arc::as_ptr(&first.unwrap()) as *const u32, Arc::as_ptr(&second.unwrap()) as *const u32);
    }

    #[test]
    fn test_resolve_singleton_on_transient_returns_none() {
        let mut container = Container::new();
        container.register::<u32, _>(|| 42u32);

        // 对瞬态服务调用 resolve（单例方法）返回 None
        let value = container.resolve::<u32>();
        assert!(value.is_none());
    }

    #[test]
    fn test_clear_container() {
        let mut container = Container::new();
        container.register_singleton::<String, _>(|| "test".to_string());
        assert!(container.is_registered::<String>());

        container.clear();
        assert!(!container.is_registered::<String>());
    }

    #[test]
    fn test_multiple_services() {
        let mut container = Container::new();
        container.register_singleton::<String, _>(|| "first".to_string());
        container.register::<i32, _>(|| 100);
        container.register::<u64, _>(|| 999u64);

        assert!(container.is_registered::<String>());
        assert!(container.is_registered::<i32>());
        assert!(container.is_registered::<u64>());

        assert!(!container.is_registered::<f32>());
    }
}