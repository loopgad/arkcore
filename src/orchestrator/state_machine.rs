//! ArkCore 状态机模块
//!
//! AI Agent 状态转换管理

use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::info;

/// Agent 状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    /// 空闲状态
    Idle,
    /// 规划状态
    Planning { task: String },
    /// 执行状态
    Executing { step: usize, total: usize },
    /// 等待审批状态
    AwaitingApproval { command: String, risk_level: String },
    /// 反思状态
    Reflecting { assessment: String },
    /// 已完成状态
    Completed { result: String },
    /// 失败状态
    Failed { reason: String },
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentState::Idle => write!(f, "Idle"),
            AgentState::Planning { task } => write!(f, "Planning: {}", task),
            AgentState::Executing { step, total } => write!(f, "Executing ({}/{})", step, total),
            AgentState::AwaitingApproval { command, risk_level } => {
                write!(f, "AwaitingApproval: {} [{}]", command, risk_level)
            }
            AgentState::Reflecting { assessment } => write!(f, "Reflecting: {}", assessment),
            AgentState::Completed { result } => write!(f, "Completed: {}", result),
            AgentState::Failed { reason } => write!(f, "Failed: {}", reason),
        }
    }
}

/// Agent 状态机
pub struct AgentStateMachine {
    state: AgentState,
    history: Vec<AgentState>,
    max_history: usize,
}

impl AgentStateMachine {
    /// 创建新的状态机实例
    pub fn new(max_history: usize) -> Self {
        Self {
            state: AgentState::Idle,
            history: Vec::new(),
            max_history,
        }
    }

    /// 执行状态转换
    pub fn transition(&mut self, new_state: AgentState) {
        info!("状态转换: {:?} -> {:?}", self.state, new_state);
        self.history.push(self.state.clone());

        // 保持历史记录在限制内
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        self.state = new_state;
    }

    /// 检查是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            AgentState::Completed { .. } | AgentState::Failed { .. }
        )
    }

    /// 获取当前状态
    pub fn current_state(&self) -> &AgentState {
        &self.state
    }

    /// 获取状态历史
    pub fn history(&self) -> &[AgentState] {
        &self.history
    }

    /// 重置状态机到初始状态
    pub fn reset(&mut self) {
        if !matches!(self.state, AgentState::Idle) {
            self.history.push(self.state.clone());
        }
        self.state = AgentState::Idle;
    }

    /// 获取已执行的步数
    pub fn executed_steps(&self) -> usize {
        self.history
            .iter()
            .filter(|s| matches!(s, AgentState::Executing { .. }))
            .count()
    }
}

impl Default for AgentStateMachine {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = AgentStateMachine::new(10);
        assert_eq!(sm.current_state(), &AgentState::Idle);
        assert!(!sm.is_terminal());
    }

    #[test]
    fn test_transition() {
        let mut sm = AgentStateMachine::new(10);
        sm.transition(AgentState::Planning {
            task: "Test task".to_string(),
        });
        assert!(matches!(sm.current_state(), AgentState::Planning { .. }));
        assert_eq!(sm.history().len(), 1);
    }

    #[test]
    fn test_terminal_states() {
        let mut sm = AgentStateMachine::new(10);

        sm.transition(AgentState::Completed {
            result: "Done".to_string(),
        });
        assert!(sm.is_terminal());

        sm.reset();
        sm.transition(AgentState::Failed {
            reason: "Error".to_string(),
        });
        assert!(sm.is_terminal());
    }

    #[test]
    fn test_history_limit() {
        let mut sm = AgentStateMachine::new(3);

        for i in 0..5 {
            sm.transition(AgentState::Executing {
                step: i,
                total: 5,
            });
        }

        // 只应保留最后 3 个状态（加上 Idle 前的那个）
        assert!(sm.history().len() <= 3);
    }

    #[test]
    fn test_reset() {
        let mut sm = AgentStateMachine::new(10);
        sm.transition(AgentState::Planning {
            task: "Test".to_string(),
        });
        sm.reset();
        assert_eq!(sm.current_state(), &AgentState::Idle);
    }
}