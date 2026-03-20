# 项目需求说明（Requirements）

本文档依据 **当前代码实现** 整理，描述 xtask_todo 工作区的能力、用户可见行为与验收要点。实现变更时应同步更新本文档。

---

## 1. 项目概述

| 项 | 说明 |
|----|------|
| **名称** | xtask_todo |
| **目标** | 提供可复用的待办（Todo）领域库、基于 `cargo xtask` 的 CLI 工作流，以及可选的 **devshell**（虚拟文件系统 + 脚本 + 宿主 Rust 工具链沙箱运行）。 |
| **Workspace** | `crates/todo`（库 **xtask-todo-lib**，crates.io 发布）、`xtask`（工作区工具，`publish = false`）。 |
| **主要入口** | `cargo xtask …`（通过 `.cargo/config.toml` 别名）；`cargo run -p xtask-todo-lib --bin cargo-devshell`（devshell）。 |

---

## 2. 范围与实现状态总览

| 能力域 | 状态 | 说明 |
|--------|------|------|
| Todo CRUD / 列表过滤排序 | 已实现 | 库 API + `cargo xtask todo` |
| 可选字段、搜索、统计、导入导出、重复任务 | 已实现 | 见 §4 |
| `--json` / `--dry-run` / 约定退出码 | 已实现 | 见 §7 |
| `todo init-ai` | 已实现 | 生成 AI 命令/技能文件 |
| xtask：fmt / clippy / coverage / git / gh / run / clean / publish | 已实现 | 见 §5 |
| Devshell：VFS、内置命令、管道、重定向、脚本、`source` | 已实现 | 见 §6 |
| Tab 补全（命令 + 路径，含子路径前缀） | 已实现 | `CompletionType::List`，路径候选含目录前缀 |
| `rustup` / `cargo` 宿主沙箱（导出 VFS → 临时目录 → 同步回 VFS） | 已实现 | 需 PATH 中可执行文件 |

**不包含**（除非后续纳入并修订本文档）：HTTP API、多用户权限、`.todo.json` 正式 schema 版本迁移策略、内核级强隔离（当前为路径级临时目录隔离）等。详见 §9。

---

## 3. Todo 领域需求（库 + CLI 行为）

以下用户故事与 `xtask-todo-lib` 及 `cargo xtask todo` 行为一致。

### US-T1 创建待办

- **故事**：用户可以创建一条待办（必须含标题）。
- **验收**：有效标题则分配 id 并持久化；空/非法标题返回错误（CLI 退出码 2），不产生有效项。

### US-T2 列出待办

- **故事**：用户可以查看待办列表。
- **验收**：无数据时可为空列表提示；有数据时输出包含 id、状态、标题与时间信息；支持过滤与排序（§4）。

### US-T3 完成待办

- **故事**：用户可将待办标为已完成。
- **验收**：未完成项完成后查询为已完成；非法/不存在 id 返回数据错误（退出码 3），不影响其他项。

### US-T4 删除待办

- **故事**：用户可删除一条待办。
- **验收**：存在则删除并持久化；不存在 id 返回数据错误（退出码 3）或约定幂等行为与实现一致。

### US-T5 时间戳与展示

- **故事**：记录创建时间；完成时记录完成时间；列表展示相对时间及已完成项用时。
- **验收**：模型含 `created_at`、`completed_at`；人类可读列表展示与 `format` 模块一致。

### US-T6 长期未完成视觉提示

- **故事**：在 TTY 下对「创建超过阈值且未完成」的项做区分展示。
- **验收**：阈值 **7 天**（`AGE_THRESHOLD_DAYS`）；非 TTY/管道不输出颜色转义序列。

### US-T7 查看单条（`show`）

- **验收**：有效 id 输出完整字段；无效 id 非 0 退出。

### US-T8 更新（`update`）

- **验收**：可更新标题及可选字段；支持 `--clear-repeat-rule`；非法 id 退出码 3。

