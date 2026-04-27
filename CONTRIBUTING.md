# Contributing to ArkCore

Thank you for contributing to ArkCore. This document provides guidance on running tests, building the project, and working with the different operating modes.

## Prerequisites

- Rust 1.85 or later (MSRV)
- Cargo (comes with Rust)

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Code Quality

Run Clippy to check for common mistakes and improve code quality:

```bash
cargo clippy
```

For stricter linting:

```bash
cargo clippy -- -D warnings
```

## Testing

### Run All Tests

```bash
# Run library tests
cargo test --lib

# Run integration tests
cargo test --test '*'
```

### Test Coverage

If you have `cargo-llvm-cov` installed:

```bash
# Generate HTML coverage report
cargo llvm-cov --lib --html

# Open the report
cargo llvm-cov --lib --html --open
```

### Running in Different Modes

#### REPL Mode

Interactive read-eval-print loop for experimenting with ArkCore:

```bash
cargo run --release -- repl
```

#### Daemon Mode

Run ArkCore as a background server:

```bash
cargo run --release -- daemon
```

## Windows Platform Notes

Some tests may fail on Windows due to Unix command compatibility issues. The sandbox module uses Unix commands like `ls` and `rm` which are not available on Windows by default.

If you encounter failures in sandbox-related tests on Windows, this is a known limitation. Consider using WSL (Windows Subsystem for Linux) for full compatibility.

## Project Structure

ArkCore consists of the following modules:

- **CLI**: Command-line interface
- **REPL**: Interactive evaluation loop
- **Server**: HTTP/WebSocket server
- **Sandbox**: Command execution sandbox
- **Memory**: Persistent memory storage
- **Orchestrator**: Task orchestration
- **LLM**: Language model integration

## Getting Help

If you encounter issues, please check the project documentation or open an issue on the repository.