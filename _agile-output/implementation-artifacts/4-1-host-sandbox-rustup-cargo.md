# Story 4.1：宿主沙箱执行 rustup/cargo

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望在**未启用 VM** 时于**宿主沙箱路径**执行 **`rustup`/`cargo`** 并正确回写，  
以便默认路径不依赖 Lima/Podman。

## 映射需求

- **FR20**（`epics.md` Story 4.1；PRD — 未启用 VM 时宿主侧沙箱：**导出—执行—回写**）
- **NFR-R1**：**无 VM / Mode S** 主路径在**不安装** Lima/Podman 时仍可用（CI 与最小依赖）

## Acceptance Criteria

1. **Given** **`SessionHolder`** 处于 **Host**（**`DEVSHELL_VM=off`** 或等效，以 **`vm/mod.rs`** / **`try_session_*`** 解析为准）且 **`rustup`/`cargo`** 在宿主 **`PATH`**  
   **When** 在 devshell 执行 **`rustup …`** 或 **`cargo …`**（经 **`run_rust_tool_builtin`** → **`VmExecutionSession::run_rust_tool`** → **`HostSandboxSession`**）  
   **Then** 走 **`sandbox::run_rust_tool`**：**`export_vfs_to_temp_dir`** → **`run_in_export_dir`** → **`sync_host_dir_to_vfs`** → 清理临时目录；语义与 **`crates/todo/src/devshell/sandbox/mod.rs`** 模块文档一致（**FR20**）。

2. **Given** **`DEVSHELL_RUST_MOUNT_NAMESPACE`**（Linux 可选，见 **`sandbox/run.rs`**）  
   **When** 开启时  
   **Then** 子进程在 **`run_in_export_dir`** 前应用私有 mount namespace（**不**引入 Podman/Docker）；非 Linux **忽略**该变量（**FR20**，**NFR-R1**）。

3. **Given** **`rustup` 或 `cargo` 不在 PATH**（**`find_in_path`** 失败）  
   **When** 调用宿主沙箱路径  
   **Then** 映射为 **`BuiltinError::RustupNotFound` / `CargoNotFound`**（见 **`builtin_impl::run_rust_tool_builtin`**），stderr 可诊断（**NFR-R1** 与可预期失败一致）。

4. **Given** 导出或回写失败（**`SandboxError::ExportFailed` / `SyncBackFailed`** 等）  
   **When** 执行工具链命令  
   **Then** **不**表现为 VFS 已更新；错误与 **`BuiltinError::Sandbox*`** 映射一致（**FR20**）。

5. **棕地**：本故事**聚焦宿主沙箱**（**`vm/session_host.rs`** + **`sandbox/run.rs`** + **`sync.rs`** / **`export.rs`**）；**不**在本故事中实现 **γ/β** VM 内执行（属 **4-2～4-3**）。若需改 **`run_rust_tool`** 签名或 **`SessionHolder`** 分派，须评估对 **4-2/4-3** 的回归。

6. **回归**：**`cargo test -p xtask-todo-lib`**（**`sandbox`** / **`vm`** 相关单测）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **棕地核对**：阅读 **`sandbox::run_rust_tool`** 全路径；对照 **`docs/requirements.md` §5.4** 表中 **`rustup`/`cargo`** 行（Mode S）。
- [x] **环境矩阵**：在 **`DEVSHELL_VM=off`**（或文档等价）下手工或集成测试 **`cargo build`** 类命令；记录 **`target/`** 可执行位与 **`restore_execute_bits_for_build_artifacts`** 行为。
- [x] **文档**：若行为与 **`docs/devshell-vm-gamma.md`** / **`sandbox` 模块头** 有出入，同步一处真源。
- [x] **验证**：`cargo test -p xtask-todo-lib sandbox`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| 宿主会话 | **`crates/todo/src/devshell/vm/session_host.rs`** — **`HostSandboxSession::run_rust_tool`** |
| 沙箱执行 | **`crates/todo/src/devshell/sandbox/run.rs`** — **`run_rust_tool`**、**`run_in_export_dir`** |
| 内置入口 | **`command/dispatch/builtin_impl.rs`** — **`run_rust_tool_builtin`** |
| 分派 | **`vm/mod.rs`** — **`SessionHolder::run_rust_tool`**（**`Self::Host`** 分支） |

