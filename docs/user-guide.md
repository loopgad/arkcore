# ArkCore 用户指南

ArkCore 是一个 Local-First OS Agent Engine，提供本地优先的 AI Agent 编排能力，支持交互式终端、守护进程和配置管理三种运行模式。

## 目录

- [安装](#安装)
- [快速开始](#快速开始)
- [交互式终端 (REPL)](#交互式终端-repl)
- [守护进程模式](#守护进程模式)
- [配置管理](#配置管理)
- [项目结构](#项目结构)
- [常见问题](#常见问题)

## 安装

### 前置要求

- Rust 1.85 或更高版本
- SQLite (通过 sqlx 绑定)
- 平台特定依赖:

  **Linux:**
  - libsqlite3-dev

  **macOS:**
  - sqlite3 (通过 Homebrew 安装)

  **Windows:**
  - 无额外依赖

### 构建

```bash
# 克隆项目
git clone https://github.com/your-org/arkcore.git
cd arkcore

# Release 构建
cargo build --release

# 开发构建
cargo build
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib orchestrator
cargo test --lib sandbox
cargo test --lib memory

# 运行文档测试
cargo test --doc
```

## 快速开始

### 首次启动

```bash
# 交互式终端模式
cargo run --release -- repl

# 启动守护进程
cargo run --release -- daemon --port 8080

# 配置 API Key
cargo run --release -- config --set-api-key "your-api-key"
```

## 交互式终端 (REPL)

REPL 模式提供交互式命令行界面，用于直接与 ArkCore 交互。

### 启动 REPL

```bash
cargo run --release -- repl
```

### 执行初始化脚本

```bash
# 启动时执行脚本
cargo run --release -- repl --init setup.script
```

### 常用命令

```
arkcore> help
arkcore> status
arkcore> history
arkcore> clear
arkcore> exit
```

### 与 Agent 交互

```
arkcore> 分析当前目录文件
[Planning] 任务: 分析当前目录文件
[Executing] Step 1/3
[Executing] Step 2/3
[Executing] Step 3/3
[Completed] 分析完成
```

## 守护进程模式

守护进程模式在后台运行 HTTP 服务器，提供 API 接口。

### 启动服务器

```bash
# 默认端口 8080
cargo run --release -- daemon

# 自定义端口
cargo run --release -- daemon --port 9090
```

### API 端点

#### 健康检查

```bash
# 基础健康检查
curl http://127.0.0.1:8080/health

# 详细健康检查
curl http://127.0.0.1:8080/health/detailed
```

#### SSE 事件流

```bash
curl http://127.0.0.1:8080/events
```

#### WebSocket

```bash
# 使用 wscat 连接
wscat -c ws://127.0.0.1:8080/ws
```

### 优雅关闭

守护进程支持优雅关闭，收到 SIGINT (Ctrl+C) 信号时会：

1. 停止接收新请求
2. 等待现有请求处理完成
3. 保存状态
4. 释放资源

默认超时时间为 30 秒。

## 配置管理

### 查看配置

```bash
cargo run --release -- config
```

### 设置 API Key

```bash
cargo run --release -- config --set-api-key "your-api-key"
```

### 配置文件

配置文件位置：

- Linux/macOS: `~/.config/arkcore/config.toml`
- Windows: `%APPDATA%\arkcore\config.toml`

示例配置：

```toml
[llm]
provider = "openai"
model = "gpt-4"
api_base = "https://api.openai.com/v1"

[database]
path = "~/.local/share/arkcore/memory.db"

[server]
port = 8080
host = "127.0.0.1"

[security]
audit_enabled = true
rate_limit_enabled = true
```

## 项目结构

```
arkcore/
├── src/
│   ├── lib.rs              # 库入口
│   ├── main.rs             # 程序入口
│   ├── cli/                # 命令行模块
│   │   ├── args.rs         # Clap 参数定义
│   │   ├── config.rs       # 配置管理
│   │   └── mod.rs
│   ├── core/               # 核心抽象层
│   │   ├── traits.rs       # Trait 定义
│   │   ├── config.rs       # 配置系统
│   │   ├── container.rs    # 依赖注入
│   │   └── mod.rs
│   ├── orchestrator/       # AI Agent 调度器
│   │   ├── state_machine.rs
│   │   └── mod.rs
│   ├── sandbox/            # 零信任沙盒
│   │   ├── executor.rs
│   │   ├── isolator.rs
│   │   ├── truncator.rs
│   │   └── mod.rs
│   ├── memory/             # 记忆存储
│   │   └── mod.rs
│   ├── services/           # 服务层
│   │   ├── health.rs
│   │   ├── ratelimit.rs
│   │   ├── retry.rs
│   │   ├── timeout.rs
│   │   ├── cache.rs
│   │   ├── pool.rs
│   │   ├── object_pool.rs
│   │   ├── async_rt.rs
│   │   └── security/
│   │       ├── apikey.rs
│   │       ├── audit.rs
│   │       ├── rbac.rs
│   │       ├── crypto.rs
│   │       ├── compliance.rs
│   │       ├── validator.rs
│   │       └── mod.rs
│   ├── platform/           # 平台适配
│   ├── web/               # Web UI
│   ├── server/            # HTTP 服务器
│   ├── repl/              # 交互式解释器
│   └── error.rs           # 错误处理
├── tests/                 # 集成测试
├── config/                # 配置文件
├── migrations/            # 数据库迁移
├── Cargo.toml
└── README.md
```

## 常见问题

### Q: 守护进程启动失败，端口被占用

**A:** 检查端口占用情况并更换端口：

```bash
# Linux/macOS
lsof -i :8080

# Windows
netstat -ano | findstr :8080

# 使用其他端口
cargo run --release -- daemon --port 9090
```

### Q: 记忆搜索没有返回预期结果

**A:** 检查 FTS5 索引状态：

```rust
// 确认关键词已正确设置
let skill = Skill {
    keywords: "rust, systems, programming".to_string(),
    // ...
};
```

### Q: 安全检查阻止了合法命令

**A:** 某些命令因安全策略被阻止。详见 [安全指南](security.md)。

### Q: 内存数据库 vs 文件数据库如何选择

**A:**
- **内存数据库**: 适合短期会话、测试环境，数据不持久化
- **文件数据库**: 适合生产环境，数据持久化

```rust
// 内存数据库
let memory = SkillMemory::new().await?;

// 文件数据库
let memory = SkillMemory::from_file("./data/memory.db").await?;
```

### Q: 如何查看详细日志

**A:** 设置 RUST_LOG 环境变量：

```bash
# 全部日志
RUST_LOG=arkcore=debug cargo run --release -- daemon

# 仅错误
RUST_LOG=arkcore=error cargo run --release -- daemon
```

### Q: 沙盒执行超时如何处理

**A:** 沙盒有默认执行超时。可以通过以下方式处理：

```rust
// 检查执行结果
let result = sandbox.execute("sleep 100").await?;
if result.was_killed {
    println!("命令执行超时被终止");
}
```
