# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-04-28

### Added

- **LLM Provider** Implemented real Ollama provider with SSE streaming support
- **Ollama Integration** Full API implementation with health check and streaming responses
- **Environment Config** Added `LlmConfig::ollama()` constructor for simplified Ollama setup

### Changed

- **Cargo.toml** Updated version from 0.1.0 to 0.2.1
- **.gitignore** Added proper ignore patterns for Rust build output

### Dependencies

- Added `reqwest` with JSON and streaming features for HTTP client support

---

## [0.2.0] - 2026-04-27

### Security

- **S1** Fixed command injection vulnerability in sandbox executor with whitelist + escaping dual defense and parameterized command execution
- **S2** Replaced insecure key derivation (SHA256 iteration) with PBKDF2-HMAC-SHA256 (600,000+ iterations per OWASP 2023)
- **S3** Fixed empty salt encryption by implementing cryptographically secure random salt generation using `getrandom` crate (32 bytes / 256 bits)
- **S4** Extended command blacklist based on OWASP Command Injection Prevention best practices

### Fixed

- **T1** Fixed integer overflow in `platform::tty::test_ansi_color_conversion` using saturating arithmetic
- **T2** Fixed floating-point precision issue in `services::ratelimit::test_token_bucket_acquire` using epsilon comparison
- **T3** Fixed assertion failure in `services::security::rbac::test_rbac_middleware`
- **T4** Fixed pattern detection issue in `services::security::validator::tests`
- **T5** Fixed key rotation issue in `services::security::crypto::test_key_rotation`
- **T6** Fixed Windows compatibility in `sandbox::executor::tests` with conditional test execution
- **T7** Fixed Windows compatibility in `orchestrator::tests` using mock executor

### Added

- **cli** Added unit tests for CLI module (command parsing, config generation, error handling)
- **repl** Added unit tests for REPL module (command parsing, history, auto-completion)
- **server** Added integration tests for server module (HTTP endpoints, WebSocket, graceful shutdown)
- **core** Added unit tests for core module (DI container, config loading, trait bounds)

### Changed

- **Q2** Moved hardcoded WebSocket URL to configuration file
- **Q3** Changed sensitive information hint in CLI config to generic message

### Code Quality

- **C1** Replaced `println!` with `tracing::info!` in REPL module
- **C2** Replaced weak test keys with secure mock keys in API key module
- **C3** Added SAFETY comments for unsafe impl绕过 in orchestrator module
- **C4** Removed deprecated `ArkCoreError` alias and updated all call sites

---

## [0.1.0] - 2026-04-27

### Added

- Initial release
- CLI REPL interface
- Dioxus Web UI
- Axum HTTP Server with WebSocket support
- SQLite-based local storage and memory management
- Task orchestration
- Security sandbox execution environment
- Encryption, authentication, and security validation
