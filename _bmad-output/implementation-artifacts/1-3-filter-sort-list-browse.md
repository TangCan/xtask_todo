# Story 1.3：过滤、排序与列表浏览

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望按状态、日期、标签等维度过滤与排序，  
以便在任务变多时仍能快速定位。

## 映射需求

- **FR3**（`epics.md` Story 1.3；`docs/requirements.md` §3.1「列表：支持过滤、排序」、§3.2 `list` 行）
- **Epic 5**：`list` 的 `--json` 成功载荷与终端输出语义一致（过滤后**无匹配**时仍为空列表语义，与 Story 1.2 的 `todo_list_json_payload` 一致）

## Acceptance Criteria

1. **Given** 工作区存在多条待办，且条目在状态、优先级、标签、`due_date` 上可区分  
   **When** 使用文档已列的 `todo list` 参数：`--status`、`--priority`、`--tags`（逗号分隔、任一词命中）、`--due-before` / `--due-after`（`YYYY-MM-DD`）、`--sort`（`created-at` | `due-date`/`due_date` | `priority` | `title`）  
   **Then** 输出顺序与过滤结果与 `docs/requirements.md`、`docs/test-cases.md` 及既有领域实现一致（**TC-T9-2**、**TC-T2-2** 等）。

2. **Given** 非法参数（例如不支持的 `--status`、非 `YYYY-MM-DD` 的日期过滤）  
   **When** 执行 `list`  
   **Then** 进程非 0 退出，约定 **退出码 2**（参数类），且**不写** `.todo.json`（**TC-T9-3**、**TC-DATE-2**）。

3. **`--json`**：在过滤/排序生效时，成功响应仍为统一 `{"status":"success","data":…}`；`data` 使用 `todo_list_json_payload`（与 Story 1.2 一致）：有结果时 `items` 为过滤后顺序；**无匹配**时 `empty: true`、`message` 与 `EMPTY_LIST_MESSAGE` 一致、`items: []`。

4. **回归**：不破坏 Story 1.1 / 1.2 已建立行为（`add` 校验顺序、空列表人类可读与 JSON、`todo --json list` 全局 flag 顺序）。

## Tasks / Subtasks

