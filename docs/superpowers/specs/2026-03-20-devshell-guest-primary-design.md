# Devshell：VM（guest）文件系统为唯一真源 — 设计规格

**日期**：2026-03-20  
**状态**：头脑风暴收敛稿（供评审 / 通过后进入 **writing-plans**）  
**前置选型**：用户已确认采用 **Mode P（guest 真源）**，与当前 **Mode S（内存 VFS + push/pull）** 并存，**默认保持 Mode S** 直至实现切换。  
**关联短文**：[`2026-03-20-devshell-vm-primary-guest-filesystem.md`](./2026-03-20-devshell-vm-primary-guest-filesystem.md)（目标与阶段总览）  
**历史规格**：[`2026-03-11-devshell-microvm-session-design.md`](./2026-03-11-devshell-microvm-session-design.md)（原「VFS 权威 + 同步」模型 — Mode P 在工程树语义上 **替代** 该文档 §3.2 对「REPL 与 guest 双份树」的假设，**不**否定会话级 VM、工具链挂载等约束）

---

## 1. 目的与成功标准

### 1.1 目的

- 用户在 **Mode P** 下面对 **同一套虚拟文件系统（工作区）**：**认知上始终只有一份树**。  
  - **VM 已就绪**：工程树由 **guest 内 Linux** 承载；`cargo`/`rustup` 与需要落盘的 builtin **经后端对 guest 操作**（等价于在 VM 里跑 Linux 语义）。  
  - **VM 未运行**：builtin 仍只操作 **该工作区的进程内表示**（如内存 `Vfs` 或与上次 guest 状态一致的镜像），**不把工程树「导出到宿主路径」** 作为真源或常规出口；宿主侧不出现第二份「导出副本」才算权威。
- **`cargo` / `rustup`** 与 **`cd` / `ls` / `cat` / `mkdir` / `touch` / `echo` 重定向写文件** 等 **针对工程树的操作** 全部落在 **上述同一虚拟工作区**；**取消** Mode S 那种工程树在内存与 guest 之间 **push/pull 对齐循环**（Mode P 下由 **单一后端视图** 统一，而非双源同步）。

### 1.2 成功标准（验收）

1. Mode P 开启且 VM 就绪时，在 guest 内 `touch /workspace/foo/x` 后，**无需 pull**，REPL `ls` 即可看到 **一致结果**（通过同一后端读到 guest）。
2. Mode P 下执行 `cargo build`，**不**调用 `push_incremental` / `pull_workspace_to_vfs` 针对 **工程树根**（`target/` 等留在 guest）。
3. **`DEVSHELL_VM=off`** 或 **Mode S** 时，行为与 **当前主线** 一致（回归测试通过）。
4. **`cargo test -p xtask-todo-lib`** 在 **无 Lima** 环境下 **不依赖** guest（Mock 或强制 Mode S）。

---

## 2. 非目标（首期）

- 不在 guest 内执行 **任意** 用户 shell 或任意二进制（仍仅 **白名单** 原语 + `rustup`/`cargo`）。
- Mode P **不沿用** 现行 **`.dev_shell.bin`** 序列化格式；会话持久化改用 **与虚拟工作区 / guest 一致** 的表示（见 §10）。
- 不在本文冻结 **β IPC** 报文格式（见 §4.3 与 Phase 4）。

---

## 3. 头脑风暴：三条技术路线（guest 侧 I/O）

| 路线 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 每操作 `limactl shell`** | 每个 `ls`/`cat`/… 拼 `sh -c` + 解析 stdout | 与当前 γ 一致、无新长连接 | **延迟高**、难做流式大文件与补全缓存 |
| **B. 持久 SSH（ControlMaster）** | 会话内 `ssh …` 长连接，向 guest 发 **小型助手脚本** 或 **结构化协议** | 延迟可控、可批量列目录 | 需管理 socket 生命周期、与 Lima 端口转发耦合 |
| **C. β IPC 文件 RPC** | 侧车或 `devshell-vm` 实现 `readdir`/`read`/`write` | 可测试、协议可版本化 | β 未就绪前 **不能** 作为唯一依赖 |

