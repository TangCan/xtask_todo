---
title: '测试运行器与测试工具链升级'
story_key: '1-1-测试运行器与测试工具链升级'
epic: 1
story: 1
status: done
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../specs/spec-xtask-todo-hardening/SPEC.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
completion_note: 'Ultimate context engine analysis completed - comprehensive developer guide created'
---

# Story 1.1: 测试运行器与测试工具链升级

## Story

作为项目维护者，
我希望使用更快且隔离性更好的测试工具链，
以便测试结果成为可靠的质量信号。

## Acceptance Criteria

1. 工作区已有单元测试和集成测试时，`cargo nextest run` 能通过全部非 doctest 测试，且既有测试断言、JSON 输出契约和退出码语义保持不变。
2. xtask 可使用 `trycmd`、`assert_cmd`、`assert_fs`、`insta`、`proptest`；`xtask-todo-lib` 可使用 `insta`、`proptest`。
3. `cargo test --doc` 独立通过全部 doctest。
4. nextest 并行运行时，测试之间不共享可变状态或产生竞态；现有 cwd/temp-dir 相关测试保持通过。

## Tasks / Subtasks

- [x] 更新 `xtask/Cargo.toml` 的 `[dev-dependencies]`，加入 `trycmd`、`assert_cmd`、`assert_fs`、`insta`、`proptest`。
  - [x] 保持 workspace 现有依赖版本策略；Cargo.lock 按仓库既有 `.gitignore` 策略不纳入版本控制。
- [x] 更新 `crates/todo/Cargo.toml` 的 `[dev-dependencies]`，加入 `insta`、`proptest`。
  - [x] 不改变库的运行时依赖、默认 feature 或 `.todo.json` 数据格式。
- [x] 验证 nextest 与现有测试隔离模型兼容。
  - [x] 检查 `cwd_test_lock`、`RestoreCwd`、临时目录和环境变量测试是否依赖进程内并发假设。
  - [x] 未发现需要修改隔离实现的缺口；未删除既有断言或降低测试覆盖。
- [x] 在本地执行 `cargo nextest run` 和 `cargo test --doc`，记录验证结果。
- [x] 更新本 Story 的 Implementation Notes，记录最终依赖版本、隔离检查和验证命令。

## Developer Context

### Scope and Boundaries

- 本 Story 只建立测试运行器和测试工具基础设施；覆盖率实现、CI 门禁和六维 list 快照属于后续 Story 1.2–1.4。
- 不新增待办业务功能，不切换 `.todo.json` 格式，不引入 just/cargo-make。
- 不改变 `--json` 输出契约或退出码：参数错误为 2，业务/数据失败为 3。

### Repository Structure

- `xtask/`：宿主编排、CLI 和集成测试。
- `crates/todo/`：`xtask-todo-lib` 领域、存储和 devshell 代码。
- `xtask/tests/`：CLI/集成测试；现有 `todo_list/list_json.rs` 等测试必须保留。
- `crates/todo/src/list/tests.rs`、`crates/todo/src/store.rs`、`crates/todo/src/model.rs`：后续快照/属性测试的目标位置，本 Story 只提供依赖。

### Existing Patterns to Preserve

- Rust edition 2021，workspace resolver 2。
- 各 crate 自己声明 clippy lint；根虚拟 workspace 不添加 `[lints]`。
- 测试命令必须覆盖 workspace，包括 `devshell-vm` 和 `beta-vm` 默认 feature。
- 现有测试中的 cwd 恢复、临时目录、环境变量清理和串行锁不可被无依据删除。
- `cargo nextest` 默认不执行 doctest，因此 doctest 必须由 `cargo test --doc` 单独运行。

### Implementation Guidance

- `cargo-nextest` 的 `run` 可替代当前非 doctest 的 `cargo test` 流程，并提供进程级测试隔离；不要把 doctest 误判为 nextest 覆盖范围。
- 新增测试依赖应放在对应 crate 的 `[dev-dependencies]`，不要提升为运行时依赖。
- `trycmd`、`assert_cmd` 和 `assert_fs` 的实际用法留给 Story 1.3；本 Story 只确保依赖可解析、可编译。
- 优先运行单个受影响 crate 的测试，再运行 workspace 全量测试；避免通过禁用测试或串行化所有测试掩盖隔离问题。

### Latest Tooling Notes

- cargo-nextest 官方文档说明 `cargo nextest run` 运行非 ignored 的测试，并兼容 cargo test 的常用选项；项目仍需单独执行 doctest。
- trycmd 官方文档定位为批量 CLI snapshot harness；Story 1.3 再创建 `.trycmd` fixtures。
- cargo-llvm-cov 支持 nextest 集成，但覆盖率迁移属于 Story 1.2，不在本 Story 修改 coverage 实现。

### Testing Requirements

- `cargo nextest run`
- `cargo test --doc`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`

### References

- [Source: _agile-output/planning-artifacts/epics-hardening.md#Story 1.1: 测试运行器与测试工具链升级]
- [Source: _agile-output/specs/spec-xtask-todo-hardening/SPEC.md#CAP-1 测试基础设施升级]
- [Source: _agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md#CAP-1 测试基础设施]
- [Source: _agile-output/project-context.md#测试相关]
- [Source: xtask/Cargo.toml]
- [Source: crates/todo/Cargo.toml]
- [Source: .github/workflows/ci.yml]
- [Official cargo-nextest running tests](https://nexte.st/docs/running/)
- [Official trycmd documentation](https://docs.rs/trycmd/latest/trycmd/)
- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)

## Dev Agent Record

### Agent Model Used

Codex GPT-5

### Debug Log References

### Completion Notes List

- Story context generated from approved hardening Epic 1 and current repository state.
- Existing detailed hardening story was preserved; this implementation story provides the standard BMad build context.
- `cargo check --workspace --all-targets` passed after resolving the new dev-dependencies.
- `cargo nextest run` passed: 471 tests passed, 1 skipped.
- `cargo test --doc` passed: 0 doctests discovered and 0 failed.
- `docs/development-guide.md` documents installing cargo-nextest and the separate doctest command.
- Automation review completed with no new tests or fixtures required; see `_bmad-output/test-artifacts/automation-summary.md`.
- Regression passed after `cargo clean`: `cargo fmt --all -- --check`, `cargo nextest run`, `cargo test --doc`, strict clippy, and rustdoc.
- Code review findings were triaged: local nextest installation was documented, CI migration was deferred to Story 1.4, and the ignored `Cargo.lock` policy was preserved.

### File List

- `_agile-output/implementation-artifacts/1-1-test-runner-and-testing-toolchain-upgrade.md`
- `_agile-output/implementation-artifacts/spec-1-1-test-runner-and-testing-toolchain-upgrade.md`
- `_bmad-output/test-artifacts/atdd-checklist-1-1-test-runner-and-testing-toolchain-upgrade.md`
- `_bmad-output/test-artifacts/automation-summary.md`
