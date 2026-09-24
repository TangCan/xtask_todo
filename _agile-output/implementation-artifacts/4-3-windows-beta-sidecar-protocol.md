# Story 4.3：Windows β 与侧车协议

Status: done

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

- [x] **棕地核对**：通读 **`BetaSession::exchange`**、**`ensure_session_started`**、**`run_rust_tool`**；对照 **`docs/devshell-vm-windows.md`** 与 **`crates/devshell-vm`** 协议说明。
- [x] **成帧**：确认 **`read_one_json_line`** 与侧车 **`write`** 均为**一行一 JSON**；记录与 **exec** 子进程 **stdout** 混用风险及 **`docs/devshell-vm-windows.md`** 已述缓解措施。
- [x] **测试**：为 **handshake** / **错误帧** / **parse 失败** 补单元测试（可 mock **`IpcStream`**）。
- [x] **验证**：`cargo test -p xtask-todo-lib --features beta-vm`、`cargo clippy -p xtask-todo-lib --all-targets --features beta-vm -- -D warnings`。

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

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask-todo-lib --features beta-vm ensure_ready_handshake_uses_single_json_line_and_accepts_handshake_ok`
- `cargo test -p xtask-todo-lib --features beta-vm ensure_ready_maps_error_frame_to_vmerror_ipc`
- `cargo test -p xtask-todo-lib --features beta-vm ensure_ready_reports_non_json_response_prefix`
- `cargo test -p xtask-todo-lib --features beta-vm`
- `cargo clippy -p xtask-todo-lib --all-targets --features beta-vm -- -D warnings`

### Completion Notes List

- 已核对 `BetaSession::ensure_ready`/`exchange`/`run_rust_tool`：握手使用 `op=handshake, version=1`，Mode S 按 `push_incremental -> sync_request(push_to_guest) -> exec -> sync_request(pull_from_guest) -> pull_workspace_to_vfs` 路径执行，满足 FR22。
- 已核对协议成帧：请求侧统一 `writeln!` 发送单行 JSON；响应侧通过 `read_one_json_line` / `StdioPipe::read_json_line` 读取单行并 `serde_json` 解析，非 JSON 会返回含 `first line prefix` 的诊断，满足 NFR-I2。
- 新增单测 `ensure_ready_handshake_uses_single_json_line_and_accepts_handshake_ok`：用 TCP mock 侧车验证 handshake 请求是一行一 JSON（含换行）且 `handshake_ok` 可通过。
- 新增单测 `ensure_ready_maps_error_frame_to_vmerror_ipc`：验证 `op:error` 帧被映射为 `VmError::Ipc(message)`，不会被当成功响应。
- 新增单测 `ensure_ready_reports_non_json_response_prefix`：验证响应为非 JSON 时错误包含前缀提示，满足可诊断排错需求。
- 文档核对：`docs/devshell-vm-windows.md` 已覆盖 exec 子进程 stdout 可能污染协议通道的风险与 stderr 转发缓解策略；本次实现与文档一致，无需改文档真源。

### File List

- `crates/todo/src/devshell/vm/session_beta/mod.rs`
- `crates/todo/src/devshell/vm/session_beta/session.rs`
- `crates/todo/src/devshell/vm/session_beta/tests.rs`
- `_bmad-output/implementation-artifacts/4-3-windows-beta-sidecar-protocol.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | **`BetaSession`** 迁入 **`session.rs`**，`mod.rs` 仅 **`ipc` + re-export**，结构更清晰；**`exchange`** 仍为单行 **`writeln!` + `read_one_json_line`/`read_json_line`**，**`op:error`** 映射 **`VmError::Ipc`**，与 AC2/4 一致。 |
| **Edge Case Hunter** | **`tests.rs`** 以 **TCP mock** 侧车串行 **`vm_env_lock`**，握手测要求请求行以换行结尾；若 **`ensure_ready`** 在 **`restore_var` 前 panic**，**`DEVSHELL_VM_SOCKET`** 可能残留（与其它 env 测相同量级）。 |
| **Acceptance Auditor** | AC1：握手 **`version:1`/`handshake_ok`** 由集成测覆盖；AC2：成帧与非 JSON **`first line prefix`** 由测覆盖；AC4：**`op:error`** 测覆盖；AC3/run_rust_tool 链为棕地搬迁无行为 diff；AC6：**`cargo test -p xtask-todo-lib --features beta-vm`**（273 passed）、**`clippy --features beta-vm -D warnings`** 已通过。 |

**待办项**：无。审查中若工作区出现 **`crates/todo/.dev_shell.bin`** 漂移，应按仓库先例 **`git restore`**，勿与故事一并提交。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — 故事与 sprint **4-3-windows-beta-sidecar-protocol** 标为 **done**；**`last_updated`** 更新为 **`2026-03-26T03:26:00Z`**；复验带 **`beta-vm`** 的 **`cargo test` / `clippy`**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
