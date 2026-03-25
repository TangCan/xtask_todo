# Story 1.1：创建待办与校验失败不脏写

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名在终端管理任务的开发者，  
我希望用 CLI 添加待办并在校验失败时不产生无效或部分写入，  
以便自动化脚本与个人工作流都不会破坏 `.todo.json`。

## 映射需求

- **FR1**（`docs/requirements.md` §3.1、`_bmad-output/planning-artifacts/epics.md`）
- **NFR-S1**：数据默认在工作区本地；不破坏 `.todo.json` 完整性（`epics.md` Story 1.1）

## Acceptance Criteria

1. **Given** 工作区已有或可无 `.todo.json`  
   **When** 执行带合法参数的 `cargo xtask todo add <title>`（或等价：`cargo run -p xtask -- todo add …` / 构建产物 `todo add …`）  
   **Then** 新待办出现在后续 `list` 中，且 `.todo.json` 经序列化后符合领域模型与 schema 预期（id 为正整数、标题非空等）。

2. **Given** 标题在领域层校验失败（trim 后为空）  
   **When** 执行 `add`  
   **Then** 进程**非 0** 退出（约定 **退出码 2**，参数/用法类），**且** `.todo.json` 内容与执行前一致（无新记录、无截断写）。

3. **Given** 标题合法但可选字段非法（例如 `--due-date` 非 `YYYY-MM-DD`、`--priority` 非允许枚举、`--repeat_rule` 无法解析等）  
   **When** 执行 `add`  
   **Then** 进程非 0 退出（约定 **退出码 2**），**且** 不向磁盘写入已部分构造的待办（与 TC-T1-3、可编程契约 `requirements.md` §6 / US-A2 一致）。

4. **`--dry-run`**：在修改类 `add` 上，开启 `--dry-run` 时**不**调用持久化写盘路径；行为与 `docs/test-cases.md` TC-A4-1 及 `requirements.md` §3.2 一致。

5. **`--json`**：失败时 stdout 为统一错误 JSON 结构（与 `xtask/src/todo/error.rs` 约定一致），成功时成功载荷可解析。

## Tasks / Subtasks

- [x] 对照 **当前实现** 核对 Story 1.1 行为是否已满足；若已满足，补全/收紧**回归测试**与文档交叉引用（AC 全覆盖）。
  - [x] 领域层：`xtask-todo-lib` — `TodoList::create` 空标题 → `TodoError::InvalidInput`（见 `crates/todo/src/list/mod.rs`）。
  - [x] CLI 层：`xtask/src/todo/cmd/dispatch.rs` — `handle_add`：失败路径须在 `save_todos` 之前返回；`dry_run` 早退；可选字段经 `patch_from_add_args`（`cmd/parse.rs`）校验。
  - [x] 确认「先 `create` 再 `patch_from_add_args` / `update`」时，若可选字段校验失败，**不得** `save_todos`（当前逻辑应无写盘；若需消除歧义，可考虑**先校验可选字段再 `create`** 作为实现优化，非功能性前提）。
- [x] 集成测试：`xtask/src/tests/todo/todo_cmd/json_dry_init.rs` — 新增 `cmd_todo_add_invalid_optional_leaves_existing_file_unchanged`（TC-T1-3 / 不脏写）；空标题与非法可选字段已有 `crud` / `json_dry_init` 覆盖。
- [x] Devshell 子集：`todo_builtin.rs` 的 `todo add` 仅为标题拼接，无 `--due-date` 等可选 flag；空标题返回 `TodoArgError` 且不 `save_todos`，与既有 `devshell/tests/run_todo.rs` 一致，无需改代码。

## Change Log

- 2026-03-25：`handle_add` 在 `create` 之前解析/校验可选字段（`patch_from_add_args`）；新增集成测试断言非法 `due_date` 时 `.todo.json` 字节级不变。

## Dev Notes

### 技术要点与防呆