### 3.1 推荐（分阶段）

- **Phase 1–2**：以 **A** 做 **最小可用**（与现有 `limactl` 调用栈一致），同时抽象 **`GuestFsOps` trait**，便于替换为 **B**。
- **Phase 2 并行优化**：在 γ 上尝试 **`limactl shell` + 单次 `sh` 批量**（减少往返），或调研 Lima 是否暴露稳定 **SSH 复用** API。
- **Phase 4**：**C** 与 **β** 对齐，**GuestFsOps** 的默认实现切到 IPC。

---

## 4. 架构总览

### 4.1 核心抽象：`WorkspaceBackend`（名称可微调）

职责：**为 devshell 提供「当前工作区」的文件与 cwd 语义**，隐藏 Mode S / Mode P。

**Mode P 与「单一虚拟 FS」**：`GuestPrimaryBackend` 在 **VM 可用** 时以 **guest** 为落盘真源；在 **VM 不可用** 时仍可退化为 **同一工作区的进程内表示**（例如仅内存树或惰性重连前的缓存），builtin **始终只通过 `WorkspaceBackend`** 访问该树，**不**引入「先导出到宿主临时目录再操作」的路径。实现上允许在 **VM 恢复** 后与 guest **再对齐**（具体策略：阻塞、提示、或后台同步 — 可在 writing-plans 中定稿）。

建议方法族（示意）：

- `logical_cwd()` / `set_logical_cwd(&str) -> Result<(), …>`
- `resolve_guest_path(logical: &str) -> Result<PathBuf, …>`（Mode P）或 **N/A**（Mode S）
- `read_file` / `write_file` / `list_dir` / `mkdir` / `remove` / `exists` …
- **Rust 工具**：`run_rust_tool(program, args) -> Result<ExitStatus, …>`  
  - Mode S：现有 sandbox 或 γ **push → shell → pull**  
  - Mode P：γ **仅** `limactl shell --workdir <guest_cwd> -- program …`（**无** 工程树 push/pull）

**实现体**：

- **`MemoryVfsBackend`**：包装现有 `Rc<RefCell<Vfs>>` + 现有 `SessionHolder`（与今天一致）。
- **`GuestPrimaryBackend`**：持有 `GammaSession`（或更一般的 **`VmExecutionSession`**）+ **`GuestFsOps`** + **逻辑路径 ↔ guest 路径** 映射器。

### 4.2 与 `command::dispatch` 的关系

- 今天：dispatch 直接拿 **`&mut Vfs`**。
- 目标：dispatch 拿 **`&mut dyn WorkspaceBackend`**（或具体 enum），**builtin** 只通过后端接口访问工程树。
- **管道 / 重定向**：字节在 **阶段间** 传递；Mode P 下写 VFS 路径的重定向 → 转为 **guest 路径写**（或经临时宿主缓冲再上传 — **不推荐** 大文件，首期可限制单文件大小或仅支持小文件）。

### 4.3 `VmExecutionSession` 演进

- 今日：`run_rust_tool(vfs, …)` 内含 sync。
- 建议：拆出 **`run_rust_tool_guest_only(&self, guest_workdir, program, args)`**（Mode P），或 **`run_rust_tool`** 内根据 **`WorkspaceMode`** 分支。
- **`GuestFsOps`** 可作为 **γ** 上 `limactl` 的薄封装，**β** 上换 IPC 实现。

---

## 5. 路径模型

### 5.1 逻辑路径（用户可见）

- 保持现有 **Unix 风格逻辑路径**（如 `/projects/hello`），**不改变** 用户心智。
- **`cd` / 补全** 基于 **逻辑路径** 展示。

### 5.2 Guest 物理路径

- **`guest_mount`**（默认 `/workspace`）+ **逻辑 cwd 的 leaf**（与现有 `guest_dir_for_cwd_inner` **一致**）。
- 映射函数 **`logical_to_guest(logical_cwd, logical_path) -> guest_path`**：  
  - 规则：**与当前 push 布局一致**，避免「Mode P 与 Mode S 同一项目路径不一致」。

