# Story 4.5：Mode P 与会话元数据

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望在**有效条件**下使用 **guest 为主工作区（Mode P）**，并把**会话元数据**落在**工作区约定路径**，  
以便与 **guest 工程视图**一致且可复盘。

## 映射需求

- **FR24**：在有效条件下可使用 **guest 为主工作区**，工程树操作与 **`rustup`/`cargo`** 针对同一视图（以 Mode P 规则与降级为准）。
- **FR25**：会话相关元数据持久化到**工作区内约定路径**（以 **`requirements §1.1`** 为准）。

## Acceptance Criteria

1. **Given** **`DEVSHELL_VM_WORKSPACE_MODE=guest`** 且 **`VmConfig::workspace_mode_effective()`** 解析为 **`WorkspaceMode::Guest`**（即 **VM 开启**、**`use_host_sandbox()`** 为假、后端 **`lima`** 或 **`beta`**，见 **`vm/config/mod.rs`**）  
   **When** 建立 **`SessionHolder`**（γ 或 β）且 **`syncs_vfs_with_host_workspace()`** 为假  
   **Then** **`SessionHolder::is_guest_primary()`** 为 **`true`**；REPL/脚本路径上文件类 builtin 经 **`GuestFsOps`**（与 **`docs/devshell-vm-gamma.md`**、guest-primary 设计一致）（**FR24**）。

2. **Given** **`DEVSHELL_VM=off`**、**`DEVSHELL_VM_BACKEND=host`/`auto`**（空视为宿主沙箱）、或 **请求 `guest` 但 VM/后端组合不满足**（见 **`workspace_mode_effective`** 的 stderr 提示）  
   **When** 会话实际为 **Mode S**（**`is_guest_primary()`** 为假）  
   **Then** **不**把 guest-primary 会话 JSON 当作**唯一**真源语义误用；**legacy `.dev_shell.bin`** 路径仍按 **`serialization`** / **`repl`** 棕地规则；与 **`requirements §1.1`** 降级表一致（**FR24**）。

3. **Given** **Mode P** 且进程退出或 **`save_on_exit`**（**`repl/mod.rs`**）  
   **When** 保存 guest-primary 元数据  
   **Then** 写入 **`devshell_session_v1`** JSON（**`logical_cwd`**、**`saved_at_unix_ms`**），**不**写 legacy **`.dev_shell.bin`**（与 **`session_store` 模块注释**一致）（**FR25**）。

4. **Given** **`session_store::session_metadata_path`** 解析顺序（**`DEVSHELL_WORKSPACE_ROOT/.cargo-devshell/session.json`** → **`cwd` 下 `.cargo-devshell/session.json`** → **`{bin_stem}.session.json`**）  
   **When** **`load_guest_primary`** / **`save_guest_primary`**  
   **Then** 读写路径与 **`docs/requirements.md` §1.1**「持久化（共同约定）」及 **`session_store.rs` 顶部说明**一致；**`apply_guest_primary_startup`** 能恢复 **`logical_cwd`** 或优雅 **`Ok(())`** 无文件时（**FR25**）。

5. **棕地**：**`export_devshell_workspace_root_env`**（γ）等须在 REPL 前设置 **`DEVSHELL_WORKSPACE_ROOT`** 时，会话 JSON 落在**与挂载对齐的宿主路径**；本故事以**核对 AC、补集成测试或文档缺口**为主，**避免**重复实现已存在于 **`session_store.rs`** / **`repl/mod.rs`** 的核心逻辑。

6. **回归**：**`cargo test -p xtask-todo-lib session_store`**、**`cargo test -p xtask-todo-lib devshell`**（相关用例）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **真值表**：**`DEVSHELL_VM*`** × **`DEVSHELL_VM_WORKSPACE_MODE`** → **`WorkspaceMode`** × **`SessionHolder` 变体** × **`is_guest_primary()`**；与 **`docs/devshell-vm-gamma.md`** 表格对照。
- [x] **会话路径**：在 **有/无 `DEVSHELL_WORKSPACE_ROOT`**、**有/无 `cwd`** 下各跑一次 **`load_guest_primary`/`save_guest_primary`** 预期（可沿用/扩展 **`session_store` 单元测试**）。
- [x] **退出路径**：核对 **`repl::save_on_exit`**（**`repl/mod.rs`**）在 **guest-primary** 与 **非 guest-primary** 下是否分别写 JSON vs legacy bin；缺测试则补。
- [x] **文档**：若实现与 **`docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md` §10** 有细微差异，在 **`requirements.md`** 或模块注释中**一句**对齐，避免双源矛盾。
- [x] **验证**：上述 **`cargo test`** / **`clippy`** 命令。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 有效 Mode P | **`vm/config/mod.rs`** — **`VmConfig::workspace_mode_effective`**、**`WorkspaceMode`** |
| 会话判定 | **`vm/mod.rs`** — **`SessionHolder::is_guest_primary`**、**`guest_primary_fs_ops_mut`** |
| JSON 与路径 | **`crates/todo/src/devshell/session_store.rs`** — **`GuestPrimarySessionV1`**、**`FORMAT_DEVSHELL_SESSION_V1`**、**`session_metadata_path`**、**`apply_guest_primary_startup`** |
| 启动/退出 | **`devshell/mod.rs`**、**`repl/mod.rs`** — **`apply_guest_primary_startup`**、**`save_on_exit`** |

