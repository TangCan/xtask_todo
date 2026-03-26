# Story 3.5：帮助与 TTY 补全

Status: done

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

- [x] **棕地核对**：**`help`** 输出 vs **`builtin_impl`** 分派表 vs **`completion/candidates`** 命令名列表 — 三处一致。
- [x] **TTY**：在 **`repl.rs`** 确认 **`CompletionType::List`**；阅读 **`completion/helper.rs`** 与 **`DevShellHelper::update`** 路径逻辑。
- [x] **文档**：若修文案，同步 **`docs/requirements.md` §5.6**（若需）、**`crates/todo/README.md`**。
- [x] **验证**：`cargo test -p xtask-todo-lib completion`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

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

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib completion`
- `cargo test -p xtask-todo-lib run_with_help_lists_builtin_command_set`
- `cargo test -p xtask-todo-lib devshell`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`

### Completion Notes List

- 已核对并统一 `help`/分派/补全命令集合：`builtin_impl::run_builtin_help`、`run_builtin_core`、`completion::BUILTIN_COMMANDS` 三处命令集一致；`export_readonly` 作为 `export-readonly` 别名已在 help 文案中显式体现。
- 新增测试 `run_with_help_lists_builtin_command_set`，校验 help 输出包含当前实现内置命令集合（含 `export_readonly` 别名、`exit, quit`）。
- 新增测试 `complete_commands_contains_builtin_and_aliases`，校验补全命令清单覆盖内置命令与别名。
- TTY/非 TTY 行为核对：`repl.rs` 的 TTY 分支使用 `Editor` + `set_completion_type(CompletionType::List)` + `DevShellHelper`；非 TTY 走 `read_line` 分支，不依赖 Tab 补全，符合 AC2/AC3。
- 文档核对：`requirements` §5.6 与当前实现一致；本次仅修正 help 文案与别名一致性，不需要额外文档文件改动。

### File List

- `crates/todo/src/devshell/command/dispatch/builtin_impl.rs`
- `crates/todo/src/devshell/tests/run_basic.rs`
- `crates/todo/src/devshell/completion/tests.rs`
- `_bmad-output/implementation-artifacts/3-5-help-tty-completion.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | diff 聚焦 help 文案别名、`help` 与补全列表回归测；与「内置命令 discoverability」目标一致，无多余行为变更。 |
| **Edge Case Hunter** | `run_with_help_lists_builtin_command_set` 对 **`exit, quit`** 使用整串子串匹配，与当前 `run_builtin_help` 输出一致；若将来调整措辞顺序需同步测试。`complete_commands_contains_builtin_and_aliases` 覆盖别名与核心内置名，不断言「仅含」列表（与 AC 范围一致）。 |
| **Acceptance Auditor** | AC1：`export-readonly|export_readonly` 与分派、`BUILTIN_COMMANDS` 一致；新增测覆盖 help 与补全。AC2/3：本次未改 `repl.rs`，故事内棕地核对可接受。AC4：无 README 矛盾改动。AC6：`cargo test -p xtask-todo-lib`、`clippy -D warnings` 已通过。 |

**待办项**：无（0 decision-needed、0 patch、0 defer）。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — 本故事与 sprint **3-5-help-tty-completion** 标为 **done**；复验 `cargo test -p xtask-todo-lib`（267 passed）、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
