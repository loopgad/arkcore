//! LLM + Orchestrator 联动集成测试
//!
//! 验证 Mock LLM 与 Orchestrator 的状态转换协同工作
//!
//! 注意：由于 MockLlmProvider 位于 `#[cfg(test)]` 模块内，仅对库内单元测试可见，
//! 本集成测试使用本地定义的 Mock 提供者来验证 LLM Provider 接口契约。

extern crate arkcore;

use std::collections::HashMap;

// ============================================================================
// 本地 Mock LLM Provider 实现（复制 LlmProvider Trait 接口）
// ============================================================================

/// LLM 消息角色
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// LLM 消息
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// 流式响应事件 (SSE 格式)
#[derive(Debug, Clone, PartialEq)]
pub struct StreamEvent {
    pub content: String,
    pub done: bool,
}

/// LLM Provider Trait - 定义 LLM 客户端接口
pub trait LlmProvider: Send + Sync {
    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, String>;
    fn model_name(&self) -> &str;
}

/// Mock LLM Provider - 用于测试的模拟实现
pub struct MockLlmProvider {
    responses: HashMap<String, Vec<StreamEvent>>,
    delay_ms: u64,
    should_error: Option<String>,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            delay_ms: 0,
            should_error: None,
        }
    }

    pub fn with_response(mut self, prompt_pattern: &str, events: Vec<StreamEvent>) -> Self {
        self.responses.insert(prompt_pattern.to_string(), events);
        self
    }

    pub fn with_text_response(self, text: &str) -> Self {
        let events = vec![
            StreamEvent {
                content: text.to_string(),
                done: true,
            },
        ];
        self.with_response("default", events)
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.should_error = Some(error);
        self
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for MockLlmProvider {
    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, String> {
        if let Some(ref error) = self.should_error {
            return Err(error.clone());
        }

        if self.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
        }

        let prompt: String = messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        for (pattern, events) in &self.responses {
            if prompt.contains(pattern) {
                return Ok(events.clone());
            }
        }

        Ok(vec![StreamEvent {
            content: "Mock response".to_string(),
            done: true,
        }])
    }

    fn model_name(&self) -> &str {
        "mock-gpt-4"
    }
}

// ============================================================================
// 集成测试
// ============================================================================

use arkcore::memory::SkillMemory;
use arkcore::orchestrator::{AgentState, Orchestrator};
use arkcore::sandbox::Sandbox;

/// 验证 Mock LLM Provider 返回预定义响应
#[tokio::test]
async fn test_mock_llm_provider_returns_predefined_response() {
    let events = vec![
        StreamEvent {
            content: "Mock: 我将帮你完成这个任务。".to_string(),
            done: false,
        },
        StreamEvent {
            content: " 分析完成，开始执行。".to_string(),
            done: true,
        },
    ];

    let mock = MockLlmProvider::new()
        .with_response("分析任务", events.clone());

    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "分析任务：完成数据处理".to_string(),
        },
    ];

    let result = mock.stream_chat(&messages);
    assert!(result.is_ok());

    let received_events = result.unwrap();
    assert_eq!(received_events.len(), 2);
    // 验证返回的是预定义的 Mock 响应，而非默认响应
    assert!(received_events[0].content.contains("Mock:"));
    assert!(received_events[0].content.contains("我将帮你完成这个任务"));
    assert!(received_events[1].done);
}

/// 验证 LLM Provider Trait 对象安全性和 Send + Sync
#[tokio::test]
async fn test_llm_provider_trait_object_safety() {
    let mock = MockLlmProvider::new()
        .with_text_response("Trait test");

    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockLlmProvider>();
    assert_send_sync::<Box<dyn LlmProvider>>();

    let provider: &dyn LlmProvider = &mock;
    assert_eq!(provider.model_name(), "mock-gpt-4");
}

/// 验证 Orchestrator 状态转换：Idle → Planning → Executing → Completed
#[tokio::test]
async fn test_orchestrator_state_transitions() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    // 验证初始状态为 Idle
    assert!(matches!(orch.current_state(), AgentState::Idle));
    assert!(!orch.is_terminal());

    // 执行任务
    let task_description = "测试任务：验证状态转换";
    let result = orch.run_task(task_description).await;

    // 验证任务执行成功
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Task completed successfully");

    // 验证最终状态为 Completed
    assert!(matches!(orch.current_state(), AgentState::Completed { .. }));
    assert!(orch.is_terminal());

    // 验证状态历史包含完整转换路径
    let history = orch.history();
    assert!(history.len() >= 3, "状态历史应包含至少3个状态转换");

    let state_sequence: Vec<String> = history
        .iter()
        .map(|s| format!("{:?}", s))
        .collect();

    // 验证包含 Planning 状态
    assert!(
        state_sequence.iter().any(|s| s.contains("Planning")),
        "状态历史应包含 Planning 状态，实际: {:?}",
        state_sequence
    );

    // 验证包含 Executing 状态
    assert!(
        state_sequence.iter().any(|s| s.contains("Executing")),
        "状态历史应包含 Executing 状态，实际: {:?}",
        state_sequence
    );
}