### US-T9 可选属性

- **字段**：描述、截止日期（YYYY-MM-DD）、优先级（low/medium/high）、标签（列表/逗号分隔）。
- **验收**：add/update/list/show 与过滤、排序行为与实现一致。

### US-T10 搜索（`search`）

- **验收**：按关键词在标题、描述、标签等约定范围内匹配；无匹配可为空列表。

### US-T11 统计（`stats`）

- **验收**：至少包含总数、未完成、已完成等统计输出。

### US-T12 导入 / 导出

- **导出**：JSON/CSV（扩展名或 `--format`）。
- **导入**：JSON/CSV；支持 `--replace` 替换当前列表（与合并策略以实现为准）。

### US-T13 重复任务

- **规则**：daily / weekly / monthly / yearly / weekdays、间隔简写（如 `2d`、`3w`）、`custom:N`；结束条件 `repeat_until`、`repeat_count`。
- **验收**：完成带重复规则的任务时可生成下一实例；**`complete` 支持 `--no-next`**，仅完成当前实例、不生成下一任务。

---

## 4. `cargo xtask todo` 命令规格

### 4.1 全局选项

| 选项 | 行为 |
|------|------|
| `--json` | 结构化 JSON 输出（成功含 `status`/`data`，失败含 `status`/`error`）。 |
| `--dry-run` | 修改类子命令仅预览，不写 `.todo.json`、不改内存列表。 |

### 4.2 子命令与要点

| 子命令 | 说明 |
|--------|------|
| `add <title>` | 可选 `--description`、`--due-date`、`--priority`、`--tags`、`--repeat-rule`、`--repeat-until`、`--repeat-count`。 |
| `list` | 可选 `--status`（completed/incomplete）、`--priority`、`--tags`、`--due-before`、`--due-after`、`--sort`（created-at / due-date / priority / title）。 |
| `show <id>` | 单条详情。 |
| `update <id> <title>` | 同上可选字段 + `--clear-repeat-rule`。 |
| `complete <id>` | `--no-next` 跳过重复下一实例。 |
| `delete <id>` | 删除。 |
| `search <keyword>` | 关键词搜索。 |
| `stats` | 统计。 |
| `export <file>` | `--format json\|csv` 或按扩展名。 |
| `import <file>` | `--replace` 可选。 |
| `init-ai` | `--for-tool`、`--output` 生成 AI 用命令文件（默认 Cursor 路径约定）。 |

### 4.3 数据与参数约定

- **持久化文件**：默认当前目录 **`.todo.json`**（与 devshell 内 `todo` 内置命令共享约定，见 §6）。
- **任务 id**：正整数；**0 非法**。不存在 id 的 show/update/complete/delete：**退出码 3**。
- **日期**：`YYYY-MM-DD`；非法则 **退出码 2**。
- **优先级 / 重复规则**：非法值 **退出码 2**。

---

## 5. 其他 `cargo xtask` 子命令（开发者工作流）

| 子命令 | 需求要点 |
|--------|----------|
| `run` | 运行当前项目约定的主程序/示例（以 xtask 实现为准）。 |
| `fmt` | 调用 `cargo fmt`。 |
| `clippy` | Clippy（pedantic + nursery 等，与项目配置一致）。 |
| `coverage` | 基于 cargo-tarpaulin 的覆盖率任务。 |
| `clean` | 清理构建产物（以实现为准）。 |
| `gh log` | 依赖宿主 **GitHub CLI (`gh`)**，展示最近 Actions 运行日志。 |
| `git add` / `git pre-commit` / `git commit` | 暂存、预提交检查、带检查的提交（默认消息等见 `--help`）。 |
| `publish` | 发布相关辅助（见 `docs/publishing.md`）。 |

**US-X1**：`cargo xtask --help` 列出子命令；已实现子命令执行并返回约定退出码。  
**US-X3**：新增子命令仅需在 xtask 源码注册，无需改 cargo 别名配置。

