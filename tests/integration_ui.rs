//! T7.4: UI 集成测试
//!
//! 测试 ArkCore Web UI 组件和主题系统
//!
//! # 测试范围
//!
//! - 主题系统（暗/亮主题）
//! - Agent 状态枚举
//! - WebSocket 消息类型
//! - 系统指标数据
//! - 连接状态

use arkcore::web::{
    config::WebUiConfig,
    theme::{dark_values, light_values, Theme},
    AgentInfo, AgentStatus, CommandOutput, ConnectionStatus, MetricsData, OutputType, WsMessage,
};

/// 测试主题枚举
#[test]
fn test_theme_variants() {
    let themes = vec![Theme::Dark, Theme::Light, Theme::System];

    for theme in themes {
        assert!(!theme.name().is_empty());
        match theme {
            Theme::Dark => assert!(theme.is_dark()),
            Theme::Light => assert!(!theme.is_dark()),
            Theme::System => {}
        }
    }
}

/// 测试主题解析
#[test]
fn test_theme_resolve() {
    // 明确的主题应保持不变
    assert_eq!(Theme::Dark.resolve(true), Theme::Dark);
    assert_eq!(Theme::Dark.resolve(false), Theme::Dark);
    assert_eq!(Theme::Light.resolve(true), Theme::Light);
    assert_eq!(Theme::Light.resolve(false), Theme::Light);

    // System 应根据系统偏好解析
    assert_eq!(Theme::System.resolve(true), Theme::Dark);
    assert_eq!(Theme::System.resolve(false), Theme::Light);
}

/// 测试主题切换
#[test]
fn test_theme_toggle() {
    assert_eq!(Theme::Dark.toggle(), Theme::Light);
    assert_eq!(Theme::Light.toggle(), Theme::Dark);
    // System 切换后保持 System
    assert_eq!(Theme::System.toggle(), Theme::System);
}

/// 测试主题显示格式
#[test]
fn test_theme_display() {
    assert_eq!(format!("{}", Theme::Dark), "dark");
    assert_eq!(format!("{}", Theme::Light), "light");
    assert_eq!(format!("{}", Theme::System), "system");
}

/// 测试暗色主题 CSS 变量
#[test]
fn test_dark_theme_css_variables() {
    let css = dark_values::get_css();

    // 验证包含关键颜色变量
    assert!(css.contains("--color-primary"));
    assert!(css.contains("--color-background"));
    assert!(css.contains("--color-surface"));
    assert!(css.contains("--color-text-primary"));
    assert!(css.contains("--color-text-secondary"));

    // 验证暗色主题背景是深色
    assert!(css.contains("#0f0f0f") || css.contains("0f0f0f"));
}

/// 测试亮色主题 CSS 变量
#[test]
fn test_light_theme_css_variables() {
    let css = light_values::get_css();

    // 验证包含关键颜色变量
    assert!(css.contains("--color-primary"));
    assert!(css.contains("--color-background"));
    assert!(css.contains("--color-surface"));
    assert!(css.contains("--color-text-primary"));
    assert!(css.contains("--color-text-secondary"));

    // 验证亮色主题背景是浅色
    assert!(css.contains("#ffffff") || css.contains("ffffff"));
}

/// 测试 CSS 变量名称
#[test]
fn test_css_var_names() {
    use arkcore::web::theme::css_vars::*;

    assert_eq!(PRIMARY, "--color-primary");
    assert_eq!(BACKGROUND, "--color-background");
    assert_eq!(SURFACE, "--color-surface");
    assert_eq!(TEXT_PRIMARY, "--color-text-primary");
    assert_eq!(SUCCESS, "--color-success");
    assert_eq!(WARNING, "--color-warning");
    assert_eq!(ERROR, "--color-error");
    assert_eq!(INFO, "--color-info");
}

/// 测试 Agent 状态枚举
#[test]
fn test_agent_status_variants() {
    let statuses = vec![
        AgentStatus::Idle,
        AgentStatus::Running,
        AgentStatus::Thinking,
        AgentStatus::Waiting,
        AgentStatus::Error,
        AgentStatus::Connected,
        AgentStatus::Disconnected,
    ];

    for status in statuses {
        let display = format!("{}", status);
        assert!(!display.is_empty());
    }
}

/// 测试 Agent 状态显示
#[test]
fn test_agent_status_display() {
    assert_eq!(format!("{}", AgentStatus::Idle), "idle");
    assert_eq!(format!("{}", AgentStatus::Running), "running");
    assert_eq!(format!("{}", AgentStatus::Thinking), "thinking");
    assert_eq!(format!("{}", AgentStatus::Waiting), "waiting");
    assert_eq!(format!("{}", AgentStatus::Error), "error");
    assert_eq!(format!("{}", AgentStatus::Connected), "connected");
    assert_eq!(format!("{}", AgentStatus::Disconnected), "disconnected");
}

/// 测试 AgentInfo 创建
#[test]
fn test_agent_info_creation() {
    let agent = AgentInfo::new("agent-001", "Test Agent");

    assert_eq!(agent.id, "agent-001");
    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.status, AgentStatus::Idle);
    assert!(agent.current_task.is_none());
    assert!(agent.start_time.is_none());
    assert_eq!(agent.cpu_usage, 0.0);
    assert_eq!(agent.memory_usage, 0.0);
}

/// 测试 AgentInfo 链式构建
#[test]
fn test_agent_info_builder() {
    let agent = AgentInfo::new("agent-001", "Test Agent")
        .with_status(AgentStatus::Running)
        .with_task("Processing data");

    assert_eq!(agent.id, "agent-001");
    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.status, AgentStatus::Running);
    assert_eq!(agent.current_task, Some("Processing data".to_string()));
}

