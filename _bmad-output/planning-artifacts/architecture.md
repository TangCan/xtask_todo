---
stepsCompleted:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
  - 7
  - 8
workflowType: architecture
lastStep: 8
status: complete
completedAt: '2026-03-25'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - docs/acceptance-report.md
  - docs/superpowers/plans/2026-03-11-devshell-microvm-session.md
  - docs/acceptance.md
  - docs/requirements.md
  - docs/devshell-vm-windows.md
  - docs/design.md
  - docs/test-cases.md
  - docs/superpowers/plans/2026-03-20-devshell-guest-primary-workspace.md
  - docs/test-coverage.md
  - docs/devshell-vm-gamma.md
  - docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md
  - docs/superpowers/specs/2026-03-20-devshell-vm-primary-guest-filesystem.md
  - docs/publishing.md
  - docs/reference/requirement_example.md
  - docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md
  - docs/dev-container.md
  - docs/tasks.md
  - docs/devshell-vm-oci-release.md
  - docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md
  - docs/superpowers/specs/2026-03-20-devshell-rust-vm-impl-plan.md
  - docs/superpowers/specs/2026-03-20-devshell-rust-vm-design.md
  - docs/superpowers/specs/2026-03-19-devshell-scripting-impl-plan.md
  - docs/superpowers/specs/2026-03-19-devshell-scripting-design.md
  - docs/superpowers/specs/2026-03-14-gh-log-impl-plan.md
  - docs/superpowers/specs/2026-03-14-gh-log-design.md
  - docs/superpowers/specs/2026-03-10-dev-shell-todo-design.md
  - docs/superpowers/specs/2026-03-13-requirements-content-design.md
project_name: xtask_todo
user_name: Richard
date: '2026-03-25'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

PRD 将能力分为 **34 条 FR（FR1–FR34）**，从架构视角可映射为以下边界：

- **Todo 领域（FR1–FR8）**：持久化、列表/过滤/排序、完成/删除、搜索与统计、导入导出、重复任务 —— 需要稳定的**领域模型**与**单一数据源**（`.todo.json` 约定），库侧以 `TodoList<S: Store>` 等抽象表达。
- **Xtask 与仓库工具（FR9–FR13）**：统一编排入口、Git/发布/acceptance/pre-commit —— **编排层**与**领域库**分离；宿主集成（`git`/`gh`）留在 xtask，不侵入可发布库的核心语义。
- **Devshell（FR14–FR19）**：交互/脚本、内置白名单、管道与重定向、todo 子集、帮助与补全 —— 需要 **Session**、**命令分派** 与 **VFS/IO** 的清晰分层。
- **Rust 工具链与可选 VM（FR20–FR25）**：宿主沙箱 vs γ（Unix）vs β（Windows 侧车）、Mode S / Mode P、会话元数据 —— 架构上必须 **可降级**、**IPC/stdio 协议隔离**、**工作区模式显式化**。
- **可编程与 AI（FR26–FR29）**：`--json`、`--dry-run`、退出码、`init-ai` —— **对外契约**（序列化错误体、稳定码）需跨 CLI/devshell 一致。
- **跨平台与发布（FR30–FR31）**：Linux/macOS/Windows MSVC、crates.io —— **条件编译**与 **MSVC 作为一等目标** 影响 crate 布局与 CI。
- **质量与追溯（FR32–FR34）**：文档 ID 对齐、测试/覆盖率、不默认承诺 HTTP API/多租户等 —— 架构保持 **终端/库优先**，避免隐式网络服务假设。

**Non-Functional Requirements:**

- **性能（NFR-P1/P2）**：本地 CLI 可交互；管道缓冲有硬上限、超限须失败可见 —— 影响 devshell 管道实现与内存策略。
- **安全（NFR-S1–S3）**：数据默认本地工作区；侧车 stdout 仅 JSON 行；子进程白名单；供应链可复现 —— 强化 **IPC 卫生** 与 **非任意 shell**。
- **集成（NFR-I1/I2）**：外部工具缺失时显式失败或 SKIP；JSON 行协议版本化、一行一对象 —— **β/侧车** 与宿主解析契约是核心架构约束。
- **可靠性与可维护性（NFR-R1/R2）**：无 Lima/Podman 时主路径可用；破坏性变更可追溯到 CHANGELOG —— **特性开关与默认 Mode S**。

