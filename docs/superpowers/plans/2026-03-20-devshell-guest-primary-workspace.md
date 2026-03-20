# Plan：Devshell guest 真源工作区（Mode P）

**总览规格：** [2026-03-20-devshell-vm-primary-guest-filesystem.md](../specs/2026-03-20-devshell-vm-primary-guest-filesystem.md)  
**详细设计：** [2026-03-20-devshell-guest-primary-design.md](../specs/2026-03-20-devshell-guest-primary-design.md)  
**状态：** 设计 **§15** 已全部收敛；本文件为 **writing-plans** 级任务分解。**默认仍为 Mode S**；Mode P 按阶段启用，不提前改默认 env。

## 前置（头脑风暴流程）

- [x] 设计文档 **§15** 开放决策已全部勾选（含 §6 **强制 sync**）
- [x] 本计划已拆为 **sprint 级 + 文件级** 任务（见下文）
- [ ] 实现前再扫一眼设计 **§1.2 成功标准** 作为每阶段 PR 验收清单

---

## 架构锚点（代码现状）

| 区域 | 路径（`crates/todo/src/devshell/`） | 备注 |
|------|-------------------------------------|------|
| VM 配置 | `vm/config.rs` | `VmConfig::from_env`，`DEVSHELL_VM*` |
| γ 会话 + push/pull | `vm/session_gamma.rs` | `VmExecutionSession::run_rust_tool` 内 `push_incremental` / `pull_workspace_to_vfs` |
| 会话分发 | `vm/mod.rs` | `SessionHolder`，`try_from_config`，`run_rust_tool` |
| 同步算法 | `vm/sync.rs` | `push_incremental`，`pull_workspace_to_vfs` |
| Builtin / 管道 | `command/dispatch.rs` | `ExecContext { vfs, vm_session }`；γ/β+Guest 时文件 builtin / 重定向走 **`GuestFsOps`**（γ：`GammaSession`；β：IPC **`guest_fs`**）；**`workspace/io.rs`** 共享读路径 |
| REPL / 脚本 | `repl.rs`，`script/exec.rs` | `Rc<RefCell<Vfs>>` + `SessionHolder`；**`read_script_source_text`**（§9）；**`session_store.rs`** — guest-primary **`.session.json`**（§10） |
| 序列化 | `serialization.rs` | `.dev_shell.bin` ↔ `Vfs`（Mode S） |

Mode P 的核心增量：**有效工作区模式**（env 解析 + 与 VM 组合降级）、**γ 上跳过工程树 sync**、**长期** `WorkspaceBackend` + `GuestFsOps` 替换 dispatch 直读 `Vfs`。

---

## Sprint 0 — Phase 0：`WorkspaceMode` + 强制 sync（无行为变化）

**目标：** 引入 `DEVSHELL_VM_WORKSPACE_MODE` 与 **`effective_workspace_mode`**（`guest` + 无 γ → **`sync`**），默认 **`sync`**；**不**改变现有 REPL 路径（有效模式恒为 `sync` 直至后续 sprint 接线）。

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S0.1 | 常量 `ENV_DEVSHELL_VM_WORKSPACE_MODE`；枚举 `WorkspaceMode { Sync, Guest }`（或 `sync`/`guest` 字符串解析） | `vm/config.rs` | 导出常量；`cfg(test)` 未设置时等价 `Sync`（与设计 §6、`cfg(test)` 段一致） |
| S0.2 | `fn workspace_mode_from_env() -> WorkspaceMode`；`VmConfig` 增加 `workspace_mode: WorkspaceMode` 或在独立 `EffectiveWorkspaceConfig` 中组合 | `vm/config.rs`（或新 `vm/workspace_mode.rs`） | 单元测试：`guest` / `sync` / 未设置 |
| S0.3 | **`effective_workspace_mode(vm: &VmConfig) -> WorkspaceMode`**：`WorkspaceMode::Guest` **仅当** `vm.enabled && !vm.use_host_sandbox() && backend 为 lima 或 beta`（与设计 §6 一致）；否则 **`Sync`**；可选首次降级时 `eprintln!`（设计允许） | `vm/config.rs` | 表驱动测试：`DEVSHELL_VM=off` + `WORKSPACE_MODE=guest` → `Sync`；`BACKEND=host` + `guest` → `Sync`；`lima` + `guest` + `enabled` → `Guest`（Unix mock backend 或构造 `VmConfig`） |
| S0.4 | `pub use` / 文档字符串指向设计 §6 | `vm/mod.rs` | `cargo test -p xtask-todo-lib vm::config` 通过 |
| S0.5 | 用户文档 | `docs/devshell-vm-gamma.md`，`docs/design.md`（§1.4 已有可补一行链到 env 表） | 说明变量名、默认值、`guest`+host 降级 |