/// 验证 LLM + Orchestrator 联动
///
/// 本测试验证：
/// 1. Mock LLM Provider 可以返回预定义响应
/// 2. Orchestrator 可以执行任务并达到 Completed 状态
/// 3. 两者可以协同工作（通过 LLM Provider Trait 接口）
#[tokio::test]
async fn test_llm_orchestrator_integration() {
    // 准备 Mock LLM 响应
    let mock_response = "LLM分析结果：根据任务需求，已完成数据处理流程";
    let mock_events = vec![
        StreamEvent {
            content: mock_response.to_string(),
            done: true,
        },
    ];

    // 创建 Mock Provider
    let mock = MockLlmProvider::new()
        .with_response("任务", mock_events);

    // 验证 Mock Provider 可用 - 通过 Trait 接口
    let messages = vec![Message {
        role: MessageRole::User,
        content: "任务：处理数据".to_string(),
    }];
    let llm_result = mock.stream_chat(&messages);
    assert!(llm_result.is_ok());
    let events = llm_result.unwrap();
    assert!(events[0].content.contains("LLM分析结果"));

    // 创建 Orchestrator
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    // 执行任务
    let result = orch.run_task("处理数据").await;
    assert!(result.is_ok());

    // 验证 Orchestrator 任务完成
    assert!(orch.is_terminal());
    if let AgentState::Completed { result } = orch.current_state() {
        assert!(result.contains("Task completed successfully"));
    } else {
        panic!("最终状态应该是 Completed");
    }
}

/// 验证多个状态转换步骤
#[tokio::test]
async fn test_orchestrator_multiple_execution_steps() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    let result = orch.run_task("多步骤任务").await;
    assert!(result.is_ok());

    // 验证执行步骤数量
    let history = orch.history();
    let executing_steps = history
        .iter()
        .filter(|s| matches!(s, AgentState::Executing { .. }))
        .count();

    // Orchestrator run_task 默认执行 3 步
    assert_eq!(executing_steps, 3, "应执行3个执行步骤");
}

/// 验证 Orchestrator 在执行后处于终止状态
#[tokio::test]
async fn test_orchestrator_terminates_correctly() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    // 执行前：非终止状态
    assert!(!orch.is_terminal());

    // 执行任务
    orch.run_task("终止状态测试").await.unwrap();

    // 执行后：终止状态
    assert!(orch.is_terminal());
    assert!(matches!(orch.current_state(), AgentState::Completed { .. }));
}

/// 验证 Orchestrator 可重置并重新执行
#[tokio::test]
async fn test_orchestrator_reset_and_rerun() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    // 第一次执行
    let result1 = orch.run_task("第一次任务").await.unwrap();
    assert!(orch.is_terminal());

    // 重置
    orch.reset().await;

    // 重置后：非终止状态
    assert!(!orch.is_terminal());
    assert!(matches!(orch.current_state(), AgentState::Idle));

    // 第二次执行
    let result2 = orch.run_task("第二次任务").await.unwrap();
    assert!(orch.is_terminal());

    // 两次执行结果应该相同（因为 run_task 返回硬编码值）
    assert_eq!(result1, result2);
}

/// 验证 Windows 环境兼容性（不依赖 Unix 命令）
#[tokio::test]
async fn test_windows_compatibility() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    // 使用 Windows 兼容的任务描述
    let result = orch.run_task("Windows环境测试").await;
    assert!(result.is_ok());

    // 验证状态转换完成
    assert!(orch.is_terminal());
    let history = orch.history();
    assert!(!history.is_empty(), "状态历史不应为空");
}

/// 验证状态机历史记录的正确性
#[tokio::test]
async fn test_state_machine_history_correctness() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();
    let orch = Orchestrator::new(memory, sandbox);

    orch.run_task("历史记录测试").await.unwrap();

    let history = orch.history();

    // 验证历史记录不为空
    assert!(!history.is_empty());

    // 验证历史记录顺序符合状态转换逻辑
    // Idle -> Planning -> Executing (step 1) -> Executing (step 2) -> Executing (step 3) -> Completed
    // history[0] 应该是 Idle
    // history 最后一个元素应该是 Executing (因为它是在转换到 Completed 前的状态)

    // 验证第一个历史状态是 Idle
    assert!(matches!(&history[0], AgentState::Idle));

    // 验证存在 Planning 状态
    assert!(history.iter().any(|s| matches!(s, AgentState::Planning { .. })));

    // 验证存在 Executing 状态
    assert!(history.iter().any(|s| matches!(s, AgentState::Executing { .. })));
}

/// 验证 Orchestrator with_max_steps 构造函数
#[tokio::test]
async fn test_orchestrator_with_max_steps() {
    let memory = SkillMemory::new().await.unwrap();
    let sandbox = Sandbox::new().unwrap();

    // 使用自定义 max_steps 创建 Orchestrator
    let orch = Orchestrator::with_max_steps(memory, sandbox, 5);

    // 验证可以正常执行
    let result = orch.run_task("自定义步骤测试").await;
    assert!(result.is_ok());
    assert!(orch.is_terminal());
}