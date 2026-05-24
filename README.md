# ArkCore

**Local-First OS Agent Engine**

[![CI](https://github.com/arkcore/arkcore/actions/workflows/ci.yml/badge.svg)](https://github.com/arkcore/arkcore/actions/workflows/ci.yml)
[![Coverage](https://github.com/arkcore/arkcore/actions/workflows/coverage.yml/badge.svg)](https://github.com/arkcore/arkcore/actions/workflows/coverage.yml)
[![Security](https://github.com/arkcore/arkcore/actions/workflows/security.yml/badge.svg)](https://github.com/arkcore/arkcore/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/github/v/release/arkcore/arkcore)](https://github.com/arkcore/arkcore/releases)

ArkCore is a **local-first, privacy-first** OS Agent engine with an extensible plugin architecture. It provides both a CLI REPL and an Axum HTTP Server for flexible interaction modes.

## Core Features

- **Local-First Architecture** — Data stored locally, no cloud dependency
- **Multiple Interaction Modes** — CLI REPL, HTTP API, and WebSocket simultaneously
- **Modular Design** — Core, LLM, Memory, Orchestrator, Sandbox modules cleanly separated
- **Secure Sandbox** — Zero-trust execution environment with 6-layer security checks
- **SQLite Persistence** — Built-in FTS5-powered local storage
- **Async Runtime** — Fully asynchronous processing powered by Tokio
- **Cross-Platform** — Full support for Linux, macOS, and Windows

## Module Architecture

```
arkcore/
├── cli/          # CLI argument parsing
├── core/         # Core abstractions (traits, config, DI)
├── llm/          # LLM provider integration
├── memory/       # SQLite FTS5 memory engine
├── orchestrator/ # Task orchestration (state machine)
├── platform/     # Cross-platform abstractions
├── repl/         # Interactive REPL engine
├── sandbox/      # Zero-trust execution sandbox
├── security/     # Security & compliance auditing
├── server/       # Axum HTTP + WebSocket server
└── services/     # Reliability & performance services
```

## Quick Start

### Installation

**From pre-built binary** (download from [Releases](https://github.com/arkcore/arkcore/releases)):

```bash
# Download the appropriate binary for your platform
chmod +x arkcore
./arkcore repl
```

**From source:**

```bash
git clone https://github.com/arkcore/arkcore.git
cd arkcore
cargo build --release
cargo install --path .
```

**Prerequisites:**

- Rust 1.85+ (MSRV)
- SQLite
- OpenSSL (Unix) or Windows SDK

### Usage

**CLI REPL mode:**

```bash
arkcore repl
```

**HTTP Server mode:**

```bash
arkcore daemon --port 8080
```

**With verbose logging:**

```bash
arkcore -v repl
```

## Configuration

ArkCore reads configuration from `~/.config/arkcore/config.toml` on first run. Alternatively, use environment variables (see `.env.example`):

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
path = "~/.arkcore/data.db"

[security]
sandbox_enabled = true
```

## Main Features

| Module | Description |
|--------|-------------|
| CLI | Command-line interface with subcommand parsing |
| REPL | Read-Eval-Print Loop with history support |
| Server | Axum HTTP server with WebSocket and SSE |
| LLM | LLM provider abstraction (OpenAI, Anthropic, Ollama) |
| Memory | Local SQLite FTS5 storage and memory management |
| Orchestrator | Task orchestration & state machine scheduling |
| Sandbox | Zero-trust command execution sandbox |
| Security | Encryption, RBAC, compliance reports |

## Development

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo llvm-cov --lib --html

# Format check
cargo fmt -- --check

# Clippy lint
cargo clippy -- -D warnings

# Release build
cargo build --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full development guidelines.

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

---

ArkCore Team
