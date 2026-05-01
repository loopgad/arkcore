# ArkCore 开发者快速入门

本文档帮助开发者快速上手 ArkCore 项目开发。

## 环境准备

### 前置依赖

- **Rust 1.85+** (推荐使用 rustup 安装)
- **SQLite3**
- **Git**

### 安装 Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# 下载 rustup-init.exe 并运行

# 验证安装
rustc --version
cargo --version
```

### 平台依赖

| 平台 | 依赖 |
|------|------|
| Linux | `libsqlite3-dev`, `libssl-dev` |
| macOS | `sqlite3` (Homebrew) |
| Windows | Visual Studio Build Tools |

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/arkcore/arkcore.git
cd arkcore
```

### 2. 构建项目

```bash
# Debug 构建
cargo build

# Release 构建 (更快的运行时性能)
cargo build --release

# 仅检查代码 (不生成二进制)
cargo check
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 运行库测试
cargo test --lib

# 运行特定模块测试
cargo test sandbox
cargo test orchestrator
cargo test llm

# 查看测试覆盖率
cargo install cargo-llvm-cov
cargo llvm-cov --lib --html
```

## 项目结构

```
arkcore/
├── src/
│   ├── cli/           # 命令行入口
│   ├── core/          # 核心 trait 和类型
│   ├── llm/           # LLM 提供者 (OpenAI, Anthropic, Ollama)
│   ├── memory/        # SQLite 持久化层
│   ├── orchestrator/  # 任务编排器
│   ├── platform/      # 平台适配 (Windows/Linux/macOS)
│   ├── sandbox/       # 安全沙箱执行
│   ├── security/      # 加密、认证、审计
│   ├── services/      # 业务服务 (缓存、重试、工具注册)
│   └── vector/        # 向量存储和 RAG 支持
├── tests/             # 集成测试
├── benches/           # 基准测试
└── docs/              # 文档
```

## 模块开发指南

### 添加新的 LLM 提供者

```rust
// src/llm/provider/ 目录下创建新文件
// 实现 LLMProvider trait

use crate::core::traits::{LLMProvider, Message, StreamEvent};

pub struct MyProvider {
    config: LlmConfig,
}

impl LLMProvider for MyProvider {
    type Error = LlmError;

    fn stream_chat(&self, messages: &[Message]) -> Result<Vec<StreamEvent>, Self::Error> {
        // 实现聊天逻辑
    }
}
```

### 添加新的工具

```rust
// src/services/tool/builtin.rs
// 实现 Tool trait

#[derive(Debug, Clone)]
pub struct MyTool;

#[async_trait::async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "描述工具功能"
    }

    async fn execute(&self, input: Value) -> Result<Value, ToolError> {
        // 实现工具逻辑
    }
}
```

### 使用向量存储

```rust
use crate::vector::{VectorStore, SqliteVectorStore};

let store = SqliteVectorStore::new(pool).await?;

// 存储向量
store.insert(embedding, metadata).await?;

// 搜索
let results = store.search(query_embedding, top_k).await?;
```

## 代码质量

### 代码格式化

```bash
cargo fmt
cargo fmt -- --check  # 检查格式
```

### Lint 检查

```bash
cargo clippy
cargo clippy -- -D warnings  # 严格模式
```

### 运行文档测试

```bash
cargo test --doc
```

## 调试技巧

### 日志设置

```bash
RUST_LOG=debug cargo run --release -- repl
```

### 查看编译依赖

```bash
cargo tree
cargo tree -i crate-name  # 查看特定依赖的依赖树
```

### 性能分析

```bash
cargo install flamegraph
cargo flamegraph --bin arkcore -- repl
```

## 下一步

- 阅读 [用户指南](docs/user-guide.md) 了解完整功能
- 阅读 [API 文档](docs/api.md) 了解接口详情
- 阅读 [安全文档](docs/security.md) 了解安全特性
- 查看 [CHANGELOG.md](CHANGELOG.md) 了解版本历史

## 获取帮助

- 提交 [GitHub Issue](https://github.com/arkcore/arkcore/issues)
- 参与 [Discussions](https://github.com/arkcore/arkcore/discussions)
