# 设计说明（Design）

本文档描述 **xtask_todo** 的技术架构、模块划分与数据流，与 [requirements.md](./requirements.md) 对齐。实现与文档不一致时，以代码为准并回写本文档。

---

## 1. 技术架构

### 1.1 总体结构

Cargo **workspace**（`resolver = "2"`）：根目录配置 members，业务在子 crate 中。

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        xtask_todo (workspace)                             │
├──────────────────────────────────────────────────────────────────────────┤
│  crates/todo (xtask-todo-lib)     │  xtask                                │
│  · Todo 领域库                     │  · cargo xtask 唯一二进制入口         │
│  · devshell（VFS/脚本/REPL/沙箱/vm）│  · argh 解析，编排 todo 与宿主命令   │
│  · 二进制 cargo-devshell           │  · publish = false                    │
│  crates/devshell-vm（β 侧车）      │                                       │
└──────────────────────────────────────────────────────────────────────────┘
```

| Crate | 职责 |
|-------|------|
| **crates/todo** | 待办领域（`TodoList`、`Store`、`Todo`…）；**不依赖 xtask**。含 **`devshell`** 与 **`cargo-devshell`**。 |
| **crates/devshell-vm** | β 侧车二进制（`devshell-vm`）；IPC 见 `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`。 |
| **xtask** | `cargo xtask`：`todo` 走 `xtask_todo_lib`；其余为 fmt/clippy/git/gh 等。 |

### 1.2 技术选型

| 层级 | 选型 | 说明 |
|------|------|------|
| 语言 / Edition | Rust 2021 | 各 member 自管 `[lints.clippy]` |
| Workspace | `members = ["crates/todo", "crates/devshell-vm", "xtask"]` | 根不设 `[lints]` |
| xtask / todo CLI | **argh** | 子命令与 flag |
| devshell 行解析 | 自研 **parser** | 管道 `\|`、重定向、引号 |
| REPL | **rustyline** + 自定义 `Completer` | **`CompletionType::List`**；**`rustyline`** 在 **`crates/todo` 根 `[dependencies]`**，保证 **Windows / Linux / macOS** 同一套依赖图（非仅 Linux target deps）。 |
| 入口别名 | `.cargo/config.toml` | `cargo xtask` → `cargo run -p xtask --` |

### 1.3 Todo 库分层（`crates/todo` 领域部分）

```
┌─────────────────────────────────────────┐
│  Public API                              │
│  TodoList<S>, Todo, TodoId, TodoPatch,   │
│  ListOptions, RepeatRule, Store, …       │
├─────────────────────────────────────────┤
│  Domain                                  │
│  model, priority, repeat, id, error      │
├─────────────────────────────────────────┤
│  Storage                                 │
│  Store trait → InMemoryStore             │
└─────────────────────────────────────────┘
```

- **`TodoList<S: Store>`**：创建、列表、`get`、`update`、`complete(id, no_next)`、`delete`、`search`、`stats`、导入等。
- **`InMemoryStore`**：进程内 Vec；**`.todo.json`** 由 **xtask** `todo/io` 与 devshell **`todo_io`** 在 crate 外完成读写。

### 1.4 Devshell 分层（`crates/todo::devshell`）

```
┌─────────────────────────────────────────────────────────────┐
│  cargo-devshell → devshell::run_main / run_main_from_args    │
├─────────────────────────────────────────────────────────────┤
│  repl          │  TTY: rustyline；非 TTY: read_line           │
│  completion    │  命令名 + VFS 路径补全                        │
│  parser        │  Pipeline / SimpleCommand / 重定向          │
│  command       │  dispatch / builtins / todo_builtin         │
│  vfs           │  内存 Vfs（Mode S 真源；Mode P 为辅助视图）   │
│  script        │  .dsh：AST、exec                              │
│  sandbox       │  export VFS → temp → rustup/cargo → sync    │
│  vm            │  SessionHolder、γ Lima / β IPC、workspace 同步│
│  session_store │  会话 JSON（路径约定见 requirements §1.1）   │
│  serialization │  Vfs 快照序列化（测试/Mode S 等；实现细节）  │
│  todo_io       │  devshell 内 `todo` ↔ `.todo.json`           │
└─────────────────────────────────────────────────────────────┘
```

**执行模型**：不执行任意宿主 shell；除 **`rustup`/`cargo`** 经 **`SessionHolder`** 外，仅 **builtin**。

**VM（`devshell::vm`）**

- **Unix 默认**：未设 **`DEVSHELL_VM`** 视为开启；未设 **`DEVSHELL_VM_BACKEND`** 默认为 **`lima`**（γ）。**关闭**：**`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`** / **`auto`**。
- **Windows 默认**：未设 **`DEVSHELL_VM_BACKEND`** 时默认为 **`beta`**（库默认 **`beta-vm`**）；**无 Lima**。侧车通过 **Podman Machine** 运行 **`devshell-vm`**（见下 **β 侧车** 与 **`docs/devshell-vm-windows.md`**）。
- **γ**：`limactl` 编排；挂载与工作区见 **`docs/devshell-vm-gamma.md`**；**`cargo xtask lima-todo`** 维护 **`lima.yaml`**。
- **β（宿主库侧）**：**`--features beta-vm`**，**`session_beta::BetaSession`** 经 **`DEVSHELL_VM_SOCKET`** 连接侧车（Unix：**UDS** / **`tcp:`**；**Windows**：默认 **`stdio`**，不经由本机监听端口）。**宿主工作区根**由 **`workspace_host::workspace_parent_for_instance`** 解析（顺序见 **requirements §5.2**）。**`session_start`** 将 **`staging_dir`**（与 **`guest_workspace`**，默认 **`/workspace`**）发给侧车，后续 **`guest_fs`** / **`exec`** 带同一 **`session_id`**。
- **Mode S / Mode P**：见 **requirements §1.1** 与 **`docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md`**。Mode P 下工程树经 **`GuestFsOps`**（γ）或 β IPC 与 guest 挂载一致；**`DEVSHELL_VM_WORKSPACE_MODE=guest`** 与 **`DEVSHELL_VM=off`** 等冲突时 **降级 Mode S**（不报错）。
- **会话持久化**：**`logical_cwd`** 等写入 **工作区树内** JSON（**requirements §1.1** 路径）；**`session_store::GuestPrimarySessionV1`** 使用 **`format = "devshell_session_v1"`**；不以「宿主进程 cwd 旁的 `*.session.json`」为唯一规范（实现含 **legacy 读路径**）。
- **非 Unix（Windows）**：**`vm_workspace_host_root`** 等对 Lima 宿主路径的 API 在 **`#[cfg(unix)]`** 下实现；**`#[cfg(not(unix))]`** 提供**桩**实现，因 **`devshell/mod.rs`** 中 **`if cfg!(unix) && … { vm_workspace_host_root() }`** 分支在非 Unix 上仍参与**类型检查**。**`WorkspaceBackendError`** 在 **`command/dispatch.rs`** 中**始终**可解析（不限于 **`cfg(unix)` import**），以支持 **`WorkspaceReadError::Backend`** 的映射。β 在 Windows 上为**完整路径**（非桩）：**`podman_machine`**、**`session_beta`**、**`workspace_host`** 等。

