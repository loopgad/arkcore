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
}
