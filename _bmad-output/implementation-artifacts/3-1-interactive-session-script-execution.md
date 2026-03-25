# Story 3.1：交互式会话与脚本执行

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望启动交互 devshell 或用 **`.dsh` 脚本**非交互执行，  
以便复现与自动化 devshell 工作流。

## 映射需求

- **FR14**（`epics.md` Story 3.1；`docs/requirements.md` **§5.3** — REPL 与 **`cargo-devshell [-e] -f script.dsh`**）
- **NFR-I1**：用法错误、脚本缺失、会话/VM 初始化失败时**可诊断**（stderr 可读、退出码非 0）

## Acceptance Criteria

1. **Given** 已通过 **`cargo install xtask-todo-lib`** 或 **`cargo run -p xtask-todo-lib --bin cargo-devshell`** 获得 **`cargo devshell`**（包名与安装路径以 **`crates/todo/README.md`** 为准）  
   **When** 在无 **` -f `** 参数时启动（**交互 REPL**）  
   **Then** 会话可进入 **`rustyline`** REPL、读取行并执行，直至 **`exit` / `quit`** 或 EOF；行为与 **`docs/requirements.md` §5.3–§5.4** 及 **`docs/design.md`** 中 REPL 约定一致（**FR14**）。

2. **Given** 存在合法 **`.dsh`** 脚本路径  
   **When** 执行 **`cargo devshell -f script.dsh`**（可选 **` -e `** 使脚本初始 **`set -e`**，见 **§5.3**）  
   **Then** 从宿主读取脚本、按 **`docs/requirements.md` §5.5** 语言规则执行，非 0 退出时进程退出码反映失败（**FR14**）。

3. **Given** 错误用法（例如 **`-f`** 与位置参数数量不符、缺脚本路径）或脚本/资源加载失败  
   **When** 调用入口 **`xtask_todo_lib::devshell::run_main` / `run_main_from_args`**  
   **Then** 不向 stderr 静默成功；错误信息与 **`crates/todo/src/devshell/mod.rs`** 既有用法字符串一致或经本故事**刻意**改进并已更新测试（**NFR-I1**）。

4. **Given** **Mode S / Mode P** 与 VM 相关环境变量（**`DEVSHELL_VM`**、**`DEVSHELL_VM_BACKEND`** 等，见 **§5.2**、**`docs/devshell-vm-gamma.md`**）  
   **When** 启动 REPL 或脚本会话  
   **Then** 会话生命周期（会话存储、VM 会话创建/降级）与 **`requirements` / `design.md`** 一致；若仅部分环境可测，在 Completion Notes 标明**已测矩阵**与 **SKIP** 条件（**FR14**）。

5. **棕地**：实现根在 **`crates/todo/src/devshell/`**（**`mod.rs`**、**`repl.rs`**、**`script`**、**`vm`** 等）；本故事以 **核对 AC、补集成测试/文档、边界修正**为主，**不**无依据重写解析器或 VFS 核心语义。

6. **回归**：**`cargo test -p xtask-todo-lib`**（含 **`devshell`** 测试）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **棕地核对**：阅读 **`crates/todo/src/devshell/mod.rs`**（**`run_main_from_args`**：REPL vs **`-f`** 分支）、**`repl.rs`**；对照 **`docs/requirements.md` §5.3–§5.5** 做检查表。
- [x] **测试**：沿用 **`crates/todo/src/devshell/tests/run_main.rs`**；为缺口（REPL  happy path mock、**`-e`**、错误用法）补测或记**手工步骤**。
- [x] **文档**：若 CLI 用法与 **`requirements`** 有出入，同步 **`crates/todo/README.md`** 或 **`requirements` §5.3**。
- [x] **验证**：`cargo test -p xtask-todo-lib devshell`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| 入口 | **`crates/todo/src/devshell/mod.rs`** — **`run_main`** / **`run_main_from_args`** |
| 二进制 | **`crates/todo/src/bin/cargo_devshell/main.rs`** → **`devshell::run_main()`** |
| REPL / 脚本 | **`repl.rs`**、**`script`** 模块 |
| 既有单测 | **`crates/todo/src/devshell/tests/run_main.rs`** |

### 架构合规（摘录）

- **Devshell FR14–FR19** 归属 **`crates/todo/src/devshell/`**（见 **`architecture.md`**）；**不**将 REPL 核心迁入 **`xtask`** crate。

### 前序故事

- Epic 2 中 **xtask** 故事与 **devshell** 正交；数据对齐见后续 **3-4**（**`todo_io`**）。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.1]
- [Source: `docs/requirements.md` — §5.1–§5.5]
- [Source: `docs/design.md` — devshell / 会话相关章节]
- [Source: `crates/todo/src/devshell/mod.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib devshell::tests::run_main`
- `cargo test -p xtask-todo-lib devshell`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`
- `cargo test -p xtask-todo-lib`

### Completion Notes List

- 已核对 `run_main_from_args` 在无 `-f` 时进入 REPL 分支、在 `-f` 时进入脚本分支；实现与 `docs/requirements.md` §5.3–§5.5 一致。
- 已核对并执行 `run_main` 相关测试覆盖：`-f` 缺参、`-f` 多余位置参数、脚本文件缺失诊断、无 `-f` 的 REPL happy path 执行。
- 错误用法与脚本加载失败均写入 `stderr` 且返回 `Err`（非 0 退出路径），满足 NFR-I1 的可诊断性要求。
- 已测矩阵：本地自动化覆盖 REPL/脚本入口、`-e` 行为、参数错误和脚本缺失；SKIP：TTY 下 `rustyline` 真实交互、Mode P/VM（`DEVSHELL_VM*`）端到端环境依赖验证，本次未在专用 VM 环境执行。
- 文档核对结果：当前 CLI 启动说明与 `requirements` §5.3 未发现需修改项，故无文档变更。

### File List

- `_bmad-output/implementation-artifacts/3-1-interactive-session-script-execution.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
