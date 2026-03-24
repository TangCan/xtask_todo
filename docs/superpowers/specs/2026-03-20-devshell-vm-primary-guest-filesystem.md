# Devshell：以 VM（guest）文件系统为唯一真源

**状态：** 架构选型已确认；**Mode P + β 侧车已部分落地**（**`GuestFsOps`**、**`devshell-vm` `exec`/`guest_fs`**、**Windows Podman**，见 **[requirements.md](../../requirements.md) §5**、**§5.8**、**[design.md](../../design.md) §1.4**）。本文仍描述**目标形态**、与 Mode S 的差异及分期路线；若与主线需求冲突，以 **`requirements.md`** 为准。  
**详细设计（头脑风暴收敛稿）：** [`2026-03-20-devshell-guest-primary-design.md`](./2026-03-20-devshell-guest-primary-design.md)（含技术路线 A/B/C、`WorkspaceBackend`、路径模型、开放决策勾选）。  
**关联：** `docs/design.md` §1.4、`docs/devshell-vm-gamma.md`、`docs/devshell-vm-windows.md`、`docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md`。  
**产品需求**：[requirements.md](../../requirements.md) **§1.1**、**§5**、**§5.8**。

---

## 1. 背景与目标

**当前默认（同步模式，下称「Mode S」）**

- REPL / 脚本使用进程内 **内存 `Vfs`** 作为「工程树」真源。
- 执行 **`cargo` / `rustup`** 时：**增量 push** → guest 内执行 → **pull** 回写 `Vfs`。
- **`.dev_shell.bin`** 序列化的是这棵内存树。

**选定方向（guest 真源模式，下称「Mode P」）**

- **guest 内可见的工程目录**（如 **`/workspace/<leaf>`**）为 **唯一真源**。
- REPL 的 **`cd` / `ls` / `cat` / `mkdir` / …** 与 **`cargo`** **一致地**针对 **同一份** guest 文件系统操作，**不再**在每次 rust 命令前后做整段 push/pull 来「对齐」内存树与磁盘。
- 可选：**宿主进程不再维护完整内存 `Vfs` 镜像**，或仅保留 **路径/cwd 状态机 + 缓存**（实现细节见下文）。

**非目标（本规格暂不承诺）**

- Mode P **不沿用** 现行 **`.dev_shell.bin`** 文件格式；会话持久化改用 **与虚拟工作区 / guest 一致** 的新载体（详见设计 **`2026-03-20-devshell-guest-primary-design.md` §10**）。
- **Todo / `.todo.json`**：Mode P 仍 **只在宿主**（与 `todo_io` 一致），见设计 **`2026-03-20-devshell-guest-primary-design.md` §11**（**A 已决**）。

---

## 2. 与 Mode S 的对比

| 维度 | Mode S（当前） | Mode P（目标） |
|------|----------------|----------------|
| 工程文件真源 | 内存 `Vfs` | guest 挂载目录 |
| `cargo` 前 | push | 无（或仅确保挂载就绪） |
| `cargo` 后 | pull | 无（或可选一致性检查） |
| `ls`/`cat` 等 | 读内存树 | 读 guest（需远程执行或代理） |
| 典型延迟 | 同步成本集中在 rust 调用 | 每个文件 builtin 可能多一次 **SSH/exec** |
| 离线/无 VM | 仍可用内存 VFS | **强依赖** VM 可用 |

---

## 3. 实现路线（建议分阶段）

### Phase 0 — 开关与文档