**β 侧车（`crates/devshell-vm`，Linux 可执行文件）**

- **协议**：stdin/stdout（或 UDS/TCP）上 **一行一条 JSON**；**`handshake`**、**`session_start`**、**`guest_fs`**、**`sync_request`**、**`exec`**、**`session_shutdown`** 等（草案见 **`docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`**）。
- **`guest_fs`**：在 **`session_start`** 之后将 **`guest_path`**（位于 **`guest_workspace`** 下）映射到宿主 **`staging_dir`** 树并读写；无会话时可为桩列表。
- **`exec`**：在 **`map_guest_to_host(guest_cwd)`** 目录下 **`Command::spawn`** **`argv`**；**非桩**。OCI 运行时镜像须含 **`cargo`**（**`containers/devshell-vm/Containerfile`**）。
- **stdio 与 JSON**：侧车进程 **stdout 专用于协议回包**。子进程 **不得**继承该 stdout（否则 **`cargo run`** 的程序输出会破坏宿主 **`read_json_line`**）。实现：**子进程 stdout/stderr 管道化**，并 **`io::copy`** 到 **侧车 stderr**；宿主/Podman 侧通常仍能看到编译与程序输出（见 **requirements §5.8**）。

**管道 / 重定向**：前段 stdout → 缓冲 → 下段 stdin；重定向在 builtin 层走 VFS 或 guest 路径（以实现为准）。

### 1.5 Xtask 角色

