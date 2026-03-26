# Story 3.4：Devshell 内 Todo 子集

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望在 devshell 中使用**待办子集**且读写与 **`.todo.json`** 约定一致，  
以便与 **`cargo xtask todo`** 数据对齐。

## 映射需求

- **FR17**（`epics.md` Story 3.4；`docs/requirements.md` **§5.4** — **`todo …`** 子集，无 **`export` / `import` / `init-ai`**）
- **数据一致性**：**`todo_io`** 与 **xtask** 侧共用 **`TodoDto`** / **`InMemoryStore`** 契约（见 **`crates/todo/src/devshell/todo_io.rs`** 模块文档）

## Acceptance Criteria

1. **Given** **`todo_io::todo_file()`** 解析为**当前工作目录**下 **`.todo.json`**（与 **`docs/requirements.md` §3.2** 默认数据文件一致）  
   **When** 在 devshell 执行**已支持**的 **`todo`** 子命令（以 **`todo_builtin.rs`** 中 **`match sub`** 为准：`list`、`add`、`show`、`update`、`complete`、`delete`、`search`、`stats`）  
   **Then** 读写的 JSON 与 **`cargo xtask todo`** 使用同一 **`TodoDto`** 序列化语义；**不**引入第二套文件格式（**FR17**）。

2. **Given** **`docs/requirements.md` §5.4** 明确 **devshell 不提供** **`export` / `import` / `init-ai`**  
   **When** 用户尝试等价能力（若通过未知子命令或其它入口）  
   **Then** 行为为**明确拒绝**或**未实现**提示，**不**静默写错文件（**FR17**）。

3. **Given** **Mode P / guest-primary**（**`todo_io`** 注释：**路径留在宿主当前目录**，不映射进 guest 工程树）  
   **When** 在 VM 相关会话中执行 **`todo`**  
   **Then** 与 **`design.md`** / **`todo_io`** 文档化策略一致；若实现与注释不符，本故事**修正实现或更新设计说明**并补测（**FR17**）。

4. **Given** 修改类子命令（**`add` / `update` / `complete` / `delete`**）  
   **When** 成功执行  
   **Then** 调用 **`save_todos`** 持久化；失败路径返回 **`BuiltinError::TodoSaveFailed`** 等可读错误（**FR17**）。

5. **棕地**：实现位于 **`crates/todo/src/devshell/command/todo_builtin.rs`**、**`todo_io.rs`**；**`builtin_impl`** 将 **`todo`** 分派至 **`run_todo_cmd`**。本故事以 **核对 AC、与 xtask `todo` 行为矩阵对齐、补测试/文档**为主，**不**在 devshell 内一次性实现 **xtask** 全量 **`todo`** 旗标（除非 **`requirements`** 明确扩展）。

6. **回归**：**`cargo test -p xtask-todo-lib`**（含 **`todo`** / **`devshell`**）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **棕地核对**：列出 **`todo_builtin`** 支持的子命令与 **`xtask/src/todo/`** 对应命令的差异表（**`--json`**、**`--dry-run`**、过滤/排序等**未**在 devshell 暴露时记入「刻意子集」）。
- [x] **路径与 Mode P**：阅读 **`todo_io`**、**`design.md`** 中与 **§11** / **guest-primary** 相关段落；核对 **`cwd`** 在 devshell/VM 下是否仍指向预期宿主目录。
- [x] **测试**：为 **`run_todo_cmd`** 关键路径或 **`todo_io`** 集成补 **单元/集成** 测试（可沿用 **`devshell/tests/`** 模式）。
- [x] **验证**：`cargo test -p xtask-todo-lib`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| 内置 `todo` | **`crates/todo/src/devshell/command/todo_builtin.rs`** — **`run_todo_cmd`** |
| 持久化 | **`crates/todo/src/devshell/todo_io.rs`** — **`load_todos` / `save_todos` / `todo_file`** |
| 分派 | **`dispatch/builtin_impl.rs`** 中 **`todo`** 分支 |

