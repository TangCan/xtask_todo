---
title: 'Release 配置与体积/启动优化'
story_key: '2-2-release-配置与体积-启动优化'
epic: 2
story: 2
status: review
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../implementation-artifacts/1-4-ci-hard-quality-gates.md'
  - '../implementation-artifacts/2-1-release-baseline-and-repeatable-measurement.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 2.2: Release 配置与体积/启动优化

## Story

作为项目维护者，
我希望优化 release 构建配置，
以便获得更小的二进制和更好的启动表现。

## Acceptance Criteria

1. release profile 的每项调整均有明确记录，至少评估 `strip`、`lto`、`codegen-units` 和 `opt-level`，且不改变 debug/test profile。
2. 使用 Story 2.1 的同一命令、同一目标集合和可比参数测量优化前后实际数值；至少一个目标的体积或冷启动 p50 有可重复改善，未产生收益的实验不保留。
3. workspace release build、debug build、nextest/doctest 和 CLI help 均成功；优化不会改变现有 CLI 输出、退出码或 `.todo.json` 行为。
4. 文档说明 strip/LTO 对调试和故障排查的影响，并保留可解释的调试信息/符号策略；基准报告明确记录实际 commit 与工作树状态。

## Tasks / Subtasks

- [x] 为 workspace 增加经过验证的 release profile 配置，记录候选参数和选择理由。
- [x] 对当前 baseline 做 before/after 测量，比较每个 binary 的 size 与冷启动 min/p50/p95。
- [x] 若候选配置没有可测收益或影响构建/诊断，则回滚该候选并记录结果；保留最终有效组合。
- [x] 更新 release benchmark 文档，说明优化假设、实际数据、平台限制和调试策略。
- [x] 增加配置/回归测试，完成 fmt、nextest、doctest、clippy、rustdoc、release build 和基准命令验证。

## Developer Context

### Scope and Boundaries

- 只调整 Cargo release 配置和相关文档/基准；不修改业务逻辑、CLI 契约、错误退出码或数据格式。
- 必须以 Story 2.1 生成的 JSON 为唯一 before 数据来源，不引用外部报告的数字。
- 选择应以当前 host/target 的实测结果为依据；不要把单一平台结果宣称为所有平台收益。
- 不引入新的构建工具或 CI 供应链动作；继续使用 `cargo xtask release-baseline`。

### Existing Patterns to Preserve

- workspace 根 Cargo.toml 是 virtual workspace；profile 配置放在 workspace 根并保持成员 crate 配置不变。
- Release binary targets 必须由 metadata/基准工具动态发现，不能复制固定目标清单。
- 测量命令使用临时工作目录隔离启动副作用，baseline 输出拒绝覆盖已有文件。

### Testing Requirements

- 测量并保存优化前后 JSON，验证 schema、commit、dirty 状态、目标排序和统计字段。
- `cargo build --workspace --release` 与 `cargo build --workspace`。
- `cargo xtask release-baseline --runs 1 --output <temporary>`，检查输出隔离。
- `cargo fmt --all -- --check`、`cargo nextest run --workspace --all-features`、`cargo test --doc --workspace --all-features`。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`、`RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`。

### Previous Story Intelligence

- Story 2.1 的基准命令会记录 `git_commit` 与 `git_worktree_dirty`，并在临时 cwd 中执行二进制；不要绕过它创建第二套测量逻辑。
- `cargo-devshell --help` 曾在工作区产生 `.dev_shell.bin`，基准隔离修复必须保留。

## Completion Notes

基于 Story 2.1 的实测 baseline 优化 release profile；本故事只保留有证据的配置收益，并记录诊断取舍。