/// 测试 WebSocket 消息序列化
#[test]
fn test_ws_message_serialization() {
    use serde_json;

    // Agent 状态消息
    let agent_info = AgentInfo::new("agent-001", "Test Agent").with_status(AgentStatus::Running);
    let msg = WsMessage::AgentStatus(agent_info);
    let json = serde_json::to_string(&msg).expect("序列化失败");
    assert!(json.contains("agent_status"));

    // 指标消息
    let metrics = MetricsData {
        cpu: 45.5,
        memory: 62.3,
        disk: 75.0,
        latency_ms: 10,
    };
    let msg = WsMessage::Metrics(metrics);
    let json = serde_json::to_string(&msg).expect("序列化失败");
    assert!(json.contains("metrics"));

    // 连接状态消息
    let msg = WsMessage::Connection(ConnectionStatus::Connected);
    let json = serde_json::to_string(&msg).expect("序列化失败");
    assert!(json.contains("connection"));

    // 命令输出消息
    let output = CommandOutput {
        content: "test output".to_string(),
        output_type: OutputType::Stdout,
        timestamp: 1234567890,
    };
    let msg = WsMessage::Output(output);
    let json = serde_json::to_string(&msg).expect("序列化失败");
    assert!(json.contains("output"));

    // 错误消息
    let msg = WsMessage::Error("Something went wrong".to_string());
    let json = serde_json::to_string(&msg).expect("序列化失败");
    assert!(json.contains("error"));
}

/// 测试连接状态
#[test]
fn test_connection_status() {
    let statuses = vec![
        ConnectionStatus::Connected,
        ConnectionStatus::Disconnected,
        ConnectionStatus::Reconnecting,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).expect("序列化失败");
        assert!(!json.is_empty());
    }
}

/// 测试输出类型
#[test]
fn test_output_type() {
    let types = vec![
        OutputType::Stdout,
        OutputType::Stderr,
        OutputType::System,
        OutputType::Result,
    ];

    for output_type in types {
        let json = serde_json::to_string(&output_type).expect("序列化失败");
        assert!(!json.is_empty());
    }
}

/// 测试命令输出结构
#[test]
fn test_command_output_structure() {
    let output = CommandOutput {
        content: "Hello, World!".to_string(),
        output_type: OutputType::Stdout,
        timestamp: 1704067200,
    };

    assert_eq!(output.content, "Hello, World!");
    assert_eq!(output.output_type, OutputType::Stdout);
    assert_eq!(output.timestamp, 1704067200);
}

/// 测试系统指标数据
#[test]
fn test_metrics_data() {
    let metrics = MetricsData {
        cpu: 45.5,
        memory: 62.3,
        disk: 75.0,
        latency_ms: 10,
    };

    assert_eq!(metrics.cpu, 45.5);
    assert_eq!(metrics.memory, 62.3);
    assert_eq!(metrics.disk, 75.0);
    assert_eq!(metrics.latency_ms, 10);
}

/// 测试 WebUI 配置默认值
#[test]
fn test_webui_config_defaults() {
    let config = WebUiConfig::default();

    assert_eq!(config.ws_url, "ws://127.0.0.1:8080/ws");
    assert_eq!(config.default_theme, Theme::Dark);
    assert!(config.animations_enabled);
    assert!(config.auto_reconnect);
    assert_eq!(config.reconnect_interval_ms, 3000);
}

/// 测试 WebUI 配置序列化
#[test]
fn test_webui_config_serialization() {
    let config = WebUiConfig::default();
    let json = serde_json::to_string(&config).expect("序列化失败");

    assert!(json.contains("ws_url"));
    assert!(json.contains("default_theme"));
    assert!(json.contains("animations_enabled"));
}

/// 测试 Agent 状态转换
#[test]
fn test_agent_state_from_status() {
    // AgentInfo 可以从 AgentStatus 创建
    let agent = AgentInfo::new("test", "Test Agent").with_status(AgentStatus::Thinking);

    assert_eq!(agent.status, AgentStatus::Thinking);
}

/// 测试主题 JSON 序列化
#[test]
fn test_theme_serialization() {
    use serde_json;

    let theme = Theme::Dark;
    let json = serde_json::to_string(&theme).expect("序列化失败");
    assert_eq!(json, "\"dark\"");

    let theme = Theme::Light;
    let json = serde_json::to_string(&theme).expect("序列化失败");
    assert_eq!(json, "\"light\"");

    let theme = Theme::System;
    let json = serde_json::to_string(&theme).expect("序列化失败");
    assert_eq!(json, "\"system\"");
}

/// 测试主题 JSON 反序列化
#[test]
fn test_theme_deserialization() {
    use serde_json;

    let dark: Theme = serde_json::from_str("\"dark\"").expect("反序列化失败");
    assert_eq!(dark, Theme::Dark);

    let light: Theme = serde_json::from_str("\"light\"").expect("反序列化失败");
    assert_eq!(light, Theme::Light);

    let system: Theme = serde_json::from_str("\"system\"").expect("反序列化失败");
    assert_eq!(system, Theme::System);
}

/// 测试 AgentStatus JSON 序列化
#[test]
fn test_agent_status_serialization() {
    use serde_json;

    let status = AgentStatus::Running;
    let json = serde_json::to_string(&status).expect("序列化失败");
    assert_eq!(json, "\"running\"");
}

/// 测试 AgentStatus JSON 反序列化
#[test]
fn test_agent_status_deserialization() {
    use serde_json;

    let status: AgentStatus = serde_json::from_str("\"running\"").expect("反序列化失败");
    assert_eq!(status, AgentStatus::Running);

    let status: AgentStatus = serde_json::from_str("\"idle\"").expect("反序列化失败");
    assert_eq!(status, AgentStatus::Idle);
}