---

## 6. Devshell（`cargo-devshell`）需求

### 6.1 启动与持久化

- **二进制**：`xtask-todo-lib` 的 `cargo-devshell`。
- **用法**：
  - `cargo-devshell [path]`：REPL；可选 VFS 快照路径，默认 **`.dev_shell.bin`**。
  - `cargo-devshell [-e] -f script.dsh`：执行脚本；`-e` 等价初始 **`set -e`**（遇错退出）。
- **验收**：退出 REPL 或脚本结束后将 VFS **保存**到上述 bin 文件（与实现一致）；文件不存在时从空 VFS 启动。

### 6.2 虚拟文件系统（VFS）

- **需求**：路径为 Unix 风格；支持 `mkdir`、文件读写、`cd`/`pwd`、`ls`；与宿主路径交互的约定以实现为准（如 `export-readonly`、`source` 可读宿主文件）。

### 6.3 内置命令（builtin）

须与 `help` 输出及 `dispatch` 一致，包括但不限于：

| 命令 | 需求要点 |
|------|----------|
| `pwd` / `cd` / `ls` / `mkdir` | 目录导航与列出。 |
| `cat` / `touch` / `echo` | 文件与输出；`cat` 可无参数读 stdin。 |
| `save [path]` | 将 VFS 保存到 bin。 |
| `export-readonly [path]` | 将 VFS 子树导出到宿主临时目录（只读用途）。 |
| `todo …` | **子集**：`list`、`add`、`show`、`update`、`complete`、`delete`、`search`、`stats`（无 `export` / `import` / `init-ai`）；与 `cargo xtask todo` 共用 **`.todo.json`** 约定。 |
| `rustup [args…]` / `cargo [args…]` | 导出 cwd 对应 VFS 子树 → 在宿主临时目录中执行 `PATH` 内的 `rustup`/`cargo` → 同步回 VFS → 删除临时目录。**不得**内建调用 `podman`/`docker` 等 OCI 运行时。Linux 可选 `DEVSHELL_RUST_MOUNT_NAMESPACE`（独立 mount namespace，libc）；详见 [dev-container.md](./dev-container.md)。**Unix γ（`cargo devshell` 默认）：** 未设置 `DEVSHELL_VM` 时视为开启 VM；未设置 `DEVSHELL_VM_BACKEND` 时 Unix 默认为 `lima`，经 Lima 在 VM 内执行；`DEVSHELL_VM=off` 或 `DEVSHELL_VM_BACKEND=host`/`auto` 回退宿主 sandbox。工作区挂载见 [devshell-vm-gamma.md](./devshell-vm-gamma.md)。 |
| `exit` / `quit` | 结束 REPL。 |
| `help` | 列出内置命令说明。 |

### 6.4 管道、重定向与解析

- **管道**：`|` 连接多条内置命令，前一条 stdout 作为下一条 stdin。
- **重定向**：`<`、`>`、`2>`（含 `2>` 作为独立 token）；重定向目标为 VFS 路径（以实现为准）。
- **需求**：解析错误应给出可读错误，不崩溃。

### 6.5 脚本（`.dsh`）

- **行延续**：`\`；注释：`#`。
- **变量**：`NAME=value`；展开 `$VAR` / `${VAR}`。
- **控制流**：`if …; then … else … fi`、`for …; do … done`、`while …; do … done`。
- **`set -e`**：之后命令失败则终止脚本。
- **`source path` / `. path`**：嵌套执行脚本；最大深度 **64**；可从 VFS 或宿主读文件。
- **验收**：脚本仅执行内置命令，不调用宿主 shell 任意可执行文件（`rustup`/`cargo` 除外且走沙箱）。

### 6.6 REPL：`source` / `.`

- 在交互行中支持 `source path`、`. path`，行为与脚本中一致（读 VFS 或宿主）。

### 6.7 Tab 补全

