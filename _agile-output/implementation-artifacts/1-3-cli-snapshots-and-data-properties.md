---
title: 'CLI 快照与数据层属性测试'
story_key: '1-3-cli-快照与数据层属性测试'
epic: 1
story: 3
status: done
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../specs/spec-xtask-todo-hardening/SPEC.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 1.3: CLI 快照与数据层属性测试

## Story

作为项目维护者，
我希望用批量快照和属性测试覆盖 CLI 与数据层边界，
以便及时发现过滤、排序和序列化回归。

## Acceptance Criteria

1. 真实 `todo` 二进制的 trycmd 套件覆盖 `--status`、`--priority`、`--tags`、`--due-before`、`--due-after` 和 `--sort`，每个维度都有匹配结果与无匹配结果。
2. 快照场景不重复已有 `list_json.rs` 的断言，并保持既有 JSON 输出契约。
3. 数据层使用 proptest 验证过滤结果不变量，并使用 insta 固定稳定排序/过滤快照；失败输入可由 proptest 持久化机制重放。
4. 非法状态、日期等 list 参数返回退出码 2，且已有 `.todo.json` 不被修改；`deferred-work.md` 的 list E2E 债务有测试证据。

## Tasks / Subtasks

- [x] 创建 trycmd harness，注册真实 `todo` 二进制并添加固定 `.todo.json` fixture。
- [x] 添加六个 list 维度的正向/无匹配快照，覆盖稳定字段和 JSON 输出。
- [x] 在 `xtask-todo-lib` 添加 proptest 过滤不变量和 insta 稳定快照。
- [x] 补充非法参数及现有数据不变的端到端证据，更新 deferred-work 记录。
- [x] 运行 trycmd、nextest、doctest 和完整回归命令。

## Developer Context

### Scope and Boundaries

- 只增加测试、fixtures 和测试文档；不改变 todo 业务实现、JSON 字段、退出码或 `.todo.json` 格式。
- trycmd 必须通过真实构建的 `todo` 二进制运行；避免复制已有 `list_json.rs` 的排序 JSON 断言。
- 属性测试使用固定、可复现的策略；不引入随机网络、时间或全局 cwd 状态。

### Existing Patterns to Preserve

- 测试工作区使用 nextest；doctest 独立执行。
- CLI 集成测试使用 `env!("CARGO_BIN_EXE_todo")`/`env!("CARGO_BIN_EXE_xtask")`。
- list 的 JSON 成功/错误 payload、参数退出码 2 和 `.todo.json` 写入边界必须保持。

### Testing Requirements

- `cargo test -p xtask --test trycmd_list_snapshots`
- `cargo nextest run`
- `cargo test --doc`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings`

## References

- [Source: _agile-output/planning-artifacts/epics-hardening.md#Story 1.3: CLI 快照与数据层属性测试]
- [Source: xtask/tests/todo_list/list_json.rs]
- [Source: crates/todo/src/model.rs]
- [Source: crates/todo/src/list/mod.rs]
- [Source: _agile-output/implementation-artifacts/deferred-work.md]

## Dev Agent Record

### Agent Model Used

Codex GPT-5

### Completion Notes List

- 已覆盖真实 `todo` 二进制的六类 list 查询、无匹配结果、固定 fixture、属性不变量及非法日期数据不变性。
- 定向测试、workspace 测试、doctest、clippy、fmt 与代码审查已完成；nextest 将在故事提交前执行最终回归。

### File List

- `xtask/tests/trycmd_list_snapshots.rs`
- `xtask/tests/trycmd/list_filters.trycmd`
- `xtask/tests/trycmd/list_filters.in/.todo.json`
- `xtask/tests/trycmd/list_filters.out/.todo.json`
- `xtask/tests/todo_list/list_json.rs`
- `crates/todo/src/tests/property.rs`
- `crates/todo/src/tests/snapshots/xtask_todo_lib__tests__property__title_sort_snapshot_is_stable.snap`
