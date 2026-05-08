# ArkCore

**Local-First OS Agent Engine**

[版本 Shield](https://img.shields.io/badge/version-0.2.1-blue.svg)
[Rust Shield](https://img.shields.io/badge/rust-1.85-orange.svg)
[许可证 Shield](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)

ArkCore 是一个本地优先的操作系统 Agent 引擎，提供 CLI REPL 和 Axum HTTP Server 两种交互方式。

## 核心特性

- **本地优先架构** - 数据存储在本地，不依赖云服务
- **多种交互方式** - 同时支持 CLI REPL、Web UI 和 HTTP API
- **模块化设计** - 核心、LLM、Memory、Orchestrator 等模块清晰分离
- **安全沙箱** - 提供安全的执行环境
- **SQLite 持久化** - 内置 SQLite 数据库支持
- **异步运行时** - 基于 Tokio 的全异步处理

## 模块架构

```
arkcore/
├── cli/          # 命令行接口
├── core/         # 核心功能
├── llm/          # LLM 集成
├── memory/       # 本地存储与记忆
├── orchestrator/ # 任务编排
├── platform/     # 平台相关功能
├── repl/         # REPL 交互引擎
├── sandbox/      # 安全沙箱
├── security/     # 安全与加密
├── server/       # HTTP 服务器
├── services/     # 业务服务
└── web/          # Dioxus Web UI
```

## 快速开始

### 安装

**从源码编译**

```bash
git clone https://github.com/arkcore/arkcore.git
cd arkcore
cargo build --release
cargo install --path .
```

**依赖要求**

- Rust 1.85+
- SQLite
- OpenSSL (Unix) 或 Windows SDK

### 使用方式

**CLI REPL 模式**

```bash
arkcore repl
```

**HTTP 服务器模式**

```bash
arkcore daemon --port 8080
```

## 主要功能

| 模块 | 功能描述 |
|------|----------|
| CLI | 命令行界面，支持交互式命令输入 |
| REPL | 读取-执行-打印循环，实时交互 |
| Server | Axum HTTP 服务器，支持 WebSocket |
| LLM | 大语言模型集成接口 |
| Memory | 本地 SQLite 存储与记忆管理 |
| Orchestrator | 任务编排与调度 |
| Sandbox | 安全沙箱执行环境 |
| Security | 加密、认证与安全验证 |

## 配置

ArkCore 默认配置位于 `~/.arkcore/config.toml`。首次运行时会自动创建。

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
path = "~/.arkcore/data.db"

[security]
encryption = true
```

## 开发

```bash
# 运行测试
cargo test

# 运行基准测试
cargo bench

# 代码格式检查
cargo fmt --check

# _clippy 检查
cargo clippy -- -D warnings
```

## 贡献

欢迎提交 Issue 和 Pull Request。重大更改请先开 Issue 讨论。

## 许可证

本项目基于 **MIT OR Apache-2.0** 许可证分发。

---

ArkCore Team