**合并策略：** 单独 PR；全库行为应与合并前一致（有效模式未接入 `GammaSession` 前恒表现为 Mode S）。

**落地记录：** S0.1–S0.4 已在 **`vm/config.rs`** / **`vm/mod.rs`** 实现；S0.5：`docs/devshell-vm-gamma.md` 环境表已增一行；`docs/design.md` §1.4 已含 Mode P 说明，未重复整表。

**Sprint 1 落地记录：** **`vm/guest_fs_ops.rs`** — `GuestFsOps`、`GuestFsError`、`MockGuestFsOps`（单测）；**Unix** `LimaGuestFsOps`（`list_dir`/`read_file`/`mkdir`/`remove`/`write_file` 经 `limactl shell` + `dd`）；路径辅助 `normalize_guest_path`、`guest_path_is_under_mount`；**`guest_project_dir_on_guest`**（与 γ 布局一致）；**`GammaSession`** 新增 `limactl_shell_output`、`limactl_shell_stdin`、`guest_mount()`。尚未接入 dispatch / REPL。

**Sprint 2 落地记录：** **`devshell/workspace/backend.rs`** — `WorkspaceBackend`、`WorkspaceBackendError`、`MemoryVfsBackend`（`Rc<RefCell<Vfs>>`）、`GuestPrimaryBackend`（`Box<dyn GuestFsOps>` + `logical_path_to_guest`）；**`vfs::resolve_path_with_cwd`** 抽出供逻辑路径解析；**`MemoryVfsBackend::remove`** 暂为 `Unsupported`（待 `Vfs::remove`）；**`GuestPrimaryBackend::run_rust_tool`** 暂为 `Unsupported`（Sprint 3）。尚未接入 dispatch。

**Sprint 3 落地记录：** **`DEVSHELL_VM_WORKSPACE_MODE=guest`** 且 **`VmConfig::workspace_mode_effective() == Guest`** 时，**γ** `GammaSession` 与 **β** `BetaSession` 在 **`run_rust_tool` / `shutdown`** 中 **不再** `push_incremental` / `pull_workspace_to_vfs`（γ）；β 同时跳过 IPC **`sync_request`** push/pull。字段 **`sync_vfs_with_workspace`**；**`GammaSession::syncs_vfs_with_host_workspace`** 供诊断。

**Sprint 4 落地记录：** **`command/dispatch.rs`** — **`impl GuestFsOps for GammaSession`**（`LimaGuestFsOps` 委托同一实现）；**`SessionHolder::guest_primary_gamma_mut`**；**`workspace_read_file` / `write` / `list_dir` / `mkdir` / `touch`** 在 γ+Guest 时经 **`logical_path_to_guest`** + **`GuestFsOps`**；**`export-readonly`** 在 guest-primary 下返回 **`ExportGuestPrimaryNotSupported`**（待 §8.1 完整语义）。**ExecContext** 仍为 **`vfs` + `vm_session`**（未改为 `dyn WorkspaceBackend`，与 **`MemoryVfsBackend`** 行为等价拆分）。**Sprint 7**：β+Guest 经 **`SessionHolder::guest_primary_fs_ops_mut`** + **`impl GuestFsOps for BetaSession`**（侧车 **`guest_fs`**）。见 **`docs/devshell-vm-gamma.md`**。