- **TTY** 下使用 **rustyline**，补全类型为 **`CompletionType::List`**（类似 bash：最长公共前缀 + 多义时第二次 Tab 列清单），**非**默认的 Circular（避免唯一补全后再次 Tab 退回半截词）。
- **路径补全**：候选为**整词替换串**，须保留已输入的目录前缀（例如 `cat src/` → `cat src/main.rs`，不得变为 `cat main.rs`）。

### 6.8 设计与扩展

- Rust 沙箱隔离的进一步设计见：`docs/superpowers/specs/2026-03-20-devshell-rust-vm-design.md` 及对应实现计划（可选更强隔离后端等）。
- 会话级 microVM（γ Lima + β 侧车）见：`docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md`、IPC 草案 `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`；侧车占位 crate **`crates/devshell-vm`**（`cargo run -p devshell-vm`）。

**US-D1**：开发者可通过 devshell 在隔离 VFS 中演练文件操作与脚本。  
**US-D2**：开发者可在 VFS 内使用宿主 `rustup`/`cargo` 并看到产物写回 VFS（在 PATH 满足前提下）。

---

## 7. AI / 可编程接口

| ID | 需求 |
|----|------|
| **US-A1** | `--json` 时输出可解析 JSON；失败结构含错误信息。 |
| **US-A2** | 退出码：**0** 成功；**1** 一般错误；**2** 参数错误；**3** 数据错误（如 todo id 不存在）。 |
| **US-A3** | `init-ai` 可向指定工具目录生成命令/技能文件，指导使用 `--json` 等调用方式。 |
| **US-A4** | `--dry-run` 对修改类 todo 命令仅预览，不写盘、不改内存列表。 |

---

## 8. 非功能与约束

### 8.1 技术栈与仓库约定

- **语言**：Rust；各 crate 的 Clippy 策略见各自 `Cargo.toml`。
- **Git hooks**：可选 `.githooks`（fmt、clippy、行数限制、test），见 README。
- **命令稳定性**：主版本内尽量保持 CLI 兼容；破坏性变更需文档与迁移说明。

### 8.2 错误与输出

- 人类可读错误优先 stderr；`--json` 模式下错误结构在 stdout 或约定通道与实现一致。
- Todo 列表颜色仅 TTY 启用。

### 8.3 规模与限制

- 不承诺单文件任务数量上限；超大规模下的性能与内存以「合理使用」为预期。

### 8.4 退出码汇总（Todo）

| 码 | 含义 | 典型场景 |
|----|------|----------|
| 0 | 成功 | |
| 1 | 一般错误 | I/O、内部错误等 |
| 2 | 参数错误 | 非法日期、非法 flag、空标题等 |
| 3 | 数据错误 | id 不存在或非法 id |

---

## 9. 不包含与未来方向

当前**不承诺**实现：

- HTTP API（如 `todo serve`）、推送提醒、自然语言对话界面  
- 多用户、权限、审计  
- `.todo.json` / `.dev_shell.bin` 的自动版本迁移流水线  
- 并发多进程安全写入同一文件的严格保证  

可参考 `docs/reference/requirement_example.md` 中的扩展想法；是否纳入以今后修订为准。

---

## 10. 文档维护

- 新增或变更功能时：**更新本文档**对应章节，并视情况更新 README、`docs/design.md`、`docs/acceptance.md` 等。
- 实现与文档冲突时：以产品决策为准——**要么改代码对齐文档，要么更新文档反映真实行为**。

---

## 11. 参考

- **示例需求来源**（历史）：`docs/reference/requirement_example.md`（AI 技能集成版 TODO 需求），本项目已裁剪并实现其中大部分 CLI 能力。  
- **发布说明**：`docs/publishing.md`。  
- **OpenSpec / 设计文档**：`docs/superpowers/specs/`、`openspec/` 下各规格（devshell 脚本、gh log、Rust VM 等）。
