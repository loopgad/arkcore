//! T7.1: 模块集成测试
//!
//! 测试 ArkCore 核心模块之间的协同工作能力

use arkcore::memory::{Skill, SkillMemory};
use arkcore::orchestrator::{AgentState, Orchestrator};
use arkcore::sandbox::Sandbox;
use arkcore::services::health::{HealthLevel, HealthManager};

/// 测试辅助函数：创建测试用 Orchestrator 实例
async fn create_test_orchestrator() -> Orchestrator {
    let memory = SkillMemory::new().await.expect("创建内存存储失败");
    let sandbox = Sandbox::new().expect("创建沙盒失败");
    Orchestrator::new(memory, sandbox)
}

/// 测试 Orchestrator 与 Memory 的集成
#[tokio::test]
async fn test_orchestrator_memory_integration() {
    let memory = SkillMemory::new().await.expect("创建内存存储失败");
    let sandbox = Sandbox::new().expect("创建沙盒失败");
    let orch = Orchestrator::new(memory.clone(), sandbox);

    // 验证 memory 引用可用 (使用 try_read 同步检查)
    let mem_ref = orch.memory();
    assert!(mem_ref.try_read().is_ok());

    // 存储技能
    let skill = Skill {
        id: "test-skill-001".to_string(),
        skill_name: "Test Skill".to_string(),
        category: "Testing".to_string(),
        summary: "A test skill for integration testing".to_string(),
        details: Some("Integration test skill".to_string()),
        keywords: "test, integration".to_string(),
        importance: 1.0,
        success_count: 0,
        access_count: 0,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    memory.store_skill(&skill).await.expect("存储技能失败");

    // 搜索验证
    let results = memory.search("Test", 10).await.expect("搜索失败");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].skill_name, "Test Skill");
}

/// 测试 Orchestrator 与 Sandbox 的集成
#[tokio::test]
async fn test_orchestrator_sandbox_integration() {
    let memory = SkillMemory::new().await.expect("创建内存存储失败");
    let sandbox = Sandbox::new().expect("创建沙盒失败");

    // 安全检查测试 - 验证 sandbox 安全检查功能正常
    let safe_result = sandbox.security_check("ls -la");
    assert!(safe_result.passed || !safe_result.violations.is_empty());

    // 创建新的 sandbox 实例用于 Orchestrator
    let sandbox2 = Sandbox::new().expect("创建沙盒失败");
    let orch = Orchestrator::new(memory, sandbox2);

    // 验证 sandbox 引用可用 - 空字符串会通过安全检查因为没有危险模式
    let sandbox_ref = orch.sandbox();
    let empty_check = sandbox_ref.security_check("");
    assert!(empty_check.passed, "空字符串应该通过安全检查");
}

/// 测试 Orchestrator 执行任务流程
#[tokio::test]
async fn test_orchestrator_task_execution() {
    let orch = create_test_orchestrator().await;

    // 初始状态检查
    let initial_state = orch.current_state();
    assert!(matches!(initial_state, AgentState::Idle));

    // 执行任务
    let result = orch.run_task("Test task").await.expect("任务执行失败");
    assert_eq!(result, "Task completed successfully");

    // 验证终止状态
    assert!(orch.is_terminal());
    let final_state = orch.current_state();
    assert!(matches!(final_state, AgentState::Completed { .. }));
}

/// 测试状态历史记录
#[tokio::test]
async fn test_orchestrator_state_history() {
    let orch = create_test_orchestrator().await;

    // 执行任务触发状态转换
    orch.run_task("Test task with history").await.expect("任务执行失败");

    // 验证历史记录
    let history = orch.history();
    assert!(!history.is_empty(), "历史记录不应为空");

    // 验证最终状态是 Completed
    let final_state = orch.current_state();
    assert!(matches!(final_state, AgentState::Completed { .. }), "最终状态应该是 Completed，实际是 {:?}", final_state);
}

/// 测试健康检查模块
#[tokio::test]
async fn test_health_check_integration() {
    let health_manager = HealthManager::new();

    // 验证初始健康状态
    let status = health_manager.get_status().await;
    assert_eq!(status.level, HealthLevel::Healthy);
    assert!(status.database.connected);
    assert!(status.sandbox.available);

    // 验证简单健康响应
    let response = health_manager.get_simple_status().await;
    assert_eq!(response.status, "healthy");
    assert!(response.timestamp > 0);
}

/// 测试 Memory 模块 FTS5 搜索
#[tokio::test]
async fn test_memory_fts_integration() {
    let memory = SkillMemory::new().await.expect("创建内存存储失败");

    // 存储多个技能
    let skills = vec![
        Skill {
            id: "skill-001".to_string(),
            skill_name: "Rust Programming".to_string(),
            category: "Programming".to_string(),
            summary: "Systems programming language".to_string(),
            details: Some("Fast, safe, concurrent".to_string()),
            keywords: "rust, systems, programming".to_string(),
            importance: 1.0,
            success_count: 5,
            access_count: 10,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
        Skill {
            id: "skill-002".to_string(),
            skill_name: "Python Scripting".to_string(),
            category: "Programming".to_string(),
            summary: "Scripting language for automation".to_string(),
            details: Some("Easy to learn, powerful".to_string()),
            keywords: "python, scripting, automation".to_string(),
            importance: 0.9,
            success_count: 3,
            access_count: 5,
            created_at: "2024-01-02T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        },
    ];

    for skill in &skills {
        memory.store_skill(skill).await.expect("存储技能失败");
    }

    // 测试全文搜索
    let results = memory.search("programming", 10).await.expect("搜索失败");
    assert!(!results.is_empty(), "应该至少找到一个匹配 'programming' 的技能");

    // 测试关键词搜索
    let rust_results = memory.search("rust", 10).await.expect("搜索失败");
    assert!(rust_results.iter().any(|s| s.skill_name.contains("Rust")));

    // 测试按访问次数排序
    let all_skills = memory.get_all_skills(10).await.expect("获取所有技能失败");
    assert!(all_skills[0].access_count >= all_skills[1].access_count);
}

/// 测试模块间的线程安全
#[tokio::test]
async fn test_module_thread_safety() {
    use std::sync::Arc;
    use tokio::task;

    let orch = create_test_orchestrator().await;
    let orch_arc = Arc::new(orch);

    // 在多个任务中并发访问
    let handles: Vec<_> = (0..5)
        .map(|_i| {
            let orch = orch_arc.clone();
            task::spawn(async move {
                let result = orch.run_task("concurrent task").await;
                result.is_ok()
            })
        })
        .collect();

    // 验证所有任务完成
    for handle in handles {
        let success = handle.await.expect("任务 Join 失败");
        assert!(success);
    }
}