**Sprint 5 落地记录：** **`completion.rs`** — **`list_dir_names_for_completion`**；**`DevShellHelper`** 持有 **`vm_session`**，路径补全在 γ guest-primary 时 **`GuestFsOps::list_dir`**，失败则回退 **Vfs**；**`todo`** 加入命令名补全。**`dispatch.rs`** — 管道非末段缓冲上限 **`PIPELINE_INTER_STAGE_MAX_BYTES`**（16 MiB）、**`BuiltinError::PipelineInterStageBufferExceeded`**、单元测试边界；**`execute_pipeline`** 文档指向设计 §8.2。

---

## Sprint 1 — Phase 1：`GuestFsOps`（trait + γ 桩 + mock）

**目标：** 抽象 guest 文件原语，**不**接入 `dispatch`；为后续 `GuestPrimaryBackend` 铺路。

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S1.1 | `trait GuestFsOps`：`list_dir`、`read_file`、`write_file`、`mkdir`、`remove`、错误类型（与设计 §4 方法族对齐；签名可迭代） | 新文件 `vm/guest_fs_ops.rs` 或 `devshell/workspace/guest_fs_ops.rs` | `Send`/`Sync` 需求按调用方定 |
| S1.2 | `LimaGuestFsOps { session: &GammaSession 或 limactl + instance + guest_mount }`：内部用 `limactl shell` + **参数化** `sh -c`（设计 §13 安全）实现 **至少** `list_dir` + `read_file`（MVP） | `vm/guest_fs_ops.rs` + 从 `session_gamma.rs` 复用 `guest_dir_for_cwd_inner`、`limactl_shell` 模式 | Unix 上可选 `#[ignore]` 集成测试；核心逻辑用 **mock `limactl`** 或注入 `Command` 替身（测试友好） |
| S1.3 | `MockGuestFsOps`（内存 HashMap 树）用于单元测试 | `guest_fs_ops.rs` 的 `#[cfg(test)]` 或 `tests/` | 不启动 VM 的 `cargo test` 覆盖 trait |

**合并策略：** trait + mock 先合并；Lima 实现可分第二个 PR，但需有 mock 通过测试。

---

## Sprint 2 — Phase 1b：`WorkspaceBackend` 骨架（仍可选未接 dispatch）

**目标：** 定义 `WorkspaceBackend` + `MemoryVfsBackend`（包装现有 `Vfs` + cwd）；`GuestPrimaryBackend` **占位**（持 `GuestFsOps` + 逻辑路径映射 §5），可先 **panic/unimplemented** 于未用方法，直至 Sprint 4。

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S2.1 | `trait WorkspaceBackend`：`logical_cwd`、`set_logical_cwd`、`read_file`、`write_file`、`list_dir`、`mkdir`…；`run_rust_tool` 委托或返回 `NotImplemented` 直到接线 | 新 `devshell/workspace/backend.rs`（或 `vm/workspace_backend.rs`） | 编译通过；仅单元测试构造 `MemoryVfsBackend` |
| S2.2 | `MemoryVfsBackend`：`Rc<RefCell<Vfs>>` + 与现有逻辑路径一致 | 同上 | 与现有 `Vfs` 测试兼容 |
| S2.3 | `GuestPrimaryBackend` 结构体字段：`GuestFsOps`、`guest_mount`、`lima_instance`、逻辑 cwd 状态机（§5 `logical_to_guest`） | 同上 | 单元测试：路径映射与 §5.3 越界 |

---

## Sprint 3 — Phase 1c：γ `run_rust_tool` 在 **有效 Guest** 时跳过 push/pull