### 5.3 越界与校验

- 所有 guest 路径必须 **规范化为落在** `guest_mount/<leaf>/…` 下，**禁止** `..` 逃逸出工作区挂载树（在 **构造路径时** 检查）。

---

## 6. 环境变量与配置（建议）

| 变量 | 取值 | 语义 |
|------|------|------|
| **`DEVSHELL_VM_WORKSPACE_MODE`** | **`sync`**（默认） | Mode S：内存 VFS 权威 + push/pull |
| | **`guest`** | Mode P：guest 真源 |
| **`DEVSHELL_VM`** / **`DEVSHELL_VM_BACKEND`** | 现有 | **已决**：若 **`DEVSHELL_VM_WORKSPACE_MODE=guest`** 但 **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`/`auto`**（无可用 guest 后端），则 **强制按 Mode S（`sync`）** 运行，**不报错**退出；实现可 **`eprintln!`** 提示已降级（可选）。Mode P 仅在 VM 开启且后端为 **lima**（或未来 **beta**）时生效。 |

**`cfg(test)`**：未设置时等价 **`sync`** + **Host** sandbox，与今日一致。

---

## 7. 分阶段交付（与短文 §3 对齐并细化）

| Phase | 交付物 | 说明 |
|-------|--------|------|
| **0** | 上述 env + 文档 + **enum `WorkspaceMode`** 解析 | 无行为变化 |
| **1** | **`GuestFsOps` + γ 实现 + 单元测试（mock limactl）** | 可先不接入 dispatch |
| **1b** | **`GammaSession::run_rust_tool` 在 Mode P 分支跳过 sync** | 需会话知悉 mode 或上层传入 |
| **2** | **dispatch 改接 `WorkspaceBackend`**；Mode P 下 builtin 走 guest | 最大工作量 |
| **3** | 脚本、**Mode P 持久化新格式**（非 legacy `.dev_shell.bin`）、todo 策略 | 见 §9–§11 |
| **4** | β **`GuestFsOps`** | 与 IPC 草案对齐 |

---

## 8. 内置命令与管道（Mode P）

### 8.1 必须覆盖的 builtin（工程树相关）

`cd` `pwd` `ls` `mkdir` `touch` `cat` `echo`（写文件分支）`export-readonly`

**`export-readonly`（已决，2026-03-20 评审）**：**不作为向宿主文件系统导出**。Mode P 下它与其它工程树 builtin 一样，只作用于 **同一虚拟工作区**：在 **VM 运行** 时从 **guest** 视图产生只读分支/副本（语义与实现细节在实现阶段定：例如仅在逻辑路径下挂只读节点、或等价操作）；在 **VM 未运行** 时从 **当前进程内工作区表示** 只读分支。**禁止**将「导出到宿主临时目录」作为 Mode P 的默认或推荐语义。

### 8.2 管道

- 前一阶段 stdout 仍在 **宿主内存**；下一阶段若为写文件且目标在 guest：**整段缓冲写入 guest**（大文件 **warn** 或限制）。
- **`cargo` 管道**：保持现有「仅第一阶段可为 rust 工具」规则；Mode P 下 rust 阶段 **cwd** 为 **guest 路径**。

---

## 9. 脚本（`.dsh`）

- 与 REPL **共用** `WorkspaceBackend`。
- **Mode P** 下 **`source` 宿主文件**：保持从 **宿主** 读脚本内容（与今日一致），但脚本内 **文件操作** 仍走 guest。

---

## 10. 会话持久化（Mode P 与 `.dev_shell.bin`）

**已决（2026-03-20 评审）**：Mode P 要求持久化形态 **与 VM 侧虚拟文件系统一致**，因此 **不再使用** 当前 **`.dev_shell.bin`** 的 **文件格式**（该格式与 Mode S 的「整棵内存 VFS 树序列化」绑定，与 guest 真源模型不一致）。

**分模式约定**：

| 模式 | 持久化 |
|------|--------|
| **Mode S** | 继续沿用 **现有** `.dev_shell.bin`（及其实现），行为与今日一致。 |
| **Mode P** | **弃用** legacy `.dev_shell.bin` **格式**；采用 **新** 会话状态载体（文件名/扩展名、是否在 guest 内落盘、是否仅元数据等由 **writing-plans / 实现** 定稿），其语义必须与 **同一虚拟工作区**（§1.1）对齐，**不**再内嵌一套与 guest 脱节的整树快照编码。 |

**迁移与兼容**：从 Mode S 保存的 `.dev_shell.bin` **打开并切到 Mode P** 时，实现可定义为：仅恢复 **非工程树** 元数据（如 cwd、书签），工程树以 **guest / 后端** 为准；**不要求** 能无损把 legacy bin 中的文件树「写回」为新格式的一字节兼容 round-trip。

**文档**：须在 `design.md` / 用户说明中写明：**Mode P 会话文件 ≠ 现行 `.dev_shell.bin`**，避免用户混用旧工具解析新文件。

---

## 11. `todo` 与 `.todo.json`

**已决（2026-03-20 评审）**：采用 **A** — **`todo` builtin** 与 **`.todo.json`** 仍在 **宿主当前工作目录**（或 `todo_io` 既有约定路径）解析，与 **`xtask todo`** 及现有 **`todo_io`** 一致；与 guest 工程树 **解耦**，不因 VM 启停或 Mode P 迁移 todo 文件位置。

**未选路径（记录）**：**B** 将 `.todo.json` 映射到 **`/workspace/…`** 需挂载与可写策略，本期 **不采用**。

---

## 12. 测试策略

- **`GuestFsOps`**：**trait + mock**，不启动真 VM。
- **集成**：**可选** `#[ignore]` + 文档说明需 Lima；CI 仍以 **Mode S** 为主。
- **回归**：全量测试在 **默认 env** 下跑 **Mode S**，保证 **零 VM** 通过。