**Scale & Complexity:**

- **主领域**：开发者工具（Rust workspace：库 + xtask CLI + 可选 devshell-vm），**非**典型 Web 全栈或移动端。
- **复杂度**：PRD 归类为 **medium**（跨平台、VM、IPC、可选环境依赖）。
- **估计架构关注点数量**：领域核心 1（todo lib）、编排 1（xtask）、交互运行时 1（devshell）、隔离执行与 IPC 1（devshell-vm + 平台后端）— 共约 **4 个主要组件域**，外加 **验收/测试** 横切。

- **Primary domain**：Rust **CLI + 库** + 可选 **VM 侧车**，强终端与脚本化。
- **Complexity level**：**medium**（与 PRD 一致）。
- **Estimated architectural components**：见上「约 4 域 + 横切」。

### Technical Constraints & Dependencies

- **棕地**：根 `Cargo.toml` workspace 已存在；交付物含 **`xtask-todo-lib`（crates.io）**、**xtask（不单独发布）**、**devshell-vm（`publish = false`，OCI 另见文档）**。
- **外部依赖**：Git、`gh`、`rustup`/`cargo`、可选 Lima/Podman；失败须可诊断并文档化 SKIP（`acceptance.md §2`）。
- **Windows**：无 Lima；β 路径依赖 **Podman + JSON 行侧车 + GHCR 镜像版本与库版本对齐**（`devshell-vm-oci-release.md`）。
- **合规**：默认不承担 HIPAA/PCI 等；`.todo.json` 非云端多租户模型。

### Cross-Cutting Concerns Identified

- **可机读 CLI 契约**：`--json`、退出码、错误体 —— 跨 **xtask todo** 与 **devshell** 子集须一致。
- **验收即门禁**：`cargo xtask acceptance`、pre-commit、MSVC `cargo check` —— 影响 crate 划分、测试布局与 CI 可重复性。
- **Mode S / Mode P 与降级**：VM 不可用或用户关闭时行为可预测 —— 会话与工作区状态机是共享设计轴。
- **stdio/协议卫生**：侧车仅向宿主输出可解析 JSON 行 —— 所有子进程封装须防止污染 stdout。
- **跨平台一致性**：同一套需求—设计—用例—验收 ID；Windows 与 Unix 能力分治但语义对齐。

## Starter Template Evaluation

### Primary Technology Domain

**Rust 开发者工具 / Cargo Workspace（棕地）**：主交付为 **可发布 crate（`xtask-todo-lib`）**、**workspace 内 `xtask` 二进制** 与 **可选 `devshell-vm`**，**不是** 以 Next.js/Vite 等 Web 应用脚手架为起点的项目。因此本步 **不** 采用「`create-*-app` 类一次性生成器」作为架构基线，而以 **现有仓库布局与 PRD/设计文档** 为准。

### Starter Options Considered

| 选项类型 | 结论 |
|----------|------|
| 通用 Web 全栈脚手架（Next、Remix、T3 等） | **不适用** — 无 PRD 级 Web UI 主路径 |
| 通用 CLI 脚手架（oclif、Commander 等，非 Rust） | **不适用** — 技术栈为 Rust |
| **`cargo new` / 空 workspace** | 仅适用于绿场；本仓库为 **棕地**，以当前 `Cargo.toml` workspace 为事实来源 |

### Selected Starter: 现有 Cargo Workspace（棕地基线）

**Rationale for Selection:**

- PRD 将项目归类为 **brownfield**，且已存在 **多 crate workspace**（`todo` / `xtask` / `devshell-vm` 等）与成体系 `docs/`。
- 「初始化」对贡献者而言是 **`git clone` + `cargo build` / `cargo test`**，而非运行第三方模板 CLI。
- 领域与契约（`--json`、退出码、Mode S/P、γ/β）已在 **需求与设计文档** 中绑定，**替换**为外部 starter 会破坏可追溯性。

**Initialization Command（新贡献者/CI 语义）:**

