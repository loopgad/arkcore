//! ArkCore 统一配置系统
//!
//! 支持多环境配置、配置文件加载、环境变量覆盖

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// 环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// 开发环境
    #[default]
    Development,
    /// 生产环境
    Production,
    /// 测试环境
    Test,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            "test" => Ok(Environment::Test),
            _ => Err(format!("未知的环境: {}", s)),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 应用名称
    pub name: String,
    /// 环境
    pub env: Environment,
    /// 日志级别
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "arkcore".to_string(),
            env: Environment::default(),
            log_level: "info".to_string(),
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    pub url: String,
    /// 连接池大小
    pub pool_size: u32,
    /// 超时时间（秒）
    #[serde(with = "serde_duration_seconds")]
    pub timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:arkcore.db".to_string(),
            pool_size: 5,
            timeout: Duration::from_secs(30),
        }
    }
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API Key
    pub api_key: Option<String>,
    /// 模型名称
    pub model: String,
    /// API 基础 URL
    pub base_url: String,
    /// 请求超时（秒）
    #[serde(with = "serde_duration_seconds")]
    pub timeout: Duration,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "gpt-4".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout: Duration::from_secs(60),
        }
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用沙箱
    pub sandbox_enabled: bool,
    /// 沙箱超时时间（秒）
    #[serde(with = "serde_duration_seconds")]
    pub sandbox_timeout: Duration,
    /// 允许执行命令列表
    pub allowed_commands: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            sandbox_enabled: true,
            sandbox_timeout: Duration::from_secs(30),
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "echo".to_string(),
                "pwd".to_string(),
                "mkdir".to_string(),
                "rm".to_string(),
                "cp".to_string(),
                "mv".to_string(),
            ],
        }
    }
}

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// 操作系统类型
    pub os_type: String,
    /// 主机名
    pub hostname: String,
    /// 工作目录
    pub working_dir: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            os_type: std::env::consts::OS.to_string(),
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            working_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
        }
    }
}

/// ArkCore 统一配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 应用配置
    pub app: AppConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// LLM 配置
    pub llm: LlmConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 平台配置
    pub platform: PlatformConfig,
}

/// Duration 序列化辅助模块
mod serde_duration_seconds {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: u64 = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// 配置加载器
pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self {
            config_path: Self::default_config_path(),
        }
    }

    /// 从指定路径加载配置
    pub fn with_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config_path = path.into();
        self
    }

    /// 默认配置路径
    fn default_config_path() -> PathBuf {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".config"))
                    .unwrap_or_else(|| PathBuf::from("."))
            });
        config_home.join("arkcore").join("config.toml")
    }

    /// 检测配置文件格式
    fn detect_format(path: &Path) -> ConfigFormat {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "yaml" | "yml" => ConfigFormat::Yaml,
                "toml" => ConfigFormat::Toml,
                "json" => ConfigFormat::Json,
                _ => ConfigFormat::Toml,
            })
            .unwrap_or(ConfigFormat::Toml)
    }

    /// 加载配置
    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("读取配置文件失败: {}", self.config_path.display()))?;

        let format = Self::detect_format(&self.config_path);
        let mut config: Config = match format {
            ConfigFormat::Toml => toml::from_str(&content).with_context(|| {
                format!("解析 TOML 配置文件失败: {}", self.config_path.display())
            })?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content).with_context(|| {
                format!("解析 YAML 配置文件失败: {}", self.config_path.display())
            })?,
            ConfigFormat::Json => serde_json::from_str(&content).with_context(|| {
                format!("解析 JSON 配置文件失败: {}", self.config_path.display())
            })?,
        };

        // 应用环境变量覆盖
        config.apply_env_overrides();

        Ok(config)
    }

    /// 保存配置
    pub fn save(&self, config: &Config) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }

        let format = Self::detect_format(&self.config_path);
        let content = match format {
            ConfigFormat::Toml => {
                toml::to_string_pretty(config).context("序列化配置为 TOML 失败")?
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(config).context("序列化配置为 YAML 失败")?
            }
            ConfigFormat::Json => {
                serde_json::to_string_pretty(config).context("序列化配置为 JSON 失败")?
            }
        };

        fs::write(&self.config_path, content)
            .with_context(|| format!("写入配置文件失败: {}", self.config_path.display()))?;

        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置格式枚举
#[derive(Debug, Clone, Copy)]
enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl Config {
    /// 从默认路径加载配置
    pub fn load() -> Result<Self> {
        ConfigLoader::new().load()
    }

    /// 保存到默认路径
    pub fn save(&self) -> Result<()> {
        ConfigLoader::new().save(self)
    }

    /// 应用环境变量覆盖
    ///
    /// 环境变量格式：`ARKCORE_*`
    /// 例如：`ARKCORE_APP_LOG_LEVEL=debug`
    fn apply_env_overrides(&mut self) {
        // 应用通用覆盖
        if let Ok(val) = std::env::var("ARKCORE_ENV") {
            if let Ok(env) = val.parse() {
                self.app.env = env;
            }
        }
        if let Ok(val) = std::env::var("ARKCORE_APP_LOG_LEVEL") {
            self.app.log_level = val;
        }

        // 数据库覆盖
        if let Ok(val) = std::env::var("ARKCORE_DATABASE_URL") {
            self.database.url = val;
        }
        if let Ok(val) = std::env::var("ARKCORE_DATABASE_POOL_SIZE") {
            if let Ok(pool_size) = val.parse() {
                self.database.pool_size = pool_size;
            }
        }

        // LLM 覆盖
        if let Ok(val) = std::env::var("ARKCORE_LLM_API_KEY") {
            self.llm.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("ARKCORE_LLM_MODEL") {
            self.llm.model = val;
        }
        if let Ok(val) = std::env::var("ARKCORE_LLM_BASE_URL") {
            self.llm.base_url = val;
        }

        // 安全覆盖
        if let Ok(val) = std::env::var("ARKCORE_SECURITY_SANDBOX_ENABLED") {
            self.security.sandbox_enabled = val.to_lowercase() != "false";
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(Environment::Test.to_string(), "test");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app.name, "arkcore");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.database.pool_size, 5);
    }

    #[test]
    fn test_config_loader_default_path() {
        let loader = ConfigLoader::new();
        assert!(loader.config_path.to_string_lossy().contains("arkcore"));
    }
}
