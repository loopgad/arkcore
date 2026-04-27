//! ArkCore CLI 配置管理
//!
//! 基于核心配置系统，CLI 专用配置接口
//! 配置文件路径: ~/.config/arkcore/config.toml

pub use crate::core::config::{
    Config, ConfigLoader, DatabaseConfig, Environment, LlmConfig, PlatformConfig,
    SecurityConfig, AppConfig,
};

use anyhow::Result;

/// 配置目录路径
pub fn config_dir() -> Result<std::path::PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".config"))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        });
    Ok(config_home.join("arkcore"))
}

/// 配置文件路径
pub fn config_path() -> Result<std::path::PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// 加载配置
///
/// 如果配置文件不存在，返回默认配置
pub fn load_config() -> Result<Config> {
    ConfigLoader::new().load()
}

/// 保存配置
pub fn save_config(config: &Config) -> Result<()> {
    ConfigLoader::new().save(config)
}

/// 设置 API Key
pub fn set_api_key(api_key: &str) -> Result<()> {
    let mut config = load_config().unwrap_or_default();
    config.llm.api_key = Some(api_key.to_string());
    save_config(&config)?;
    Ok(())
}

/// 显示当前配置
pub fn show_config() -> Result<()> {
    let config = load_config()?;
    println!("ArkCore 配置:");
    println!("  应用名称: {}", config.app.name);
    println!("  环境: {}", config.app.env);
    println!("  日志级别: {}", config.app.log_level);
    println!("  LLM 模型: {}", config.llm.model);
    // Mask credentials in URL if present
    let masked_base_url = mask_url_credentials(&config.llm.base_url);
    println!("  API Base URL: {}", masked_base_url);
    if config.llm.api_key.is_some() {
        println!("  API Key: [已设置]");
    } else {
        println!("  API Key: [未设置]");
    }
    // Mask credentials in database URL if present
    let masked_db_url = mask_url_credentials(&config.database.url);
    println!("  数据库 URL: {}", masked_db_url);
    println!("  数据库连接池: {}", config.database.pool_size);
    println!("  沙箱启用: {}", config.security.sandbox_enabled);
    Ok(())
}

/// Mask credentials in URLs (e.g., postgresql://user:pass@host -> postgresql://user:***@host)
fn mask_url_credentials(url: &str) -> String {
    // Check if URL contains credentials pattern (user:pass@host)
    if url.contains('@') && (url.contains("://") || url.starts_with("//")) {
        // Simple heuristic: if URL has :// and contains @, likely has credentials
        if let Some(at_pos) = url.find('@') {
            let before_at = &url[..at_pos];
            if before_at.contains(':') {
                // Has credentials, mask them
                if let Some(colon_pos) = before_at.find(':') {
                    let scheme_end = url.find("://").map(|p| p + 3).unwrap_or(0);
                    let user = &before_at[..colon_pos];
                    return format!("{}://{}:***@{}", &url[..scheme_end], user, &url[at_pos + 1..]);
                }
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_ok());
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.llm.model, "gpt-4");
    }

    #[test]
    fn test_config_path() {
        let path = config_path();
        assert!(path.is_ok());
        // Path should end with arkcore/config.toml
        let path_binding = path.unwrap();
        let path_str = path_binding.to_string_lossy();
        assert!(path_str.ends_with("arkcore") || path_str.contains("arkcore"));
    }

    #[test]
    fn test_config_path_contains_config_toml() {
        let path = config_path().unwrap();
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_config_re_exports() {
        // Verify that re-exported types are accessible
        let _config = Config::default();
        let _loader = ConfigLoader::new();
        let _db_config = DatabaseConfig::default();
        let _llm_config = LlmConfig::default();
        let _platform_config = PlatformConfig::default();
        let _security_config = SecurityConfig::default();
        let _app_config = AppConfig::default();
        let _env = Environment::Development;
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(format!("{}", Environment::Development), "development");
        assert_eq!(format!("{}", Environment::Production), "production");
    }
}