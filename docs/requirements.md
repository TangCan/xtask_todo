# 项目需求说明（Requirements）

描述 **xtask_todo** 工作区对外能力、用户可见行为与验收要点。功能变更时请同步更新本文档。

---

## 1. 概述

| 项 | 说明 |
|----|------|
| **名称** | xtask_todo |
| **目标** | 可复用的 **Todo** 领域库（`xtask-todo-lib`）、**`cargo xtask`** 工作流 CLI，以及可选的 **devshell**（内置语言、管道、**`rustup`/`cargo`** 与 VM 协作）。 |
| **Workspace** | `crates/todo`（库，crates.io 发布）、`xtask`（`publish = false`）。 |
| **入口** | `cargo xtask …`；`cargo run -p xtask-todo-lib --bin cargo-devshell`（devshell）。 |

### 1.1 Devshell 工作区：Mode S 与 Mode P

| 模式 | 条件（摘要） | 工程树真源 | 工具链 |
|------|----------------|------------|--------|
| **S（sync）** | 默认；或未进入 guest 有效组合 | 进程内 **VFS**；与 γ Lima 协作时在 **`rustup`/`cargo` 等**前后对 **宿主工作区目录** **`push_incremental` / `pull_workspace_to_vfs`** | 默认 **宿主**临时目录 **sandbox** |
| **P（guest-primary）** | `DEVSHELL_VM_WORKSPACE_MODE=guest` 且 VM 可用、后端 **`lima`** 或 **`beta`** | **Guest** 挂载树（与 **`limactl shell`** 内同一视图）；**`GuestFsOps`** | **`rustup`/`cargo`** 在 **guest** 内执行 |

**持久化（共同约定）**

- Guest **根文件系统**由 Lima 实例目录下的 **`qcow2`**（如 **`diffdisk`**）承载；**工程树**在典型配置下为 **挂载进 guest 的宿主目录**，与 **`/workspace/…`** 对齐。
- **`logical_cwd`** 等会话字段**仅**写入 **逻辑工作区树内**的 JSON，例如 **`${DEVSHELL_VM_GUEST_WORKSPACE}/.cargo-devshell/session.json`**（默认 **`/workspace/.cargo-devshell/session.json`**）；宿主侧与挂载对齐为 **`${DEVSHELL_WORKSPACE_ROOT}/.cargo-devshell/session.json`**。
- 细节与环境变量见 **`docs/devshell-vm-gamma.md`**；Mode P 行为见 **`docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md`**。
- **会话 JSON（guest-primary / Mode P）**：`format` 字段为 **`devshell_session_v1`**（见实现 **`session_store::GuestPrimarySessionV1`**）；**`logical_cwd`**、**`saved_at_unix_ms`** 等字段与 **`docs/design.md`** 一致。

### 1.2 平台与构建目标

| 维度 | 说明 |
|------|------|
| **`xtask-todo-lib` / `cargo install`** | **Linux、macOS、Windows（MSVC）** 均可编译与安装；**`rustyline`** 为通用依赖，REPL 在 Windows 上可用。详见 **`crates/todo/README.md`**（含 Windows 安装说明）。 |
| **VM / Lima / β / Mode P / Linux mount-namespace sandbox** | **γ（Lima）** 以 **Unix** 为主。**Windows** 未设置 **`DEVSHELL_VM_BACKEND`** 时默认 **`beta`**（库默认 **`beta-vm`**）：通过 **Podman Machine** 运行侧车 **`devshell-vm`**（**JSON 行**协议，默认 **`DEVSHELL_VM_SOCKET=stdio`**），在 **`session_start` 的 `staging_dir`** 上对 **`rustup`/`cargo`** 做 **`exec`**（真实子进程；OCI 镜像含 **`cargo`**）。方式：**宿主编译的 Linux ELF**（`podman machine ssh`）或 **`podman run` + GHCR 镜像**（无 ELF 时自动拉取）。**宿主工作区根**解析见 **§5.2**（可为 **`cargo metadata` workspace**、**`DEVSHELL_VM_WORKSPACE_PARENT`** 或 **`…/cargo-devshell-exports/vm-workspace/<实例>/`** 等）。也可用 **`DEVSHELL_VM=off`** / **`DEVSHELL_VM_BACKEND=host`** 仅用宿主沙箱。详见 **`docs/devshell-vm-windows.md`**。 |
| **交叉编译自检** | 仓库 **pre-commit**（见 **§4**、**§7.2**）对 **`xtask-todo-lib`** 执行 **`x86_64-pc-windows-msvc` 目标 `cargo check`**，避免仅 Linux 开发时引入不可在 Windows 上编译的代码。 |