```bash
git clone <repository-url> && cd xtask_todo
cargo build --workspace
cargo xtask acceptance   # 或按 docs/acceptance.md 的环境说明处理 SKIP
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**

- PRD / 仓库当前约定：**Rust Edition 2021**（各 member 在各自 `Cargo.toml` 中声明）。
- **版本事实（网络核对）**：**Rust 2024 Edition** 随 **Rust 1.85.0**（2025-02-20）进入 stable；若未来统一升级 edition，需在 **CHANGELOG、MSVC 与 CI** 上单独评估，**非**本架构文档默认变更。

**Styling Solution:** 不适用（非浏览器 UI 主产品）。

**Build Tooling:** **Cargo** + workspace `resolver` / features（见 `design.md` §1）；**`cargo xtask`** 作为编排入口。

**Testing Framework:** **`cargo test`**；覆盖率由 **`cargo xtask coverage`** 与 `xtask/src/coverage.rs` 排除规则约束（PRD / `test-coverage.md`）。

**Code Organization:** **领域在 `xtask-todo-lib`**；**编排与 Git/发布在 `xtask`**；**VM/侧车在 `devshell-vm`** — 与 PRD「Technical Architecture Considerations」一致。

**Development Experience:** **pre-commit**（`.githooks/pre-commit` 与 `cargo xtask git pre-commit` 对齐）、**acceptance** 一键回归。

**Note:** 若新增 **独立** 绿场子项目（例如仅文档用的小型工具），可再单独评估是否 `cargo new`，但 **不改变** 上述主 workspace 基线。

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions（阻塞实现一致性）：**

- **Crate 边界**：领域（`xtask-todo-lib`）/ 编排（`xtask`）/ β 侧车（`devshell-vm`）职责分离；**不**在库内绑定 `git`/`gh` 宿主业务规则（见 `design.md` §1.5）。
- **持久化形态**：Todo 权威数据为工作区内 **`.todo.json`**；**`Store` trait + `InMemoryStore`** 表达进程内状态；**文件 I/O** 仅在 **xtask `todo/io`** 与 **devshell `todo_io`**，**不**塞进领域核心抽象之下隐式全局化。
- **对外契约**：**`--json`**、**`requirements §6` 退出码**、**`--dry-run`** 语义在 **xtask todo** 与 **devshell 受支持子集** 间保持一致（PRD FR26–FR29）。
- **VM / IPC**：**一行一条 JSON**、**handshake**、侧车 **stdout 专用于协议**；子进程 **stdout/stderr 管道化** 避免污染宿主 **`read_json_line`**（`design.md` §1.4、`requirements §5.8`）。

**Important Decisions（显著塑造架构）：**

- **CLI 解析**：**argh**（derive、`FromArgs`）；crates.io 上 **0.1.x** 线持续维护（**google/argh**；具体版本以 workspace **`Cargo.lock`** 为准，网络核对日期 2026-03 仍有新版本发布）。
- **Workspace**：根 **`resolver = "2"`**；members **`crates/todo`、`crates/devshell-vm`、`xtask`**（`design.md` §1.1–§1.2）。
- **跨平台**：**Windows MSVC** 为 **一等** 目标；非 Unix 上 **γ/Lima 相关 API** 以 **`cfg`** 与**桩**保证类型检查（`design.md` §1.4）。
- **Mode S / Mode P**：**有效工作区模式**与 **冲突时降级 Mode S** 规则（`requirements §1.1`、guest-primary 设计文档）。

**Deferred Decisions（Post-MVP / 路线图）：**

- **更长驻 guest 连接**、更强 **β IPC**、**Mode P** 会话/导出完整语义（PRD Vision / `docs/superpowers/specs/`）。
- **Rust 2024 Edition** 全 workspace 迁移（需单独变更集与 CI 验证）。

### Data Architecture

| 决策 | 内容 |
|------|------|
| **存储介质** | **本地文件** `.todo.json`（用户工作区）；**无**服务端数据库、**无**多租户云端数据模型（PRD、NFR-S1）。 |
| **领域模型** | **`TodoList<S: Store>`**、**`Todo`**、**`TodoId`**、**`ListOptions`**、**`RepeatRule`** 等；**`Store`** 抽象持久化后端。 |
| **进程内状态** | **`InMemoryStore`**；与磁盘同步由 **crate 边界外** 显式 load/save。 |
| **导入/导出** | FR7 约定交换格式；**替换策略**以产品/文档为准，**不**引入隐式 HTTP 同步。 |
| **迁移** | 大版本 **`.todo.json`** 语义变更需 **文档与 CHANGELOG**；**未**承诺自动迁移流水线（PRD FR34、FR「迁移指南」）。 |

### Authentication & Security

| 决策 | 内容 |
|------|------|
| **身份与授权** | **不适用** 云端账户模型；**无** 产品级 HTTP API 权限层（除非未来显式增改 PRD）。 |
| **数据驻留** | **`.todo.json`** 与 devshell 会话数据默认 **工作区内**；**磁盘权限与备份**由用户负责（NFR-S1）。 |
| **子进程与 VM** | **白名单**（builtin + `rustup`/`cargo` 经 **`SessionHolder`**）；**非**任意宿主 shell（NFR-S2）。 |
| **供应链** | **crates.io** + **语义化版本**；合并与发布前 **`cargo xtask acceptance`**、**pre-commit**（NFR-S3）。 |

### API & Communication Patterns

| 决策 | 内容 |
|------|------|
| **人机界面** | **终端 CLI**（`cargo xtask`、**`cargo devshell`**）；**主路径** 非 Web UI。 |
| **成功/失败载荷** | 支持子命令的 **`--json`**：**结构化** 成功/错误体（与 **`requirements §6`** 退出码一致）。 |
| **错误与退出码** | **0=成功，1=一般，2=用法/参数，3=数据/状态**（以 `requirements §6` 为权威）。 |
| **β 侧车** | **JSON 行**、**版本化 handshake**、**一行一对象**（NFR-I2）；传输 **stdio** / **UDS** / **`tcp:`**（平台见 `design.md` §1.4）。 |
| **可观测性** | 侧车/编译输出 **stderr** 侧可见；**协议 stdout** 与 **子进程输出** 分离（`design.md` §1.4「stdio 与 JSON」）。 |

### Frontend Architecture

| 决策 | 内容 |
|------|------|
| **范围** | **不适用** 浏览器 SPA/SSR 主产品；**CLI `--help`**、**终端补全**、**TTY/非 TTY** 行为以 `requirements` / `design` 为准（PRD 跳过应用商店/视觉稿类要求）。 |

### Infrastructure & Deployment

| 决策 | 内容 |
|------|------|
| **构建与测试** | **Cargo workspace**；**`cargo test`**；**`cargo xtask acceptance`** 汇总 **SKIP/手工** 项（`acceptance.md`）。 |
| **质量门禁** | **`.githooks/pre-commit`** 与 **`cargo xtask git pre-commit`** 对齐；**`x86_64-pc-windows-msvc` `cargo check`**（NF-5）。 |
| **覆盖率** | **`cargo xtask coverage`**；**xtask-todo-lib / xtask** 阈值与排除规则见 `test-coverage.md` / `xtask/src/coverage.rs`。 |
| **发布** | **`xtask-todo-lib`** → **crates.io** 含 **`cargo devshell`**；**`publishing.md`** 清单；**`devshell-vm`** 镜像 **GHCR** 与 **`CARGO_PKG_VERSION` 对齐**（`devshell-vm-oci-release.md`）。 |
| **CI** | 以 **acceptance** 与文档化 **SKIP** 为准；**Lima/Podman 全链路** 自动化属 **Growth**（PRD）。 |

### Decision Impact Analysis

**Implementation Sequence（建议）：**

1. 变更 **Todo 领域** → 同步 **`xtask-todo-lib`** 测试与 **`.todo.json`** 契约说明。  
2. 变更 **CLI 行为** → 同步 **`requirements §6`**、**`--json` 示例**、**xtask 集成测试**。  
3. 变更 **devshell 命令/todo 子集** → 同步 **`todo_io`** 与 **VFS/会话** 行为。  
4. 变更 **β/IPC** → 同步 **`devshell-vm`**、**宿主 `session_beta`**、**`test-cases.md` TC-D-VM-\***。  
5. **发布/镜像** → **`publishing.md`**、**`devshell-vm-oci-release.md`**、**CHANGELOG**。

**Cross-Component Dependencies：**

- **`TodoList`** ↔ **xtask / devshell** 文件边界。  
- **`SessionHolder`** ↔ **γ/β** ↔ **guest 挂载与 workspace 根解析**。  
- **acceptance** ↔ **三 crate 测试** + **NF 文件检查** + **MSVC check**。

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified（AI 代理易分歧处）：**

约 **8** 类：**crate 边界**、**错误/退出码**、**JSON 字段命名**、**测试位置**、**cfg 与桩**、**侧车 IPC 与 stdio**、**环境变量命名**、**文档与 ID 对齐**。

### Naming Patterns

**Database Naming Conventions:** 不适用（无应用数据库）。

**API Naming Conventions（CLI / JSON）：**

- **子命令与 flag**：遵循 **argh** 派生结构与 **`requirements`** 已列名称；**不**随意重命名对外稳定 CLI。
- **JSON 字段**：成功/错误体字段名以 **`requirements` / 现有 `xtask` 输出** 为准；**新增**字段需考虑 **`--json` 稳定性** 与 **CHANGELOG**。

**Code Naming Conventions（Rust）：**

- **模块与类型**：**`snake_case`** 模块目录、**`PascalCase`** 类型/ trait、**`SCREAMING_SNAKE_CASE`** 常量（与 **Rust 惯例** 及 **clippy** 配置一致）。
- **错误类型**：领域错误与 **CLI 映射** 分层；**不**在库深处打印 **`eprintln!`** 替代可测试错误类型（编排层再决定 stderr）。

### Structure Patterns

**Project Organization：**

- **`crates/todo`**：**领域 + devshell + `cargo-devshell`**；领域测试在 **`crates/todo/src/tests/`** 或 crate 约定路径（以现有布局为准）。
- **`xtask`**：**唯一 `cargo xtask` 入口**；`todo`、**`acceptance`**、**`git`** 等子模块分文件。
- **`crates/devshell-vm`**：**侧车二进制**；协议与 **`server.rs`** 等保持与 **`docs/superpowers/specs/*ipc*`** 一致。
- **集成/大型场景**：优先 **`devshell/tests/`**、**`xtask/src/tests/`** 等现有模式，**不**在领域根散落无归属的 `main`。

**File Structure Patterns：**

- **权威需求/设计**：**`docs/requirements.md`**、**`docs/design.md`**；变更行为时 **同步** 或 **显式记 TECH DEBT**。
- **验收与报告**：**`docs/acceptance.md`**、**`docs/acceptance-report.md`**（由 **`cargo xtask acceptance`** 维护的部分 **勿手改** 除非流程说明允许）。

### Format Patterns

**API Response Formats（`--json`）：**

- **成功**：稳定 **`status`/`ok`** 等字段以 **当前实现与文档示例** 为准；**扩展** 时保持向后兼容或 **semver**。
- **失败**：**机读** `code`/`message`（或现有结构）；**与退出码 2/3** 一致；**不**向协议 stdout 混入人类长文。

**Data Exchange Formats：**

- **`.todo.json`**：序列化格式以 **领域与 `requirements §2`** 为准；**不**在补丁中引入隐式字段语义。

### Communication Patterns

**Event System Patterns：** 不适用全局事件总线；**devshell 管道** 为 **进程内阶段缓冲**，遵守 **`PIPELINE_INTER_STAGE_MAX_BYTES`** 等上限（PRD NFR-P2）。

**State Management Patterns：**

- **Mode S / Mode P**：**显式环境变量** 与 **降级** 规则；**不**依赖未文档化的全局可变状态。
- **会话 JSON**：**`format`/`logical_cwd`** 等以 **`requirements §1.1`** 与 **`design.md`** 为准。

### Process Patterns

**Error Handling Patterns：**

- **库**：返回 **`Result`** 与领域错误；**xtask**：映射到 **退出码** 与 **stderr/`--json`**。
- **VM/侧车**：超时 **`exec_timeout`**；**可解析错误帧**；**不**吞掉 **`read_json_line`** 失败。

**Loading State Patterns：** 不适用 SPA；**CLI** 长时间操作须有 **文档化** 行为（超时、 spinner 若存在以现有代码为准）。

### Enforcement Guidelines

**All AI Agents MUST:**

- **改行为先查** `docs/requirements.md` / `docs/design.md` / **`test-cases.md`** 对应 ID。  
- **新 JSON 契约** 同时更新 **`requirements` 或示例** 与 **CHANGELOG**（若对外可见）。  
- **跨平台** 变更后 **运行或说明** **`x86_64-pc-windows-msvc` `cargo check`** 与 **`cargo xtask acceptance`** 预期。  
- **侧车/子进程**：**不**让子进程继承 **协议 stdout**；**遵守** `requirements §5.8`。

**Pattern Enforcement：**

- **验证**：**`cargo fmt`**、**`cargo clippy`**、**`cargo test`**、**`cargo xtask acceptance`**。  
- **违规处理**：在 PR 中 **回写文档** 或 **回滚代码**；**不**合并「仅代码变、需求不变」的静默行为漂移。

### Pattern Examples

**Good Examples：**

- 在 **`xtask`** 增加子命令：更新 **`argh`** 结构、**帮助文**、**`--json` 测试**、**退出码表**。  
- 在 **`devshell-vm`** 改协议：先更新 **IPC 草案/版本** 与 **宿主解析**，再改 **侧车**。

**Anti-Patterns：**

- 在 **`xtask-todo-lib`** 直接 **`std::process::Command("git")`** 执行业务。  
- 向 **侧车协议通道** 打印 **调试 `println!`**。  
- **仅 Linux** 上能通过、**Windows MSVC** 上因 **`cfg`** 遗漏而 **类型错误或链接失败**。

## Project Structure & Boundaries

### Complete Project Directory Structure

以下为 **产品交付与工程门禁相关** 目录树（省略 IDE/技能仓库等大目录如 `.cursor/`、`.claude/`、`.specstory/`；以仓库根 `xtask_todo` 为准）。

```
xtask_todo/
├── Cargo.toml                 # workspace：resolver = "2"，members 见下
├── .cargo/
│   └── config.toml            # cargo xtask 等别名
├── .githooks/
│   └── pre-commit             # 与 cargo xtask git pre-commit 对齐
├── .github/
│   └── workflows/             # CI（若有）
├── containers/
│   └── devshell-vm/           # OCI 构建上下文（侧车镜像）
├── crates/
│   ├── todo/                  # 包名 xtask-todo-lib：领域 + devshell + cargo-devshell
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── model.rs, store.rs, repeat.rs, …   # 领域
│   │   │   ├── list/
│   │   │   ├── tests/                             # 领域单元测试
│   │   │   ├── bin/cargo_devshell/               # cargo devshell 入口
│   │   │   └── devshell/                          # REPL/parser/vfs/vm/sandbox/…
│   │   └── tests/                                 # crate 级集成测试（若有）
│   └── devshell-vm/             # β 侧车二进制（publish = false）
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── server.rs        # JSON 行协议
│       │   ├── guest_fs.rs
│       │   └── tests.rs
│       └── tests/               # 如 tcp_subprocess 等
├── xtask/                       # cargo xtask 唯一入口（publish = false）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs, lib.rs, run.rs
│   │   ├── todo/                # todo 子命令、io、与领域衔接
│   │   ├── acceptance/          # 一键验收与 acceptance-report
│   │   ├── bin/                 # 若有多二进制入口
│   │   ├── lima_todo/, gh.rs, ghcr.rs, git.rs, publish.rs, coverage.rs, …
│   │   └── tests/               # xtask 集成/子命令测试
│   └── tests/
│       └── integration.rs
├── docs/                        # 权威需求/设计/验收/用例（与 PRD inputDocuments 对齐）
│   ├── requirements.md, design.md, acceptance.md, test-cases.md, …
│   ├── devshell-vm-*.md, publishing.md, …
│   ├── reference/
│   ├── snippets/
│   └── superpowers/             # 草案与计划（specs/plans）
└── _bmad-output/
    └── planning-artifacts/
        ├── prd.md
        └── architecture.md      # 本文档
```

### Architectural Boundaries

**API Boundaries（对外契约）：**

- **CLI**：**`cargo xtask …`**、**`cargo devshell`**（由 **`xtask-todo-lib`** 安装提供）；**无**产品级 HTTP API。
- **机读输出**：**`--json`**、**退出码**（`requirements §6`）；**边界**在 **`xtask/src/todo/`** 与 **`crates/todo/src/devshell/`**（todo 子集）之间须语义一致。
- **β 侧车**：**宿主 `session_beta`（`crates/todo`）** ↔ **`devshell-vm` 进程**；**仅**通过 **JSON 行 + 约定传输**（stdio/UDS/tcp）。

**Component Boundaries：**

- **`xtask-todo-lib`**：**领域 + devshell 运行时**；**不**直接实现 **`git commit` 业务**。
- **`xtask`**：编排、**`.todo.json` 文件 I/O**（todo 路径）、**acceptance**、**git/gh/lima** 等宿主工具。
- **`devshell-vm`**：**侧车协议服务**；**不**嵌入 `xtask` 二进制。

**Data Boundaries：**

- **`.todo.json`**：磁盘真源由 **xtask `todo/io`** 与 **`devshell/todo_io`** 读写；**`TodoList<InMemoryStore>`** 为进程内视图。
- **会话/元数据**：**工作区内 JSON**（见 **`requirements §1.1`**）；**不**写入未文档化全局路径。

### Requirements to Structure Mapping（FR 类别 → 位置）

| FR 类别（PRD） | 主要落点 |
|----------------|----------|
| **Todo FR1–FR8** | `crates/todo/src/{model,store,list,…}`；CLI 编排 **`xtask/src/todo/`** |
| **Xtask/仓库 FR9–FR13** | **`xtask/src/`**（`git.rs`、`publish.rs`、`acceptance/` 等） |
| **Devshell FR14–FR19** | **`crates/todo/src/devshell/`**（`command/`、`parser`、`vfs`、`repl`…） |
| **VM / rustup FR20–FR25** | **`crates/todo/src/devshell/vm/`**（γ/β/SessionHolder/sandbox）；**`crates/devshell-vm/`** |
| **JSON/dry-run/init-ai FR26–FR29** | **`xtask/src/todo/`** + 文档 **`requirements`/`design`** |
| **跨平台/发布 FR30–FR31** | 各 crate **`Cargo.toml`**、**`docs/publishing.md`**、**`containers/devshell-vm/`** |
| **质量/追溯 FR32–FR34** | **`xtask/src/acceptance/`**、**`docs/test-cases.md`**、**`docs/`** 体系 |

### Integration Points

**Internal Communication：**

- **领域调用**：**`TodoList` API**（库内）。  
- **Devshell**：**`command/dispatch`** → builtins / **todo_builtin** / **SessionHolder**。  
- **VM**：**`session_gamma` / `session_beta`** ↔ **子进程或侧车**。

**External Integrations：**

- **Git / gh / rustup / cargo / limactl / podman**：**xtask** 或 **devshell vm 层** 显式调用；缺失时 **SKIP 或报错**（见 **`acceptance.md`**）。

**Data Flow（摘要）：**

- **Todo**：**磁盘 `.todo.json`** → **load** → **`InMemoryStore`** → 用户操作 → **save**（**`--dry-run` 跳过写**）。  
- **β**：**宿主 JSON 请求** → **侧车 `server.rs`** → **guest_fs/exec** → **回包 JSON 行**。

### File Organization Patterns

**Configuration Files：** 根 **`Cargo.toml`** workspace；**`.cargo/config.toml`**；各 member 独立 **`[lints.clippy]`**（见根 `Cargo.toml` 注释）。

**Source Organization：** 按 **`design.md` §1** 分层；**新文件**放入对应 **crate + 子模块**，避免跨 crate 循环依赖。

**Test Organization：** **`crates/todo/src/tests/`**、**`crates/todo/src/devshell/**/tests`**、**`xtask/src/tests/`**、**`crates/todo/tests/`**、**`xtask/tests/`**、**`devshell-vm` 内 `tests.rs` / `tests/`** — 与现有模块邻居测试习惯一致。

**Asset Organization：** **OCI**：**`containers/devshell-vm/`**；**Lima yaml** 等由 **`xtask lima-todo`** 与 **`docs/devshell-vm-gamma.md`** 约束。

### Development Workflow Integration

**Development：** **`cargo build --workspace`**；**`cargo xtask`** 别名见 **`.cargo/config.toml`**。

**Build：** **Cargo** 标准 `target/`（gitignore）；**feature** 如 **`beta-vm`** 见各 **`Cargo.toml`**。

**Deployment：** **crates.io** 发布 **`xtask-todo-lib`**；**GHCR** 发布侧车镜像（**`docs/devshell-vm-oci-release.md`**）；**无**传统服务端部署块。

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility：**  
Rust workspace、**argh**、**`.todo.json` + `Store`**、**xtask 编排**、**devshell-vm 侧车** 与 PRD/设计文档 **无冲突**；**Mode S/P** 与 **β/γ** 分治在「默认可降级」上一致。**Rust 2021（当前）** 与文中提及 **Rust 2024 edition（可选未来迁移）** 已区分，**不**强制混用。

**Pattern Consistency：**  
实现模式（JSON、退出码、crate 边界、stdio）与 **核心架构决策** 相互支撑；**反模式** 列表直接对应 **NFR-S2 / requirements §5.8**。

**Structure Alignment：**  
目录树与 **FR → 路径映射** 覆盖 PRD 分类；**三 crate** 边界与 **`design.md` §1** 一致。

### Requirements Coverage Validation

**Epic/Feature Coverage：**  
未使用独立 Epic 文件；以 **PRD FR1–FR34** 与 **NFR** 为准 —— 均在 **项目上下文**、**核心决策** 或 **结构映射** 中有对应支撑或显式排除（如 **无 HTTP API**）。

**Functional Requirements Coverage：**  
Todo / Xtask / Devshell / VM / JSON&AI / 跨平台发布 / 质量追溯 均有 **架构落点**；**FR34**「不默认承诺」项在 **数据/API/安全** 决策中重申。

**Non-Functional Requirements Coverage：**  
**NFR-P1/P2**（交互与管道缓冲）、**NFR-S1–S3**、**NFR-I1/I2**、**NFR-R1/R2** 均在 **性能/安全/集成/可靠性** 相关段落或 **patterns** 中可追溯到 **门禁或文档 ID**。

### Implementation Readiness Validation

**Decision Completeness：**  
关键技术（**workspace、argh、IPC 形态**）已记录；**精确版本**以 **`Cargo.lock`** 与 **发布流程** 为准（符合棕地实践）。

**Structure Completeness：**  
已给出 **可导航** 的目录树与 **FR 映射**；**非产品** 目录（IDE 技能库）已省略，避免噪音。

**Pattern Completeness：**  
**MUST/反例** 与 **验证命令** 已写清；**AI 代理** 可按 **docs + test-cases ID** 对齐实现。

### Gap Analysis Results

| 优先级 | 说明 |
|--------|------|
| **Critical** | 无阻塞缺口：当前文档可作为实现与评审的单一架构参考。 |
| **Important** | **Growth** 项（更全 CI 矩阵、Lima/Podman 自动化）属 **路线图**，非本架构文档缺失。 |
| **Nice-to-have** | 可选生成 **`project-context.md`**（`bmad-bmm-generate-project-context`）以浓缩 LLM 规则；**Validate PRD** 若需对外交付可再跑。 |

### Validation Issues Addressed

本轮验证 **未发现** 需立即修订的矛盾项；若 **PRD 或 design** 后续变更，应 **同步** 本文件 **§ 核心架构决策** 或 **§ 实现模式** 相关条目。

### Architecture Completeness Checklist

**Requirements Analysis**

- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**Architectural Decisions**

- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**Implementation Patterns**

- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**Project Structure**

- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** **READY FOR IMPLEMENTATION**（作为 **AI 与人工** 的架构一致性强约束）

**Confidence Level:** **high**（与现有 `docs/design.md`、`requirements.md`、PRD 对齐）

**Key Strengths：**  
棕地结构清晰；**crate 与 IPC 边界** 明确；**验收与 MSVC** 门禁可执行。

**Areas for Future Enhancement：**  
Edition 升级、更长驻 guest、更强 β IPC —— 见 PRD Vision / `docs/superpowers/specs/`。

### Implementation Handoff

**AI Agent Guidelines：**

- 严格按本文与 **`docs/requirements.md`**、**`docs/design.md`** 实现；冲突时 **先改文档** 或 **显式记债务**。
- 遵循 **§ Implementation Patterns & Consistency Rules** 中的 **MUST** 与 **Anti-Patterns**。
- 变更 **协议或 CLI 契约** 时同步 **测试** 与 **`test-cases.md`** / **验收 ID**。

**First Implementation Priority：**  
以棕地为基线：**`git clone` + `cargo build --workspace` + `cargo xtask acceptance`**；新功能按 **§ Core Architectural Decisions** 中的 **Implementation Sequence** 顺序影响面评估。

---

## Workflow Completion（Step 8）

**BMM 架构工作流已结束。** 文档路径：`_bmad-output/planning-artifacts/architecture.md`（frontmatter：`status: complete`，`stepsCompleted: 1–8`）。
