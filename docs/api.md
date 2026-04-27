# ArkCore API 文档

ArkCore 是一个 Local-First OS Agent Engine，提供模块化的 AI Agent 编排、零信任沙盒执行和 SQLite FTS5 记忆存储能力。

## 目录

- [模块概览](#模块概览)
- [核心组件 API](#核心组件-api)
- [Orchestrator API](#orchestrator-api)
- [Sandbox API](#sandbox-api)
- [SkillMemory API](#skillmemory-api)
- [Server API](#server-api)
- [Services API](#services-api)
- [错误处理](#错误处理)

## 模块概览

```
arkcore
├── core        # 核心抽象层（traits, config, container）
├── services    # 服务实现
│   ├── orchestrator  # AI Agent 调度器
│   ├── sandbox      # 零信任执行沙盒
│   ├── memory        # SQLite FTS5 记忆引擎
│   └── security      # 安全服务
├── platform   # 平台适配（Windows/Unix）
├── web         # Web UI
├── cli         # 命令行解析
├── repl        # 交互式解释器
└── server      # Axum HTTP 服务器
```

## 核心组件 API

### lib.rs - 库入口

```rust
// 引入 arkcore 库
use arkcore::{Orchestrator, SkillMemory, Sandbox, Error};

// 重新导出常用类型
pub use arkcore::core::{Config, Container};
pub use arkcore::error::ArkCoreError;
```

### main.rs - 程序入口

支持三种运行模式：

```rust
enum Commands {
    /// 交互式终端模式
    Repl { init: Option<String> },
    /// 后台守护进程模式
    Daemon { port: u16 },
    /// 配置管理
    Config { set_api_key: Option<String> },
}
```

## Orchestrator API

AI Agent 调度器 - 基于状态机的任务编排。

### 创建实例

```rust
use arkcore::orchestrator::Orchestrator;
use arkcore::memory::SkillMemory;
use arkcore::sandbox::Sandbox;

// 标准创建
let memory = SkillMemory::new().await?;
let sandbox = Sandbox::new()?;
let orchestrator = Orchestrator::new(memory, sandbox);

// 自定义最大步数
let orchestrator = Orchestrator::with_max_steps(memory, sandbox, 20);
```

### 状态管理

```rust
// 获取当前状态
let state = orchestrator.current_state();

// 获取状态历史
let history = orchestrator.history();

// 检查是否处于终止状态
if orchestrator.is_terminal() {
    println!("任务已完成");
}

// 重置状态机
orchestrator.reset();
```

### 任务执行

```rust
// 执行任务
let result = orchestrator.run_task("分析文件").await?;

// 带审批的执行
let result = orchestrator
    .run_with_approval("删除文件", "rm -rf /tmp/test")
    .await?;
```

### AgentState 状态机

```rust
pub enum AgentState {
    Idle,
    Planning { task: String },
    Executing { step: usize, total: usize },
    AwaitingApproval { command: String, risk_level: String },
    Reflecting { assessment: String },
    Completed { result: String },
    Failed { reason: String },
}
```

### 组件访问

```rust
// 获取记忆引用
let memory = orchestrator.memory();

// 获取沙盒引用
let sandbox = orchestrator.sandbox();
```

## Sandbox API

零信任执行沙盒 - 六层安全检测架构。

### 安全检测

```rust
use arkcore::sandbox::{security_check, truncate_output, SecurityCheckResult};

let check = security_check("ls -la");
if !check.passed {
    println!("危险命令: {:?}", check.violations);
}
```

### 命令执行

```rust
use arkcore::sandbox::Sandbox;

let sandbox = Sandbox::new()?;

// 执行命令
let result = sandbox.execute("ls -la").await?;

println!("stdout: {}", result.stdout);
println!("stderr: {}", result.stderr);
println!("exit_code: {:?}", result.exit_code);
println!("is_success: {}", result.is_success());
```

### ExecutionResult 结构

```rust
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub was_killed: bool,
    pub execution_time_ms: u64,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool { ... }
}
```

### 安全检测层

1. **危险参数检测** - `-exec`, `-delete`, `-i` 等
2. **空字节注入检测**
3. **Shell 操作符检测** - `|`, `;`, `&`, `$`, `` ` ``
4. **路径遍历检测** - `..`, `/proc/`, `/sys/`
5. **环境变量注入检测** - `LD_PRELOAD`, `DYLD_*`
6. **危险内置命令检测** - `eval`, `exec`, `source`

### 进程隔离

```rust
use arkcore::sandbox::isolator::{IsolationConfig, ProcessIsolator};

// Linux: namespace 隔离 (PID, Network, Mount, UTS, IPC)
// Windows: Job Objects 隔离
// macOS: Sandbox

let config = IsolationConfig::default();
let isolator = ProcessIsolator::new(config)?;
```

## SkillMemory API

SQLite FTS5 记忆引擎 - 基于全文搜索的技能存储与检索。

### 创建实例

```rust
use arkcore::memory::{SkillMemory, Skill};

// 内存数据库
let memory = SkillMemory::new().await?;

// 文件数据库
let memory = SkillMemory::from_file("./data/memory.db").await?;
```

### Skill 数据结构

```rust
pub struct Skill {
    pub id: String,
    pub skill_name: String,
    pub category: String,
    pub summary: String,
    pub details: Option<String>,
    pub keywords: String,          // 逗号分隔
    pub importance: f64,
    pub success_count: i32,
    pub access_count: i32,
    pub created_at: String,        // ISO 8601
    pub updated_at: String,        // ISO 8601
}
```

### 存储与检索

```rust
// 存储技能
let skill = Skill {
    id: "skill-001".to_string(),
    skill_name: "Rust Programming".to_string(),
    category: "Programming".to_string(),
    summary: "Systems programming language".to_string(),
    details: Some("Fast, safe, concurrent".to_string()),
    keywords: "rust, systems, programming".to_string(),
    importance: 1.0,
    success_count: 0,
    access_count: 0,
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-01T00:00:00Z".to_string(),
};
memory.store_skill(&skill).await?;

// 全文搜索
let results = memory.search("rust", 10).await?;
for skill in results {
    println!("{}", skill.skill_name);
}

// 获取单个技能
let skill = memory.get_skill("skill-001").await?;

// 获取所有技能（按访问次数排序）
let all_skills = memory.get_all_skills(100).await?;
```

### 计数器更新

```rust
// 更新访问计数
memory.increment_access("skill-001").await?;

// 更新成功计数
memory.increment_success("skill-001").await?;

// 删除技能
memory.delete_skill("skill-001").await?;
```

## Server API

Axum HTTP 服务器 - 支持 SSE 和 WebSocket。

### 创建与启动

```rust
use arkcore::server::Server;
use arkcore::orchestrator::Orchestrator;

let orchestrator = Orchestrator::new(memory, sandbox);
let server = Server::with_default_config(orchestrator, 8080);

server.run().await?;
```

### ServerConfig

```rust
pub struct ServerConfig {
    pub port: u16,                      // 端口号，默认 8080
    pub shutdown_timeout: Duration,     // 关闭超时，默认 30 秒
}

let config = ServerConfig::new(9090)
    .with_shutdown_timeout(Duration::from_secs(60));
```

### HTTP 端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/health/detailed` | GET | 详细健康检查 |
| `/events` | GET | SSE 事件流 |
| `/ws` | GET | WebSocket 升级 |

### 优雅关闭

```rust
// 触发关闭
server.shutdown().await;

// 保存状态
server.save_state().await?;

// 清理资源
server.cleanup().await;
```

### ShutdownHandle

```rust
use arkcore::server::{ShutdownHandle, ShutdownState};

let handle = ShutdownHandle::new(shutdown_tx);

// 请求关闭
handle.shutdown().await;

// 强制终止
handle.terminate().await;
```

## Services API

### 健康检查服务

```rust
use arkcore::services::health::{HealthManager, HealthStatus, HealthLevel};

let manager = HealthManager::new();
let status = manager.check_health().await;

println!("Status: {:?}", status.level);
```

### 限流服务

```rust
use arkcore::services::ratelimit::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::default_config();
let result = limiter.check_rate_limit("user-123", "ip").await?;

if !result.allowed {
    println!("Rate limit exceeded, retry after {:?}", result.retry_after);
}
```

### 重试服务

```rust
use arkcore::services::retry::{Retry, RetryConfig, RetryStrategy};

let retry = Retry::new(RetryConfig::exponential(3, Duration::from_millis(100)));
let result = retry.execute(|| async { /* operation */ }).await;
```

### 超时服务

```rust
use arkcore::services::timeout::{TimeoutManager, TimeoutConfig};

let manager = TimeoutManager::new();
manager.timeout(Duration::from_secs(5), async { /* operation */ }).await;
```

### 缓存服务

```rust
use arkcore::services::cache::{CacheManager, CacheConfig};

let cache = CacheManager::new(CacheConfig::default());
cache.get(&cache_key).await;
cache.set(&cache_key, value, Duration::from_secs(300)).await;
```

### 连接池服务

```rust
use arkcore::services::pool::{SqlitePoolManager, PoolConfig};

let pool_manager = SqlitePoolManager::new(PoolConfig::default());
let pool = pool_manager.get_pool().await?;
```

### 对象池服务

```rust
use arkcore::services::object_pool::{ByteBufferPool, ObjectPoolConfig};

let buffer_pool = ByteBufferPool::new(ObjectPoolConfig::default());
let buffer = buffer_pool.acquire().await?;
```

## 安全服务 API

### API Key 服务

```rust
use arkcore::services::security::apikey::ApiKeyService;

let service = ApiKeyService::new();
let key = service.generate_key().await?;
let validated = service.validate_key(&key).await?;
```

### 审计服务

```rust
use arkcore::services::security::audit::AuditEvent;

let event = AuditEvent::new("user-123", "execute_command")
    .with_resource("ls")
    .with_result("success");
service.log_event(event).await?;
```

### RBAC 服务

```rust
use arkcore::services::security::rbac::RbacService;

let rbac = RbacService::new();
let allowed = rbac.check_permission("user-123", "execute", "sandbox").await?;
```

### 加密服务

```rust
use arkcore::services::security::crypto::CryptoService;

let crypto = CryptoService::new();
let encrypted = crypto.encrypt(data, key).await?;
let decrypted = crypto.decrypt(encrypted, key).await?;
```

### 合规报告

```rust
use arkcore::services::security::compliance::ComplianceReporter;

let reporter = ComplianceReporter::new();
let report = reporter.generate_report(ReportType::Monthly).await?;
```

## 错误处理

### ArkCoreError

```rust
use arkcore::error::ArkCoreError;
use arkcore::Error;

match result {
    Ok(value) => println!("{}", value),
    Err(Error::ArkCore(e)) => {
        match e.kind() {
            ErrorKind::NotFound => println!("资源不存在"),
            ErrorKind::PermissionDenied => println!("权限不足"),
            ErrorKind::InvalidInput => println!("输入无效"),
            _ => println!("其他错误: {}", e),
        }
    }
    Err(e) => println!("错误: {}", e),
}
```

### 错误种类

```rust
pub enum ErrorKind {
    NotFound,
    InvalidInput,
    PermissionDenied,
    SecurityViolation,
    ResourceExhausted,
    Internal,
    Io,
    Timeout,
}
```

## CLI API

### 命令行参数

```rust
use arkcore::cli::{Cli, Commands};

let cli = Cli::parse();

match cli.command {
    Commands::Repl { init } => { /* 交互模式 */ }
    Commands::Daemon { port } => { /* 守护进程 */ }
    Commands::Config { set_api_key } => { /* 配置管理 */ }
}
```

### 配置管理

```rust
use arkcore::cli::config::{load_config, save_config, set_api_key, Config};

let config = load_config()?;
save_config(&config)?;
set_api_key("your-api-key")?;
```

## 类型别名

```rust
// 库级别类型别名
pub type Result<T> = result::Result<T, Error>;
pub type Error = ArkCoreError;

// 核心模块类型别名
pub use crate::core::Error;
pub use crate::core::Config;
pub use crate::core::Container;
```