- 引入显式配置（名称待定），例如环境变量 **`DEVSHELL_VM_WORKSPACE_MODE=sync|guest`** 或 Cargo feature **`guest-primary-vm`**，**默认保持 `sync`**，避免破坏现有用户与测试。**有效模式解析**：若请求 **`guest`** 但 **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`/`auto`**，则 **强制 `sync`**、不报错（见设计 **`2026-03-20-devshell-guest-primary-design.md` §6**）。
- 在 **`docs/devshell-vm-gamma.md`**、**`docs/design.md`** 标明默认与实验模式。

### Phase 1 — 远程文件原语

- 抽象 **`GuestFsOps`**（或扩展 **`VmExecutionSession`**）：在 guest 内执行 **受限** 命令或 sftp/scp 子集，实现：`list_dir`、`read_file`、`write_file`、`mkdir`、`remove`、**`set_cwd` 的逻辑等价**（guest 路径下的 cwd）。
- **路径映射**：REPL 逻辑路径（如 `/projects/foo`）↔ guest 路径（如 **`/workspace/foo`**），与现有 **`guest_dir_for_cwd_inner`** 规则对齐。
- **`cargo` / `rustup`**：在 Mode P 下直接 **`limactl shell --workdir <guest_cwd>`**，**不再**调用 **`push_incremental` / `pull_workspace_to_vfs`**（对工程树部分）。

### Phase 2 — REPL / dispatch 接入

- **`command::dispatch`**：在 Mode P 下，文件类 builtin 走 **`GuestFsOps`**，而非 **`Vfs`**。
- **管道 / 重定向**：定义字节流在宿主与 guest 之间的传输策略（仍不执行任意宿主 shell）。
- **补全**：路径补全需基于 guest 目录列表（延迟与缓存策略）。

### Phase 3 — 脚本、持久化、Todo

- **`.dsh` 脚本**：与 REPL 相同后端（guest 真源）。
- **会话持久化**：Mode P **弃用** legacy **`.dev_shell.bin`** 格式，引入 **新格式**（与 guest 真源一致；细节见设计 §10）；Mode S 仍用现行 bin。
- **`todo` builtin / `.todo.json`**：**已决** 仍在 **宿主工作区**（与设计 §11 **A** 一致）。

### Phase 4 — β 侧车

- **`SessionHolder::Beta`** 在 Mode P 下对 IPC 增加 **「文件操作」** 消息类型，或侧车直接访问与 Lima **同一** 挂载语义，避免双份同步逻辑。

---

## 4. 风险与约束

- **性能**：每个 `ls` 若都 **`limactl shell`**，交互体验可能明显下降；可能需要 **长驻 SSH 会话**、**批量 API** 或 **virtiofs 在宿主再挂一层**（架构再评估）。
- **安全**：guest 内执行面扩大，需保持 **命令白名单**，禁止任意 shell。
- **测试**：`cfg(test)` 默认无 VM；需提供 **mock `GuestFsOps`** 或 **可切换回内存 Vfs** 的测试后端。
- **Windows**：γ 不在 Windows 上编译；Mode P 同样 **Unix-centric**。

---

## 5. 决策记录

- **2026-03-20**：产品方向选择 **Mode P（以 VM 内文件系统为唯一真源）**；实现按上表分阶段推进，**默认行为在代码落地前仍为 Mode S**。

---

## 6. 后续文档维护

- 实现某一 Phase 后，更新 **`docs/design.md`** 数据流图与 **`docs/requirements.md`** 中 devshell 相关条目。
- **任务勾选清单：** [`docs/superpowers/plans/2026-03-20-devshell-guest-primary-workspace.md`](../plans/2026-03-20-devshell-guest-primary-workspace.md)。

---

## 7. 实现进度快照（仓库维护，非规格承诺）

| 主题 | 状态（截至文档更新） |
|------|----------------------|
| **β 侧车 `devshell-vm`** | **`guest_fs`**（`session_start` 后宿主映射）、**`exec`**（真实子进程）、**stdio** 下子进程输出与 JSON **stdout** 分离；见 **requirements §5.8** |
| **Windows** | 默认 **β + Podman**，无 **γ**；见 **devshell-vm-windows.md** |
| **Mode P 全量**（与本文 §3 所有阶段） | 部分落地；与 **guest-primary-design** 对照时以 **requirements.md**、**design.md §1.4** 为准 |
| **§4 风险「Windows Mode P Unix-centric」** | **β** 已提供 Windows 路径；γ 仍为 Unix only |
