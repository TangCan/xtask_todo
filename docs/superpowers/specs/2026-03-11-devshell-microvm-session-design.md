# Devshell Rust 隔离：会话级 microVM 方案（γ → β）

**日期**：2026-03-11  
**状态**：头脑风暴已定稿（用户确认同步模型与实现路线）  
**范围**：将当前「VFS 导出 → 宿主目录执行 `cargo`/`rustup` → 同步回 VFS」升级为 **三平台真 VM**、**会话级生命周期**、**guest 虚拟盘承载工程树**；工具链来自 **宿主只读挂载**；工程树与 VFS 采用 **会话起止全量 + 每道 rust 命令前后增量** 同步。  
**实现路线**：**Phase 0（γ）** 用现有 CLI 工具验证端到端；**Phase 1+（β）** 收敛为 **侧车守护进程 + 稳定 IPC**。

---

## 1. 目标与非目标

### 1.1 目标

- **三平台**（Linux / macOS / Windows）在 **真 VM**（独立内核）内执行 `rustup` / `cargo`，与当前 devshell **会话**同寿命：进入 REPL（或 `dev_shell` 进程）起 VM，**会话内复用**，`exit` 关闭。
- **工程树**：会话开始时 VFS cwd 子树 **写入 guest 可写盘**（如 `/workspace`）；**exit** 时 **整棵拉回 VFS**。
- **与 REPL 一致**：采纳 **「2 + 同步钩子」**（见 §3），避免「只 exit 同步」导致 REPL 与编译互不可见。
- **工具链（宿主挂载）**：将宿主 `RUSTUP_HOME`、`CARGO_HOME`（或文档约定的等价路径）**只读**挂入 guest；guest 内 **不得写** 工具链树。
- **可回退**：VM 不可用时（无 KVM、权限不足等）可 **明确报错** 并支持 **`DEVSHELL_VM=off`**（或配置项）回退到现有 **宿主目录 sandbox**（当前 `sandbox.rs` 行为）。

### 1.2 非目标（首期）

- 不在 guest 内重复实现完整 rustup 镜像分发（**C** 已选：宿主挂载）。
- 不把通用「任意宿主 shell 命令」纳入 VM（仍仅 `rustup`/`cargo` 管线，与 `2026-03-20-devshell-rust-vm-design.md` 一致）。
- **γ 阶段**不承诺单一依赖、也不承诺 API 稳定；以 **验证数据流与 UX** 为主。

---

## 2. 平台与后端映射（概念）

| 宿主 | Phase 0（γ）建议载体 | Phase 1+（β）目标 |
|------|----------------------|------------------|
| **Linux** | Lima / QEMU+KVM、或同类「一条命令起 Linux VM」工具 | 侧车进程 + Firecracker 或 QEMU（实现选型在 β 规格中定） |
| **macOS** | Lima（Virtualization.framework）、Multipass 等 | 侧车 + Apple Virtualization（或继续委托 Lima 作实现细节） |
| **Windows** | WSL2 / Hyper-V 管理 CLI / Multipass（按团队可装性选一种） | 侧车 + WHP/Hyper-V 抽象 |

**命名**：规格层使用后端 id：`linux-*`、`macos-*`、`windows-*`；γ 与 β 可映射到不同具体驱动，但 **IPC 契约与同步语义** 在 β 冻结。

---

## 3. 工程树与 VFS 同步（用户已确认「是」）

### 3.1 原则

guest 盘上 **`/workspace`**（名称可配置）为 **cargo 所见工程根**；内存 **VFS** 仍为 REPL 权威视图，二者通过固定节拍对齐。

### 3.2 节拍

1. **会话开始（VM Ready）**  
   - **全量 push**：VFS cwd 子树 → guest `/workspace`。

2. **每次 `rustup` / `cargo` 调用前**  
   - **增量 push**：VFS → `/workspace`（使 REPL 中的编辑对本次编译可见）。

3. **每次 `rustup` / `cargo` 调用后**  
   - **增量 pull**：`/workspace` → VFS（`target/`、生成源码等回 VFS）。

4. **会话结束（`exit` / EOF /进程退出）**  
   - **全量 pull**：兜底一致性，再 **关闭 VM**。

### 3.3 冲突与一致性（首期简化）

- 默认策略：**后写覆盖**（以最后一次成功同步为准），并可在 stderr 打 **warn**（可选）。
- 不在首期做交互式三方合并。

