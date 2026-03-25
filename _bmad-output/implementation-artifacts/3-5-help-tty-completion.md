# Story 3.5：帮助与 TTY 补全

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望获得与**内置命令一致的帮助**，并在 **TTY** 下获得**命令/路径补全**，  
以便减少查阅成本（**UX-DR1～UX-DR3**）。

## 映射需求

- **FR18**（`epics.md` Story 3.5；PRD — 与内置命令一致的帮助）
- **FR19**（PRD — TTY 下命令与路径补全）
- **UX-DR1**：主路径为终端 CLI；**`--help` / README / `requirements`** 为体验真源（**NF-4** 交叉）
- **UX-DR2**：TTY 补全与产品约定一致（对应 **FR19**）
- **UX-DR3**：成功/错误在人机与 **`--json`**（若适用）下可理解或可机读 — devshell **`todo list --json`** 等以 **`todo_builtin`** 为准

## Acceptance Criteria

1. **Given** 用户在 **REPL** 中执行内置 **`help`**（**`run_builtin_help`**，**`command/dispatch/builtin_impl.rs`**）  
   **When** 与 **`docs/requirements.md` §5.4** 所列命令集合对照  
   **Then** 文案覆盖**当前已实现**内置命令（**`pwd`/`cd`/`ls`/`mkdir`/`cat`/`touch`/`echo`/`save`/`export-readonly`/`todo`/`rustup`/`cargo`/`exit`/`quit`/`help`** 等，以代码为准）；**新增/删除**内置命令时 **help 文本同步更新**（**FR18**，**UX-DR1**）。

2. **Given** **`docs/requirements.md` §5.6**（**rustyline**、**`CompletionType::List`**、路径补全保留目录前缀）  
   **When** **`stdin.is_terminal()`** 为真，**`repl.rs`** 使用 **`Editor`** + **`DevShellHelper`**（**`completion/`**）  
   **Then** **`editor.set_completion_type(CompletionType::List)`** 行为与 §5.6 一致；补全候选与 **`completion_context`** / **`complete_commands`** / **`complete_path`**（或等价实现）一致（**FR19**，**UX-DR2**）。

3. **Given** **非 TTY**（脚本、管道、重定向）  
   **When** 进入 **`read_line`** 分支（见 **`repl.rs`** 注释）  
   **Then** **不**错误依赖 Tab 补全；与 **`requirements`** / **`acceptance.md` D4** 手工/环境说明一致（**FR19**）。

4. **Given** **`crates/todo/README.md`**（或安装说明）描述 **`cargo devshell`** 用法  
   **When** 与 **`run_main_from_args`** 实际用法（**`dev_shell [-e] -f script.dsh`**、**`dev_shell [path]`** 等）对照  
   **Then** 无矛盾；若产品要求二进制级 **`--help`**，则实现或文档化「以 **`help`** 内置命令为准」（**UX-DR1**，**NF-4**）。

5. **棕地**：实现位于 **`repl.rs`**、**`completion/*`**、**`builtin_impl`（help）**；本故事以 **核对 AC、补全/帮助单测、文案同步**为主，**不**更换为**非 rustyline** 的补全后端，除非架构变更。

6. **回归**：**`cargo test -p xtask-todo-lib`**（含 **`completion/tests.rs`**）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：**`help`** 输出 vs **`builtin_impl`** 分派表 vs **`completion/candidates`** 命令名列表 — 三处一致。
- [ ] **TTY**：在 **`repl.rs`** 确认 **`CompletionType::List`**；阅读 **`completion/helper.rs`** 与 **`DevShellHelper::update`** 路径逻辑。
- [ ] **文档**：若修文案，同步 **`docs/requirements.md` §5.6**（若需）、**`crates/todo/README.md`**。
- [ ] **验证**：`cargo test -p xtask-todo-lib completion`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| REPL TTY | **`crates/todo/src/devshell/repl.rs`** — **`Editor::new`**、**`set_completion_type(List)`**、**`DevShellHelper`** |
| 补全 | **`crates/todo/src/devshell/completion/`** — **`helper.rs`**、**`candidates.rs`**、**`context.rs`** |
| 帮助 | **`command/dispatch/builtin_impl.rs`** — **`run_builtin_help`** |

### 架构合规（摘录）

- **Tab 补全**为 **REPL 内**行为；**不**与 `cargo` 全局 `rustup` 补全混用。

### 前序故事

- **3-2**～**3-4**：内置命令与 **`todo`** 子集稳定后，本故事统一 **help/补全** 列表。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.5]
- [Source: `docs/requirements.md` — §5.4、§5.6]
- [Source: `docs/design.md` — §4.3 Tab 补全]
- [Source: `crates/todo/src/devshell/repl.rs`]
- [Source: `crates/todo/src/devshell/completion/`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
