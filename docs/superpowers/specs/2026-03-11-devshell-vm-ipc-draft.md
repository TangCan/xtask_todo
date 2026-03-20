# `devshell-vm` IPC 草案（β）

**日期**：2026-03-11  
**状态**：草案（β 实现前可改）  
**目的**：定义 **`cargo-devshell`**（客户端）与侧车 **`devshell-vm`**（服务端）之间的 **无大块内联负载** 协议，与规格 [`2026-03-11-devshell-microvm-session-design.md`](./2026-03-11-devshell-microvm-session-design.md) §3 同步语义一致。

---

## 1. 传输

| 模式 | 说明 |
|------|------|
| **Unix 套接字（首选）** | 抽象地址或文件系统路径，由环境变量 **`DEVSHELL_VM_SOCKET`** 指定（实现可约定 `unix:///path` 或裸路径）。 |
| **标准输入/输出（调试）** | 每行一条 JSON（JSON Lines），仅用于开发与测试；生产由侧车监听 socket。 |

**成帧**：一条逻辑消息 = **一行** UTF-8 JSON 对象（`\n` 结尾），无跨行 JSON。

**字节序**：不适用（文本 JSON）。

---

## 2. 版本与握手

连接建立后，**客户端必须先发送**：

```json
{"op":"handshake","version":1,"client":"cargo-devshell","client_version":"…"}
```

**服务端**回复：

```json
{"op":"handshake_ok","version":1,"server":"devshell-vm","server_version":"…"}
```

或错误：

```json
{"op":"error","code":"version_mismatch","message":"…"}
```

**`version`**：协议主版本；不兼容变更递增 major，字段扩展递增 minor（可在 `handshake` 中带 `capabilities` 数组）。

---

## 3. 会话

### 3.1 `session_start`

客户端创建会话（对应 REPL / 脚本进程生命周期）：

```json
{
  "op": "session_start",
  "session_id": "<uuid>",
  "staging_dir": "/abs/path/on/host/workspace_parent",
  "guest_workspace": "/workspace",
  "backend": "lima",
  "backend_config": { "instance": "devshell-rust" }
}
```

**`staging_dir`**：主机上 **双方约定** 的同步根目录；**大文件不经 IPC 传输**，仅在此目录读写（与 γ 的 `push_incremental` / `pull` 布局一致：`staging_dir` 下叶子目录名 = VFS cwd 最后一段）。

**服务端**回复 `session_ok` 或 `error`。

### 3.2 `session_shutdown`

```json
{"op":"session_shutdown","session_id":"<uuid>","stop_vm":false}
```

服务端：最终 **pull 语义**（若由侧车负责写回客户端可读队列，则见下文）；可选停止 VM。回复 `shutdown_ok`。

---

## 4. 同步（避免 base64 大包）

**原则**：IPC 只传 **路径与操作意图**；文件字节在 **`staging_dir`** 上由 **客户端（devshell 库）** 或 **已挂载的 guest 视图** 完成。

### 4.1 `sync_request`（push 或 pull 意图）

客户端在 **已把 VFS 内容写到 `staging_dir`**（或已从 guest 同步到 `staging_dir`）后，可发送：

```json
{
  "op": "sync_request",
  "session_id": "<uuid>",
  "direction": "push_to_guest",
  "vfs_cwd_leaf": "hello",
  "manifest": [
    {"path":"Cargo.toml","action":"upsert","size":128,"mtime_unix_ms":0},
    {"path":"src/main.rs","action":"upsert","size":256,"mtime_unix_ms":0}
  ]
}
```

- **`direction`**：`push_to_guest` | `pull_from_guest`（具体由侧车在 guest 内执行 rsync/cp 或等价步骤）。
- **`manifest`**：相对 **该 vfs 子树根** 的路径列表；`action`：`upsert` | `delete`（可选）。
- **服务端**在 guest 内应用后回复 `sync_ok` 或 `error`。

> **说明**：若 guest 与主机已通过 **Lima 挂载** 共享 `staging_dir`，则 `sync_request` 可退化为 **无操作 + 确认**，仅用于节拍与错误统一；β 也可改为侧车内嵌 QEMU 时在 guest 与 staging 之间复制。

### 4.2 `sync_pull_result`（可选）

服务端主动推送「guest 已更新这些路径」时（长轮询或另一 socket 方向），可用扩展 op；**首期可省略**，由客户端在 `exec` 完成后发 `sync_request` + `pull_from_guest`。

---

## 5. 执行 `rustup` / `cargo`

```json
{
  "op": "exec",
  "session_id": "<uuid>",
  "guest_cwd": "/workspace/hello",
  "argv": ["cargo","build"],
  "env": { "RUST_BACKTRACE": "1" }
}
```

**服务端**：

1. 可选：先处理未决的 `sync_request`（push）。
2. 在 guest 内 `exec` / `spawn` 子进程，继承或覆盖 `env`。
3. 收集 **退出码**；**无论是否为零**，尽量完成 **pull** 到 `staging_dir`（与 γ / 计划一致）。
4. 回复：

```json
{"op":"exec_result","session_id":"<uuid>","exit_code":101,"signal":null}
```

`signal`：若被信号终止则填信号编号。

---

## 6. 错误与超时

统一错误帧：

```json
{"op":"error","session_id":"<uuid>","code":"staging_io","message":"…"}
```

**超时**：客户端对 `exec` 应设 **可配置超时**；超时时发 **`exec_cancel`**（草案，可实现为杀 guest 子进程或会话级 cancel）。

---

## 7. 与 γ 的关系

| γ（当前） | β（侧车） |
|-----------|-----------|
| `xtask-todo-lib` 直接 `limactl` | 客户端连接 `devshell-vm`，由侧车调 Lima / 其他驱动 |
| 同步在库内 `push_incremental` / `pull_workspace_to_vfs` | 同步可由侧车协调 + **同一 `staging_dir` 约定** |

协议 **不强制** guest 内工具链来源（仍由挂载 / 镜像解决）。

---

## 8. 安全提示

- **套接字权限**：仅本用户或显式 ACL；避免 world-writable socket 路径。
- **`staging_dir`**：会话结束可清理；勿指向用户 home 根目录。
- **`env`**：侧车应 **过滤** 危险变量或仅白名单 `RUST_*`、`CARGO_*` 等（实现阶段定）。

---

## 9. 相关路径

- 微 VM 会话设计：[`2026-03-11-devshell-microvm-session-design.md`](./2026-03-11-devshell-microvm-session-design.md)
- 实现计划：[`../plans/2026-03-11-devshell-microvm-session.md`](../plans/2026-03-11-devshell-microvm-session.md)
- 用户向 γ 说明：[`../../devshell-vm-gamma.md`](../../devshell-vm-gamma.md)
