# Story 4.3：Windows β 与侧车协议

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名 Windows 开发者，  
我希望在启用 VM 时通过 **β** 与 **JSON 行**侧车执行 **`rustup`/`cargo`**，  
且**协议通道不被污染**。

## 映射需求

- **FR22**（`epics.md` Story 4.3；PRD — Windows **β** + 侧车执行工具链）
- **NFR-S2**：侧车 **IPC/stdio** 不被非协议输出污染（与 **requirements §5.8** 一致）
- **NFR-I2**：JSON 行协议须**版本化 handshake**、**一行一对象**成帧，便于宿主解析与排错

## Acceptance Criteria

1. **Given** 以 **`cargo build -p xtask-todo-lib --features beta-vm`** 构建且 **`SessionHolder::Beta`** 激活（**`DEVSHELL_VM_BACKEND=beta`** 等，见 **`docs/devshell-vm-windows.md`**）  
   **When** 调用 **`BetaSession::ensure_ready`**  
   **Then** 经 **`exchange`** 发送 **`op: handshake`**（**`version: 1`**、**`client`/`client_version`**），并校验响应 **`op: handshake_ok`**；失败返回 **`VmError::Ipc`** 且信息可诊断（**FR22**，**NFR-I2**）。

2. **Given** 已连接 **`IpcStream`**（**`stdio`** / **TCP** / **Unix UDS**，见 **`session_beta/ipc.rs`**）  
   **When** 调用 **`exchange`**（任意 `op`）  
   **Then** 请求**单行** JSON **`writeln!`**；响应经 **`read_one_json_line`** 解析为 **`serde_json::Value`**；非 JSON 或空行报错并包含**首行前缀**提示（**NFR-I2**）。

3. **Given** **`run_rust_tool`**（**`impl VmExecutionSession for BetaSession`**）  
   **When** 执行 **Mode S**（**`sync_vfs_with_workspace`** 为真）  
   **Then** **`push_incremental`** → **`sync_request`**（**`push_to_guest`**）→ **`exec`**（**`argv`**、可选 **`timeout_ms`**）→ **`sync_request`**（**`pull_from_guest`**）→ **`pull_workspace_to_vfs`**（失败时告警）；与 **`docs/devshell-vm-gamma.md`** / **`devshell-vm-windows.md`** 叙述一致（**FR22**）。

4. **Given** 侧车返回 **`op: error`**  
   **When** **`exchange`** 处理响应  
   **Then** 映射为 **`VmError::Ipc(message)`**，**不**当作成功帧（**NFR-S2**，**NFR-I2**）。

5. **棕地**：实现位于 **`crates/todo/src/devshell/vm/session_beta/`**（**`mod.rs`**、**`ipc.rs`**）；侧车二进制在 **`crates/devshell-vm/`**。本故事以 **核对 AC、协议与错误路径测试、文档**为主，**不**在无 RFC 的情况下更改 **JSON** 字段命名（破坏性变更须 **semver** 与 **`docs/superpowers/specs/*ipc*`** 一致）。

6. **回归**：**`cargo test -p xtask-todo-lib --features beta-vm`**（若 CI 未全量 feature，在 Completion Notes 说明）；**`cargo clippy -p xtask-todo-lib --all-targets --features beta-vm -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：通读 **`BetaSession::exchange`**、**`ensure_session_started`**、**`run_rust_tool`**；对照 **`docs/devshell-vm-windows.md`** 与 **`crates/devshell-vm`** 协议说明。
- [ ] **成帧**：确认 **`read_one_json_line`** 与侧车 **`write`** 均为**一行一 JSON**；记录与 **exec** 子进程 **stdout** 混用风险及 **`docs/devshell-vm-windows.md`** 已述缓解措施。
- [ ] **测试**：为 **handshake** / **错误帧** / **parse 失败** 补单元测试（可 mock **`IpcStream`**）。
- [ ] **验证**：`cargo test -p xtask-todo-lib --features beta-vm`、`cargo clippy -p xtask-todo-lib --all-targets --features beta-vm -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| β 会话 | **`crates/todo/src/devshell/vm/session_beta/mod.rs`** — **`BetaSession`**、**`exchange`** |
| IPC | **`session_beta/ipc.rs`** — **`connect_ipc`**、**`SocketSpec`** |
| 分发 | **`vm/mod.rs`** — **`SessionHolder::Beta`**（**`feature = "beta-vm"`**） |

### 架构合规（摘录）

- **β** 与 **γ** 共享 **`VmExecutionSession`**；**协议** 与 **devshell-vm** 侧车**成对演进**。

### 前序故事

- **4-1**（宿主沙箱）、**4-2**（γ）：本故事聚焦 **β + JSON 行 IPC**。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 Story 4.3]
- [Source: `docs/devshell-vm-windows.md`]
- [Source: `docs/requirements.md` — §5.8]
- [Source: `crates/todo/src/devshell/vm/session_beta/mod.rs`]
- [Source: `crates/devshell-vm/` — 侧车协议实现]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
