# ArkCore 本地 CI 验证脚本
# 与 .github/workflows/ci.yml 保持严格对齐
# 用法: .\scripts\ci.ps1

param(
    [switch]$SkipTests,
    [switch]$SkipClippy,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$FAILED = $false

function Write-Step {
    param([string]$Message)
    Write-Host "`n=== $Message ===" -ForegroundColor Cyan
}

function Run-Command {
    param(
        [string]$Name,
        [string]$Command
    )
    Write-Host "Running: $Command" -ForegroundColor Gray
    $output = Invoke-Expression $Command 2>&1
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        Write-Host "FAILED (exit code: $exitCode)" -ForegroundColor Red
        $script:FAILED = $true
    }
    return $exitCode
}

Write-Host "ArkCore Local CI Validation" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

# 1. Format Check (CI: cargo fmt -- --check)
Write-Step "Format Check"
if (Run-Command "cargo fmt -- --check" "cargo fmt -- --check") {
    # Format issues found
    Write-Host "Hint: Run 'cargo fmt' to fix formatting" -ForegroundColor Yellow
}

# 2. Clippy (CI: cargo clippy -- -D warnings)
if (-not $SkipClippy) {
    Write-Step "Clippy Lint"
    if (Run-Command "cargo clippy -- -D warnings" "cargo clippy -- -D warnings") {
        Write-Host "Lint errors detected" -ForegroundColor Red
    }
}

# 3. Tests (CI: cargo test --lib && cargo test --doc && cargo test --test '*')
if (-not $SkipTests) {
    Write-Step "Library Tests (cargo test --lib)"
    if (Run-Command "cargo test --lib" "cargo test --lib") {
        Write-Host "Library tests failed" -ForegroundColor Red
    }

    Write-Step "Doc Tests (cargo test --doc)"
    if (Run-Command "cargo test --doc" "cargo test --doc") {
        Write-Host "Doc tests failed" -ForegroundColor Red
    }

    Write-Step "Integration Tests (cargo test --test '*')"
    if (Run-Command "cargo test --test '*'" "cargo test --test '*'") {
        Write-Host "Integration tests failed" -ForegroundColor Red
    }
}

# 4. Build (CI: cargo build --release)
if (-not $SkipBuild) {
    Write-Step "Release Build (cargo build --release)"
    if (Run-Command "cargo build --release" "cargo build --release") {
        Write-Host "Build failed" -ForegroundColor Red
    }
}

# Summary
Write-Host "`n================================" -ForegroundColor Green
if ($FAILED) {
    Write-Host "CI VALIDATION FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "CI VALIDATION PASSED" -ForegroundColor Green
    exit 0
}