**目标：** `GammaSession::run_rust_tool`（或 `SessionHolder` 包装层）在 **`effective_workspace_mode == Guest`** 时 **不**调用 `push_incremental` / `pull_workspace_to_vfs`（工程树）；**shutdown** 行为需单独定：无 pull 时是否仍 warning — 建议与设计 §1.1 一致：**不**强制 pull（文档写明 Vfs 可能 stale for rust-only views）。

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S3.1 | 将 **`effective_workspace_mode`** 传入 `SessionHolder` 或 `GammaSession::new` / 每次 `run_rust_tool`（优先 **构造期** 注入 `WorkspaceMode`，避免每调用读 env） | `vm/mod.rs`，`vm/session_gamma.rs` | `Guest`：**无** `push_incremental`/`pull`；`Sync`：保持现状 |
| S3.2 | `try_session_rc` / REPL 入口：用 `VmConfig::from_env()` + `effective_workspace_mode` 组装 | `vm/mod.rs`，`repl.rs` / `bin` 入口 | 无 Lima 的 `cargo test` 仍走 Host + `Sync` |
| S3.3 | 文档：Mode P 下 `cargo` 后 Vfs 不自动同步 | `devshell-vm-gamma.md` | 与设计一致 |

**风险：** 在 **未**完成 Sprint 4（dispatch 接 guest）前，用户开 `guest` + γ 时 REPL `ls` 仍读 **Vfs**，`cargo` 改 guest — **预期不一致**。缓解：**文档标注「实验性」** 或 feature gate **`DEVSHELL_VM_WORKSPACE_MODE=guest`** 直至 Sprint 4；或在 Sprint 3 合并说明 **仅给 dogfood rust 路径**。

---

## Sprint 4 — Phase 2：`dispatch` → `WorkspaceBackend` + builtin 工程树走 guest

**目标：** `ExecContext` 携带 `&mut dyn WorkspaceBackend`（或 enum）；Mode **Guest** + γ 时文件 builtin 走 **`GuestFsOps`**；重定向写文件 → guest（§8.2）。

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S4.1 | `run_builtin` / `execute_pipeline` 签名迁移；`MemoryVfsBackend` 路径下行为与现有一致（回归测试） | `command/dispatch.rs`，`command/mod.rs`，`types.rs` | 全 `devshell` 测试绿 |
| S4.2 | `cd`/`pwd`/`ls`/`cat`/… 分支到 `GuestPrimaryBackend` | `dispatch.rs` 或拆 `builtin_fs.rs` | 与设计 §8.1 列表对齐 |
| S4.3 | `rustup`/`cargo` 已 Sprint 3 跳过 sync；确认与 backend cwd 一致 | `dispatch.rs` | 与 §1.2 成功标准 1–2 对齐 |

**估计工作量最大**；可拆多个 PR：先接 backend 再逐个 builtin。

---

## Sprint 5 — Phase 2b：补全、管道、重定向（Mode P）

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S5.1 | `completion.rs`：Guest 下列目录经 `GuestFsOps` 或缓存 | `completion.rs` | TTY 测试或手动脚本 |
| S5.2 | 管道大文件缓冲策略（§8.2 warn/limit） | `dispatch.rs` | 文档 + 测试阈值 |

---

## Sprint 6 — Phase 3：脚本、**新会话格式**、todo

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S6.1 | `.dsh` / `script/exec.rs`：与 REPL 同 `WorkspaceBackend`（§9） | `script/exec.rs`，`repl.rs` | 脚本测试 |
| S6.2 | **新** 会话持久化格式（**非** `.dev_shell.bin`）；仅 Mode P 写入；Mode S 不变（§10） | `serialization.rs` + 新模块 `session_store.rs`（名待定） | 迁移路径：旧 bin → 打开仅恢复元数据 |
| S6.3 | **todo**：确认仍 `todo_io` 宿主路径（§11 **A**）— 一般 **无代码变更**，仅回归测 | `todo_io.rs`，`todo_builtin.rs` | 与 `xtask todo` 一致 |