### 架构合规（摘录）

- **降级不报错**：**`guest` 请求与 VM 关闭冲突时**仍 **`WorkspaceMode::Sync`**（见 **`workspace_mode_effective`**），与 Epic **可预测降级**一致。
- **Todo 路径**：**`.todo.json`** 仍在**宿主 cwd**（**`todo_io`** / 设计 **§11**），勿在 Mode P 故事中改为 guest 内路径。

### 前序故事

- **4-2 / 4-3**：γ / β 会话与 **`GuestFsOps`**。
- **4-4**：VM 关/宿主降级；本故事在 **Mode P 有效**与 **无效**边界上与之衔接。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 Story 4.5]
- [Source: `docs/requirements.md` — §1.1]
- [Source: `docs/devshell-vm-gamma.md` — `DEVSHELL_VM_WORKSPACE_MODE`]
- [Source: `docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md`]
- [Source: `crates/todo/src/devshell/session_store.rs`]
- [Source: `crates/todo/src/devshell/vm/config/mod.rs`]
- [Source: `crates/todo/src/devshell/repl/mod.rs` — `save_on_exit`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib run_readline_guest_primary_eof_writes_session_json_not_legacy_bin`
- `cargo test -p xtask-todo-lib session_store`
- `cargo test -p xtask-todo-lib devshell`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`

### Completion Notes List

- 已核对 Mode P 有效条件：`VmConfig::workspace_mode_effective` 仅在 VM 开启且 backend 为 `lima`/`beta` 时返回 `Guest`，并通过 `SessionHolder::is_guest_primary()` 体现；无效组合会降级到 `Sync`（含 stderr 提示），满足 FR24 边界。
- 已核对会话路径优先级并复用现有测试矩阵：`session_store::session_metadata_path` 顺序为 `DEVSHELL_WORKSPACE_ROOT/.cargo-devshell/session.json` → `cwd/.cargo-devshell/session.json` → `{bin_stem}.session.json`，与 `requirements §1.1` 一致。
- 新增测试 `run_readline_guest_primary_eof_writes_session_json_not_legacy_bin`：在 guest-primary 会话下通过 EOF 触发 `repl::save_on_exit`，断言写入 `.cargo-devshell/session.json` 且不写 `.dev_shell.bin`，直接覆盖 AC3。
- 已确认非 guest-primary 分支继续沿用 legacy `.dev_shell.bin` 保存（`run_io::run_with_eof_triggers_save_on_exit` 等既有用例保持通过），未引入语义回归。
- 文档对齐检查：`session_store.rs` 模块注释与 `docs/requirements.md §1.1` 的会话 JSON 约定一致，本次无需额外文档改动。

### File List

- `crates/todo/src/devshell/repl/mod.rs`
- `crates/todo/src/devshell/repl/tests.rs`
- `xtask/src/coverage.rs`
- `_bmad-output/implementation-artifacts/4-5-mode-p-session-metadata.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | **`repl` 拆为 `mod.rs` + `tests.rs`**，抽出 **`process_line` / `StepResult`** 便于单测；**`save_on_exit`** 在 **guest-primary** 下跳过 legacy **`.dev_shell.bin`** 并调用 **`session_store::save_guest_primary`**，与 **AC3/FR25** 一致。 |
| **Edge Case Hunter** | **`run_readline_guest_primary_eof_*`** 依赖 **`SessionHolder::try_from_config`（beta + guest）**；审查补充 **`#[cfg(feature = "beta-vm")]`**，避免 **`default-features = false`** 时测试与 **β** 后端不可用冲突（默认 feature 仍运行该测）。 |
| **Acceptance Auditor** | **AC3** 由 EOF 非 TTY **`run`** 路径 + **`.cargo-devshell/session.json`** + **`devshell_session_v1`** 断言覆盖；**xtask** **`coverage.rs`** 将排除路径改为 **`repl/*`** 与目录结构一致。**AC6**：**`cargo test -p xtask-todo-lib`**（276 passed / 1 ignored，默认 features）、**`session_store::`**（4 passed）、**`--no-default-features`**（273 passed）、**`clippy -D warnings`** 通过。 |

**已处理**

- [x] [Review][Patch] **guest-primary REPL 集成测** 增加 **`#[cfg(feature = "beta-vm")]`**（`repl/tests.rs`）。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — 故事 **4-5** 与 **Epic 4** 标为 **done**；sprint **`last_updated`** **`2026-03-26T04:31:00Z`**；修正文档内 **`repl.rs`** → **`repl/mod.rs`** 引用；复验 **`cargo test` / clippy**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