- **todo 子命令**：加载/保存 **`.todo.json`**、调用 **`TodoList`**、人类可读 / `--json`、`--dry-run`、退出码。
- **其他子命令**：fmt、clippy、coverage、git、gh、publish、**`acceptance`**（见 **`docs/acceptance.md` §8**）等——**不内嵌 Todo 领域规则**。
- **`acceptance`**：编排 **`cargo test`**（**xtask-todo-lib** / **xtask** / **devshell-vm**）、**NF-1/2/6** 文件检查、**Windows MSVC 交叉 `cargo check`**，**写 `docs/acceptance-report.md`**；实现见 **`xtask/src/acceptance/`**。
- **入口**：`xtask/src/main.rs` → **`xtask::run()`**（**`lib.rs`** 便于测试）。

### 1.6 Pre-commit 与 Windows 交叉编译

- **`.githooks/pre-commit`**：暂存 **`.rs` 行数**、**`cargo fmt --check`**、**`cargo clippy --all-targets`**（pedantic/nursery、**`-D warnings`**）、**`cargo test`**，以及 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**。
- **`cargo xtask git pre-commit`**（**`xtask/src/git.rs`**）在仓库根执行上述脚本（**`sh .githooks/pre-commit`**），与 **`git config core.hooksPath=.githooks`** 下真实 **`git commit`** 钩子一致。
- **开发者依赖**：**`rustup target add x86_64-pc-windows-msvc`**；否则最后一步 **`cargo check`** 失败。

---

## 2. 数据流

### 2.1 Todo 领域

```mermaid
flowchart LR
    subgraph Callers["调用方"]
        XtaskTodo[xtask todo]
        DevshellTodo[devshell builtin todo]
    end
    subgraph Lib["xtask_todo_lib"]
        TL[TodoList + Store]
    end
    XtaskTodo --> TL
    DevshellTodo --> todo_io[todo_io] --> TL
```

- **xtask**：`todo/io` ↔ **`.todo.json`** ↔ `TodoList`；**`--dry-run`** 跳过写。
- **devshell**：**`todo_io`** 在 cwd 约定下读写 **同一 `.todo.json`**；内置 **`todo`** 为子集（无 export/import/init-ai）。

### 2.2 `cargo xtask` 调用链

```mermaid
sequenceDiagram
    participant Dev as 开发者
    participant Xtask as xtask
    participant Lib as xtask_todo_lib
    participant FS as 文件系统

    Dev->>Xtask: cargo xtask todo list
    Xtask->>FS: 读 .todo.json
    Xtask->>Lib: TodoList 操作
    Lib-->>Xtask: 结果 / 错误
    Xtask->>Xtask: format / --json
    Xtask->>FS: 写回（非 dry-run）
```

- **退出码**：todo 子命令 **2** 参数错误、**3** 数据错误；其余失败多为 **1**。

### 2.3 持久化

| 数据 | 位置 | 说明 |
|------|------|------|
| 待办列表 | **`.todo.json`** | JSON 数组；id、title、completed、时间戳、可选字段。 |
| Devshell 会话 | **工作区内** **`…/.cargo-devshell/session.json`** | 与 **requirements §1.1** 一致；宿主侧与 **`DEVSHELL_WORKSPACE_ROOT`** 挂载对齐。 |
| Vfs 快照 | 实现内序列化 | Mode S 等路径；**非**对外规范文件名（见 **requirements**）。 |

**宿主文本编码**：**`devshell::host_text`** 用于 **`.dsh`**、`source`、**`.todo.json`**（UTF-8/UTF-16 BOM）。沙箱 **copy/sync** 仍为按字节。

### 2.4 列表展示与时间（xtask）

- **`xtask/src/todo/format.rs`**：TTY 着色；**7 天**未完成 **ANSI 黄色**（**`AGE_THRESHOLD_DAYS`**）。
- 人类可读：相对时间、已完成项用时。

### 2.5 Rust 工具链沙箱（devshell）

```mermaid
flowchart LR
    VFS[Vfs cwd 子树]
    Exp[export_vfs_to_temp_dir]
    Tmp[临时目录]
    Run[宿主或 VM 内子进程]
    Sync[sync 回 Vfs]
    VFS --> Exp --> Tmp --> Run --> Sync --> VFS
```

- **`sandbox`**：`export_vfs_to_temp_dir` → **`run_in_export_dir`**（PATH 中 **`cargo`/`rustup`**）→ **`sync_host_dir_to_vfs`**。导出基目录 **`DEVSHELL_EXPORT_BASE`** / 默认 cache 路径；Linux 可选 **`DEVSHELL_RUST_MOUNT_NAMESPACE`**。详见 **[dev-container.md](./dev-container.md)**。
- **Mode P + VM**：**Unix** 上 **`rustup`/`cargo`** 经 γ **`limactl shell`** 或 β **`exec`**；**Windows** 上经 **β `exec`**（无 γ）。Mode S 仍以 **sandbox 导出目录** 跑宿主工具链为主。

