# Story 1.5：查看详情与更新可选字段

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望查看单条详情并更新描述、截止日期、优先级、重复规则等可选字段，  
以便维护任务上下文。

## 映射需求

- **FR5**（`epics.md` Story 1.5；`docs/requirements.md` §3.1「单条 / 更新」、§3.2 `show` / `update`）
- **Epic 5**：`show` / `update` 的成功与失败 JSON、`exit_code` 与 `error.rs` 约定一致；`--dry-run` 不写盘（与 Story 1.2 / 1.4 对齐）

## Acceptance Criteria

1. **Given** 工作区存在 id 为 `N` 的待办（含可选字段或部分为空）  
   **When** 执行 `todo show <N>`（或等价：`cargo xtask todo show <N>`）  
   **Then** 人类可读输出展示标题、状态、时间、描述/截止/优先级/标签/重复等（与 `handle_show` 一致）；`todo --json show <N>` 的 `data` 为 `todo_to_json` 形状，退出码 **0**（**TC-T7-1**，与 `docs/test-cases.md` US-T7 一致）。

2. **Given** id 为 **0** 或**不存在**  
   **When** 执行 `show`  
   **Then** 退出码 **2**（id 0）或 **3**（不存在）；`--json` 时 `status: error` 且 `error.code` 与退出码一致（**TC-A2-2**、**TC-A2-3**、**TC-T7-2**）。

3. **Given** 存在待办 id `N`  
   **When** 执行 `todo update <N> <title>` 并可选带 `--description`、`--due-date`、`--priority`、`--tags`、`--repeat-rule`、`--repeat-until`、`--repeat-count`、`--clear-repeat-rule`  
   **Then** 成功路径更新内存并 `save_todos`；非法可选字段与 `add` 相同校验路径（`patch_from_add_args`），失败时退出码 **2** 且**不写盘**；成功时 `--json` 成功载荷与实现一致（**TC-T8-1**、**TC-T9-1**、**TC-T13-5** 相关）。

4. **`--dry-run update`**：不调用持久化写盘；与 `docs/test-cases.md` **TC-A4-1** 一致；若 id 不存在（与 `complete` dry-run 对齐）应返回数据错误（**3**）而非静默成功。

5. **回归**：不破坏 Story 1.1～1.4（`add` 校验、`list`/`search` JSON、`complete`/`delete` 错误码与集成测试）。

## Tasks / Subtasks

