---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
classification:
  projectType: developer_tool
  domain: general
  complexity: medium
  projectContext: brownfield
inputDocuments:
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
workflowType: prd
briefCount: 0
researchCount: 0
brainstormingCount: 0
projectDocsCount: 27
---

# 产品需求文档（PRD）— xtask_todo

**作者：** Richard  
**日期：** 2026-03-25

## Executive Summary

**xtask_todo** 面向在 **Rust 工作区**内用终端完成日常工作的开发者：提供可发布的 **Todo 领域库（`xtask-todo-lib`）**、**`cargo xtask` 工作流 CLI**，以及可选的 **devshell**（VFS、脚本、REPL、管道/重定向、`rustup`/`cargo` 与 VM 协作）。棕地目标不是「再写一版需求」，而是把 **可机读接口（`--json` 等）、约定退出码、CI/pre-commit 可验证、以及 `cargo xtask acceptance` 可追溯** 与 **可选 γ（Lima）/ β（Podman + `devshell-vm`）** 放在同一工作区，使 **Linux/macOS/Windows（MSVC）** 下均能持续演进而不牺牲可验证性。

**问题与价值**：在「个人/小团队任务管理 + 仓库内自动化」场景下，工具需要 **可脚本化、可被 AI 消费、可跨平台安装**，同时 **VM/侧车是隔离与工具链一致性的可选路径**，而非绑架宿主。本仓库用 **需求 / 设计 / 验收 / 测试用例** 成体系对齐，避免能力与文档脱节。

### What Makes This Special

- **一体分层**：库 + xtask + 可选 devshell/VM；**Mode S（内存 VFS + 同步）** 与 **Mode P（guest 真源）** 显式建模，降级行为可写清、可测。
- **跨平台 VM 叙事**：Unix 上 **γ（Lima）**；Windows **无 Lima**，默认 **β + JSON 行侧车 + OCI 镜像版本对齐**，与 **`requirements` / `design` / `test-cases`** 一致。
- **可验收**：`cargo xtask acceptance`、NF 与 D 系列验收 ID、以及 MSVC 交叉 `cargo check` 等门禁，使棕地演进**可回归、可签字**。
- **核心洞察**：**默认仍可在宿主沙箱或关闭 VM 下工作**；重量级环境（Lima/Podman）**增强**而非**前提**。

## Project Classification

| 维度 | 结论 |
|------|------|
| **项目类型** | **developer_tool**（库 + CLI；强终端与 devshell 形态） |
| **领域** | **general**（软件开发 / 工程效率；非垂直监管行业） |
| **复杂度** | **medium**（跨平台、VM、IPC、可选环境依赖） |
| **情境** | **brownfield**（已有成体系 `docs/` 与实现） |

## Success Criteria

### User Success

- **Todo / CLI**：开发者能用 **`cargo xtask todo`** 完成 CRUD、列表过滤、导入导出、重复任务等 **`requirements §3`** 行为；**`--json`** 输出可被脚本解析；**退出码**符合 **`requirements §6`**（0/1/2/3）。
- **可编程/AI**：**`init-ai`** 能生成可用技能/命令入口；**`--dry-run`** 对数据零写入。
- **Devshell**：在 **无 VM** 时仍可用 **sandbox** 跑 **`rustup`/`cargo`**；在 **可选 VM** 下，Unix **γ** / Windows **β** 路径与 **`requirements §5`、`design §1.4`** 一致；**Mode P** 下工程树与 guest 视图一致（以 **`test-cases.md`** 与手工项为准）；TTY 交互下支持通过 **↑/↓** 快速回放历史命令。
- **「值得用」的瞬间**：一键 **`cargo xtask acceptance`** 绿、或 **`todo`/`devshell`** 在典型工作流中无需查源码即可完成任务。

### Business Success

