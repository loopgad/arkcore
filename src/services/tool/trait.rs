//! 工具接口定义
//!
//! 参考 LangChain 的 Tool Calling 机制设计的工具接口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 工具参数 Schema
///
/// 用于描述工具参数的 JSON Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// 参数类型
    #[serde(rename = "type")]
    pub schema_type: String,
    /// 参数描述
    pub description: Option<String>,
    /// 是否必需
    pub required: bool,
    /// 默认值
    #[serde(default)]
    pub default: Option<Value>,
    /// 最小值（用于数值类型）
    pub min: Option<f64>,
    /// 最大值（用于数值类型）
    pub max: Option<f64>,
    /// 枚举值（用于枚举类型）
    pub enum_values: Option<Vec<String>>,
}

impl Schema {
    /// 创建字符串类型的 Schema
    pub fn string(description: Option<&str>, required: bool) -> Self {
        Self {
            schema_type: "string".to_string(),
            description: description.map(|s| s.to_string()),
            required,
            default: None,
            min: None,
            max: None,
            enum_values: None,
        }
    }

    /// 创建数值类型的 Schema
    pub fn number(description: Option<&str>, required: bool, min: Option<f64>, max: Option<f64>) -> Self {
        Self {
            schema_type: "number".to_string(),
            description: description.map(|s| s.to_string()),
            required,
            default: None,
            min,
            max,
            enum_values: None,
        }
    }

    /// 创建布尔类型的 Schema
    pub fn boolean(description: Option<&str>, required: bool) -> Self {
        Self {
            schema_type: "boolean".to_string(),
            description: description.map(|s| s.to_string()),
            required,
            default: None,
            min: None,
            max: None,
            enum_values: None,
        }
    }

    /// 创建整数类型的 Schema
    pub fn integer(description: Option<&str>, required: bool, min: Option<i64>, max: Option<i64>) -> Self {
        Self {
            schema_type: "integer".to_string(),
            description: description.map(|s| s.to_string()),
            required,
            default: None,
            min: min.map(|v| v as f64),
            max: max.map(|v| v as f64),
            enum_values: None,
        }
    }
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 是否成功
    pub success: bool,
    /// 输出内容
    pub output: String,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 消耗的 token 数（可选）
    pub tokens_used: Option<u64>,
}

impl ToolResult {
    /// 创建成功结果
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
            tokens_used: None,
        }
    }

    /// 创建错误结果
    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
            tokens_used: None,
        }
    }

    /// 设置消耗的 token 数
    pub fn with_tokens(mut self, tokens: u64) -> Self {
        self.tokens_used = Some(tokens);
        self
    }
}

/// 工具接口
///
/// 所有工具必须实现此 trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// 获取工具名称
    fn name(&self) -> &str;

    /// 获取工具描述
    fn description(&self) -> &str;

    /// 获取工具参数 Schema
    fn parameters(&self) -> Schema;

    /// 执行工具
    ///
    /// # Arguments
    ///
    /// * `params` - JSON 格式的参数
    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError>;
}

/// 工具执行错误
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// 参数无效
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// 参数缺失
    #[error("Missing required parameter: {0}")]
    MissingParam(String),

    /// 执行失败
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// 工具不存在
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// 工具已存在
    #[error("Tool already exists: {0}")]
    AlreadyExists(String),

    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON 错误
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<ToolError> for ToolResult {
    fn from(err: ToolError) -> Self {
        ToolResult::err(err.to_string())
    }
}
