---
title: 'CI 硬质量门禁'
story_key: '1-4-ci-硬质量门禁'
epic: 1
story: 4
status: done
baseline_commit: '516ef7c'
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../implementation-artifacts/1-3-cli-snapshots-and-data-properties.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 1.4: CI 硬质量门禁

## Story

作为项目维护者，
我希望 CI 自动执行测试、格式、lint、覆盖率和供应链检查，
以便低质量或存在已知漏洞的变更无法合并。

## Acceptance Criteria

1. CI 在 Linux、macOS 和 Windows 上安装并执行 `cargo nextest run`，并单独执行 `cargo test --doc`；任一失败都阻断 job。
2. CI 执行 `cargo fmt --all -- --check` 与 clippy，clippy 使用 `-D warnings`，任一警告或格式问题都使 job 失败。
3. CI 执行 `cargo llvm-cov`/`cargo xtask coverage`，保留机器可读覆盖率工件，并对两个核心 crate 应达到的覆盖率阈值执行失败门禁；阈值和调整方式有文档说明。
4. CI 执行 `cargo deny check` 与 `cargo audit`，许可证、来源、重复版本或 RustSec 检查失败时阻断 job；配置文件和版本策略纳入仓库。
5. 工作流命令在 Linux、macOS、Windows 上使用跨平台可用的步骤和路径，不恢复 tarpaulin 或 Linux-only ptrace 方案。

## Tasks / Subtasks

- [x] 重构 `.github/workflows/ci.yml` 为跨平台质量矩阵，并安装 nextest、llvm-cov、deny、audit 所需工具。
- [x] 将 nextest、doctest、fmt、clippy、覆盖率与供应链检查拆成可定位失败原因的步骤。
- [x] 为覆盖率阈值、JSON 工件上传和阈值调整流程补充文档。
- [x] 新增并验证 `deny.toml` 或等价 cargo-deny 配置与 `cargo audit` 执行方式。
- [x] 用本地等价命令验证 YAML、格式、lint、测试、覆盖率与供应链门禁，并记录平台限制。

## Developer Context

### Scope and Boundaries

- 只修改 CI 配置、供应链配置和质量门禁文档；不改变 todo 业务行为或 JSON/退出码契约。
- 既有 `.githooks/pre-commit` 是本地门禁，不应被 CI 依赖替代；CI 必须显式运行自己的命令。
- 覆盖率沿用 Story 1.2 的 `cargo llvm-cov` 路径和排除意图，不引回 cargo-tarpaulin。

### Existing Patterns to Preserve

- 当前工作流使用 `actions/checkout@v4` 与 `actions-rust-lang/setup-rust-toolchain@v1`。
- workspace 包含 `xtask-todo-lib`、`devshell-vm` 和 `xtask`，测试需覆盖默认 feature workspace。
- `docs/test-coverage.md` 已记录 95% 团队目标与 `cargo xtask coverage`；本 Story 应把门禁来源和报告形式补全，而不是创建第二套阈值。
- nextest 不运行 doctest，必须保留独立的 `cargo test --doc --workspace` 步骤。

### Likely Files

- `.github/workflows/ci.yml`
- `deny.toml`（或仓库既有 cargo-deny 配置位置）
- `docs/test-coverage.md`、`docs/development-guide.md` 或供应链检查文档
- 仅在验证需要时调整脚本/配置，不修改产品源码。

### Testing Requirements

- `cargo fmt --all -- --check`
- `cargo nextest run`
- `cargo test --doc --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo xtask coverage` 或等价的 `cargo llvm-cov` 阈值检查
- `cargo deny check`
- `cargo audit`
- 使用 YAML 解析/静态检查确认矩阵在三类 runner 上语法和 shell 语义成立。

### Previous Story Intelligence

- Story 1.3 的 trycmd 场景依赖真实二进制和 `.in/.out` 文件系统快照；CI 必须保留测试资源，不用工作目录共享来注入数据。
- Story 1.3 最终回归为 484 个 nextest 测试（1 个既有环境跳过），提交钩子还验证 rustdoc 与 Windows MSVC 编译；CI 可复用这些命令但不能假设本地 hook 已执行。
- 覆盖率实现已处理 Windows 路径分隔符，CI 仍应避免把 POSIX shell 假设写进核心命令。

## Completion Notes

已完成六组合平台质量矩阵、固定质量工具版本、nextest/doctest/rustdoc/fmt/clippy/llvm-cov 门禁、覆盖率 JSON 工件、cargo-deny 与 cargo-audit 供应链门禁。当前本地等价验证已通过；Windows ARM runner 由 GitHub-hosted `windows-11-arm` 提供执行环境。
