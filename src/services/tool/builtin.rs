//! 内置工具实现
//!
//! 提供常用的内置工具：文件读写、计算器、日期时间等

use crate::services::tool::trait::{Schema, Tool, ToolError, ToolResult};
use async_trait::async_trait;
use chrono::{DateTime as ChronoDateTime, Local, Utc};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

/// 文件读取工具
pub struct FileRead;

impl FileRead {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileRead {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileRead {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "读取文件内容。输入文件路径，返回文件内容。"
    }

    fn parameters(&self) -> Schema {
        Schema::string(Some("要读取的文件路径"), true)
    }

    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let path = params
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("path must be a string".to_string()))?;

        // 安全检查：防止路径遍历
        if path.contains("..") || path.contains('~') {
            return Err(ToolError::InvalidParams(
                "Path traversal detected".to_string(),
            ));
        }

        // 如果是相对路径，转换为绝对路径（基于当前工作目录）
        let full_path = if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            let cwd = std::env::current_dir().map_err(|e| ToolError::IoError(e))?;
            cwd.join(path).to_string_lossy().to_string()
        };

        let content = fs::read_to_string(&full_path)
            .await
            .map_err(|e| ToolError::IoError(e))?;

        Ok(ToolResult::ok(content))
    }
}

/// 文件写入工具
pub struct FileWrite;

impl FileWrite {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileWrite {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileWrite {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "写入内容到文件。参数：path（文件路径）, content（要写入的内容）"
    }

    fn parameters(&self) -> Schema {
        Schema {
            schema_type: "object".to_string(),
            description: Some("包含 path 和 content 的对象".to_string()),
            required: true,
            default: None,
            min: None,
            max: None,
            enum_values: None,
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingParam("path".to_string()))?;

        let content = params
            .get("content")
            .ok_or_else(|| ToolError::MissingParam("content".to_string()))?;

        // 安全检查：防止路径遍历
        if path.contains("..") || path.contains('~') {
            return Err(ToolError::InvalidParams(
                "Path traversal detected".to_string(),
            ));
        }

        // 如果是相对路径，转换为绝对路径（基于当前工作目录）
        let full_path = if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            let cwd = std::env::current_dir().map_err(|e| ToolError::IoError(e))?;
            cwd.join(path).to_string_lossy().to_string()
        };

        // 确保父目录存在
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ToolError::IoError(e))?;
        }

        fs::write(&full_path, content.to_string())
            .await
            .map_err(|e| ToolError::IoError(e))?;

        Ok(ToolResult::ok(format!("Successfully wrote to {}", full_path)))
    }
}

/// 计算器工具
pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for Calculator {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "执行数学计算。参数：expression（数学表达式字符串）。支持：+, -, *, /, %, ^, sqrt, sin, cos, tan, log, abs"
    }

    fn parameters(&self) -> Schema {
        Schema::string(Some("数学表达式，如 2+3*4 或 sqrt(16)"), true)
    }

    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let expression = params
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("expression must be a string".to_string()))?;

        // 简单的表达式求值（避免 eval 安全问题）
        let result = evaluate_expression(expression)?;

        Ok(ToolResult::ok(result.to_string()))
    }
}

/// 简单的数学表达式求值
fn evaluate_expression(expr: &str) -> Result<f64, ToolError> {
    let expr = expr.trim();

    // 移除常见的数学函数
    let expr = expr.replace("sqrt(", "math_sqrt(");
    let expr = expr.replace("sin(", "math_sin(");
    let expr = expr.replace("cos(", "math_cos(");
    let expr = expr.replace("tan(", "math_tan(");
    let expr = expr.replace("log(", "math_log(");
    let expr = expr.replace("abs(", "math_abs(");

    // 使用 meval 进行表达式求值
    meval::eval_str(&expr).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
}

/// 日期时间工具
pub struct DateTime;

impl DateTime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DateTime {
    fn name(&self) -> &str {
        "datetime"
    }

    fn description(&self) -> &str {
        "获取当前日期时间。参数：format（可选，日期格式字符串）。默认格式：%Y-%m-%d %H:%M:%S"
    }

    fn parameters(&self) -> Schema {
        Schema {
            schema_type: "object".to_string(),
            description: Some("可选的 format 参数".to_string()),
            required: false,
            default: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
            min: None,
            max: None,
            enum_values: None,
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");

        let now: ChronoDateTime<Local> = Local::now();
        let formatted = now.format(format).to_string();

        Ok(ToolResult::ok(formatted))
    }
}

/// 获取 UTC 日期时间
pub struct UtcDateTime;

impl UtcDateTime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UtcDateTime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for UtcDateTime {
    fn name(&self) -> &str {
        "utc_datetime"
    }

    fn description(&self) -> &str {
        "获取当前 UTC 日期时间。参数：format（可选，日期格式字符串）。默认格式：%Y-%m-%d %H:%M:%S"
    }

    fn parameters(&self) -> Schema {
        Schema {
            schema_type: "object".to_string(),
            description: Some("可选的 format 参数".to_string()),
            required: false,
            default: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
            min: None,
            max: None,
            enum_values: None,
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");

        let now: ChronoDateTime<Utc> = Utc::now();
        let formatted = now.format(format).to_string();

        Ok(ToolResult::ok(formatted))
    }
}

/// 获取所有内置工具
pub fn all_builtin_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(FileRead::new()) as Arc<dyn Tool>,
        Arc::new(FileWrite::new()) as Arc<dyn Tool>,
        Arc::new(Calculator::new()) as Arc<dyn Tool>,
        Arc::new(DateTime::new()) as Arc<dyn Tool>,
        Arc::new(UtcDateTime::new()) as Arc<dyn Tool>,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_calculator_add() {
        let calc = Calculator::new();
        let result = calc.execute(json!("2 + 3")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "5");
    }

    #[tokio::test]
    async fn test_calculator_complex() {
        let calc = Calculator::new();
        let result = calc.execute(json!("2 + 3 * 4")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "14");
    }

    #[tokio::test]
    async fn test_calculator_sqrt() {
        let calc = Calculator::new();
        let result = calc.execute(json!("sqrt(16)")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "4");
    }

    #[tokio::test]
    async fn test_datetime_default_format() {
        let dt = DateTime::new();
        let result = dt.execute(json!({})).await.unwrap();
        assert!(result.success);
        // 验证格式包含日期和时间
        assert!(result.output.contains("-"));
        assert!(result.output.contains(":"));
    }

    #[tokio::test]
    async fn test_datetime_custom_format() {
        let dt = DateTime::new();
        let result = dt
            .execute(json!({"format": "%Y-%m-%d"}))
            .await
            .unwrap();
        assert!(result.success);
        // 验证只有日期
        assert!(result.output.matches("-").count() >= 2);
    }

    #[tokio::test]
    async fn test_utc_datetime() {
        let dt = UtcDateTime::new();
        let result = dt.execute(json!({})).await.unwrap();
        assert!(result.success);
    }
}
