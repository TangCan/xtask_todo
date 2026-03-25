# Story 1.8：重复任务

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望使用重复规则、下一实例与 `--no-next` 等能力，  
以便管理周期性工作。

## 映射需求

- **FR8**（`epics.md` Story 1.8；`docs/requirements.md` §3.1「重复规则」、§3.2、`RepeatRule`）
- **Epic 5**：`complete` 带 `--no-next` 的 JSON/退出码；非法 `repeat_count` 等 → 退出码 **2**（与 `todo_error` 约定一致）

## Acceptance Criteria

1. **Given** 待办含有效 `RepeatRule`（如 `daily`、`weekly`、`2d`、`custom:N` 等，以领域 `FromStr` 为准）  
   **When** 执行 `complete <id>`（默认生成下一实例）  
   **Then** 行为与 `TodoList::complete` / 存储一致：完成当前条、按规则插入下一待办（若适用）（**TC-T13-1**、**TC-T13-3**、**TC-T13-4**）。

2. **Given** 同上  
   **When** 执行 `complete <id> --no-next`  
   **Then** **不**创建下一实例（**TC-T13-2**）。

3. **Given** `add` / `update` 带 `--repeat-rule` / `--repeat-until` / `--repeat-count`  
   **When** 参数合法  
   **Then** 持久化与领域模型一致；`--clear-repeat-rule` 清除规则（**TC-T13-5**、**TC-T8-2**）。

4. **Given** 非法 `repeat_count` 等 CLI 输入  
   **When** 执行 `add` / `update`  
   **Then** 退出码 **2**（**TC-T13-6**、**TC-DATE-1** 相关）。

5. **端到端**：`cargo xtask todo add …` 带重复选项后 `list` 可见预期字段（已有 **`xtask_todo_add_with_repeat_options_then_list`**，**TC-T13-7**）；若缺「`complete` 后下一实例」的 **xtask 集成**断言，则在本故事补全。

6. **回归**：不破坏 Story 1.1～1.7（含 **import/export** 与既有集成模块）。

## Tasks / Subtasks

- [x] **棕地核对**：`RepeatRule` / `TodoList::complete`（`crates/todo`）；`handle_complete`（`xtask/src/todo/cmd/dispatch.rs`）；`patch_from_add_args` 与 repeat 相关 flag（`parse.rs`）；`crates/todo/src/tests/priority_repeat.rs`、`repeat.rs`、`advanced.rs`；`xtask` 内 `todo_cmd`、`todo_error`、`integration.rs`。将 AC 与 `docs/test-cases.md`（**TC-T13-1～7**）映射到断言。
- [x] **端到端缺口（若缺）**：为「完成重复任务后生成下一实例」与「`--no-next`」补充 `xtask/tests/` 集成测试（`--json` 可选），与领域单测不重复即可。
- [x] **init-ai**：核对 `init_ai.rs` 中 `complete`、`add`/`update` 重复相关行与 `args.rs` 一致。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask --test integration`、`cargo clippy -p xtask --all-targets -- -D warnings`。

### Review Findings

- [x] [Review][Patch] `sprint-status.yaml` 文件头 `# last_updated`（`01:00`）与 `last_updated` 字段（`01:35`）不一致 — 已统一为 `2026-03-27T01:35:00Z`（BMad code review）。

## Change Log

- 2026-03-27：补充 `xtask` 端到端重复完成用例（默认生成下一实例 + `--no-next` 不生成）；同步故事与 sprint 状态到 `review`。
- 2026-03-27：code review 修正 `sprint-status.yaml` 注释与 `last_updated` 一致。
- 2026-03-27：审查通过，故事标 `done`；`sprint-status` 中 `1-8-recurring-tasks` 与 `epic-1` 同步为 `done`。

## Dev Notes

### 棕地现状（避免重复造轮）

- **领域**：重复规则解析、完成时下一实例逻辑集中在 **`xtask-todo-lib`**（见 `priority_repeat.rs`、`list/mod.rs` 等）。
- **CLI**：`TodoCompleteArgs::no_next`；`add`/`update` 的 repeat 选项已在 `args.rs`。
- **已有测试**：库内覆盖较全；`xtask/tests/integration.rs` 已有带 repeat 的 `add` 用例。

### 实现与契约要点

- **下一实例**：由领域计算下一 `due_date` / id；勿在 CLI 层重复实现规则。
- **`--no-next`**：仅影响 `complete` 路径，不改变已完成条目的存储语义。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| complete / repeat | `xtask/src/todo/cmd/dispatch.rs` — `handle_complete` |
| 领域 | `crates/todo/src/list/mod.rs`、`model`、repeat 相关模块 |
| 测试 | `crates/todo/src/tests/priority_repeat.rs`、`xtask/tests/` |

### 架构合规（摘录）

- **Crate 边界**：重复业务规则在库；CLI 调用 `TodoList` API（`architecture.md`）。

### 测试与追溯

- `docs/test-cases.md`：**TC-T13-1**～**TC-T13-7**。

### 前序故事（1.7）

- 导入导出与默认 `.todo.json` I/O 已稳定；本故事勿引入隐式迁移或第二数据源。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.8]
- [Source: `docs/requirements.md` §3.1、§3.2]
- [Source: `docs/test-cases.md` — US-T13]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Todo 领域]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

### Completion Notes List

- 复用领域层 `TodoList::complete` 规则，CLI 不重复实现下一实例逻辑。
- 已新增集成测试覆盖 `complete` 生成下一实例（含 `due_date` 递推）及 `complete --no-next`。
- 既有 `add/update` 的重复参数合法/非法路径测试与 `init-ai` 文案一致性保持不变。
- 验证命令通过：`cargo test -p xtask-todo-lib`、`cargo test -p xtask --test integration`、`cargo clippy -p xtask --all-targets -- -D warnings`。

### File List

- `xtask/tests/complete_delete/mod.rs`
- `_bmad-output/implementation-artifacts/1-8-recurring-tasks.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 测试与 `docs/test-cases.md` ID 可追溯
