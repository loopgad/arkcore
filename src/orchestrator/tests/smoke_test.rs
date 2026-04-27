//! Orchestrator 状态机冒烟测试
//!
//! 测试状态转换、历史记录、终止状态识别和重置功能

use super::super::{AgentState, AgentStateMachine};
use std::sync::{Arc, RwLock};

/// 测试完整流程: Idle -> Planning -> Executing -> Completed
#[test]
fn test_state_transition_idle_to_completed() {
    let mut sm = AgentStateMachine::new(100);

    // 验证初始状态
    assert!(matches!(sm.current_state(), AgentState::Idle));
    assert!(!sm.is_terminal());

    // Idle -> Planning
    sm.transition(AgentState::Planning {
        task: "Test task".to_string(),
    });
    assert!(matches!(sm.current_state(), AgentState::Planning { .. }));
    assert_eq!(sm.history().len(), 1);

    // Planning -> Executing
    sm.transition(AgentState::Executing { step: 1, total: 3 });
    assert!(matches!(sm.current_state(), AgentState::Executing { .. }));
    assert_eq!(sm.history().len(), 2);

    // Executing -> Completed
    sm.transition(AgentState::Completed {
        result: "Success".to_string(),
    });
    assert!(matches!(sm.current_state(), AgentState::Completed { .. }));
    assert!(sm.is_terminal());
    assert_eq!(sm.history().len(), 3);
}

/// 测试完整流程: Idle -> Planning -> Executing -> Failed
#[test]
fn test_state_transition_idle_to_failed() {
    let mut sm = AgentStateMachine::new(100);

    // 初始状态
    assert!(matches!(sm.current_state(), AgentState::Idle));

    // Idle -> Planning
    sm.transition(AgentState::Planning {
        task: "Failing task".to_string(),
    });

    // Planning -> Executing
    sm.transition(AgentState::Executing { step: 1, total: 3 });

    // Executing -> Failed
    sm.transition(AgentState::Failed {
        reason: "Execution error".to_string(),
    });

    assert!(matches!(sm.current_state(), AgentState::Failed { .. }));
    assert!(sm.is_terminal());
    assert_eq!(sm.history().len(), 3);
}

/// 测试状态转换后可以继续转换（即使不合逻辑）
/// 注意: 状态机本身不阻止非法转换，只是记录历史
#[test]
fn test_state_transition_recording() {
    let mut sm = AgentStateMachine::new(100);

    // 从 Completed 可以转换到其他状态（状态机不阻止）
    sm.transition(AgentState::Completed {
        result: "Done".to_string(),
    });
    assert!(sm.is_terminal());

    // 转换到 Failed
    sm.transition(AgentState::Failed {
        reason: "Overwritten".to_string(),
    });
    assert!(sm.is_terminal());
    assert!(matches!(sm.current_state(), AgentState::Failed { .. }));

    // 历史记录了所有转换
    assert_eq!(sm.history().len(), 2);
}

/// 测试历史记录限制 (max_history)
#[test]
fn test_history_limit() {
    let mut sm = AgentStateMachine::new(3); // 限制为 3

    // 执行多次转换
    for i in 0..5 {
        sm.transition(AgentState::Executing {
            step: i,
            total: 5,
        });
    }

    // 历史记录应该被限制在 max_history
    assert!(sm.history().len() <= 3);
}

/// 测试 max_history=0 的边界情况
#[test]
fn test_history_limit_zero() {
    let mut sm = AgentStateMachine::new(0);

    sm.transition(AgentState::Planning {
        task: "Test".to_string(),
    });

    // max_history=0 时，历史可能为空或只有一项（取决于实现）
    // 关键是不要 panic
    let _ = sm.history();
    assert!(matches!(sm.current_state(), AgentState::Planning { .. }));
}

/// 测试 is_terminal() 正确识别终止状态
#[test]
fn test_is_terminal() {
    let mut sm = AgentStateMachine::new(100);

    // 非终止状态
    assert!(!sm.is_terminal());

    sm.transition(AgentState::Planning {
        task: "Test".to_string(),
    });
    assert!(!sm.is_terminal());

    sm.transition(AgentState::Executing { step: 1, total: 1 });
    assert!(!sm.is_terminal());

    sm.transition(AgentState::AwaitingApproval {
        command: "ls".to_string(),
        risk_level: "low".to_string(),
    });
    assert!(!sm.is_terminal());

    sm.transition(AgentState::Reflecting {
        assessment: "Good".to_string(),
    });
    assert!(!sm.is_terminal());

    // 终止状态 - Completed
    sm.transition(AgentState::Completed {
        result: "Done".to_string(),
    });
    assert!(sm.is_terminal());
}

