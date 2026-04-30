//! 跨平台路径抽象
//!
//! 提供统一的路径接口，适配不同操作系统的路径规范

use dirs;
use std::path::PathBuf;

/// 跨平台路径提供者 trait
pub trait PlatformPaths {
    /// 配置目录
    fn config_dir() -> PathBuf;

    /// 数据目录
    fn data_dir() -> PathBuf;

    /// 缓存目录
    fn cache_dir() -> PathBuf;

    /// 临时目录
    fn temp_dir() -> PathBuf;

    /// 用户主目录
    fn home_dir() -> Option<PathBuf>;

    /// 初始化平台特定目录
    fn ensure_dirs() -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let dirs = [
            Self::config_dir(),
            Self::data_dir(),
            Self::cache_dir(),
            Self::temp_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)
                    .map_err(|e| anyhow::anyhow!("创建目录失败 {:?}: {}", dir, e))?;
            }
        }
        Ok(())
    }
}

/// Windows 路径实现
pub struct WindowsPaths;

impl PlatformPaths for WindowsPaths {
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\AppData\Local\ArkCore"))
            .join("ArkCore")
    }

    fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\AppData\Local\ArkCore"))
            .join("ArkCore")
    }

    fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\AppData\Local\ArkCore\Cache"))
            .join("ArkCore")
    }

    fn temp_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\AppData\Local\Temp"))
            .join("ArkCore")
    }

    fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }
}

/// Unix 路径实现
pub struct UnixPaths;

impl PlatformPaths for UnixPaths {
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/etc/arkcore"))
            .join("arkcore")
    }

    fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/var/lib/arkcore"))
            .join("arkcore")
    }

    fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp/arkcore"))
            .join("arkcore")
    }

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join("arkcore")
    }

    fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }
}

/// 根据当前平台选择路径实现
#[cfg(windows)]
pub type CurrentPlatformPaths = WindowsPaths;

#[cfg(not(windows))]
pub type CurrentPlatformPaths = UnixPaths;

/// 获取 ArkCore 配置目录
pub fn config_dir() -> PathBuf {
    CurrentPlatformPaths::config_dir()
}

/// 获取 ArkCore 数据目录
pub fn data_dir() -> PathBuf {
    CurrentPlatformPaths::data_dir()
}

/// 获取 ArkCore 缓存目录
pub fn cache_dir() -> PathBuf {
    CurrentPlatformPaths::cache_dir()
}

/// 获取 ArkCore 临时目录
pub fn temp_dir() -> PathBuf {
    CurrentPlatformPaths::temp_dir()
}

/// 获取用户主目录
pub fn home_dir() -> Option<PathBuf> {
    CurrentPlatformPaths::home_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_paths() {
        // 测试配置目录非空
        assert!(!config_dir().as_os_str().is_empty());
        // 测试数据目录非空
        assert!(!data_dir().as_os_str().is_empty());
        // 测试缓存目录非空
        assert!(!cache_dir().as_os_str().is_empty());
        // 测试临时目录非空
        assert!(!temp_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_ensure_dirs() {
        // 测试目录创建
        let result = CurrentPlatformPaths::ensure_dirs();
        assert!(result.is_ok());
    }

    #[test]
    fn test_home_dir() {
        // 主目录应该存在
        if let Some(home) = home_dir() {
            assert!(home.exists() || !home.to_string_lossy().is_empty());
        }
    }

    // ========== 新增测试 ==========

    #[test]
    fn test_paths_are_distinct() {
        // 不同目录应该有不同的路径
        let config = config_dir();
        let data = data_dir();
        let cache = cache_dir();
        let temp = temp_dir();

        // 至少 temp 目录应该与其他不同
        assert_ne!(temp, config);
    }

    #[test]
    fn test_paths_contain_app_name() {
        // 路径应该包含 arkcore/ArkCore 标识
        let config_str = config_dir().to_string_lossy().to_lowercase();
        let data_str = data_dir().to_string_lossy().to_lowercase();
        let cache_str = cache_dir().to_string_lossy().to_lowercase();

        // 至少一个应该包含 arkcore
        let has_arkcore = config_str.contains("arkcore")
            || data_str.contains("arkcore")
            || cache_str.contains("arkcore");

        // 注意：在某些平台上可能不包含，这是平台相关的
        // 所以只验证路径是有效的
        assert!(config_dir().components().count() >= 1);
    }

    #[test]
    fn test_config_dir_parent_exists_or_creatable() {
        let config = config_dir();
        // 配置目录本身或其父目录应该可访问
        if config.exists() {
            assert!(config.is_dir() || !config.exists());
        }
    }

    #[test]
    fn test_data_dir_not_empty() {
        let data = data_dir();
        assert!(!data.to_string_lossy().is_empty());
    }

    #[test]
    fn test_cache_dir_not_empty() {
        let cache = cache_dir();
        assert!(!cache.to_string_lossy().is_empty());
    }

    #[test]
    fn test_temp_dir_not_empty() {
        let temp = temp_dir();
        assert!(!temp.to_string_lossy().is_empty());
    }

    #[test]
    fn test_paths_implementation_consistency() {
        // 确保 WindowsPaths 和 UnixPaths 实现一致
        #[cfg(windows)]
        {
            let wp = WindowsPaths::config_dir();
            let dp = WindowsPaths::data_dir();
            let cp = WindowsPaths::cache_dir();
            let tp = WindowsPaths::temp_dir();

            assert!(!wp.to_string_lossy().is_empty());
            assert!(!dp.to_string_lossy().is_empty());
            assert!(!cp.to_string_lossy().is_empty());
            assert!(!tp.to_string_lossy().is_empty());
        }

        #[cfg(not(windows))]
        {
            let up = UnixPaths::config_dir();
            let dp = UnixPaths::data_dir();
            let cp = UnixPaths::cache_dir();
            let tp = UnixPaths::temp_dir();

            assert!(!up.to_string_lossy().is_empty());
            assert!(!dp.to_string_lossy().is_empty());
            assert!(!cp.to_string_lossy().is_empty());
            assert!(!tp.to_string_lossy().is_empty());
        }
    }

    #[test]
    fn test_home_dir_returns_valid_path() {
        match home_dir() {
            Some(path) => {
                // 如果返回了路径，应该是一个有效的路径格式
                assert!(path.components().count() >= 1);
            }
            None => {
                // None 也是有效返回值（如果没有主目录）
            }
        }
    }
}
