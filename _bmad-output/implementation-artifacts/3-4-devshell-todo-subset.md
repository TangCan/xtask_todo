# Story 3.4：Devshell 内 Todo 子集

Status: ready-for-dev

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

- [ ] **棕地核对**：列出 **`todo_builtin`** 支持的子命令与 **`xtask/src/todo/`** 对应命令的差异表（**`--json`**、**`--dry-run`**、过滤/排序等**未**在 devshell 暴露时记入「刻意子集」）。
- [ ] **路径与 Mode P**：阅读 **`todo_io`**、**`design.md`** 中与 **§11** / **guest-primary** 相关段落；核对 **`cwd`** 在 devshell/VM 下是否仍指向预期宿主目录。
- [ ] **测试**：为 **`run_todo_cmd`** 关键路径或 **`todo_io`** 集成补 **单元/集成** 测试（可沿用 **`devshell/tests/`** 模式）。
- [ ] **验证**：`cargo test -p xtask-todo-lib`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

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

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
