//! 工具注册表
//!
//! 管理和调度所有可用工具的中心注册表

use crate::services::tool::trait::{Schema, Tool, ToolError, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工具注册表
///
/// 管理所有注册的工具，支持工具的注册、查找和调用
pub struct ToolRegistry {
    /// 工具存储
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个工具
    ///
    /// # Arguments
    ///
    /// * `tool` - 要注册的工具
    ///
    /// # Returns
    ///
    /// 如果工具已存在则返回错误
    pub async fn register<T: Tool + 'static>(&self, tool: T) -> Result<(), ToolError> {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().await;

        if tools.contains_key(&name) {
            return Err(ToolError::AlreadyExists(name));
        }

        tools.insert(name, Arc::new(tool));
        Ok(())
    }

    /// 批量注册工具
    pub async fn register_many<T: Tool + 'static>(&self, tools: Vec<T>) -> Result<(), ToolError> {
        for tool in tools {
            self.register(tool).await?;
        }
        Ok(())
    }

    /// 注销一个工具
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.write().await.remove(name)
    }

    /// 获取工具信息（不执行）
    pub async fn get(&self, name: &str) -> Option<ToolInfo> {
        let tools = self.tools.read().await;
        tools.get(name).map(|tool| ToolInfo {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: tool.parameters(),
        })
    }

    /// 列出所有工具
    pub async fn list(&self) -> Vec<ToolInfo> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            })
            .collect()
    }

    /// 执行工具
    ///
    /// # Arguments
    ///
    /// * `name` - 工具名称
    /// * `params` - JSON 格式的参数
    pub async fn execute(&self, name: &str, params: Value) -> Result<ToolResult, ToolError> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        tool.execute(params).await
    }

    /// 检查工具是否存在
    pub async fn contains(&self, name: &str) -> bool {
        self.tools.read().await.contains_key(name)
    }

    /// 获取工具数量
    pub async fn len(&self) -> usize {
        self.tools.read().await.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.tools.read().await.is_empty()
    }

    /// 清空所有工具
    pub async fn clear(&self) {
        self.tools.write().await.clear();
    }
}

/// 工具信息（用于列表和描述）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 Schema
    pub parameters: Schema,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::tool::trait::{Schema, Tool};
    use async_trait::async_trait;
    use serde_json::json;

    /// 测试用工具
    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn parameters(&self) -> Schema {
            Schema::string(Some("Test parameter"), true)
        }

        async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
            Ok(ToolResult::ok(params.to_string()))
        }
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await.unwrap();

        let info = registry.get("test_tool").await;
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "test_tool");
        assert_eq!(info.description, "A test tool");
    }

    #[tokio::test]
    async fn test_register_duplicate() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await.unwrap();

        let result = registry.register(TestTool).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::AlreadyExists(_)));
    }

    #[tokio::test]
    async fn test_execute() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await.unwrap();

        let result = registry
            .execute("test_tool", json!({"test": "value"}))
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_execute_not_found() {
        let registry = ToolRegistry::new();

        let result = registry.execute("nonexistent", json!({})).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await.unwrap();

        let list = registry.list().await;
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await.unwrap();

        let removed = registry.unregister("test_tool").await;
        assert!(removed.is_some());

        let exists = registry.contains("test_tool").await;
        assert!(!exists);
    }
}
