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
| **VM / Lima / β / Mode P / Linux mount-namespace sandbox** | **γ（Lima）** 以 **Unix** 为主。**Windows** 默认 **`DEVSHELL_VM_BACKEND=beta`**（库默认 **`beta-vm`**），首次连接可 **自动 Podman** 起侧车；也可用 **`DEVSHELL_VM=off`** / **`DEVSHELL_VM_BACKEND=host`** 仅用宿主沙箱。详见 **`docs/devshell-vm-windows.md`**。 |
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
| VM：`rustup`/`cargo` 经 Lima 或回退宿主 sandbox | 见 **`docs/devshell-vm-gamma.md`** |

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

在 **Mode P** 且 γ 就绪时，REPL 对**工程树**的操作与在 **Lima guest** 同一挂载下（默认 **`/workspace/…`**）一致；经 **`GuestFsOps`**。降级为 **Mode S** 时使用内存 VFS + push/pull。

### 5.2 工作区路径（γ）

- **宿主工作区根**（与 guest 挂载对齐）：默认 **`$XDG_CACHE_HOME/cargo-devshell-exports/vm-workspace/<实例名>`**（或 **`DEVSHELL_VM_WORKSPACE_PARENT`** 覆盖）；进程导出 **`DEVSHELL_WORKSPACE_ROOT`**。
- **Guest 逻辑根**：**`DEVSHELL_VM_GUEST_WORKSPACE`**（默认 **`/workspace`**）。
- **实例名**：**`DEVSHELL_VM_LIMA_INSTANCE`**（默认 **`devshell-rust`**）。

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
| `rustup` / `cargo` | Mode S：导出 → 宿主执行 → 回写；Mode P：guest 内执行。Unix 默认经 Lima；可 **`DEVSHELL_VM=off`** 等回退。 |
| `exit` / `quit` / `help` | 退出与帮助。 |

### 5.5 语言特性

- **管道** `|`；**重定向** `<`、`>`、`2>`。
- **脚本**：`#` 注释、`\` 续行、变量、`if`/`for`/`while`、**`set -e`**、**`source`** / **`.`**（深度上限 **64**）。

### 5.6 Tab 补全

- **rustyline**，**`CompletionType::List`**；路径补全保留目录前缀。

### 5.7 Mode P 补充

- **β**：`--features beta-vm`，侧车 **`devshell-vm`**，**`DEVSHELL_VM_SOCKET`** 等见 **`docs/devshell-vm-gamma.md`**。
- **验收**：REPL 与 guest 同路径下 **`ls`/`cat`/写文件** 一致；**`source`** 读宿主脚本等例外以设计文档为准。

### 5.8 扩展阅读

- **`docs/devshell-vm-gamma.md`**
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
- **全功能 devshell（VM、guest 挂载、Mode P 与 Linux 沙箱扩展）**：开发与验收以 **Unix** 环境为准；**Windows** 侧重 **库 + `cargo-devshell` REPL** 与 **Todo** 内置命令，不保证与 Lima 文档 1:1 行为一致。

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
| Devshell | **§5** |
| **`--json`**、退出码、`init-ai` | **§6** |
| 非功能约束 | **§7** |
| 平台、`cargo install`、pre-commit 交叉检查 | **§1.2**、**§7.1**、**§7.2** |
| 文档维护 | **§8** |

- **`docs/publishing.md`** — 发布流程  
- **`docs/design.md`** — 设计总览  
- **`docs/acceptance.md` §8** — **`cargo xtask acceptance`** 自动化验收与报告  
- **`docs/reference/requirement_example.md`** — 原始需求示例（仅作参考）
