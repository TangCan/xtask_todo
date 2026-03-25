# Story 1.7：导入与导出

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望按约定格式导出并自文件导入，且可选择替换策略，  
以便迁移与备份。

## 映射需求

- **FR7**（`epics.md` Story 1.7；`docs/requirements.md` §3.2 `export` / `import`）
- **Epic 5**：成功/失败 JSON、`exit_code`；`--dry-run` 对 **import**（及修改类路径）不写 **`.todo.json`**（与既有契约一致）

## Acceptance Criteria

1. **Given** 当前工作区有待办列表  
   **When** 执行 `todo export <file>`（可选 `--format json|csv`，或按扩展名推断）  
   **Then** 文件写入与 `save_todos_to_path_with_format` / `io.rs` 一致；`--json` 成功载荷含导出计数与文件路径（**TC-T12-1**）。

2. **Given** 存在有效 **JSON 或 CSV** 导入文件  
   **When** 执行 `todo import <file>`（默认**合并**：为导入条目分配新 id 并并入当前列表）  
   **Then** 合并后 `.todo.json` 反映新条目；`--json` 成功载荷含 `merged`、`count`（**TC-T12-2**）。

3. **Given** 同上  
   **When** 执行 `todo import <file> --replace`  
   **Then** 当前列表**完全替换**为文件内容（仅文件中的任务）；`--json` 成功载荷含 `replaced`、`count`（**TC-T12-3**）。

4. **`import --dry-run`**：不写入 **`.todo.json`**；`--json` 仍返回成功预览结构（与 `handle_import` 实现一致；**TC-A4-1** 类）。

5. **错误路径**：文件不存在、格式非法等 → **退出码 1**（`TodoCliError::General`）或约定的一般错误；**不**破坏 `.todo.json` 完整性（与 Story 1.1 棕地一致）。

6. **回归**：不破坏 Story 1.1～1.6（`todo` 子命令、`search`/`stats` 集成测等）。

## Tasks / Subtasks

- [x] **棕地核对**：`handle_export` / `handle_import`（`xtask/src/todo/cmd/dispatch.rs`）；`TodoExportArgs` / `TodoImportArgs`（`xtask/src/todo/args.rs`）；`save_todos_to_path_with_format`、`load_todos_from_path`（`xtask/src/todo/io.rs`）；现有 `xtask/src/tests/todo/todo_cmd_io.rs`（`cmd_todo_export_and_import_merge_replace`、CSV 等）。将 AC 与 `docs/test-cases.md`（**TC-T12-1～3**、**TC-DATE-1** 若适用）映射到断言。
- [x] **端到端缺口（若缺）**：在 `xtask/tests/` 下为 `cargo xtask todo --json export` / `import` 增补集成测试（临时目录、临时文件），覆盖成功 JSON、失败时文件不破坏（可选）。
- [x] **init-ai**：核对 `xtask/src/todo/init_ai.rs` 中 `export` / `import` 与 `args.rs` 一致。
- [x] **验证命令**：`cargo test -p xtask-todo-lib`、`cargo test -p xtask --test integration`、`cargo clippy -p xtask --all-targets -- -D warnings`。

### Review Findings

- [x] [Review][Patch] `sprint-status.yaml` 文件头 `# last_updated` 与 `last_updated` 字段不一致 — 已统一为 `2026-03-26T23:45:00Z`（BMad code review）。
- [x] [Review][Patch] `crates/todo/.dev_shell.bin` 不应随本故事提交 — 已从索引/工作区恢复（勿纳入 commit）。

## Change Log

- 2026-03-26：`load_todos_for_import`（`io.rs`）供 `import` 专用——路径必须存在；非 CSV 时 JSON 须语法合法。`handle_import` 改用该函数。`xtask/tests/todo_list/import_export.rs` 增补 TC-T12-1～3、TC-A4-1、AC5（缺失/非法 JSON）。`sprint-status` 更新为 review。
- 2026-03-26：code review 修正 sprint 注释与 `.dev_shell.bin` 误改。
- 2026-03-27：code review 通过；故事从 `review` 标为 `done`。

## Dev Notes

### 棕地现状（避免重复造轮）

- **CLI**：`handle_export` / `handle_import` 已实现；`import` 支持 `--replace` 与 `dry_run` 分支。
- **I/O**：`io.rs` 含 JSON/CSV 序列化与加载路径。
- **已有测试**：`todo_cmd_io.rs`（合并/替换/CSV 等）。

### 实现与契约要点

- **格式推断**：扩展名 `.csv` → CSV；否则 JSON（与 `handle_export` 一致）。
- **合并**：`list.add_todo` 逐条加入；id 由领域分配（见库实现）。
- **替换**：`InMemoryStore::from_todos` + `save_todos` 写回默认 `.todo.json`。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| export / import | `xtask/src/todo/cmd/dispatch.rs` |
| 参数 | `xtask/src/todo/args.rs` |
| I/O | `xtask/src/todo/io.rs` |
| 单元/集成测 | `xtask/src/tests/todo/todo_cmd_io.rs`、`xtask/tests/` |

### 架构合规（摘录）

- **Crate 边界**：领域在 **`xtask-todo-lib`**；CLI 与文件 I/O 在 **`xtask`**（`architecture.md`）。

### 测试与追溯

- `docs/test-cases.md`：**TC-T12-1**、**TC-T12-2**、**TC-T12-3**。

### 前序故事（1.6）

- 集成测试按模块划分；新增 `tests` 子模块时在 `integration.rs` 增加 `mod …;`。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.7]
- [Source: `docs/requirements.md` §3.2]
- [Source: `docs/test-cases.md` — US-T12]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Todo 与 I/O]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

### Completion Notes List

- `import` 现使用 `load_todos_for_import`：与「浏览」用的 `load_todos_from_path`（缺文件返回空）区分，满足 AC5 缺失文件 / JSON 语法错误 → `General` / 退出码 1。
- `--dry-run` 与全局 `--json` 的 CLI 顺序为 `todo --json --dry-run import <file>`（见 `cargo xtask todo --help`）。
- `cargo test -p xtask --test integration` 全量通过；`cargo clippy -p xtask --all-targets -- -D warnings` 通过。

### Implementation Plan

1. 核对 `handle_export` / `handle_import` JSON 字段与 AC。
2. 实现 `load_todos_for_import` 并接入 `handle_import`。
3. 集成测试 `todo_list/import_export.rs` 覆盖 TC-T12、dry-run、错误路径。

### File List

- `xtask/src/todo/io.rs`
- `xtask/src/todo/cmd/dispatch.rs`
- `xtask/tests/todo_list/import_export.rs`
- `xtask/tests/todo_list/mod.rs`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-7-import-export.md`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 测试与 `docs/test-cases.md` ID 可追溯