- [x] **棕地核对**：确认本故事大部分行为已在代码库中实现 — 领域层 `TodoList::list_with_options`（`crates/todo/src/list/mod.rs`）、CLI 解析 `list_options_from_args`（`xtask/src/todo/cmd/parse.rs`）、分发 `handle_list`（`xtask/src/todo/cmd/dispatch.rs`）。将 epics AC 与 `docs/test-cases.md`（**US-T9**、**TC-T9-1～3**、**TC-DATE-2**）逐条对照到测试文件与断言。
- [x] **测试覆盖缺口**：在 `xtask/src/tests/todo/todo_cmd/list_options.rs` 与 `crates/todo/src/tests/list_options.rs` 已有覆盖基础上，若缺少 **端到端** `cargo xtask todo --json list` + 过滤/排序 的 JSON 形状与顺序断言，则补集成或 `cmd_todo` 测试（注意全局 `--json` 在子命令前：`todo --json list …`）。
- [x] **init-ai / 文档**：核对 `xtask/src/todo/init_ai.rs` 中 `list` 行是否完整列出过滤与排序 flag；若 `docs/requirements.md` §3.2 与实现不一致，以代码与 `test-cases.md` 为准做最小修订或故事内记录偏差处理。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`（及预提交等价命令若适用）。

### Review Findings

- [x] [Review][Patch] 非法 `list` 参数路径应不写入 `.todo.json`（AC2）— 已在 `xtask_todo_list_invalid_status_exit_code_2` 增加无存储时 `!exists`；新增 `xtask_todo_list_invalid_status_preserves_existing_store` 校验已有 `.todo.json` 字节级不变 — [`xtask/tests/integration.rs`](../../xtask/tests/integration.rs)
- [x] [Review][Patch] TC-DATE-2 端到端 — 已新增 `xtask_todo_list_invalid_due_before_exit_code_2`、`xtask_todo_list_invalid_due_after_exit_code_2`（退出码 2、`error` JSON、无盘写入） — [`xtask/tests/integration.rs`](../../xtask/tests/integration.rs)
- [x] [Review][Defer] AC1 中其它 flag（如 `--priority`、`--due-before`/`--due-after`、多键排序）的 E2E 覆盖仍主要依赖下层单测；本 diff 仅补齐部分集成断言 — deferred, pre-existing [`xtask/tests/integration.rs`](../../xtask/tests/integration.rs)

## Change Log

- 2026-03-25：补充集成测试 `xtask_todo_list_json_sort_due_date_orders_items`、`xtask_todo_list_json_filter_no_match_empty_payload`、`xtask_todo_list_invalid_status_exit_code_2`；棕地核对与 init-ai 无代码变更需求。
- 2026-03-26：代码审查后补测：`invalid_*` 路径不创建/不篡改 `.todo.json`；TC-DATE-2 集成覆盖非法 `--due-before` / `--due-after`。

## Dev Notes

### 棕地现状（避免重复造轮）

- **领域**：`ListOptions` / `ListFilter` / `ListSort` 定义于 `crates/todo/src/model.rs`；过滤与排序逻辑在 `TodoList::list_with_options`（`crates/todo/src/list/mod.rs`）。
- **CLI**：`TodoListArgs`（`xtask/src/todo/args.rs`）已含 `--status`、`--priority`、`--tags`、`--due-before`、`--due-after`、`--sort`；`list_options_from_args` 将字符串解析为领域类型并校验日期格式与 status 枚举词。
- **已有测试**：`crates/todo/src/tests/list_options.rs`、`crates/todo/src/list/tests.rs`、`xtask/src/tests/todo/todo_cmd/list_options.rs`（含 `cmd_todo_list_with_filters_and_sort`、非法 status/due 等）。

### 实现与契约要点

- **日期过滤**：`due_before` / `due_after` 对**无** `due_date` 的条目过滤结果为排除（领域实现已体现）；与测试一致即可，勿改语义除非需求变更。
- **排序**：`due-date` 与 `due_date` 在 CLI 解析中均映射到 `ListSort::DueDate`（`parse.rs`）；无 due 日期的条目在按 due 排序时的相对顺序以当前 `list_with_options` 为准，变更需同步单测。
- **JSON**：`handle_list` 已用 `todo_list_json_payload(&items)`；过滤后空集应与 Story 1.2 空列表 JSON 形状一致。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| list 分发 | `xtask/src/todo/cmd/dispatch.rs` — `handle_list` |
| CLI 解析 | `xtask/src/todo/cmd/parse.rs` — `list_options_from_args` |
| 领域列表 | `crates/todo/src/list/mod.rs` — `list_with_options` |
| JSON 载荷 | `xtask/src/todo/error.rs` — `todo_list_json_payload` |
| 集成 / 命令测试 | `xtask/tests/integration.rs`、`xtask/src/tests/todo/todo_cmd/list_options.rs` |

### 架构合规（摘录）

- **Crate 边界**：过滤/排序规则在 **`xtask-todo-lib`**；CLI 仅解析与调用 `TodoList` API（`architecture.md` — Data Architecture、FR1–FR8 映射）。
- **CLI 稳定命名**：不随意重命名对外 flag（`architecture.md` — API Naming Conventions）。

### 测试与追溯

- `docs/test-cases.md`：**TC-T9-2**（list 过滤排序）、**TC-T9-3**（非法 status 等）、**TC-DATE-2**（非法 `--due-before` / `--due-after`）、**TC-T2-2**（多条顺序，与 `list_options` 相关）。

### 前序故事经验（1.2）

- 全局 **`--json`**：`todo --json list`，非 `todo list --json`（argh 结构）。
- 空列表 JSON：`empty`、`message`、`items` 约定见 `error.rs` / Story 1.2。

### Git 近期上下文

- 近期提交含 todo 校验与 BMad 工件；本故事以 **list 过滤/排序验证与缺口测试** 为主。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.3]
- [Source: `docs/requirements.md` §3.1、§3.2]
- [Source: `docs/test-cases.md` — US-T9、TC-T9-*、TC-DATE-2]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Todo 领域、ListOptions、CLI 边界]

## Dev Agent Record

### Agent Model Used

Cursor Agent（GPT-5.1）

### Debug Log References

（无）

### Completion Notes List

- 棕地核对：`list_with_options`、`list_options_from_args`、`handle_list` 已满足 AC；未改领域/CLI 核心逻辑。
- 新增集成测试：`xtask_todo_list_json_sort_due_date_orders_items`（`--json list --sort due-date` 与 `due_date` 顺序）、`xtask_todo_list_json_filter_no_match_empty_payload`（过滤无匹配时 `empty`/`message`/`items`）、`xtask_todo_list_invalid_status_exit_code_2`（TC-T9-3 端到端退出码 2 + 错误 JSON）。
- 审查后补测：`xtask_todo_list_invalid_due_before_exit_code_2`、`xtask_todo_list_invalid_due_after_exit_code_2`（TC-DATE-2）；`xtask_todo_list_invalid_status_preserves_existing_store`；非法 `list` 空目录路径断言不创建 `.todo.json`。
- `init-ai` 中 `list` 行已含全部过滤与排序 flag；`docs/requirements.md` §3.2 与实现一致，无需改文档。
- 已运行 `cargo test -p xtask-todo-lib`、`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings` 均通过。

### File List

- `xtask/tests/integration.rs`
- `_bmad-output/implementation-artifacts/1-3-filter-sort-list-browse.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化）
- [x] 测试与 `docs/test-cases.md` ID 可追溯（TC-T9-2、TC-T9-3、集成补充）