- **退出码**：`xtask/src/todo/error.rs` — `EXIT_PARAMETER = 2`，`EXIT_DATA = 3`，`EXIT_GENERAL = 1`。创建/校验类输入错误应走 **Parameter（2）**，与 `requirements.md` §3.1「非法标题 → 退出码 2」一致。
- **持久化边界**：权威文件为工作区 **`.todo.json`**；load/save 在 `xtask/src/todo/io.rs`。领域核心为 `TodoList` + `InMemoryStore`（`architecture.md` 数据架构决策）。
- **CLI 解析**：argh（`TodoArgs` / `TodoAddArgs` 在 `xtask/src/todo/args.rs`）。

### 须触摸的源码位置（棕地）

| 区域 | 路径 |
|------|------|
| 子命令分发与 add 处理 | `xtask/src/todo/cmd/dispatch.rs`（`handle_add`） |
| 可选字段解析与日期格式 | `xtask/src/todo/cmd/parse.rs`（`patch_from_add_args`、`is_yyyy_mm_dd`） |
| JSON / 退出码 | `xtask/src/todo/error.rs` |
| 文件 DTO 与 load/save | `xtask/src/todo/io.rs` |
| 领域创建与校验 | `crates/todo/src/list/mod.rs` |
| 独立 `todo` 二进制入口 | `xtask/src/bin/todo.rs` → `run_standalone` |

### 架构合规（摘录）

- **Crate 边界**：待办领域在 **`xtask-todo-lib`**；**`xtask`** 负责 CLI、`.todo.json` I/O（`architecture.md` Core Architectural Decisions）。
- **不**在库内绑定 git；**不**隐式全局持久化 — load/save 在调用链上显式（同文档）。

### 测试要求

- `cargo test -p xtask-todo-lib`（领域单测，如 `crates/todo/src/tests/crud.rs`、`list/tests.rs`）。
- `cargo test -p xtask` — 关注 `xtask/tests/todo/`、`xtask/src/todo/mod.rs` 内联测试。
- 验收报告对照：`docs/acceptance.md` / `docs/test-cases.md` — **§2 Todo（T1-1～）** 与 **TC-T1-1、TC-T1-3**。

### 前序故事

- 无（Epic 1 首故事）。

### Git / 近期提交提示

- 近期提交多为 devshell-vm / 文档；本故事以 **todo CLI + lib** 为主，与 VM 无直接依赖。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 Story 1.1]
- [Source: `docs/requirements.md` §3.1、§3.2、`§6` 退出码]
- [Source: `docs/test-cases.md` — US-T1 / TC-T1-1、TC-T1-3、TC-A4-1]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Data Architecture、Crate 边界]

## Dev Agent Record

### Agent Model Used

Cursor Agent（GPT-5.1）

### Debug Log References

（无）

### Completion Notes List

- `handle_add`：在 `dry_run` 早退之后、`list.create` 之前执行 `patch_from_add_args`，使非法可选字段不会先占用新 id，并与「失败不写盘」语义一致。
- 新增 `cmd_todo_add_invalid_optional_leaves_existing_file_unchanged`：先成功 `add seed`，再对非法 `due_date` 的 `add` 断言失败且 `.todo.json` 与失败前完全一致。
- 已运行 `cargo test -p xtask`、`cargo test -p xtask-todo-lib`、`cargo clippy -p xtask --all-targets` 均通过。

### File List

- `xtask/src/todo/cmd/dispatch.rs`
- `xtask/src/tests/todo/todo_cmd/json_dry_init.rs`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-1-create-todo-validation-no-dirty-write.md`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 测试与 `docs/test-cases.md` ID 可追溯（TC-T1-3、既有 TC-T1-1/TC-A4-1 相关用例）

### Review Findings

- [x] [Review][Defer] `handle_add` 在 `--dry-run` 时仍早于 `patch_from_add_args` 返回，非法可选参数不会在 dry-run 路径被校验 [`xtask/src/todo/cmd/dispatch.rs:76`] — deferred, pre-existing（AC4 仅要求不写盘；是否与「非 dry-run 一致校验」需产品/文档拍板）
