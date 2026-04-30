//! Orchestrator 模块
//!
//! AI Agent 调度器 - 基于状态机的任务编排

mod state_machine;

pub use state_machine::{AgentState, AgentStateMachine};

use crate::memory::SkillMemory;
use crate::sandbox::Sandbox;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Orchestrator 主结构
///
/// 负责协调 LLM、记忆和沙盒执行
pub struct Orchestrator {
    state_machine: RwLock<AgentStateMachine>,
    memory: Arc<RwLock<SkillMemory>>,
    sandbox: Arc<Sandbox>,
    #[allow(dead_code)]
    max_steps: usize,
}

impl Orchestrator {
    /// 创建新的 Orchestrator 实例
    pub fn new(memory: SkillMemory, sandbox: Sandbox) -> Self {
        Self {
            state_machine: RwLock::new(AgentStateMachine::new(100)),
            memory: Arc::new(RwLock::new(memory)),
            sandbox: Arc::new(sandbox),
            max_steps: 10,
        }
    }

    /// 创建带自定义最大步数的 Orchestrator
    pub fn with_max_steps(memory: SkillMemory, sandbox: Sandbox, max_steps: usize) -> Self {
        Self {
            state_machine: RwLock::new(AgentStateMachine::new(100)),
            memory: Arc::new(RwLock::new(memory)),
            sandbox: Arc::new(sandbox),
            max_steps,
        }
    }

    /// 获取当前状态
    pub fn current_state(&self) -> AgentState {
        // 使用 try_read() 返回 Result，避免 panic
        self.state_machine
            .try_read()
            .map(|guard| guard.current_state().clone())
            .unwrap_or(AgentState::Idle)
    }

    /// 获取状态历史
    pub fn history(&self) -> Vec<AgentState> {
        self.state_machine
            .try_read()
            .map(|guard| guard.history().to_vec())
            .unwrap_or_default()
    }

    /// 检查是否处于终止状态
    pub fn is_terminal(&self) -> bool {
        self.state_machine
            .try_read()
            .map(|guard| guard.is_terminal())
            .unwrap_or(true)
    }

    /// 执行任务
    pub async fn run_task(&self, task: &str) -> Result<String> {
        // 规划阶段
        self.state_machine
            .write()
            .await
            .transition(AgentState::Planning { task: task.to_string() });

        // 模拟执行阶段
        let total_steps = 3;
        for step in 1..=total_steps {
            if self.state_machine.read().await.is_terminal() {
                break;
            }

            self.state_machine
                .write()
                .await
                .transition(AgentState::Executing {
                    step,
                    total: total_steps,
                });

            // 模拟执行延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        // 完成
        let result = "Task completed successfully".to_string();
        self.state_machine
            .write()
            .await
            .transition(AgentState::Completed { result: result.clone() });

        Ok(result)
    }

    /// 执行带审批的命令
    pub async fn run_with_approval(&self, task: &str, command: &str) -> Result<String> {
        // 规划
        self.state_machine
            .write()
            .await
            .transition(AgentState::Planning { task: task.to_string() });

        // 安全检查
        let check = self.sandbox.security_check(command);
        let risk_level = if check.passed {
            "low".to_string()
        } else {
            "high".to_string()
        };

        // 等待审批
        self.state_machine
            .write()
            .await
            .transition(AgentState::AwaitingApproval {
                command: command.to_string(),
                risk_level: risk_level.clone(),
            });

        // 如果风险高，拒绝执行
        if risk_level == "high" {
            self.state_machine
                .write()
                .await
                .transition(AgentState::Failed {
                    reason: "Security check failed".to_string(),
                });
            anyhow::bail!("Security check failed: {:?}", check.violations);
        }

        // 执行
        self.state_machine
            .write()
            .await
            .transition(AgentState::Executing { step: 1, total: 1 });

        let result = self.sandbox.execute(command).await?;
        let is_success = result.is_success();
        let output = if is_success {
            result.stdout
        } else {
            result.stderr
        };
        let exit_code = result.exit_code;

        // 反思
        self.state_machine
            .write()
            .await
            .transition(AgentState::Reflecting {
                assessment: if is_success {
                    "Command executed successfully".to_string()
                } else {
                    format!("Command failed with exit code: {:?}", exit_code)
                },
            });

        // 完成
        self.state_machine
            .write()
            .await
            .transition(AgentState::Completed { result: output.clone() });

        Ok(output)
    }

    /// 重置状态机
    pub async fn reset(&self) {
        self.state_machine.write().await.reset();
    }

    /// 获取记忆引用
    pub fn memory(&self) -> &Arc<RwLock<SkillMemory>> {
        &self.memory
    }

    /// 获取沙盒引用
    pub fn sandbox(&self) -> &Arc<Sandbox> {
        &self.sandbox
    }
}


/// SAFETY: Orchestrator is safe to send across thread boundaries because:
/// - All interior mutability is protected by Arc<RwLock<...>> or similar synchronization
/// - The Sandbox and SkillMemory are both Send + Sync
/// - No raw pointers or unsafe data structures are used
///
/// This allows Orchestrator to be used in async contexts that may move between threads
unsafe impl Send for Orchestrator {}

/// SAFETY: Orchestrator is safe to share references across threads because:
/// - All state access is guarded by synchronization primitives (Arc<RwLock>)
/// - The internal components (Sandbox, SkillMemory) are all thread-safe
/// - No mutable references are leaked outside the struct
unsafe impl Sync for Orchestrator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_task() {
        let memory = SkillMemory::new().await.unwrap();
        let sandbox = Sandbox::new().unwrap();
        let orch = Orchestrator::new(memory, sandbox);

        let result = orch.run_task("Test task").await.unwrap();
        assert_eq!(result, "Task completed successfully");
        assert!(orch.is_terminal());
    }

    #[tokio::test]
    async fn test_run_with_approval_safe_command() {
        let memory = SkillMemory::new().await.unwrap();
        let sandbox = Sandbox::new().unwrap();
        let orch = Orchestrator::new(memory, sandbox);

        let result = orch
            .run_with_approval("List files", "ls -la")
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_run_with_approval_dangerous_command() {
        let memory = SkillMemory::new().await.unwrap();
        let sandbox = Sandbox::new().unwrap();
        let orch = Orchestrator::new(memory, sandbox);

        // 危险命令 - 使用 find with -exec 是跨平台可检测的危险模式
        // 在 Unix 上: find / -exec rm {} \; 会被检测
        // 在 Windows 上: 类似模式也会被检测
        let result = orch
            .run_with_approval("Delete files", "find . -exec rm -rf {} \\;")
            .await;

        assert!(result.is_err());
        assert!(orch.is_terminal());
    }

    #[test]
    fn test_state_transitions() {
        let mut sm = AgentStateMachine::new(10);

        sm.transition(AgentState::Planning {
            task: "Test".to_string(),
        });
        assert!(matches!(sm.current_state(), AgentState::Planning { .. }));

        sm.transition(AgentState::Executing { step: 1, total: 3 });
        assert!(matches!(sm.current_state(), AgentState::Executing { .. }));

        sm.transition(AgentState::Completed {
            result: "Done".to_string(),
        });
        assert!(sm.is_terminal());
    }
}