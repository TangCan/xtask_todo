# Story 4.2：Unix γ（Lima）路径

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名在 macOS/Linux 上的开发者，  
我希望在启用 VM 时通过 **γ 后端**在隔离环境执行工具链命令，  
以便与宿主隔离。

## 映射需求

- **FR21**（`epics.md` Story 4.2；PRD — Unix 上 **γ/Lima** 执行 **`rustup`/`cargo`**）
- **NFR-I1**：**`limactl`** 缺失、实例未就绪、挂载未配置等场景下**可诊断**（显式错误/SKIP 说明，见 epic AC）

## Acceptance Criteria

1. **Given** **`SessionHolder`** 为 **`Gamma`**（**`#[cfg(unix)]`**；**`DEVSHELL_VM_BACKEND=lima`** 或 **`auto`** 解析到 γ，以 **`vm/config`** / **`try_session_*`** 为准）且 **`limactl`** 可用  
   **When** 调用 **`run_rust_tool`**（**`vm/mod.rs`** → **`GammaSession::run_rust_tool`**，**`session_gamma/session/exec.rs`**）  
   **Then** 执行路径包含：**`limactl_ensure_running`** →（Mode S 时）**`push_incremental`** → **`limactl_shell`** 在 **guest 工作目录** 运行 **`program` + `args`** →（成功路径）**`pull_workspace_to_vfs`** 或文档化警告；与 **`docs/devshell-vm-gamma.md`** 中 **Mode S push/pull** 描述一致（**FR21**）。

2. **Given** **`ensure_ready`**（**`VmExecutionSession::ensure_ready`** for **γ**）  
   **When** 首次会话  
   **Then** 包含 **`limactl_ensure_running`** 及 **`maybe_ensure_guest_build_essential`** / **`maybe_guest_todo_probe_hint_and_install`** 等（以 **`exec.rs`** 实现为准），失败时 **`VmError::Lima`** 或等价消息可定位（**FR21**，**NFR-I1**）。

3. **Given** **`limactl` 不在 PATH**、实例不存在、或 **`lima_diagnostics`** 判定 guest 配置不当  
   **When** 用户尝试 γ 路径  
   **Then** **不**静默成功；错误信息或 **`lima_diagnostics::warn_if_guest_misconfigured` / `emit_tool_failure_hints`** 输出与 **`docs/devshell-vm-gamma.md` §前置条件** 可对照（**NFR-I1**）。

4. **Given** **非 Unix** 或 **γ 未编译**（**`cfg(unix)`**）  
   **When** 构建或运行  
   **Then** **不**错误链接 **gamma** 符号；与 **`SessionHolder`** 的 **cfg** 分派一致（**FR21** 范围仅限 Unix）。

5. **棕地**：实现位于 **`crates/todo/src/devshell/vm/session_gamma/`**（**`GammaSession`**、**`exec.rs`**、**`lima_diagnostics`**、**`sync`**）；本故事以 **核对 AC、补集成测试（可 `#[ignore]` + 环境说明）、文档交叉引用**为主，**不**重复实现 **4.1** 宿主 **`sandbox::run_rust_tool`** 逻辑。

6. **回归**：**`cargo test -p xtask-todo-lib`**（**`vm`** / **`session_gamma`** 单元测试）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **棕地核对**：阅读 **`session_gamma/session/exec.rs`** 中 **`VmExecutionSession::run_rust_tool`** 全路径；对照 **`docs/devshell-vm-gamma.md`** 心智模型（§简单心智模型、§前置条件）。
- [x] **环境矩阵**：在具备/不具备 Lima 的环境下列出 **SKIP** 与 **显式失败** 预期；若缺测试，添加 **`#[cfg(unix)]`** 测试或 **`#[ignore]`** 说明。
- [x] **与 4.1 边界**：确认 **`Self::Host`** vs **`Self::Gamma`** 在 **`VmConfig`** 下的选择条件（**`vm/mod.rs`**、**`try_session_rc_or_host`**）。
- [x] **验证**：`cargo test -p xtask-todo-lib`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| γ 执行 | **`crates/todo/src/devshell/vm/session_gamma/session/exec.rs`** — **`impl VmExecutionSession for GammaSession`** |
| 会话类型 | **`crates/todo/src/devshell/vm/session_gamma/mod.rs`** — **`GammaSession`** |
| 分派 | **`crates/todo/src/devshell/vm/mod.rs`** — **`SessionHolder::Gamma`**、**`run_rust_tool`** |
| 诊断 | **`lima_diagnostics.rs`** |

### 架构合规（摘录）

- **γ** 使用 **`limactl`** 子进程，**不**在宿主内嵌 QEMU；与 **architecture.md** VM 分层一致。

### 前序故事

- **4-1**（宿主沙箱）：**无 VM** 路径；本故事覆盖 **Unix + Lima 启用** 路径。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 Story 4.2]
- [Source: `docs/devshell-vm-gamma.md`]
- [Source: `docs/requirements.md` — §1.1、§5.8]
- [Source: `crates/todo/src/devshell/vm/session_gamma/session/exec.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib try_session_rc_reports_lima_missing_when_backend_lima`
- `cargo test -p xtask-todo-lib try_session_rc_or_host_falls_back_to_host_when_lima_unavailable`
- `cargo test -p xtask-todo-lib`
- `cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`

### Completion Notes List

- 已核对 γ 执行主路径：`GammaSession::run_rust_tool` 依次执行 `limactl_ensure_running`、(Mode S) `push_incremental`、`limactl_shell`、(Mode S) `pull_workspace_to_vfs`（失败转 warning），与 `docs/devshell-vm-gamma.md` 描述一致。
- 已核对 `ensure_ready` 路径：首次会话包含 `limactl_ensure_running`、`maybe_ensure_guest_build_essential`、`maybe_guest_todo_probe_hint_and_install`，失败向 `VmError::Lima` 传播。
- 新增 Unix 测试 `try_session_rc_reports_lima_missing_when_backend_lima`：在无 `limactl` PATH 条件下断言 `try_session_rc` 返回 `Err` 且 stderr 含 `limactl not found in PATH`，覆盖可诊断失败路径（NFR-I1）。
- 新增 Unix 测试 `try_session_rc_or_host_falls_back_to_host_when_lima_unavailable`：断言 `try_session_rc_or_host` 会降级到 `Self::Host` 并输出 `VM unavailable`，确认与 4.1 的边界分派契约。
- 环境矩阵结论：Lima 可用时走 γ 路径；Lima 不可用时为显式失败或可诊断降级；`sandbox` 中 Linux mount namespace 相关测试保持 `#[ignore]`（需专用权限环境），与文档说明一致。
- 文档核对结果：`docs/devshell-vm-gamma.md`、`docs/requirements.md` 与当前实现一致，无需修改文档真源。

### File List

- `crates/todo/src/devshell/vm/mod.rs`
- `_bmad-output/implementation-artifacts/4-2-unix-gamma-lima-path.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