### 架构合规（摘录）

- **宿主沙箱**不调用 OCI；与 **architecture.md**「VM / rustup FR20–FR25」分层一致。

### 前序故事

- Epic 3 **devshell** 基础（REPL、VFS、内置命令）已建立；本故事收紧 **无 VM** 工具链路径。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 Story 4.1]
- [Source: `docs/requirements.md` — §5.4 `rustup`/`cargo`（Mode S）]
- [Source: `crates/todo/src/devshell/sandbox/mod.rs`]
- [Source: `crates/todo/src/devshell/sandbox/run.rs`]
- [Source: `crates/todo/src/devshell/vm/session_host.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib run_with_rust_tools_missing_in_path_are_diagnostic`
- `cargo test -p xtask-todo-lib sandbox`
- `cargo test -p xtask-todo-lib`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`

### Completion Notes List

- 已核对宿主沙箱主路径：`run_rust_tool_builtin -> SessionHolder::run_rust_tool(Self::Host) -> sandbox::run_rust_tool`，流程为导出、在导出目录执行、回写、清理，与 FR20 一致。
- 补充集成测试 `run_with_rust_tools_missing_in_path_are_diagnostic`，在空 PATH 下验证 `cargo`/`rustup` 诊断输出分别包含 `cargo not found in PATH` 与 `rustup not found in PATH`，对应 `BuiltinError::CargoNotFound/RustupNotFound` 映射。
- 既有 `sandbox` 测试已覆盖关键行为：导出/回写、`find_in_path` 失败、`restore_execute_bits_for_build_artifacts` 修复 `target` 下 ELF 可执行位、Linux `DEVSHELL_RUST_MOUNT_NAMESPACE`（`#[ignore]` 专用环境）。
- `DEVSHELL_VM=off` / `DEVSHELL_VM_BACKEND=host` 路径经 `VmConfig` 与 `SessionHolder::Host` 逻辑保持一致；未引入 γ/β 相关改动（符合故事边界）。
- 文档核对结果：`requirements` §5.4 与 `docs/devshell-vm-gamma.md` 当前描述与实现一致，无需更新文档真源。

### File List

- `crates/todo/src/devshell/tests/run_basic.rs`
- `_bmad-output/implementation-artifacts/4-1-host-sandbox-rustup-cargo.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | 新增测试在全局 `PATH` 上加互斥锁并设无效路径，触发 `cargo`/`rustup` 失败路径；断言 stderr 含 `cargo not found in PATH` / `rustup not found in PATH`，与 **`BuiltinError` Display**（`types.rs`）一致，贴合 **AC3**。 |
| **Edge Case Hunter** | 测试在 `run_with` 成功返回后恢复 `PATH`；若在 `set_var` 与恢复之间 **panic**，环境可能残留 — 可用 `Drop` 守卫收紧（非本故事阻塞）。`path_env_lock` 与 poison 恢复与仓库其他全局 env 测风格一致。 |
| **Acceptance Auditor** | **AC3** 由新集成测覆盖；**AC1/2/4/5** 由既有 `sandbox`/`vm` 路径与故事棕地核对支撑（diff 未改沙箱实现）。**AC6**：`cargo test -p xtask-todo-lib`（268 passed）、`clippy -D warnings` 已通过。 |

**已处理（审查当日）**

- [x] [Review][Patch] **`crates/todo/.dev_shell.bin`** — 与 4.1 无关的本地二进制漂移已 **`git restore`**，避免与功能变更一并提交（与同仓库 1-7 / 1-4 审查先例一致）。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — 故事与 sprint **4-1-host-sandbox-rustup-cargo** 标为 **done**；同步 sprint `last_updated`；复验 `cargo test -p xtask-todo-lib`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`；清理误改的 **`.dev_shell.bin`**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
