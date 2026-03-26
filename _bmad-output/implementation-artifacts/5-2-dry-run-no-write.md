# Story 5.2：Dry-run 无写盘

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望在**修改类命令**上使用 **`--dry-run`** 时**不写入** **`.todo.json`** 等约定持久化文件，  
以便安全预览（**US-A4**）。

## 映射需求

- **FR27**：**`--dry-run`** 下修改类命令**不**持久化到约定数据文件（与 **`requirements §3.2`**、**§6 US-A4** 一致）。
- **UX-DR3**：与 **5.1** 的 **`--json`** 可组合；成功预览时 **`data`** 含 **`would_*`** / **`merged`** / **`replaced`** 等字段（见 **`dispatch.rs`**）。

## Acceptance Criteria

1. **Given** **`handle_add` / `handle_update` / `handle_complete` / `handle_delete`**（**`xtask/src/todo/cmd/dispatch.rs`**）且 **`dry_run == true`**  
   **When** 命令成功返回（含 **`--json`** 预览）  
   **Then** **不**调用 **`save_todos`**；磁盘上的 **`.todo.json`**（由 **`load_todos`/`save_todos`** 约定路径决定）**不变**（**FR27**）。

2. **Given** **`handle_import`** 且 **`--dry-run`**  
   **When** **`replace`** 为真或假（merge）  
   **Then** **`save_todos`** 仅在 **`!dry_run`** 时调用（见 **`dispatch.rs`** 中 **`if !dry_run`**）；合并/替换仅作用于**内存中** **`TodoList`**，**不**落盘（与 **`xtask/tests/todo_list/import_export.rs`** **`xtask_todo_import_dry_run_merge_preserves_store_tc_a4`** 一致）（**FR27**）。

3. **Given** **`--dry-run`** 与 **数据错误**（例如 **`update`/`complete`/`delete`** 指向不存在 id）  
   **When** 执行  
   **Then** 仍**不**写盘；退出码与 **`TodoCliError::Data`** 一致（见 **`show_update/mod.rs`** 等）（**FR27**）。

4. **Given** **非修改类**子命令（**`list`/`show`/`search`/`stats`/`export`**）  
   **When** 用户传入全局 **`--dry-run`**（若 CLI 解析允许）  
   **Then** 行为以实现为准：**`export`** 当前**无** **`dry_run`** 参数，**会**写入**导出目标文件**；本故事**不**要求为 **`export`** 增加 dry-run，但须在 **Dev Notes** 中**一句**说明，避免与「全局不写盘」误解混淆。

5. **棕地**：**`xtask/src/tests/todo/todo_cmd/json_dry_init.rs`**、**`list_options.rs`** 等已覆盖 **`dry_run` add** 与多条 **`dry_run`** 路径；本故事以 **核对 AC、补边界测试（若缺）** 为主。

6. **回归**：**`cargo test -p xtask`**（**`todo`** 与 **`import_export`** 相关）、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **矩阵**：**`add`/`update`/`complete`/`delete`/`import`** × **`--dry-run`** × **有/无 `--json`** — 确认**均无** **`save_todos`**（**`import`** 两处 **`if !dry_run`**）。
- [x] **export**：确认 **`handle_export`** 无 **`dry_run`**；文档或 **`--help`** 是否需要注明「导出始终写目标文件」（**最小**变更）。
- [x] **集成**：若缺少 **`replace` + `--dry-run`** 的磁盘不变性测试，按 **`import_export`** 模式补一条。
- [x] **验证**：**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 全局 flag | **`TodoArgs::dry_run`**（**`xtask/src/todo/args.rs`**） |
| 实现 | **`dispatch.rs`** — 各 **`handle_*`** 早退 **`return Ok(())`** 或 **`if !dry_run { save_todos(...) }`** |
| 测试 | **`json_dry_init.rs`**、**`list_options.rs`**、**`import_export.rs`** |

### 架构合规（摘录）

- **`load_todos`** 在 **`cmd_todo`** 入口已执行；**`dry_run`** 路径须在**任何** **`save_todos`** 之前返回（**`add`** 在验证/分配 id 前即返回 — 注意 **`add` dry_run** **不**调用 **`list.create`**，故**不**消耗 id）。

### 前序故事

- **5.1**：**`--json`** 与 **`--dry-run`** 组合的成功 JSON 形状。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 Story 5.2]
- [Source: `docs/requirements.md` — §3.2、`--dry-run`；§6 **US-A4**]
- [Source: `xtask/src/todo/cmd/dispatch.rs`]
- [Source: `xtask/tests/todo_list/import_export.rs` — dry-run import]
- [Source: `xtask/src/tests/todo/todo_cmd/json_dry_init.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask xtask_todo_import_dry_run_replace_preserves_store`
- `cargo test -p xtask && cargo clippy -p xtask --all-targets -- -D warnings`

### Completion Notes List

- 已逐项核对 `dispatch.rs`：`add/update/complete/delete` 在 `dry_run` 分支均于 `save_todos` 前早退；`import` 在 `replace`/`merge` 两分支均仅在 `if !dry_run` 时落盘，满足 FR27。
- 复核现有单测覆盖矩阵：`json_dry_init.rs` 覆盖 `dry_run add`；`list_options.rs` 覆盖 `dry_run update/complete/delete` 且含 `json` 与非 `json` 路径；`import_export.rs` 已覆盖 `dry_run merge`。
- 新增集成测试 `xtask_todo_import_dry_run_replace_preserves_store`，验证 `--dry-run --replace` 仅返回预览 JSON，不改写 `.todo.json`，并确认后续 `list` 仍为原本数据。
- 在 `docs/requirements.md` §3.2 追加一句说明：`export` 当前仍会写目标文件，不受 `--dry-run` 影响，避免“全局 dry-run 一律不写盘”的歧义。

### File List

- `xtask/tests/todo_list/import_export.rs`
- `docs/requirements.md`
- `_bmad-output/implementation-artifacts/5-2-dry-run-no-write.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
