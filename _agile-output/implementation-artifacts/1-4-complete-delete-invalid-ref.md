# Story 1.4：完成、删除与非法引用

Status: done

## Story

作为一名开发者，  
我希望完成或删除待办，并在 id 不存在或非法时得到约定错误语义，  
以免误删或静默失败。

## 映射需求

- **FR4**（`epics.md` Story 1.4；`docs/requirements.md` §3.1「完成 / 删除」、§6 与 **US-A2** 退出码）
- **Epic 5**：`--json` 失败时 stdout 为统一 `{"status":"error","error":{...}}`，与 `exit_code()` 一致（与 Story 1.2 / 1.3 契约对齐）

## Acceptance Criteria

1. **Given** 工作区存在至少一条待办，且 id 有效  
   **When** 执行 `complete <id>` / `delete <id>`（含可选 `--no-next` 于 `complete`）  
   **Then** 成功路径更新内存与 `.todo.json`；人类可读与 `--json` 成功载荷与 `handle_complete` / `handle_delete` 实现一致（**TC-T3-1**、**TC-T4-1**、**TC-X4-2**）。

2. **Given** id 为 **0** 或无法解析为有效 `TodoId`  
   **When** 执行 `complete` / `delete`  
   **Then** 进程退出码 **2**（参数类），错误信息可读；`--json` 时为 `status: error` 且 `error.code == 2`（**TC-A2-2**）。

3. **Given** id 语法有效但**不存在**于当前存储  
   **When** 执行 `complete` / `delete`  
   **Then** 进程退出码 **3**（数据类），`todo not found` 语义；`--json` 时 `error.code == 3`（**TC-A2-3**）。

4. **失败路径不写盘**：在 **2** / **3** 类错误下，`.todo.json` 不因本次命令而被错误覆盖或截断（与 Story 1.1 棕地一致；可参考 `invalid_optional_leaves_existing_file_unchanged` 类断言）。

5. **回归**：不破坏 Story 1.1～1.3 已建立行为（`add`/`list`/`init-ai`、全局 `--json` 顺序）。

## Tasks / Subtasks

- [x] **棕地核对**：领域层 `TodoList::complete` / `delete`（`crates/todo`）；CLI `handle_complete` / `handle_delete`（`xtask/src/todo/cmd/dispatch.rs`）；`TodoCliError` 与 `exit_code`（`xtask/src/todo/error.rs`）；现有 `xtask/src/tests/todo/todo_cmd/crud.rs`（`*_id_zero_*`、`*_nonexistent_*`）与 `list_options.rs` 中 dry-run complete/delete。
- [x] **端到端缺口**：在 `xtask/tests/integration.rs` 增补 `cargo xtask todo` 对 `complete` / `delete` 的 `--json` 成功/失败（2/3）与 `error.message`、不写盘 / 保留存储断言（`todo --json complete|delete …`）。
- [x] **init-ai**：`xtask/src/todo/init_ai.rs` 已含 `complete <id> [--no-next]`、`delete <id>`，与 `args.rs` 一致，无需改。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask --test integration`、`cargo clippy -p xtask --all-targets -- -D warnings` 均通过（注：`cargo test -p xtask --lib` 在部分环境下 `lima_todo::cmd_lima_todo_print_only_no_build_smoke` 可能因临时目录 workspace 元数据失败，与本故事无关）。

### Review Findings

- [x] [Review][Patch] AC4 对称性 — 已补 `xtask_todo_json_delete_nonexistent_preserves_store`（失败 `delete` 不篡改 `.todo.json`）— [`xtask/tests/integration.rs`](../../xtask/tests/integration.rs)
- [x] [Review][Patch] `crates/todo/.dev_shell.bin` — 审查结论：勿与功能变更一并提交；若本地出现该 diff，使用 `git restore` 或从暂存区剔除（是否 `.gitignore` 由仓库策略决定）

## Change Log

- 2026-03-26：自 `epics.md` / `test-cases.md` 生成故事；下一迭代实施棕地核对与集成测试补全。
- 2026-03-26：完成集成测试与棕地核对；故事进入 `review`。
- 2026-03-26：BMad code review 后补 `xtask_todo_json_delete_nonexistent_preserves_store`；故事标为 `done`。

## Dev Notes

### 棕地现状

- **领域**：`TodoList::complete`、`TodoList::delete` 与 `TodoError::NotFound`（`crates/todo`）。
- **CLI**：`TodoCompleteArgs` / `TodoDeleteArgs`；`cmd_todo` 经 `xtask`/`todo` 二进制统一 JSON 错误打印（`xtask/src/lib.rs`、`xtask/src/bin/todo.rs` 调 `print_json_error`）。
- **已有测试**：`cmd_todo_complete_id_zero_errors`、`cmd_todo_complete_nonexistent_id_returns_exit_code_3`、`cmd_todo_delete_id_zero_errors`、`cmd_todo_delete_nonexistent_id_returns_exit_code_3`、`cmd_todo_complete_and_delete`（`crud.rs`）。

### 测试与追溯

- `docs/test-cases.md`：**TC-A2-2**、**TC-A2-3**、**TC-T3-***、**TC-T4-***、**TC-X4-2**。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.4]
- [Source: `docs/requirements.md` §3.1、§6、US-A2]
- [Source: `docs/test-cases.md` — TC-A2、TC-T3、TC-T4、TC-X4]

## Dev Agent Record

### Agent Model Used

Cursor Agent（GPT-5.1）

### Debug Log References

（无）

### Completion Notes List

- 棕地：`handle_complete` / `handle_delete`、`TodoCliError` 映射与 `crud.rs` 单测已覆盖核心路径；未改领域/CLI 业务逻辑。
- 新增 `xtask/tests/integration.rs`：`xtask_todo_json_complete_then_delete_success`、`xtask_todo_json_complete_no_next_success`、`xtask_todo_json_complete_id_zero_exit_2`、`xtask_todo_json_delete_id_zero_exit_2`、`xtask_todo_json_complete_nonexistent_exit_3`、`xtask_todo_json_delete_nonexistent_exit_3`、`xtask_todo_json_complete_nonexistent_preserves_store`、`xtask_todo_json_delete_nonexistent_preserves_store`。
- `init-ai` 无需修改。

### File List

- `xtask/tests/integration.rs`
- `_bmad-output/implementation-artifacts/1-4-complete-delete-invalid-ref.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化）
- [x] 测试与 `docs/test-cases.md` ID 可追溯（TC-A2-2/3、TC-T3-1、TC-T4-1、端到端 TC-X4-2 类）