---

## 2. 能力范围（摘要）

| 能力域 | 说明 |
|--------|------|
| Todo CRUD、过滤、排序、可选字段、搜索、统计、导入导出、重复任务 | 库 + `cargo xtask todo` |
| `--json`、`--dry-run`、约定退出码 | 见 §6 |
| `todo init-ai` | 生成 AI 命令/技能文件 |
| xtask：`fmt`、`clippy`、`coverage`、`git`、`gh`、`run`、`clean`、`publish` 等 | 见 §4 |
| Devshell：VFS、内置命令、管道、重定向、脚本、`source`、Tab 补全 | 见 §5 |
| VM：`rustup`/`cargo` — Unix 经 **Lima（γ）** 或回退宿主 sandbox；**Windows** 经 **β + Podman**（侧车 **`exec`**）或回退宿主 sandbox | 见 **`docs/devshell-vm-gamma.md`**、**`docs/devshell-vm-windows.md`** |

**当前不承诺**：HTTP API、多用户权限、`.todo.json` 自动迁移流水线、内核级强隔离、多进程并发写同一文件的强保证。

---

## 3. Todo 领域（库 + CLI）

### 3.1 用户故事与验收

- **创建**：有效标题则分配 id 并持久化；非法标题 → 退出码 **2**。
- **列表**：支持过滤、排序；空列表可接受。
- **完成 / 删除**：非法或不存在 id → **退出码 3**（或约定幂等）。
- **时间**：`created_at`、`completed_at`；列表相对时间展示与实现一致。
- **长期未完成**：TTY 下 **7 天**阈值视觉区分（`AGE_THRESHOLD_DAYS`）；非 TTY 不着色。
- **单条 / 更新 / 搜索 / 统计 / 导入导出 / 重复规则**：与实现一致；**`complete`** 支持 **`--no-next`**（不生成下一重复实例）。

### 3.2 `cargo xtask todo` 规格

**全局选项**

| 选项 | 行为 |
|------|------|
| `--json` | 成功/失败均为可解析 JSON。 |
| `--dry-run` | 修改类命令仅预览，不写 **`.todo.json`**。 |

**子命令（要点）**

| 子命令 | 说明 |
|--------|------|
| `add` | 可选 `--description`、`--due-date`、`--priority`、`--tags`、重复相关选项。 |
| `list` | `--status`、`--priority`、`--tags`、日期范围、`--sort`。 |
| `show` / `update` / `complete` / `delete` / `search` / `stats` | 见 `--help`。 |
| `export` / `import` | JSON/CSV；`import` 可选 `--replace`。 |
| `init-ai` | `--for-tool`、`--output`。 |

**数据约定**

- 数据文件默认 **`.todo.json`**（与 devshell 内 **`todo`** 子集共用）。
- 任务 id 为正整数，**0 非法**；日期 **`YYYY-MM-DD`**。

---

## 4. 其他 `cargo xtask` 子命令