- [x] **棕地核对**：`handle_show` / `handle_update`（`xtask/src/todo/cmd/dispatch.rs`）；`TodoShowArgs` / `TodoUpdateArgs`（`xtask/src/todo/args.rs`）；`patch_from_add_args`（`xtask/src/todo/cmd/parse.rs`）；领域 `TodoList::update`（`crates/todo`）；现有 `crud.rs`（`cmd_todo_show_found_and_not_found`、`cmd_todo_update_and_update_id_zero_errors`）、`list_options.rs`（`cmd_todo_show_with_all_fields`、`cmd_todo_update_clear_repeat_rule_clears_repeat_rule`、dry-run update）、`json_dry_init.rs`（show json）。将 AC 与 `docs/test-cases.md`（**TC-T7-***、**TC-T8-1**、**TC-A2-***、**TC-A4-1**）映射到断言。
- [x] **端到端缺口（若缺）**：在 `xtask/tests/integration.rs` 增补 `cargo xtask todo` 对 `show` / `update` 的 `--json` 成功与典型失败（2/3），以及「非法 update 可选字段不脏写 `.todo.json`」类断言（对齐 Story 1.1 / 1.4 模式）。
- [x] **init-ai**：核对 `xtask/src/todo/init_ai.rs` 中 `show` / `update` 行与 `args.rs` 一致；必要时更新一行说明。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`。

## Change Log

- 2026-03-26：`handle_update` 在 `--dry-run` 分支增加与 `complete` 一致的「id 不存在 → 退出码 3」检查；新增 `xtask/tests/show_update/mod.rs` 集成测试并 `mod show_update` 挂到 `integration` 测试入口。
- 2026-03-26：BMad（BMM）code review 通过；故事标为 `done`。

## Dev Notes

### 棕地现状（避免重复造轮）

- **CLI**：`handle_show` 已输出人类可读字段与 `todo_to_json`；`handle_update` 在 dry-run 之后、`patch_from_add_args` → `list.update` → `save_todos`。
- **领域**：`TodoList::update` 与 `TodoPatch`（`crates/todo`）；可选字段校验复用与 `add` 相同的解析逻辑。
- **已有测试**：见 `xtask/src/tests/todo/todo_cmd/crud.rs`、`list_options.rs`、`json_dry_init.rs`；`crates/todo/src/tests/advanced.rs` 等（test-cases 索引）。

### 实现与契约要点

- **退出码**：`TodoCliError::Parameter` → **2**，`Data` → **3**（`error.rs`）；与 Story 1.4 一致。
- **update 标题**：`TodoUpdateArgs` 含 `title` positional；`patch.title` 在 `patch_from_add_args` 之后设置。
- **`--clear-repeat-rule`**：`patch.repeat_rule_clear` 与 `TodoList::update` 行为以现有 tests 为准。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| show / update | `xtask/src/todo/cmd/dispatch.rs` |
| 参数 | `xtask/src/todo/args.rs` |
| 解析 | `xtask/src/todo/cmd/parse.rs` — `patch_from_add_args` |
| JSON | `xtask/src/todo/error.rs` — `todo_to_json` |
| 集成测试 | `xtask/tests/integration.rs` |

### 架构合规（摘录）

- **Crate 边界**：领域在 **`xtask-todo-lib`**；CLI 编排与 I/O 在 **`xtask`**（`architecture.md`）。
- **不**在库内绑定 git；load/save 显式（同文档）。

### 测试与追溯

- `docs/test-cases.md`：**TC-T7-1**、**TC-T7-2**、**TC-T8-1**、**TC-A2-2**、**TC-A2-3**、**TC-A4-1**、**TC-T13-5**（show/update 与 repeat）。

### 前序故事经验（1.4）

- **`todo --json <subcommand>`** 全局 flag 顺序；`print_json_error` 与 `exit_code` 对齐。
- 失败路径 **.todo.json** 字节级不变类测试（1.4 集成模式）。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.5]
- [Source: `docs/requirements.md` §3.1、§3.2]
- [Source: `docs/test-cases.md` — US-T7、US-T8、TC-A2、TC-A4、TC-T13]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Todo 领域、CLI 边界]

## Dev Agent Record

### Agent Model Used

Cursor Agent（GPT-5.1）

### Debug Log References

（无）

### Completion Notes List

- 棕地核对：`handle_show` / `handle_update` 与 `crud` / `list_options` 单测已覆盖主要路径；补集成测试覆盖真实 `xtask` 子进程。
- **行为修正**：`handle_update` 的 `--dry-run` 在 id 不存在时返回 `TodoCliError::Data`（退出码 3），与 `handle_complete` / `handle_delete` 的 dry-run 及 AC4 一致。
- 新增 `xtask/tests/show_update/mod.rs`：`xtask_todo_json_show_success`、`xtask_todo_json_show_id_zero_exit_2`、`xtask_todo_json_show_nonexistent_exit_3`、`xtask_todo_json_update_success`、`xtask_todo_json_update_invalid_optional_preserves_store`、`xtask_todo_json_update_dry_run_nonexistent_exit_3`。
- `init-ai` 中 `show` / `update` 行已与 `args.rs` 一致，未改。
- 已运行 `cargo test -p xtask-todo-lib`、`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings` 均通过。

### File List

- `xtask/src/todo/cmd/dispatch.rs`
- `xtask/tests/show_update/mod.rs`
- `xtask/tests/integration.rs`
- `_bmad-output/implementation-artifacts/1-5-detail-update-optional-fields.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化）
- [x] 测试与 `docs/test-cases.md` ID 可追溯