- **发布物**：**`xtask-todo-lib`** 在 **crates.io** 可安装；**`cargo install`** 用户可按 **`docs/devshell-vm-windows.md`** 在 **Podman 可选** 下走通 β（或显式降级 **host**）。
- **演进纪律**：破坏性 CLI/行为变更在 **主版本策略** 与 **CHANGELOG** 中可追踪（**`requirements §7`**、验收 **NF-3/NF-4** 为文档/流程项）。
- **协作与可追溯**：需求—设计—用例—验收 ID 可对照（**`test-cases.md`**、**`acceptance.md`**）。

### Technical Success

- **自动化门禁**：**`cargo xtask acceptance`** 退出码 **0**（含合理 **SKIP**）；覆盖 **`cargo test`**（**xtask-todo-lib / xtask / devshell-vm**）、**NF-1/2/6** 文件检查、**`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**（未装 target 时 **SKIP** 与报告一致）。
- **覆盖率目标**：**`cargo xtask coverage`** 对 **xtask-todo-lib**、**xtask** 摘要 **≥95%**（以 **`xtask/src/coverage.rs`** 排除规则为准）。
- **Pre-commit 对齐**：**`.githooks/pre-commit`** 与 **`cargo xtask git pre-commit`** 与 **`requirements §4/§7.2`** 描述一致。

### Measurable Outcomes

| 结果 | 可验证信号 |
|------|------------|
| 回归可重复 | CI/本地 **`cargo xtask acceptance`** 与 **`acceptance-report.md`** 一致 |
| 跨平台不翻车 | **MSVC `cargo check`** 不因平台条件编译而长期失败 |
| VM 可选 | **`DEVSHELL_VM=off` / `BACKEND=host`** 下核心路径仍可用 |

## Product Scope

### MVP - Minimum Viable Product

棕地语境下，**MVP = 当前对外承诺的「已发布 + 可验收」基线**：**Todo + xtask + devshell 宿主路径** + **文档化的 VM 可选路径** + **acceptance + MSVC 检查** 作为合并与发布前的默认条。

### Growth Features (Post-MVP)

- **环境与矩阵**：更多 **CI 矩阵**、Lima/Podman **全链路** 自动化（当前部分为 **手工/环境**，见 **`acceptance.md §2`**）。
- **体验与工具**：**`gh log`**、**lima-todo**、覆盖率排除项收紧等随 **`docs/tasks`** 与路线图演进。

### Vision (Future)

- **更长驻的 guest 连接**、更强 **β IPC**、以及 **Mode P** 下更完整的 **会话/导出** 语义（见 **`docs/superpowers/specs/`** 与 **guest-primary** 设计），在 **不破坏默认 Mode S** 的前提下迭代。

## User Journeys

### 旅程 1：日常开发者 —— Todo 与脚本化成功路径

**小周**在特性分支上修 bug，习惯用终端管任务。**开场**：仓库根已有 `.todo.json`，需要快速记一条「修完某 issue」的待办。**发展**：执行 `cargo xtask todo add "fix: …" --json`，脚本解析 `status`；完成后 `complete`，列表里状态正确。**高潮**：CI 里同一套命令在 Linux runner 上可重复。**结局**：个人任务流与自动化一致，无需打开 GUI。

**揭示的能力需求**：Todo CRUD、`--json`、稳定退出码、与 **`.todo.json`** 路径约定一致。

### 旅程 2：同一开发者 —— 参数/数据错误与恢复（边缘路径）

**开场**：误传非法日期或不存在 id。**发展**：CLI 返回 **2/3** 与可读 **stderr**；若加 `--json`，错误结构可机读。**高潮**：用户用 **dry-run** 预览 `add`/`update`，确认无误再实写。**结局**：自动化与人工都不会静默破坏数据。

**揭示的能力需求**：**`requirements §6`** 退出码、**`--dry-run`**、JSON 错误体。

### 旅程 3：维护者/贡献者 —— 合并前门禁与发布

**开场**：准备 PR 或发版。**发展**：本地跑 **`cargo xtask acceptance`**，必要时 **`cargo xtask git pre-commit`**；关注 **NF-1/2/6**、**MSVC check** 是否绿或合理 SKIP。**高潮**：**`acceptance-report.md`** 与人工验收表对照，签字有据。**结局**：棕地演进不靠「感觉全绿」。

**揭示的能力需求**：**acceptance** 子命令、**`.githooks/pre-commit`** 与文档一致、**`publishing.md`** 流程。

### 旅程 4：Windows + Podman 用户 —— β 侧车全链路（环境与排错）

**开场**：在 Windows 上 **`cargo install xtask-todo-lib`**，希望 **`cargo devshell`** 里跑 **`cargo new`/`run`**。**发展**：按 **`devshell-vm-windows.md`** 处理 **stdio**、镜像拉取或 **ELF** 路径；若出现 **「sidecar response is not JSON」**，按 **requirements §5.8** 与日志排查旧侧车/stdout 污染。**高潮**：在挂载工作区上看到真实工程树。**结局**：Unix γ 文档不必强行套在 Windows 上；**降级 host** 仍可用。

**揭示的能力需求**：**β**、**OCI 版本对齐**、**exec 超时**、**stdio 与协议隔离**、TTY 历史命令回放（↑/↓）、文档化 **SKIP/手工** 项。

### 旅程 5：AI/集成方 —— 可机读与技能生成

**开场**：IDE 或 Agent 要批量操作 todo。**发展**：统一 **`--json`** 成功/失败；运行 **`todo init-ai`** 生成技能片段。**高潮**：工具行为可被提示词与脚本稳定依赖。**结局**：「规范驱动」协作可落地。

**揭示的能力需求**：**US-A1～A4**、**init-ai** 输出结构。

### Journey Requirements Summary

| 旅程 | 主要能力域 |
|------|------------|
| 1 | Todo 领域 API、xtask todo、持久化约定 |
| 2 | 错误模型、退出码、dry-run、JSON 错误 |
| 3 | acceptance、git/pre-commit、发布与元数据 |
| 4 | devshell VM、β、Windows 路径、降级、TTY 历史命令回放 |
| 5 | JSON CLI、init-ai、AI 可消费契约 |

## Domain-Specific Requirements

### 合规与监管

本产品**默认不**承担 **HIPAA、PCI-DSS、金融行业牌照**等强监管场景的合规责任；**`.todo.json`** 为**用户本地工作区**文件，无多租户账户与云端托管。**若**在受监管或企业策略环境下使用，由部署方自行评估数据分级、出境与留存。

### 技术与安全约束

- **供应链**：依赖 **crates.io** 与 **语义化版本**；合并与发布前依赖 **`cargo xtask acceptance`**、**pre-commit** 与 **MSVC 交叉检查**，降低不可编译或行为漂移进入主线。
- **子进程与 VM**：**`rustup`/`cargo`**、**`limactl`**、**Podman** 等属**用户环境**；实现须 **失败可诊断**、**可降级**（宿主 sandbox、**`DEVSHELL_VM=off`** / **`BACKEND=host`**），避免在 VM 不可用时静默进入不一致状态。
- **侧车 IPC**：协议 **stdout** 仅用于 JSON 行；子进程 **stdout/stderr** 不得破坏宿主 **`read_json_line`**（与 **requirements §5.8**、**TC-D-VM-4** 一致）。

### 集成与运行环境

- **Git**、**GitHub CLI（`gh`）**、**Rust toolchain**、可选 **Lima/Podman** 为外部依赖；**`requirements` / `devshell-vm-*`** 已列前置条件与 **SKIP/手工** 项。

### 风险与缓解

| 风险 | 缓解 |
|------|------|
| 仅 Linux 开发导致 Windows 编译破损 | **NF-5/D8**、**`x86_64-pc-windows-msvc` `cargo check`** |
| VM/侧车不可用时的路径分裂 | **effective workspace mode** 降级、**Mode S** 默认可用 |
| 侧车/镜像与库版本不一致 | **GHCR 标签与 `CARGO_PKG_VERSION` 对齐**（**`docs/devshell-vm-oci-release.md`**） |

## Innovation & Novel Patterns

### Detected Innovation Areas

- **同一工作区内的三重界面**：可发布 **库**、**`cargo xtask` 编排**、可选 **进程内 devshell 语言** —— 三者共享 **Todo 与验收语义**，减少「脚本一套、交互另一套」的分裂。
- **Mode S / Mode P 显式化**：在「内存 VFS + push/pull」与「guest 工程树真源」之间做**可降级**的**工作区模式**，而不是把 VM 当作黑箱。
- **Unix γ / Windows β 分治**：在**无 Lima** 的 Windows 上以 **JSON 行 + stdio + OCI 侧车** 对齐 **Unix Lima** 能力边界，并以 **协议隔离（stdout）** 保证可解析性。
- **验收即产品**：**`cargo xtask acceptance`** 将 **NF/D/测试** 部分固化为**可重复命令**，与 **AI/自动化** 友好。

### Market Context & Competitive Landscape

- **Todo/CLI**：独立工具很多；**本仓库差异**在于与 **Rust workspace、xtask、devshell、VM** 的**同一套需求—测试—验收**闭环。
- **开发环境隔离**：容器、VM、远程开发方案成熟；本方案强调 **可选重量级依赖**、**宿主可跑**、**crates.io 可安装**，定位在「个人/小团队 + 可验证」而非全托管平台。

### Validation Approach

- **自动化**：**acceptance**、**pre-commit**、**MSVC check**、**`cargo test`/`coverage`**。
- **环境依赖项**：Lima/Podman 路径以 **SKIP/手工** 明示（**`acceptance.md §2`**），避免「假绿」。
- **协议与侧车**：**`devshell-vm`** 单元/集成测试（**TC-D-VM-***）与 **Windows 手工**对照。

### Risk Mitigation

| 创新点 | 若未达预期 |
|--------|------------|
| Mode P 复杂度 | 默认 **Mode S**；**`DEVSHELL_VM_WORKSPACE_MODE`** 可关 |
| β/镜像漂移 | **版本化 GHCR**、**`DEVSHELL_VM_CONTAINER_IMAGE`** 覆盖 |
| 「创新」难维护 | **需求/设计/用例**三处同步，**breaking** 走版本与 CHANGELOG |

## Developer Tool Specific Requirements

### Project-Type Overview

**xtask_todo** 作为 **developer_tool**：对外以 **Rust crate（`xtask-todo-lib`）** 为主交付物，辅以 **workspace 内 `xtask` 二进制**（不发布 crates.io）与 **侧车 `devshell-vm`**（**`publish = false`**，OCI 见 **`docs/devshell-vm-oci-release.md`**）。目标用户为 **Rust 工作区内的开发者**与 **AI/脚本集成方**（**`--json`**、**`init-ai`**）。

### Technical Architecture Considerations

- **Workspace**：根 **`Cargo.toml`** 管理 **`crates/todo`**、**`xtask`**、**`crates/devshell-vm`**；**resolver**、**feature**（如 **`beta-vm`**）与 **`cfg` 分平台** 见 **`design.md §1`**。
- **公开 API 与边界**：领域逻辑在 **`xtask-todo-lib`**；**`.todo.json` I/O** 在 **xtask** 与 **devshell `todo_io`**；**不**在库内直接绑定全局 `git`/`gh` 业务规则（**xtask** 编排）。
- **跨平台**：**Windows MSVC** 为 **一等**编译目标（**pre-commit / NF-5**）；**非 Unix** 上 **γ** 相关 API 以 **桩/分支** 保证类型检查（**`design.md §1.4`**）。

### language_matrix

| 语言 / 版本 | 范围 |
|-------------|------|
| **Rust** | **2021**（各 member 自管 **`[lints.clippy]`**） |
| 其他 | **无**内嵌解释型 DSL 运行时；**devshell** 为自研 shell 语法（**`.dsh`**），非通用 Bash 超集 |

### installation_methods

| 方式 | 对象 |
|------|------|
| **Workspace / `cargo install --path`** | 贡献者、本仓库克隆 |
| **`cargo install xtask-todo-lib`** | crates.io 用户（**`cargo devshell`** 随库二进制发布） |
| **`rustup target add x86_64-pc-windows-msvc`** | 开发机交叉自检（**§7.2**） |
| **Podman + GHCR** | Windows **β** 侧车（可选，见 **`devshell-vm-windows.md`**） |

### api_surface

| 类别 | 要点 |
|------|------|
| **库** | **`TodoList<S: Store>`**、**`Todo`**、**`TodoId`**、**`ListOptions`**、**`RepeatRule`** 等（**`design.md §1.3`**） |
| **CLI（xtask）** | **`cargo xtask todo …`**、**`--json`** / **`--dry-run`**、退出码约定（**`requirements §3/§6`**） |
| **Devshell** | **builtin** 白名单、**`rustup`/`cargo`** 经 **SessionHolder**；**非**任意宿主 shell |

### code_examples

- **单元/集成测试**即契约示例：**`crates/todo/src/tests/`**、**`xtask/src/tests/`**、**`devshell/tests/`**。
- **AI 技能**：**`todo init-ai --for-tool … --output …`** 生成调用说明（**`requirements §3/§6`**）。

### migration_guide

- **语义化版本**：**`xtask-todo-lib`** 在 **`crates/todo/Cargo.toml`**；破坏性变更需 **CHANGELOG** 与主版本策略（**`requirements §7`**、验收 **NF-3**）。
- **数据文件**：**`.todo.json`** 由产品定义；**未承诺**自动迁移流水线（**`requirements §2`**）；大版本升级需文档说明。

### Implementation Considerations

- **跳过（CSV）**：**应用商店合规**、**视觉/UX 稿** 不作为本 PRD 必需要求；开发者体验以 **CLI、帮助、`--help` 与 README** 为准（**NF-4**）。
- **包与文档**：**`cargo publish -p xtask-todo-lib --dry-run`** 与 **`publishing.md`** 为发布前检查清单。

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP 方法**：**可验证的开发者工具 MVP（problem-solving + platform）** —— 以 **`cargo xtask acceptance`** 与 **pre-commit** 为「是否可合并/可发布」的底线，而不是以「功能数量最少」为唯一目标。

**资源与技能**：**Rust** 全栈（领域 + CLI + 可选 devshell/vm）；**发布**需熟悉 **crates.io** 与 **CI**；**VM 路径**需有人能维护 **Lima/Podman** 文档与 **手工验收**。

### MVP Feature Set（Phase 1）

**支撑的核心旅程**：旅程 **1–3、5**（Todo/脚本化、错误恢复、门禁与 AI 可机读）；旅程 **4（Windows β 全链路）** 在 MVP 中列为 **环境依赖 + 手工**，自动化为 **Growth**。

**能力边界**与上文 **Product Scope → MVP** 一致；此处强调 **合并/发布底线**：**acceptance + pre-commit + MSVC check（或文档化 SKIP）**，以及 **可追溯文档集**（**`requirements` / `design` / `acceptance` / `test-cases`**）。

### Post-MVP Features

**Phase 2（Growth）**

- **更多 CI 矩阵**、Lima/Podman **端到端自动化**（减少 **`acceptance.md §2`** 手工项）。
- **`gh log`、lima-todo、覆盖率** 等体验与度量增强（见 **`docs/tasks`**、**`test-coverage.md`**）。

**Phase 3（Expansion / Vision）**

- **更长驻 guest 连接**、更强 **β IPC**、**Mode P** 会话/导出完整语义（见 **`superpowers/specs/`**、guest-primary 规划）。

### Risk Mitigation Strategy

| 类别 | 缓解 |
|------|------|
| **技术** | **Mode S 默认**；**β** 可 **host** 降级；**跨平台** 靠 **MSVC check** 与 **cfg** 审查 |
| **市场/采用** | **crates.io 可安装**、**README Windows**、**GHCR 版本对齐** |
| **资源** | **acceptance** 一键回归；**SKIP** 明示；大功能 **分 PR**、**guest-primary** 已分 sprint（见计划文档） |

## Functional Requirements

### 待办领域（Todo）

- **FR1**：终端用户可以通过命令行创建待办项，并在合法校验失败时得到非成功结果且**不**产生无效数据。
- **FR2**：终端用户可以列出待办项，并在无数据时得到可理解的空结果。
- **FR3**：终端用户可以按约定规则过滤、排序待办列表（含状态、日期、标签等维度，以产品约定为准）。
- **FR4**：终端用户可以将待办项标为完成或删除，并在引用不存在或非法标识时得到约定错误语义。
- **FR5**：终端用户可以查看单条待办详情，并可以更新可选字段（描述、截止日期、优先级、重复规则等，以产品约定为准）。
- **FR6**：终端用户可以按关键词搜索待办项并查看统计摘要。
- **FR7**：终端用户可以将待办数据导出为约定交换格式，并可以从文件导入（含替换策略选项）。
- **FR8**：终端用户可以使用重复任务相关能力（创建下一实例、终止条件、`--no-next` 等，以产品约定为准）。

### Xtask 工作流与仓库工具

- **FR9**：开发者可以通过单一入口调用工作区编排的开发者子命令（格式化、静态分析、测试、清理等，以产品约定为准）。
- **FR10**：开发者可以运行与 Git 相关的辅助子命令（暂存、带检查的提交等，以产品约定为准）。
- **FR11**：开发者可以运行与发布相关的辅助子命令（以 **`publishing.md`** 为准）。
- **FR12**：开发者可以运行一键验收命令，得到**汇总报告**并区分自动化通过与需人工/环境的项。
- **FR13**：贡献者可以在合并前运行与 CI 对齐的提交前检查（或与该检查等价的路径）。

### Devshell：交互、语言与内置能力

- **FR14**：用户可以以交互式会话启动 devshell，或以脚本文件非交互执行。
- **FR15**：用户可以在会话内进行目录导航、列目录、读写文件、创建目录等**内置**文件操作（以白名单命令为准）。
- **FR16**：用户可以使用管道与重定向组合多条内置命令。
- **FR17**：用户可以在 devshell 内使用待办能力的**受支持子集**，并与工作区约定数据文件一致。
- **FR18**：用户可以获得与内置命令一致的帮助信息。
- **FR19**：用户在 TTY 环境下可以获得命令与路径补全（以产品约定行为为准）。
- **FR35**：用户在 TTY 交互会话中可以通过 **↑/↓** 浏览并复用当前会话已执行的命令历史；执行复用命令时应保持与手动输入一致的解析与执行语义。

### Rust 工具链执行与环境（含可选 VM）

- **FR20**：用户可以在 devshell 内调用 **`rustup`/`cargo`**，并在**未启用 VM** 时通过宿主侧沙箱路径执行（导出—执行—回写语义，以产品约定为准）。
- **FR21**：用户在启用 VM 时，可以在 **Unix** 类环境下通过 **γ 类后端**在隔离环境中执行 **`rustup`/`cargo`**（以产品约定为准）。
- **FR22**：用户在 **Windows** 上在启用 VM 时，可以通过 **β 类后端**与侧车协议执行 **`rustup`/`cargo`**（以产品约定为准）。
- **FR23**：用户可以将 VM **关闭**或选择**仅宿主**执行路径，且核心 devshell 能力仍可用（以降级规则为准）。
- **FR24**：用户在**有效条件**下可以使用 **guest 为主工作区**的模式，使工程树操作与 **`rustup`/`cargo`** 针对同一视图（以 **Mode P** 规则与降级为准）。
- **FR25**：用户可以持久化与会话相关的元数据到**工作区内**约定路径（以 **`requirements §1.1`** 为准）。

### 可编程接口与 AI 集成

- **FR26**：用户可以在支持的子命令上使用**结构化 JSON** 输出成功与失败结果。
- **FR27**：用户可以在修改类命令上使用 **dry-run**，使**不**写入约定数据文件。
- **FR28**：用户可以使用**约定退出码**区分成功、一般错误、参数错误与数据错误（以 **`requirements §6`** 为准）。
- **FR29**：用户可以生成面向外部工具（含 AI 助手）的**初始化/技能**材料（以 **`init-ai`** 约定为准）。

### 跨平台与发布物

- **FR30**：用户可以在 **Linux、macOS、Windows（MSVC）** 上获取**可编译**的库与二进制交付物（以产品声明为准）。
- **FR31**：用户可以通过**包注册表**安装主库（含 **`cargo devshell`** 入口，以发布策略为准）。

### 质量、追溯与文档一致性

- **FR32**：维护者可以**追溯**需求、设计、用例与验收 ID（以仓库文档体系为准）。
- **FR33**：维护者可以运行**自动化测试**与**覆盖率**工作流（以 `xtask` 与排除规则为准）。
- **FR34**：系统**不得**将本 PRD 未列能力**默认**承诺为 **HTTP API、多用户权限、`.todo.json` 自动迁移流水线**等（与 **`requirements §2`**「当前不承诺」一致）；若未来纳入，须**显式增改 FR**。

## Non-Functional Requirements

### Performance

- **NFR-P1**：交互式 CLI 在典型工作区规模下，**单次子命令**（无外部网络等待时）应在**可交互等待**内返回；**不**对 **`cargo`/`rustup` 编译时长**作上限承诺（受用户工程与机器影响）。
- **NFR-P2**：**管道阶段缓冲**有**明确上限**（以 **`PIPELINE_INTER_STAGE_MAX_BYTES`** 等产品常量为准），超限须**失败可见**，避免静默耗尽内存。

### Security

- **NFR-S1**：**`.todo.json`** 与 devshell 会话数据默认位于**用户工作区**；产品**不**将待办内容作为**云端账户**模型处理；用户需自行管理磁盘访问权限与备份。
- **NFR-S2**：**侧车/VM 路径**须保证 **IPC 通道**不被非协议输出污染（与 **requirements §5.8** 一致）；**子进程**执行面保持**白名单/可控**（非任意 shell）。
- **NFR-S3**：依赖 **crates.io** 与 **语义化版本**；发布流程须能复现构建与检查（见 **pre-commit / acceptance**）。

### Integration

- **NFR-I1**：与 **Git**、**`gh`**、**`rustup`/`cargo`**、可选 **Lima/Podman** 的集成须**在缺失时显式失败或 SKIP**，并**文档化**前置条件（与 **`acceptance.md §2`** 一致）。
- **NFR-I2**：**JSON 行侧车协议**须版本化握手并保持 **一行一对象** 成帧，以便宿主解析与排错。

### Reliability & Maintainability

- **NFR-R1**：**主路径**（无 VM / Mode S）须在**不安装** Lima/Podman 的环境下保持可用，以支持 CI 与最小依赖用户。
- **NFR-R2**：**破坏性变更**须在主版本与 **CHANGELOG** 中可追溯（与 **NF-3** 对齐）。

### 不适用（本产品中刻意不单列）

- **大规模多租户扩展**、**Web 可达性 WCAG 全量** 等不作为本 PRD 的独立 NFR 块；若未来产品形态变化，再增补对应类别。
