# Story 1.6：搜索与统计摘要

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望按关键词搜索并查看统计摘要，  
以便快速评估工作量与范围。

## 映射需求

- **FR6**（`epics.md` Story 1.6；`docs/requirements.md` §3.1「搜索 / 统计」、§3.2 `search` / `stats`）
- **Epic 5**：`search` 无匹配时的 `--json` 载荷与 `list` 空结果一致（`todo_list_json_payload`，见 Story 1.2）；`stats` 的 `--json` 成功体含 `total` / `incomplete` / `complete`

## Acceptance Criteria

1. **Given** 多条待办，标题/描述/标签中含可区分文本  
   **When** 执行 `todo search <keyword>`（或 `cargo xtask todo search <keyword>`）  
   **Then** 命中集合与领域 `TodoList::search` 一致（标题、描述、标签子串、大小写不敏感；`keyword` trim 后小写匹配）（**TC-T10-1**，与 `crates/todo/src/tests/advanced.rs` 对齐）。

2. **Given** 无命中或空库  
   **When** 执行 `search`  
   **Then** 人类可读为 `EMPTY_LIST_MESSAGE`（`No tasks.`）；`todo --json search …` 使用 `todo_list_json_payload`，`empty: true` 与 `message` 与 Story 1.2 一致（**TC-T10-2**）。

3. **Given** 任意非空待办集合  
   **When** 执行 `todo stats`  
   **Then** 终端行含 total / incomplete / complete 计数且与 `TodoList::stats` 一致；`todo --json stats` 的 `data` 为 `{ "total", "incomplete", "complete" }`（**TC-T11-1**）。

4. **边界（领域已有行为）**：`keyword` 经 trim 后**为空字符串**时，`search` 返回**全部**待办（与 `crates/todo/src/list/mod.rs` 实现一致）；若产品要求「空关键词报错」，需单开变更，本故事以**文档化 + 测试锁定当前行为**为准。

5. **回归**：不破坏 Story 1.1～1.5（`add`/`list`/`show`/`update`、集成测试模块 `todo_list` / `show_update` / `complete_delete`）。

## Tasks / Subtasks

- [x] **棕地核对**：`handle_search` / `handle_stats`（`xtask/src/todo/cmd/dispatch.rs`）；`TodoSearchArgs`（`xtask/src/todo/args.rs`）；`TodoList::search` / `stats`（`crates/todo/src/list/mod.rs`）；`crates/todo/src/tests/advanced.rs`、`xtask/src/tests/todo/todo_cmd_io.rs`（`cmd_todo_search_and_stats`）等。将 AC 与 `docs/test-cases.md`（**TC-T10-1**、**TC-T10-2**、**TC-T11-1**）映射到断言。
- [x] **端到端缺口（若缺）**：在 `xtask/tests/` 下为 `cargo xtask todo --json search …`、`--json stats` 增补集成测试（注意全局 `--json`：`todo --json search <kw>`），覆盖至少一次命中、零命中 JSON 形状、stats 数值与人工可算预期一致。
- [x] **init-ai**：核对 `xtask/src/todo/init_ai.rs` 中 `search` / `stats` 与 `args.rs` 一致。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask --test integration`、`cargo clippy -p xtask --all-targets -- -D warnings`。（注：`cargo test -p xtask --lib` 在本仓库存在与 Story 无关的既有失败：`run_subcommand_coverage`、`cmd_lima_todo_print_only_no_build_smoke`。）

### Review Findings

- [x] [Review][Patch] `sprint-status.yaml`：`last_updated` 与文件头注释不一致且相对前次回退 — 已统一为 `2026-03-26T21:00:00Z`（BMad code review）。

## Change Log

- 2026-03-25：补充 `search_is_case_insensitive`（领域）；`xtask/tests/todo_list/mod.rs` 增补 TC-T10-1/T10-2/T11-1 集成与空关键词 `--json` 用例；`sprint-status` 更新为 review。
- 2026-03-26：code review 修正 `sprint-status.yaml` 时间戳一致性。

## Dev Notes

### 棕地现状（避免重复造轮）

- **领域**：`TodoList::search` / `stats`（`crates/todo/src/list/mod.rs`）。
- **CLI**：`handle_search` 使用 `todo_list_json_payload`；`handle_stats` 使用内联 `serde_json::json!` 三字段。
- **已有测试**：`advanced.rs`（search/stats）；`xtask` 内 `todo_cmd_io.rs` 等。

### 实现与契约要点

- **`search`**：与 `list` 共用 `print_todo_list_items` / `todo_list_json_payload`（空结果语义统一）。
- **`stats`**：JSON 字段名 **`incomplete`**（不是 `pending`），与现有实现一致。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| search / stats | `xtask/src/todo/cmd/dispatch.rs` |
| 参数 | `xtask/src/todo/args.rs` — `TodoSearchArgs` |
| 领域 | `crates/todo/src/list/mod.rs` |
| JSON | `xtask/src/todo/error.rs` — `todo_list_json_payload`（search） |
| 集成测试 | `xtask/tests/`（`todo_list` / 新模块，与现有 `mod` 组织一致） |

### 架构合规（摘录）

- **Crate 边界**：搜索/统计规则在 **`xtask-todo-lib`**；CLI 仅调用 `TodoList` API（`architecture.md`）。

### 测试与追溯

- `docs/test-cases.md`：**TC-T10-1**、**TC-T10-2**、**TC-T11-1**。

### 前序故事（1.5）

- 集成测试按功能分模块：`common`、`complete_delete`、`show_update`、`todo_list`；新测勿破坏 `cargo test -p xtask --test integration` 总入口。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.6]
- [Source: `docs/requirements.md` §3.1、§3.2]
- [Source: `docs/test-cases.md` — US-T10、US-T11]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Todo 领域]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

### Completion Notes List

- 棕地实现已满足 AC；未改 `dispatch`/`init_ai` 行为，仅增加领域与集成测试锁定 TC-T10/T11 与空关键词行为。
- `cargo test -p xtask --test integration` 与 `cargo test -p xtask-todo-lib` 通过；`cargo clippy -p xtask --all-targets -- -D warnings` 通过。

### Implementation Plan

1. 核对 `handle_search` / `handle_stats` 与 `todo_list_json_payload`、`stats` JSON 三字段。
2. 领域：`search_is_case_insensitive`；保留既有 `search("   ")` 全量行为。
3. 集成：`xtask todo --json search` / `stats` 与空库、无命中、命中、空关键词、`stats` 计数。

### File List

- `crates/todo/src/tests/advanced.rs`
- `xtask/tests/todo_list/mod.rs`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-6-search-stats-summary.md`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 测试与 `docs/test-cases.md` ID 可追溯