**Sprint 6 落地记录：** **`workspace/io.rs`** — **`read_logical_file_bytes`** / **`WorkspaceReadError`**；**`logical_to_guest_abs`**（Unix）；**`dispatch`** 读路径委托。**`script::read_script_source_text`**：`exec_source` + REPL **`source` / `.`** 与 builtin 同序（guest → VFS → 宿主）。**`SessionHolder::is_guest_primary()`**；**`save_on_exit`** 在 guest-primary 时 **跳过** legacy **`.dev_shell.bin`**，写入 **`session_store`** JSON（`.session.json`）。**`session_store.rs`** — **`devshell_session_v1`** 元数据；启动时 **`apply_guest_primary_startup`**。**`export-readonly`**（Mode P）：**`workspace/guest_export.rs`** — guest → VFS **`/.__export_ro_*`** 镜像。**`todo_io`** 模块注释 §11。

---

## Sprint 7 — Phase 4：β `GuestFsOps` + 文档回写

| # | 任务 | 文件 / 位置 | 验收 |
|---|------|-------------|------|
| S7.1 | `BetaSession` 实现 `GuestFsOps` 或桥 IPC（设计 Phase 4） | `vm/session_beta.rs` | `feature beta-vm` CI |
| S7.2 | `docs/requirements.md`、`test-coverage.md` 补充 Mode P / env | 根 `docs/` | 与实现一致 |

**Sprint 7 落地记录：** **`impl GuestFsOps for BetaSession`**（JSON **`guest_fs`**：`list_dir` / `read_file` / `write_file` / `mkdir` / `remove`，`content_base64`）；**`SessionHolder::guest_primary_fs_ops_mut`**、**`is_guest_primary()`**；**`devshell-vm`** 桩实现 **`guest_fs`**；**`workspace/io.rs`** / **`dispatch`** / **`completion`** 统一走 γ 或 β guest-primary。**`docs/requirements.md`** §5 / §1.1（Mode P）、**`test-coverage.md`** β 说明。

---

## 依赖顺序（简图）

```
S0 ─┬─> S1 ─> S2 ─┬─> S3 ─> S4 ─> S5
    │             └──────────────> S6（可与 S5 部分并行，视人手）
S4/S6 ─────────────────────────────> S7（β 可晚于 γ 核心）
```

---

## 原 P0–P4 勾选（与 Sprint 映射）

- [x] **P0** → **Sprint 0**（`WorkspaceMode`、`DEVSHELL_VM_WORKSPACE_MODE`、`VmConfig::workspace_mode_effective`；**尚未**接入 `GammaSession`/dispatch，行为与此前一致）
- [x] **P1** → **Sprint 1**（trait + Mock + `LimaGuestFsOps`；**尚未**接入 dispatch）
- [x] **P1b** → **Sprint 2**（`WorkspaceBackend` 骨架；**尚未**接入 dispatch）
- [x] **P1c** → **Sprint 3**（γ/β 在 **Guest** 有效模式下跳过工程树 push/pull；**尚未**接入 dispatch）
- [x] **P2** → **Sprint 4**（γ guest-primary：dispatch 文件类 builtin + 重定向；`export-readonly` 占位错误；β 未接）
- [x] **P2b** → **Sprint 5**（γ guest-primary 路径补全；管道阶段内存上限 16MiB）
- [x] **P3** → **Sprint 6**（`read_script_source_text`；guest-primary 跳过 legacy bin 退出保存；**`session_store`** 文档桩；**todo_io** §11 注释）
- [x] **P4** → **Sprint 7**（β `GuestFsOps` + `guest_fs` IPC；`docs/requirements.md` / `test-coverage.md` Mode P）

---

## 回滚与标志

- 任意阶段：**unset `DEVSHELL_VM_WORKSPACE_MODE`** 或设为 **`sync`** → Mode S。
- **`guest` + host**：自动 **`sync`**（设计 §6），无需用户改 env。
