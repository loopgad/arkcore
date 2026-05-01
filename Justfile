# ArkCore 本地验证脚本
# 与 .github/workflows/ci.yml 保持严格对齐
#
# Windows: .\scripts\ci.ps1
# Unix/macOS: just (如果安装) 或直接运行以下命令

# 运行所有测试 (对应 CI test job)
test:
    cargo test --lib && cargo test --doc && cargo test --test '*'

# 仅运行库测试
test-lib:
    cargo test --lib

# 仅运行文档测试
test-doc:
    cargo test --doc

# 仅运行集成测试
test-integration:
    cargo test --test '*'

# 代码质量检查 (对应 CI clippy job)
check: fmt clippy

# 格式化检查
fmt:
    cargo fmt -- --check

# 自动修复格式
fmt-fix:
    cargo fmt

# Lint 检查
clippy:
    cargo clippy -- -D warnings

# 构建 (对应 CI build job)
build:
    cargo build --release

# 开发构建
build-dev:
    cargo build

# 运行所有检查 (提交前必跑)
pre-commit: fmt-fix check test build

# 快速检查 (不运行测试)
ci-quick: fmt check build

# 帮助
default:
    @echo "ArkCore 本地验证命令:"
    @echo ""
    @echo "  just test           # 运行所有测试 (对应 CI)"
    @echo "  just check          # 代码质量检查"
    @echo "  just fmt            # 格式检查"
    @echo "  just fmt-fix        # 自动修复格式"
    @echo "  just clippy         # Lint 检查"
    @echo "  just build          # Release 构建"
    @echo "  just pre-commit     # 提交前必跑: 格式+检查+测试+构建"
    @echo "  just ci-quick       # 快速检查 (不测试)"
    @echo ""
    @echo "Windows 用户 (无 just):"
    @echo "  .\scripts\ci.ps1            # 完整检查"
    @echo "  .\scripts\ci.ps1 -SkipTests # 跳过测试快速检查"
