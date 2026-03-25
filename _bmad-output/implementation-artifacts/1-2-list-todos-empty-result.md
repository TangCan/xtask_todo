# Story 1.2：列出待办与空结果

Status: done

## Story

作为一名开发者，  
我希望在无数据时看到可理解的空列表语义，  
以便区分「无待办」与「命令失败」。

## 映射需求

- **FR2**（`docs/requirements.md` §3.1、`epics.md` Story 1.2）
- **Epic 5**：`list` / `search` 的 `--json` 成功载荷与终端语义一致（`architecture.md` / `error.rs` 约定）

## Acceptance Criteria

1. **Given** 工作区无 `.todo.json` 或文件为空数组、或无任何有效条目  
   **When** 执行 `todo list`（或等价：`cargo xtask todo list` / `cargo run -p xtask -- todo list`）  
   **Then** 人类可读输出**明确表示**空集（当前：`No tasks.`）。

2. **Given** 同上  
   **When** 执行 `todo --json list`（全局 `--json` 在子命令前）  
   **Then** stdout 为统一成功 JSON，`data` 含 `"items": []`，并含 **`"empty": true`** 与 **`"message": "No tasks."`**，退出码 **0**（与「失败 JSON」区分）。

3. **`search`** 在 `--json` 且无匹配时与 **`list`** 使用同一空列表载荷形状（`items` + `empty` + `message`），与人类可读一致。

## Tasks / Subtasks

- [x] 抽取共享空列表 JSON 载荷（`todo_list_json_payload`），与 `format::EMPTY_LIST_MESSAGE` 对齐。
- [x] `handle_list` / `handle_search` 使用上述载荷；更新 `init-ai` 生成文档中的 `--json` 用法说明。
- [x] 单元测试：`todo_list_json_payload` 空/非空；集成测试：`xtask todo --json list` 空仓库 JSON 断言。

## Change Log

- 2026-03-25：`todo list` / `search` 的 `--json` 成功载荷增加 `empty` 与空集 `message`；集成测试覆盖 `cargo xtask todo --json list`。
- 2026-03-25：Code review 通过；故事标记为 done，无待修复项（见 Review Findings 中的 defer）。

## Dev Notes

### 技术要点

- **全局 `--json`**：`TodoArgs` 上为 `todo --json list`，**不是** `todo list --json`（见 `xtask/src/lib.rs` / `argh`）。
- **常量**：`xtask/src/todo/format.rs` — `EMPTY_LIST_MESSAGE` 供终端与 JSON 共用。

### 测试

- `cargo test -p xtask todo_list_json_payload`
- `cargo test -p xtask --test integration xtask_todo_list_empty_json_matches_empty_semantics`
- `cargo clippy -p xtask --all-targets -- -D warnings`

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.2]
- [Source: `docs/requirements.md` §3.1 列表]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

（无）

### Completion Notes List

- 在 `error.rs` 增加 `todo_list_json_payload`，空结果时 `empty: true` + `message` 与 `EMPTY_LIST_MESSAGE` 一致；非空时 `empty: false` 且无 `message`。
- 集成测试使用 `xtask todo --json list` 顺序，避免 `todo list --json` 被 argh 拒绝。

### File List

- `xtask/src/todo/format.rs`
- `xtask/src/todo/error.rs`
- `xtask/src/todo/cmd/dispatch.rs`
- `xtask/src/todo/init_ai.rs`
- `xtask/tests/integration.rs`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-2-list-todos-empty-result.md`

## 完成状态

- [x] 所有 AC 已验证（自动化）
- [x] 仅修改故事文件允许区域与实现代码

### Review Findings

- [x] [Review][Defer] `load_todos` / `load_todos_from_path`：JSON 解析失败时 `unwrap_or_default()` 视为空列表，损坏的 `.todo.json` 与「无待办」在成功路径上不可区分；若需区分数据错误与空集，应另开故事改 I/O 行为 [`xtask/src/todo/io.rs` 约 77–106 行] — deferred, pre-existing
- [x] [Review][Defer] `handle_add`：`--dry-run` 在 `patch_from_add_args` 之前返回，dry-run 不校验非法可选参数；与 Story 1.1 / `deferred-work.md` 已记录项一致 [`xtask/src/todo/cmd/dispatch.rs` 约 76–90 行] — deferred, pre-existing
