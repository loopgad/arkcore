//! Agent 状态 Hook
//!
//! 管理 Agent 状态和历史。

use super::super::{AgentInfo, AgentStatus, MetricsData};

/// Agent 状态 Hook 状态
#[derive(Debug, Clone)]
pub struct AgentState {
    /// 当前 Agent 信息
    pub current_agent: Option<AgentInfo>,
    /// 所有 Agent 列表
    pub agents: Vec<AgentInfo>,
    /// 系统指标
    pub metrics: Option<MetricsData>,
    /// 状态更新历史
    pub history: Vec<AgentInfo>,
    /// 最大历史记录数
    max_history: usize,
}

/// Agent 状态 Hook
///
/// # 示例
///
/// ```rust,ignore
/// let agent_state = UseAgentState::new();
///
/// // 更新 Agent 状态
/// agent_state.update_agent(agent_info);
///
/// // 获取当前 Agent
/// if let Some(agent) = agent_state.current() {
///     println!("Current: {}", agent.name);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UseAgentState {
    state: AgentState,
}

impl UseAgentState {
    /// 创建新的 Agent 状态 Hook
    pub fn new() -> Self {
        Self {
            state: AgentState {
                current_agent: None,
                agents: Vec::new(),
                metrics: None,
                history: Vec::new(),
                max_history: 100,
            },
        }
    }

    /// 创建带有最大历史记录数的 Agent 状态 Hook
    pub fn with_max_history(max: usize) -> Self {
        Self {
            state: AgentState {
                current_agent: None,
                agents: Vec::new(),
                metrics: None,
                history: Vec::new(),
                max_history: max,
            },
        }
    }

    /// 更新 Agent 信息
    pub fn update_agent(&mut self, agent: AgentInfo) {
        // 记录到历史
        if let Some(ref current) = self.state.current_agent {
            if current.status != agent.status || current.current_task != agent.current_task {
                self.state.history.push(current.clone());
                // 限制历史记录长度
                if self.state.history.len() > self.state.max_history {
                    self.state.history.remove(0);
                }
            }
        }

        // 更新当前 Agent
        self.state.current_agent = Some(agent.clone());

        // 更新 agents 列表
        if let Some(pos) = self.state.agents.iter().position(|a| a.id == agent.id) {
            self.state.agents[pos] = agent;
        } else {
            self.state.agents.push(agent);
        }
    }

    /// 更新系统指标
    pub fn update_metrics(&mut self, metrics: MetricsData) {
        self.state.metrics = Some(metrics);
    }

    /// 获取当前 Agent
    pub fn current(&self) -> Option<&AgentInfo> {
        self.state.current_agent.as_ref()
    }

    /// 获取所有 Agent
    pub fn agents(&self) -> &[AgentInfo] {
        &self.state.agents
    }

    /// 获取系统指标
    pub fn metrics(&self) -> Option<&MetricsData> {
        self.state.metrics.as_ref()
    }

    /// 获取状态历史
    pub fn history(&self) -> &[AgentInfo] {
        &self.state.history
    }

    /// 根据 ID 获取 Agent
    pub fn get_agent(&self, id: &str) -> Option<&AgentInfo> {
        self.state.agents.iter().find(|a| a.id == id)
    }

    /// 清除历史记录
    pub fn clear_history(&mut self) {
        self.state.history.clear();
    }

    /// 清除所有数据
    pub fn clear(&mut self) {
        self.state.current_agent = None;
        self.state.agents.clear();
        self.state.metrics = None;
        self.state.history.clear();
    }

    /// 获取状态引用
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// 创建默认的演示 Agent
    pub fn with_demo_data() -> Self {
        let mut state = Self::new();

        state.update_agent(
            AgentInfo::new("agent-1", "Main Agent")
                .with_status(AgentStatus::Running)
                .with_task("Processing user request...")
        );

        state.update_agent(
            AgentInfo::new("agent-2", "Worker Agent")
                .with_status(AgentStatus::Idle)
        );

        state.update_agent(
            AgentInfo::new("agent-3", "Monitor Agent")
                .with_status(AgentStatus::Thinking)
                .with_task("Analyzing system metrics")
        );

        state.update_metrics(MetricsData {
            cpu: 45.5,
            memory: 62.3,
            disk: 38.0,
            latency_ms: 25,
        });

        state
    }
}

impl Default for UseAgentState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_agent() {
        let mut state = UseAgentState::new();
        let agent = AgentInfo::new("test-1", "Test Agent");
        state.update_agent(agent);

        assert!(state.current().is_some());
        assert_eq!(state.agents().len(), 1);
    }

    #[test]
    fn test_history() {
        let mut state = UseAgentState::with_max_history(3);

        state.update_agent(
            AgentInfo::new("test", "Agent")
                .with_status(AgentStatus::Idle)
        );

        state.update_agent(
            AgentInfo::new("test", "Agent")
                .with_status(AgentStatus::Running)
        );

        state.update_agent(
            AgentInfo::new("test", "Agent")
                .with_status(AgentStatus::Thinking)
        );

        // 历史记录应该有 2 条（状态变化）
        assert!(state.history().len() <= 2);
    }

    #[test]
    fn test_demo_data() {
        let state = UseAgentState::with_demo_data();
        assert!(!state.agents().is_empty());
        assert!(state.metrics().is_some());
    }
}