### 架构合规（摘录）

- **领域** 在 **`xtask_todo_lib`**；devshell **不**重复实现 **`TodoList`** 业务规则，仅**薄封装** + **I/O**。

### 前序故事

- **3-2**（文件操作）：VFS 与宿主路径；本故事聚焦 **`.todo.json`** 与 **todo** 子集。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.4]
- [Source: `docs/requirements.md` — §3.2、§5.4]
- [Source: `docs/design.md` — Todo / devshell / 持久化（§4.1）]
- [Source: `crates/todo/src/devshell/todo_io.rs`]
- [Source: `crates/todo/src/devshell/command/todo_builtin.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib devshell::tests::run_todo`
- `cargo test -p xtask-todo-lib todo_file_points_to_current_dir_dot_todo_json`
- `cargo test -p xtask-todo-lib`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`

### Completion Notes List

- 子命令差异核对：devshell `todo` 仅支持 `list/add/show/update/complete/delete/search/stats`；明确不支持 `export/import/init-ai`，与 `requirements` §5.4 的“刻意子集”一致。
- 新增测试 `run_with_todo_subset_rejects_unsupported_subcommands`，覆盖 `todo export/import/init-ai` 的明确拒绝（未知子命令提示），防止误写盘。
- 新增测试 `run_with_todo_add_persists_dot_todo_json_in_current_dir`，确认修改类命令会写入当前目录 `.todo.json`，与 `todo_io::todo_file()` 契约一致。
- 新增测试 `todo_file_points_to_current_dir_dot_todo_json`，验证 `todo_io` 路径确实绑定宿主 `cwd`（与 Mode P 注释策略一致：不映射进 guest 工程树）。
- 现有实现已在 `add/update/complete/delete` 成功后调用 `save_todos`，失败路径映射到 `BuiltinError::TodoSaveFailed`；本次未发现需改动的实现缺陷。
- 文档核对：`requirements` 与实现一致，无需修改文档文件。

### File List

- `crates/todo/src/devshell/tests/run_todo.rs`
- `crates/todo/src/devshell/todo_io.rs`
- `_bmad-output/implementation-artifacts/3-4-devshell-todo-subset.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-25）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | 用户路径清晰：devshell 内 `todo add` → 当前目录 `.todo.json`；`export`/`import`/`init-ai` 被拒且 stderr 含「未知子命令」与允许列表，符合 FR17 子集与 §5.4。 |
| **Edge Case Hunter** | `run_with_todo_add_persists_*` 在 `cwd_mutex` 下 chdir，失败时依赖测试框架/进程退出清理临时目录；与现有 devshell 测试风格一致。未知子命令测试对三条命令共用同一 stderr 断言，若将来单条消息格式分叉需拆分断言。 |
| **Acceptance Auditor** | AC1：`todo_file` + `add` 持久化测到；AC2：三条禁子命令测到；AC3：`todo_file_points_to_current_dir_dot_todo_json` 与 Mode P 注释一致；AC4：`save_todos`/`TodoSaveFailed` 棕地已存在，本故事以补测为主；AC5/6：clippy 与本包测试通过（见下方变更记录）。 |

**状态**：PO 已确认 **done**（**2026-03-25**）；**Epic 3** 仍为 **in-progress**（**3-5** 未完结）。

## Change Log

- **2026-03-25**：PO 签收 — 故事与 sprint 中 **3-4-devshell-todo-subset** 标为 **done**；**epic-3** 保持 **in-progress**。
- **2026-03-25**：代码审查 — 对齐 `sprint-status.yaml` 顶部 `# last_updated` 与 `last_updated` 字段；验证 `cargo test -p xtask-todo-lib` 过滤运行 `subset_rejects_unsupported` / `persists_dot_todo_json` / `todo_file_points` 及 `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings` 通过。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