---

## 13. 风险摘要

- **性能**：路线 A 下 REPL 可能「钝」— 必须通过 **批量 API** 或 **B** 优化。
- **安全**：`GuestFsOps` 的 `sh -c` 构造必须 **参数化**，禁止字符串拼接用户输入为裸 shell。
- **规格冲突**：与 **2026-03-11** §3「VFS 权威 + 同步」**并列存在**时，须在 **`design.md`** 标明：**Mode S 遵循 03-11；Mode P 遵循本文**。

---

## 14. 评审通过后下一步（头脑风暴终端）

1. 开放决策 **已全部收敛**（§6、§8.1、§10、§11）；若正文需修订仍可在本文件直接改。  
2. **writing-plans（已完成）：** **`docs/superpowers/plans/2026-03-20-devshell-guest-primary-workspace.md`** — Sprint 0–7、文件锚点、P0–P4 映射。  
3. **禁止**在未通过评审前合并 Mode P 默认行为变更。

---

## 15. 开放决策（评审记录 — 均已决）

- [x] **§8.1** `export-readonly`：**已决** — 同一虚拟工作区内只读分支/视图；**不**导出到宿主；VM 开时用 guest，VM 关时用进程内工作区表示（见 §1.1、§4.1、§8.1）。  
- [x] **§10** 持久化：**已决** — Mode P **不沿用** legacy `.dev_shell.bin` 格式；新格式与虚拟工作区 / guest 一致；Mode S 仍用现行 bin（见 §10）。  
- [x] **§11** todo：**已决** — **A**：宿主 cwd / `todo_io` 约定；与 guest 解耦（见 §11）。  
- [x] **§6** `host` backend 遇 `guest` mode：**已决** — **强制按 Mode S（`sync`）**，不报错（见 §6 表格）。

---

*本文件遵循 `.cursor/skills/superpowers/skills/brainstorming/SKILL.md`：设计先行，实现待 writing-plans 与用户批准。*