| 子命令 | 说明 |
|--------|------|
| `run` / `fmt` / `clippy` / `coverage` / `clean` | 开发者任务，行为以 xtask 实现为准。 |
| `gh log` | 依赖宿主 **`gh`**。 |
| `git add` / `git pre-commit` / `git commit` | 暂存与带检查的提交；**`git pre-commit`** 与 **`cargo xtask git pre-commit`** 执行 **`.githooks/pre-commit`**：`cargo fmt --check`、暂存 **`.rs` 行数 ≤500**、**`cargo clippy`**（pedantic/nursery、**`-D warnings`**）、**`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`**（与 **CI** 一致）、**`cargo test`**、**`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**（需 **`rustup target add x86_64-pc-windows-msvc`**）。 |
| `publish` | 见 **`docs/publishing.md`**。 |
| **`acceptance`** | 按 **`docs/acceptance.md`** 运行可自动化验收（`cargo test` 各包、**NF-1/2/6** 文件检查、**NF-5/D8** Windows **`cargo check`** 等），并生成 **`docs/acceptance-report.md`**（可用 **`-o`** / **`--stdout-only`**）；无法自动化的项在报告中列为人工/环境。 |

---

## 5. Devshell（`cargo-devshell`）

### 5.1 目标

在 **Mode P** 且 VM 就绪时（**Unix**：**γ（Lima）**；**Windows**：**β（Podman + 侧车）**），REPL 对**工程树**的操作与在 **guest 挂载树**（默认 **`/workspace/…`**）一致；经 **`GuestFsOps`** / β IPC。降级为 **Mode S** 时使用内存 VFS + push/pull（与 γ 协作时另含 **`push_incremental` / `pull_workspace_to_vfs`**）。

### 5.2 工作区路径（宿主 ↔ guest）

- **宿主工作区根**（与 γ 挂载 / β **`session_start` 的 `staging_dir`** 对齐）：由实现 **`workspace_parent_for_instance`** 解析，顺序为：**`DEVSHELL_VM_WORKSPACE_PARENT`**（若设置且非空）→ 否则在默认 **`DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT`** 下，若 **`cargo metadata`**（自当前目录）可解析 **workspace_root**，则取该目录（便于工程直接落在克隆树内）→ 否则为 **`${导出父目录}/vm-workspace/<实例名>`**。其中 **导出父目录** 为 **`DEVSHELL_EXPORT_BASE`**，或 **`XDG_CACHE_HOME`/`%LOCALAPPDATA%`** 下的 **`…/cargo-devshell-exports`**（见 **`sandbox::devshell_export_parent_dir()`**）。进程导出 **`DEVSHELL_WORKSPACE_ROOT`**。
- **Guest 逻辑根**：**`DEVSHELL_VM_GUEST_WORKSPACE`**（默认 **`/workspace`**）。
- **实例名**：**`DEVSHELL_VM_LIMA_INSTANCE`**（默认 **`devshell-rust`**；路径段中的实例名经消毒，非字母数字可变为 **`_`**）。

### 5.3 启动

- **REPL**：`cargo-devshell`（具体参数以实现为准）。
- **脚本**：`cargo-devshell [-e] -f script.dsh`（`-e` → 初始 **`set -e`**）。

### 5.4 内置命令（须与 `help` 一致）

| 命令 | 需求要点 |
|------|----------|
| `pwd` / `cd` / `ls` / `mkdir` | 导航与列出。 |
| `cat` / `touch` / `echo` | 文件与输出。 |
| `save` | 会话状态 → **§1.1** 工作区内 JSON。 |
| `export-readonly` | Mode S：VFS → 宿主临时目录；Mode P：guest 子树镜像到 VFS **`/.__export_ro_*`**。 |
| `todo …` | 子集（无 `export`/`import`/`init-ai`）；**`.todo.json`**。 |
| `rustup` / `cargo` | Mode S：导出 → 宿主执行 → 回写；Mode P：guest 内执行（**`exec`**）。**Unix** 默认经 **Lima（γ）**；**Windows** 在 VM 开启时经 **β 侧车**（Podman + **`devshell-vm`**），在 **`staging_dir`** 上执行 **`cargo new`/`cargo run`** 等与 **Unix guest 挂载树**一致；可 **`DEVSHELL_VM=off`** 等回退宿主沙箱。 |
| `exit` / `quit` / `help` | 退出与帮助。 |

### 5.5 语言特性

- **管道** `|`；**重定向** `<`、`>`、`2>`。
- **脚本**：`#` 注释、`\` 续行、变量、`if`/`for`/`while`、**`set -e`**、**`source`** / **`.`**（深度上限 **64**）。