---

## 4. 实现路线（用户已确认：先 γ 再 β）

### 4.1 Phase 0 — γ（编排现有工具）

- **目的**：验证 §3 同步语义、会话绑定、三平台「能跑通一条 `cargo build`」；收集延迟与失败模式。
- **做法**：  
  - `cargo-devshell`（或薄封装）通过 **子进程** 调用选定 CLI：创建/启动 VM、挂载/同步目录、在 guest 内执行 `cargo`（如 `limactl shell` / `ssh` / 等价物）。  
  - **配置**：镜像名、CLI 路径、后端选择等用 **环境变量或 TOML**，不保证跨版本 CLI 稳定。
- **交付**：文档（prerequisites、已知限制）、最小自动化测试（以 **mock / Linux CI only** 为主，其余手工矩阵）。

### 4.2 Phase 1+ — β（侧车守护）

- **目的**：收敛依赖、稳定协议、便于测试与签名分发。
- **做法**：  
  - 独立二进制 **`devshell-vm`**（或同名服务）：负责 VM 生命周期、挂载、在 guest 内执行命令。  
  - **`cargo-devshell`** 仅通过 **本地 IPC**（Unix socket / Windows named pipe）发送：会话创建、push/pull、exec、shutdown。  
  - 协议版本化（major/minor）、超时与取消（后续规格扩展）。
- **与 γ 的关系**：γ 中验证的 **同步节拍与错误语义** 原样保留；仅 **执行后端** 从「shell 拼 CLI」换为 **守护进程内嵌驱动**。

### 4.3 不推荐首期路径

- **α（全在 `xtask-todo-lib` 内嵌三平台 VM API）**：可作为 β 之后的优化方向，**不作为 γ/β 前置条件**。

---

## 5. 与现有代码的衔接

- **`sandbox.rs`**：保留 **宿主目录** 路径作为 **`DEVSHELL_VM=off`** 与 **γ/β 失败回退** 的实现基础。  
- **VM 模式开启时**：`run_rust_tool`（及等价入口）改为：确保会话 VM → §3 同步 → **guest 内 exec** → 同步 → 返回状态码。  
- **文档**：`docs/design.md` §2.5、`docs/requirements.md`、`docs/dev-container.md` 需在实现阶段更新，区分 **宿主 sandbox** / **VM 会话** / **回退**。

---

## 6. 错误处理（摘要）

| 场景 | 行为 |
|------|------|
| VM 启动失败 | 清晰 stderr 信息；若允许回退则落宿主 sandbox，否则非零退出 |
| 同步失败 | 本次命令失败；不静默丢数据 |
| guest 内 `cargo` 非零 | 与当前一致映射为 **工具链非零退出**（如 `RustToolNonZeroExit`），并仍尝试 **pull**（若设计为「失败也 pull」需在实现计划中写死） |
| 工具链挂载不可读 | 启动前检测；失败并提示路径 |

---

## 7. 测试策略（摘要）

- **单元**：路径拼接、同步 diff 逻辑（可脱离 VM）。  
- **集成**：Linux CI 可选装一种 γ 工具链跑冒烟；macOS/Windows 以 **发布前手工清单** 或专用 runner 记录。  
- **回归**：`DEVSHELL_VM=off` 与现有 `sandbox` 单测保持通过。

---

## 8. 后续步骤（流程）

1. 用户审阅本文档，确认无歧义。  
2. 使用 **writing-plans** 技能编写 **`2026-03-11-devshell-microvm-session-impl-plan.md`**（分 γ / β 里程碑与可交付物）。  
3. 实现时按 plan 分 PR，避免一次性超大变更。

---

## 9. 决策记录（头脑风暴摘要）

| 项 | 选择 |
|----|------|
| VM 形态 | 轻量 microVM 思路（具体引擎 γ/β 分阶段） |
| 平台 | Linux / macOS / Windows 均需真 VM（多后端） |
| 生命周期 | 与 devshell **会话**绑定 |
| 工具链 | **C**：宿主 `RUSTUP_HOME` / `CARGO_HOME` **只读**挂 guest |
| 工程树 | **2 + 钩子**：盘在 guest；起止全量 + 每 rust 命令前后增量 |
| 实现顺序 | **先 γ（编排现有工具）验证，再 β（侧车）收敛** |