---

## 3. 接口与模块映射

### 3.1 Todo 库（摘要）

| 类型 | 说明 |
|------|------|
| `TodoId` | `NonZeroU64`，0 非法。 |
| `Todo` / `TodoPatch` / `ListOptions` | 见 **`crates/todo/src/list/`**、**`model.rs`**。 |
| `TodoList<S: Store>` / `InMemoryStore` | 领域门面与默认存储。 |

### 3.2 Xtask（`xtask/src/lib.rs`）

| `XtaskSub` | 职责 |
|------------|------|
| `Run` / `Clean` / `Clippy` / `Coverage` / `Fmt` / `Gh` / `Git` / `Publish` | 开发者任务。 |
| `Todo` | **`todo/cmd/dispatch.rs`** + **`args.rs`** + **`error.rs`**。 |

### 3.3 Devshell 入口

| 函数 | 用途 |
|------|------|
| **`run_main` / `run_main_from_args` / `run_with`** | 二进制与测试入口。 |

**Builtin**：**`command/dispatch.rs`**；**`todo`**：**`command/todo_builtin.rs`**。

### 3.4 与 requirements 章节的对应

| requirements | 设计落点 |
|--------------|----------|
| §3 Todo | `TodoList`、`Store`、`xtask/todo/*`、`format.rs` |
| §4 其他 xtask | **`XtaskSub`**、`run_with` |
| §5 Devshell | **`devshell::*`**、`sandbox`、`vm`（**`session_gamma`、`session_beta`、`workspace_host`、`podman_machine`**）、**`session_store`**、`completion`、`repl` |
| §5.2 / §5.8 工作区路径与 Windows β | **`workspace_host::workspace_parent_for_instance`**、**`crates/devshell-vm`**（**`exec`** / **`guest_fs`**）、**`docs/devshell-vm-windows.md`** |
| §6 AI / 退出码 | **`--json`**、**`TodoCliError`**、**`print_json_error`** |
| §7 非功能 | Clippy、stderr、TTY 颜色 |
| §1.2 / §7.1～§7.2 平台与 pre-commit | **§1.2**、**§1.6**、**§4.5** |

---

## 4. 关键设计决策

### 4.1 持久化与领域分离

- **`.todo.json`** I/O 在 **xtask** 与 **devshell `todo_io`**；库保持 **`InMemoryStore`**。

### 4.2 Devshell 与 xtask 分离

- **xtask** 依赖 **xtask-todo-lib** 仅用于 **Todo**；**REPL** 通过 **`cargo-devshell`** 二进制。

### 4.3 Tab 补全

- **`CompletionType::List`**；路径候选为**含目录前缀的整词**。

### 4.4 脚本与 REPL

- 脚本 / `source` 作用域不污染后续 REPL 行（以实现为准）。

### 4.5 跨平台构建与 CI 前置检查

- **发布 `xtask-todo-lib`**：须能在 **Windows MSVC** 上通过 **`cargo check`**（crates.io 用户 **`cargo install`**）；仓库用 **§1.6** 的 pre-commit 步骤在 **Linux/macOS 开发机**上交叉验证。
- **仅 Unix 的代码路径**（**`GuestFsOps`**、**`logical_path_to_guest`** 等）用 **`#[cfg(unix)]`** 包住**调用**；**`workspace/io.rs`** 等在非 Unix 上对 **`vm_session`** 使用 **`let _ = …`** 避免 **unused** 告警。

---

## 5. 扩展与维护

- 新 Todo 能力：领域 + xtask `args`/`dispatch` + **requirements.md**。
- 新 xtask 子命令：**`XtaskSub`** + **`run_with`**。
- 新 devshell builtin：**`dispatch.rs`** + **`BUILTIN_COMMANDS`** + 帮助文案。
- VM / 沙箱：见 **`docs/superpowers/specs/`**、**`sandbox.rs`**、**`crates/devshell-vm`**、**`docs/devshell-vm-windows.md`**。

---

## 6. 参考

- [requirements.md](./requirements.md)（**§9** 含章节索引表，便于与旧文档章节号区分）
- [publishing.md](./publishing.md)
- [devshell-vm-gamma.md](./devshell-vm-gamma.md)（γ / β 总览）
- [devshell-vm-windows.md](./devshell-vm-windows.md)（Windows β、Podman、OCI 侧车）
- [devshell-vm-oci-release.md](./devshell-vm-oci-release.md)（侧车镜像发布与版本对齐）
- `docs/superpowers/specs/` — 专题设计