### 5.6 Tab 补全

- **rustyline**，**`CompletionType::List`**；路径补全保留目录前缀。

### 5.7 Mode P 补充

- **β**：`--features beta-vm`，侧车 **`devshell-vm`**，**`DEVSHELL_VM_SOCKET`**（Unix：**UDS** / **`tcp:`**；**Windows**：默认 **`stdio`**）等见 **`docs/devshell-vm-gamma.md`**、**`docs/devshell-vm-windows.md`**。
- **验收**：REPL 与 guest 同路径下 **`ls`/`cat`/写文件** 一致；**`source`** 读宿主脚本等例外以设计文档为准。

### 5.8 Windows 与 β（Podman）— 当前实现要点

在 **Mode P**、**`DEVSHELL_VM_BACKEND=beta`**（Windows 默认）且 **Podman** 可用时：

| 要点 | 说明 |
|------|------|
| **侧车** | 进程 **`devshell-vm`**；与宿主通过 **一行一条 JSON** 通信（默认 **stdio**，不经由 Windows 本机监听端口）。 |
| **`session_start`** | **`staging_dir`** 为侧车 OS 视角下的宿主工作区目录（方式 B 下为容器内 **`/workspace`** 的挂载源；**`DEVSHELL_VM_BETA_SESSION_STAGING`** 可覆盖）。 |
| **`exec`** | 在映射后的宿主目录上 **真实 `spawn`** 子进程（非桩）；OCI 运行时镜像须含 **`cargo`**（**`containers/devshell-vm/Containerfile`**：`apt install cargo`）。 |
| **`exec` 超时** | 请求可带 **`timeout_ms`**；侧车可读 **`DEVSHELL_VM_EXEC_TIMEOUT_MS`** 作为默认。超时时侧车 **终止子进程** 并返回 **`exec_timeout`**：**Linux/Unix 侧车** 为**进程组**（含 **`sh -c`** 子树）；**Windows 宿主上若运行原生 MSVC 侧车**则仅保证终止**直接**子进程（见 **`docs/devshell-vm-windows.md`** 说明）。 |
| **stdio 与程序输出** | 子进程 **不得**向侧车用于 IPC 的 **stdout** 写入非 JSON 内容（例如 **`cargo run`** 启动的二进制 **`println!`**）。实现上将子进程 **stdout/stderr 管道化并转发到侧车 stderr**，保证宿主下一行 **`read_json_line`** 仅收到 **`exec_result`** 等协议行；编译与程序输出仍可在终端侧通过 **stderr** 可见（取决于 Podman/终端转发方式）。 |
| **镜像与版本** | **`cargo install`** 用户默认拉取 **`ghcr.io/tangcan/xtask_todo/devshell-vm:v<与库相同版本>`**；行为以 **`docs/devshell-vm-windows.md`**、**`docs/devshell-vm-oci-release.md`** 为准。 |

### 5.9 扩展阅读

- **`docs/devshell-vm-gamma.md`**（γ / β 总览）
- **`docs/devshell-vm-windows.md`**（Windows β / Podman / OCI）
- **`docs/devshell-vm-oci-release.md`**（侧车镜像发布与版本对齐）
- **`docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md`**
- **`docs/superpowers/specs/2026-03-20-devshell-rust-vm-design.md`**
- **`docs/dev-container.md`**（`DEVSHELL_RUST_MOUNT_NAMESPACE` 等）

