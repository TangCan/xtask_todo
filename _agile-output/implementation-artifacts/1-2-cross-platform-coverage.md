---
title: '跨平台覆盖率能力'
story_key: '1-2-跨平台覆盖率能力'
epic: 1
story: 2
status: done
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../specs/spec-xtask-todo-hardening/SPEC.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 1.2: 跨平台覆盖率能力

## Story

作为项目维护者，
我希望使用 source-based 覆盖率工具生成各 crate 的覆盖率摘要，
以便在 Linux、macOS 和 Windows 上获得一致的质量信号。

## Acceptance Criteria

1. 已安装 `cargo-llvm-cov` 和 `llvm-tools-preview` 时，`cargo xtask coverage` 使用 llvm-cov 运行覆盖率，输出各 crate 摘要，且不调用 cargo-tarpaulin。
2. 现有覆盖率文件排除规则通过 llvm-cov 的 `--ignore-filename-regex` 等价传递，VM、Lima、REPL 胶水代码的排除意图保持不变。
3. 缺少 `llvm-tools-preview` 或 coverage 工具时，命令失败并明确提示安装方式，不输出误导性的成功覆盖率结果。
4. 现有 fake/fail 测试钩子覆盖 llvm-cov 输出格式、成功路径和失败路径，错误传播语义保持不变。

## Tasks / Subtasks

- [x] 将 `xtask/src/coverage.rs` 的执行器从 `cargo tarpaulin` 改为 `cargo llvm-cov`，保留每 crate 摘要和错误传播。
- [x] 将既有排除规则转换为 `--ignore-filename-regex`，并保留 crate-specific 目标与测试参数。
- [x] 更新 fake/fail 测试钩子和解析测试，覆盖 llvm-cov 文本输出及工具缺失提示。
- [x] 更新 README、覆盖率文档和命令帮助中的工具名称及安装前置条件。
- [x] 运行覆盖率相关测试及 workspace 回归命令，确认无业务契约变化。

## Developer Context

### Scope and Boundaries

- 只替换覆盖率实现和文档；不设置覆盖率阈值，不修改 CI 门禁，不实现 Story 1.4 的供应链检查。
- 不改变 `cargo xtask coverage` 的表格摘要、crate 名称、退出错误语义或既有测试钩子环境变量。
- 不改变运行时依赖、`.todo.json` 格式、CLI JSON 输出或退出码语义。

### Existing Patterns to Preserve

- Rust 2021、workspace resolver 2 和纯 Rust cargo-xtask 架构。
- `XTASK_COVERAGE_TEST_FAKE` 与 `XTASK_COVERAGE_TEST_FAKE_FAIL` 测试钩子。
- 当前 VM/Lima/REPL/宿主胶水排除集合及 `--test-threads` 测试参数。
- fake cargo 脚本通过 `CARGO` 环境变量注入，测试不依赖真实外部 coverage 工具。

### Testing Requirements

- 覆盖率模块单测：解析 llvm-cov summary、fake 成功、fake 失败、工具启动失败。
- `cargo nextest run -p xtask`
- `cargo test --doc`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings`

## References

- [Source: _agile-output/planning-artifacts/epics-hardening.md#Story 1.2: 跨平台覆盖率能力]
- [Source: _agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md#CAP-1 测试基础设施]
- [Source: xtask/src/coverage.rs]
- [Source: docs/test-coverage.md]

## Dev Agent Record

### Agent Model Used

Codex GPT-5

### Completion Notes List

- `cargo test -p xtask coverage -- --nocapture` passed: 14 coverage-related tests, including JSON success, argument forwarding, Windows separators, and non-zero child exit.
- Real `cargo xtask coverage` passed with `cargo-llvm-cov`: `xtask-todo-lib` 96.09%, `xtask` 95.38%.
- Missing-tool guidance names both `rustup component add llvm-tools-preview` and `cargo install cargo-llvm-cov`; malformed/non-JSON output cannot produce a successful percentage.
- Automation summary: `_bmad-output/test-artifacts/automation-summary-1-2.md`.
- Code review findings were fixed: portable path separators, strict JSON-only success parsing, complete command argument assertions, and command-level failure handling.
- Full regression after `cargo clean`: 477 nextest tests passed, 1 skipped; doctests, fmt, strict clippy, and rustdoc passed.

### File List

- `xtask/src/coverage.rs`
- `xtask/src/todo/mod.rs`
- `crates/todo/tests/integration.rs`
- `README.md`
- `docs/test-coverage.md`
- `_agile-output/implementation-artifacts/1-2-cross-platform-coverage.md`
- `_agile-output/implementation-artifacts/spec-1-2-cross-platform-coverage.md`
- `_bmad-output/test-artifacts/atdd-checklist-1-2-cross-platform-coverage.md`
- `_bmad-output/test-artifacts/automation-summary-1-2.md`