/// 测试 reset() 重置功能
#[test]
fn test_reset_from_various_states() {
    let mut sm = AgentStateMachine::new(100);

    // 从 Planning 重置
    sm.transition(AgentState::Planning {
        task: "Test".to_string(),
    });
    sm.reset();
    assert!(matches!(sm.current_state(), AgentState::Idle));
    assert!(!sm.is_terminal());

    // 从 Executing 重置
    sm.transition(AgentState::Executing { step: 1, total: 3 });
    sm.reset();
    assert!(matches!(sm.current_state(), AgentState::Idle));

    // 从 Completed 重置
    sm.transition(AgentState::Completed {
        result: "Done".to_string(),
    });
    sm.reset();
    assert!(matches!(sm.current_state(), AgentState::Idle));

    // 从 Failed 重置
    sm.transition(AgentState::Failed {
        reason: "Error".to_string(),
    });
    sm.reset();
    assert!(matches!(sm.current_state(), AgentState::Idle));
}

/// 测试 reset() 不从 Idle 状态添加历史记录
#[test]
fn test_reset_from_idle_no_history() {
    let mut sm = AgentStateMachine::new(100);

    // 当前就是 Idle
    sm.reset();
    assert!(matches!(sm.current_state(), AgentState::Idle));
    assert_eq!(sm.history().len(), 0); // 不应添加历史
}

/// 测试 reset() 会保存当前非 Idle 状态到历史
#[test]
fn test_reset_preserves_history() {
    let mut sm = AgentStateMachine::new(100);

    sm.transition(AgentState::Planning {
        task: "Important task".to_string(),
    });
    sm.reset();

    // 历史记录了 Planning 状态
    assert_eq!(sm.history().len(), 1);
    assert!(matches!(
        sm.history()[0],
        AgentState::Planning { .. }
    ));
}

/// 测试 executed_steps() 正确计数
#[test]
fn test_executed_steps() {
    let mut sm = AgentStateMachine::new(100);

    assert_eq!(sm.executed_steps(), 0);

    sm.transition(AgentState::Executing { step: 1, total: 3 });
    assert_eq!(sm.executed_steps(), 1);

    sm.transition(AgentState::Executing { step: 2, total: 3 });
    assert_eq!(sm.executed_steps(), 2);

    sm.transition(AgentState::Planning {
        task: "Interruption".to_string(),
    });
    // Planning 不是 Executing，所以计数不变
    assert_eq!(sm.executed_steps(), 2);

    sm.transition(AgentState::Completed {
        result: "Done".to_string(),
    });
    // Completed 后计数不变
    assert_eq!(sm.executed_steps(), 2);
}

/// 测试多步执行流程
#[test]
fn test_multi_step_execution() {
    let mut sm = AgentStateMachine::new(100);

    sm.transition(AgentState::Planning {
        task: "Multi-step task".to_string(),
    });

    let total_steps = 5;
    for step in 1..=total_steps {
        sm.transition(AgentState::Executing {
            step,
            total: total_steps,
        });
    }

    assert_eq!(sm.executed_steps(), total_steps);
    assert_eq!(sm.history().len(), 1 + total_steps); // Planning + 5 Executing

    sm.transition(AgentState::Completed {
        result: "All steps done".to_string(),
    });

    assert!(sm.is_terminal());
}

/// 测试 AwaitingApproval 和 Reflecting 状态
#[test]
fn test_approval_and_reflecting_states() {
    let mut sm = AgentStateMachine::new(100);

    sm.transition(AgentState::Planning {
        task: "Approval task".to_string(),
    });

    sm.transition(AgentState::AwaitingApproval {
        command: "rm -rf".to_string(),
        risk_level: "high".to_string(),
    });
    assert!(!sm.is_terminal());

    sm.transition(AgentState::Failed {
        reason: "Rejected".to_string(),
    });
    assert!(sm.is_terminal());
}

/// 测试使用 Arc<RwLock<AgentStateMachine>> 的线程安全访问
#[test]
fn test_orchestrator_state_machine_wrapper() {
    let sm = Arc::new(RwLock::new(AgentStateMachine::new(100)));

    // 读取初始状态
    {
        let state = sm.read().unwrap();
        assert!(matches!(*state.current_state(), AgentState::Idle));
    }

    // 写入转换
    {
        let mut state = sm.write().unwrap();
        state.transition(AgentState::Planning {
            task: "Concurrent task".to_string(),
        });
    }

    // 再次读取
    {
        let state = sm.read().unwrap();
        assert!(matches!(*state.current_state(), AgentState::Planning { .. }));
    }
}

/// 测试 Default 实现
#[test]
fn test_default_state_machine() {
    let sm = AgentStateMachine::default();
    assert!(matches!(sm.current_state(), AgentState::Idle));
    assert!(!sm.is_terminal());
    assert_eq!(sm.history().len(), 0);
    // Default max_history 是 100
}

/// 测试状态显示 (Display trait)
#[test]
fn test_state_display() {
    let state = AgentState::Idle;
    assert_eq!(format!("{}", state), "Idle");

    let state = AgentState::Planning {
        task: "Test".to_string(),
    };
    assert_eq!(format!("{}", state), "Planning: Test");

    let state = AgentState::Executing { step: 2, total: 5 };
    assert_eq!(format!("{}", state), "Executing (2/5)");

    let state = AgentState::Completed {
        result: "OK".to_string(),
    };
    assert_eq!(format!("{}", state), "Completed: OK");

    let state = AgentState::Failed {
        reason: "Error".to_string(),
    };
    assert_eq!(format!("{}", state), "Failed: Error");
}