---

## 6. AI / 可编程接口

| ID | 需求 |
|----|------|
| **US-A1** | `--json` 可机读输出。 |
| **US-A2** | 退出码：**0** 成功；**1** 一般错误；**2** 参数错误；**3** 数据错误。 |
| **US-A3** | `init-ai` 可生成工具用命令/技能文件。 |
| **US-A4** | `--dry-run` 不写盘。 |

---

## 7. 非功能

- **语言**：Rust；Clippy 策略见各 **`Cargo.toml`**。
- **错误**：人类可读信息优先 **stderr**；`--json` 下结构以实现为准。
- **颜色**：Todo 列表仅 TTY 着色。
- **兼容性**：主版本内尽量保持 CLI 稳定；破坏性变更需说明。

### 7.1 目标平台与 `xtask-todo-lib` 构建

- **crates.io**：以 **`xtask-todo-lib`** 当前版本为准；**Windows** 用户请使用 **`crates/todo/README.md`** 中标注的最低版本（例如修复 **`cargo install`** 所需的依赖与 **`cfg`** 调整）。
- **Devshell + VM**：**Unix** 上以 **Lima（γ）** 为完整参考（**`limactl shell`**、**lima.yaml** 挂载等）。**Windows** 上 **无 Lima**；在 **Podman** 与 **`beta-vm`** 可用时，**`rustup`/`cargo`** 经 **β 侧车**在挂载工作区上执行（**§5.8**），验收场景包括 **`cargo new`**、**`cargo run`** 等与挂载目录一致。**Windows** 仍不保证与 **γ** 文档（如 **`lima-todo`**、**`limactl shell`** 交互流程）逐条一致；**`DEVSHELL_VM=off`** / **`DEVSHELL_VM_BACKEND=host`** 下为宿主沙箱，无 Podman 依赖。

### 7.2 Pre-commit 与 Windows 交叉编译检查

- **`cargo xtask git pre-commit`** 与启用 **`git config core.hooksPath .githooks`** 后的 **`git commit`** 前钩子，均运行同一 **`pre-commit`** 脚本（见 **§4** 表）。
- 若本地未安装 **`x86_64-pc-windows-msvc`** 标准库，**`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`** 会失败；请先执行 **`rustup target add x86_64-pc-windows-msvc`**。

---

## 8. 文档维护

- 功能变更时更新本文档及 README、**`docs/design.md`**、**`docs/acceptance.md`** 等（视需要）。
- 文档与代码不一致时：按产品决策**改代码或改文档**对齐。

---

## 9. 参考

**章节索引（对外引用请用本节，避免与旧版文档章节号混淆）**

| 主题 | 章节 |
|------|------|
| 概述、Mode S / P、会话路径 | **§1**（含 **§1.1**） |
| 能力范围与不承诺 | **§2** |
| Todo、**`cargo xtask todo`** | **§3** |
| 其他 **`cargo xtask`** 子命令（含 **`acceptance`**） | **§4** |
| Devshell（含 **Windows β**） | **§5**（**§5.8**） |
| β **`exec` 超时**（**`timeout_ms`** / **`DEVSHELL_VM_EXEC_TIMEOUT_MS`**） | **§5.8**、**[test-cases.md](./test-cases.md) TC-D-VM-7** |
| **`--json`**、退出码、`init-ai` | **§6** |
| 非功能约束 | **§7** |
| 平台、`cargo install`、pre-commit 交叉检查 | **§1.2**、**§7.1**、**§7.2** |
| 文档维护 | **§8** |

- **`docs/publishing.md`** — 发布流程  
- **`docs/design.md`** — 设计总览  
- **`docs/acceptance.md` §8** — **`cargo xtask acceptance`** 自动化验收与报告  
- **`docs/devshell-vm-windows.md`** — Windows **β** / Podman / OCI 侧车  
- **`docs/reference/requirement_example.md`** — 原始需求示例（仅作参考）
