---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
---

# xtask_todo - Epic Breakdown

## Overview

本文档将 PRD 与架构决策分解为以用户价值为中心的 Epic 与用户故事，供实现与排期使用。输入文档未包含独立 UX 规格文件；终端/CLI 体验以 PRD 与 `docs/` 为准。

## Requirements Inventory

### Functional Requirements

```
FR1: 终端用户可以通过命令行创建待办项，并在合法校验失败时得到非成功结果且不产生无效数据。
FR2: 终端用户可以列出待办项，并在无数据时得到可理解的空结果。
FR3: 终端用户可以按约定规则过滤、排序待办列表（含状态、日期、标签等维度，以产品约定为准）。
FR4: 终端用户可以将待办项标为完成或删除，并在引用不存在或非法标识时得到约定错误语义。
FR5: 终端用户可以查看单条待办详情，并可以更新可选字段（描述、截止日期、优先级、重复规则等，以产品约定为准）。
FR6: 终端用户可以按关键词搜索待办项并查看统计摘要。
FR7: 终端用户可以将待办数据导出为约定交换格式，并可以从文件导入（含替换策略选项）。
FR8: 终端用户可以使用重复任务相关能力（创建下一实例、终止条件、`--no-next` 等，以产品约定为准）。
FR9: 开发者可以通过单一入口调用工作区编排的开发者子命令（格式化、静态分析、测试、清理等，以产品约定为准）。
FR10: 开发者可以运行与 Git 相关的辅助子命令（暂存、带检查的提交等，以产品约定为准）。
FR11: 开发者可以运行与发布相关的辅助子命令（以 publishing.md 为准）。
FR12: 开发者可以运行一键验收命令，得到汇总报告并区分自动化通过与需人工/环境的项。
FR13: 贡献者可以在合并前运行与 CI 对齐的提交前检查（或与该检查等价的路径）。
FR14: 用户可以以交互式会话启动 devshell，或以脚本文件非交互执行。
FR15: 用户可以在会话内进行目录导航、列目录、读写文件、创建目录等内置文件操作（以白名单命令为准）。
FR16: 用户可以使用管道与重定向组合多条内置命令。
FR17: 用户可以在 devshell 内使用待办能力的受支持子集，并与工作区约定数据文件一致。
FR18: 用户可以获得与内置命令一致的帮助信息。
FR19: 用户在 TTY 环境下可以获得命令与路径补全（以产品约定行为为准）。
FR20: 用户可以在 devshell 内调用 rustup/cargo，并在未启用 VM 时通过宿主侧沙箱路径执行（导出—执行—回写语义，以产品约定为准）。
FR21: 用户在启用 VM 时，可以在 Unix 类环境下通过 γ 类后端在隔离环境中执行 rustup/cargo（以产品约定为准）。
FR22: 用户在 Windows 上在启用 VM 时，可以通过 β 类后端与侧车协议执行 rustup/cargo（以产品约定为准）。
FR23: 用户可以将 VM 关闭或选择仅宿主执行路径，且核心 devshell 能力仍可用（以降级规则为准）。
FR24: 用户在有效条件下可以使用 guest 为主工作区的模式，使工程树操作与 rustup/cargo 针对同一视图（以 Mode P 规则与降级为准）。
FR25: 用户可以持久化与会话相关的元数据到工作区内约定路径（以 requirements §1.1 为准）。
FR26: 用户可以在支持的子命令上使用结构化 JSON 输出成功与失败结果。
FR27: 用户可以在修改类命令上使用 dry-run，使不写入约定数据文件。
FR28: 用户可以使用约定退出码区分成功、一般错误、参数错误与数据错误（以 requirements §6 为准）。
FR29: 用户可以生成面向外部工具（含 AI 助手）的初始化/技能材料（以 init-ai 约定为准）。
FR30: 用户可以在 Linux、macOS、Windows（MSVC）上获取可编译的库与二进制交付物（以产品声明为准）。
FR31: 用户可以通过包注册表安装主库（含 cargo devshell 入口，以发布策略为准）。
FR32: 维护者可以追溯需求、设计、用例与验收 ID（以仓库文档体系为准）。
FR33: 维护者可以运行自动化测试与覆盖率工作流（以 xtask 与排除规则为准）。
FR34: 系统不得将本 PRD 未列能力默认承诺为 HTTP API、多用户权限、.todo.json 自动迁移流水线等；若未来纳入，须显式增改 FR。
FR35: 用户在 TTY 交互会话中可以通过 ↑/↓ 浏览并复用当前会话已执行的命令历史；执行复用命令时应保持与手动输入一致的解析与执行语义。
```

### NonFunctional Requirements

```
NFR-P1: 交互式 CLI 在典型工作区规模下，单次子命令（无外部网络等待时）应在可交互等待内返回；不对 cargo/rustup 编译时长作上限承诺。
NFR-P2: 管道阶段缓冲有明确上限（以 PIPELINE_INTER_STAGE_MAX_BYTES 等产品常量为准），超限须失败可见，避免静默耗尽内存。
NFR-S1: .todo.json 与 devshell 会话数据默认位于用户工作区；产品不将待办内容作为云端账户模型处理。
NFR-S2: 侧车/VM 路径须保证 IPC 通道不被非协议输出污染；子进程执行面保持白名单/可控（非任意 shell）。
NFR-S3: 依赖 crates.io 与语义化版本；发布流程须能复现构建与检查（见 pre-commit / acceptance）。
NFR-I1: 与 Git、gh、rustup/cargo、可选 Lima/Podman 的集成须在缺失时显式失败或 SKIP，并文档化前置条件。
NFR-I2: JSON 行侧车协议须版本化握手并保持一行一对象成帧，以便宿主解析与排错。
NFR-R1: 主路径（无 VM / Mode S）须在不安装 Lima/Podman 的环境下保持可用。
NFR-R2: 破坏性变更须在主版本与 CHANGELOG 中可追溯（与 NF-3 对齐）。
```

### Additional Requirements

（来自 Architecture，影响 Epic 划分与实现顺序。）

- **棕地基线**：以现有 Cargo workspace 为 starter，非 Web/oclif 模板；新贡献者路径为 `git clone` + `cargo build --workspace` + `cargo xtask acceptance`。
- **Crate 边界**：`xtask-todo-lib`（领域）/ `xtask`（编排、`.todo.json` I/O、acceptance、git/gh）/ `devshell-vm`（β 侧车）；库内不绑定 git/gh 业务规则。
- **持久化**：权威数据为工作区 `.todo.json`；`Store` + `InMemoryStore`；load/save 在 crate 边界外显式化。
- **对外契约**：`--json`、requirements §6 退出码、`--dry-run` 在 xtask todo 与 devshell todo 子集间一致。
- **VM / IPC**：一行一条 JSON、handshake；侧车 stdout 专用于协议；子进程 stdout/stderr 管道化，避免污染宿主 `read_json_line`（requirements §5.8）。
- **CLI**：argh（derive）；JSON 字段与退出码变更需考虑稳定性与 CHANGELOG。
- **跨平台**：Windows MSVC 为一等目标；非 Unix 上 γ/Lima 以 cfg/桩保证类型检查。
- **Mode S / Mode P**：有效工作区模式与冲突时降级 Mode S（requirements §1.1、guest-primary 设计）。
- **质量门禁**：pre-commit 与 `cargo xtask git pre-commit` 对齐；`x86_64-pc-windows-msvc` cargo check；`cargo xtask coverage` 阈值与排除规则。
- **发布**：`xtask-todo-lib` → crates.io；devshell-vm 镜像 GHCR 与 `CARGO_PKG_VERSION` 对齐（devshell-vm-oci-release.md）。
- **实现顺序提示**（架构）：先领域/契约 → CLI → devshell/todo_io → β/IPC → 发布/镜像。

### UX Design Requirements

（规划产物中无独立 `*ux*.md`；以下为 PRD 明示的终端/可访问性相关约束，便于故事验收对齐。）

```
UX-DR1: 主路径为终端 CLI（cargo xtask、cargo devshell）；体验以 --help、README 与 requirements 为准，无单独视觉稿交付要求（PRD/NF-4）。
UX-DR2: TTY 下命令与路径补全行为与产品约定一致（对应 FR19）。
UX-DR3: 错误与成功在人机与 --json 两种模式下均可理解或可机读（与 FR26–FR28 一致）。
```

### FR Coverage Map

```
FR1: Epic 1 — 创建待办与校验
FR2: Epic 1 — 列出与空状态
FR3: Epic 1 — 过滤与排序
FR4: Epic 1 — 完成/删除与错误语义
FR5: Epic 1 — 详情与更新字段
FR6: Epic 1 — 搜索与统计
FR7: Epic 1 — 导入导出
FR8: Epic 1 — 重复任务
FR9: Epic 2 — 工作区编排子命令
FR10: Epic 2 — Git 辅助
FR11: Epic 2 — 发布辅助
FR12: Epic 2 — 一键验收
FR13: Epic 2 — 提交前检查
FR14: Epic 3 — 交互/脚本会话
FR15: Epic 3 — 内置文件操作
FR16: Epic 3 — 管道与重定向
FR17: Epic 3 — devshell 内 todo 子集
FR18: Epic 3 — 帮助
FR19: Epic 3 — 补全
FR20: Epic 4 — 宿主沙箱执行 rustup/cargo
FR21: Epic 4 — Unix γ/Lima 路径
FR22: Epic 4 — Windows β 与侧车
FR23: Epic 4 — VM 关闭与降级
FR24: Epic 4 — Mode P / guest 工作区
FR25: Epic 4 — 会话元数据持久化
FR26: Epic 5 — JSON 成功/失败载荷
FR27: Epic 5 — dry-run
FR28: Epic 5 — 退出码约定
FR29: Epic 5 — init-ai 技能材料
FR30: Epic 6 — 跨平台可编译交付物
FR31: Epic 6 — crates.io 与 cargo devshell
FR32: Epic 7 — 需求/设计/用例/验收追溯
FR33: Epic 7 — 测试与覆盖率工作流
FR34: Epic 7 — 产品边界与不默认承诺项
FR35: Epic 8 — Devshell 历史命令回放（↑/↓）
```

### NFR → Epic 对照（摘要）

| NFR | 主要 Epic |
|-----|-----------|
| NFR-P1 | 全局；Epic 5 对 CLI 可交互性敏感 |
| NFR-P2 | Epic 3（管道缓冲） |
| NFR-S1 | Epic 1、4（数据路径） |
| NFR-S2, NFR-I2 | Epic 4（侧车/IPC） |
| NFR-S3 | Epic 2、6、7 |
| NFR-I1 | Epic 2、4 |
| NFR-R1 | Epic 4 |
| NFR-R2 | Epic 6、7 |

## Epic List

### Epic 1：终端待办与数据生命周期

用户在终端内完成待办的创建、浏览、过滤、完成/删除、搜索、导入导出与重复任务，并在错误输入下得到安全、可理解的反馈，且不破坏 `.todo.json` 数据完整性。

**FRs covered:** FR1–FR8  
**NFR 相关:** NFR-P1、NFR-S1（局部）

### Epic 2：工作流编排、仓库与质量门禁

开发者通过单一 `cargo xtask` 入口完成格式化/测试/清理等编排，使用 Git 与发布辅助命令，运行一键验收与合并前检查，使合并与发布流程可重复、可与 CI 对齐。

**FRs covered:** FR9–FR13  
**NFR 相关:** NFR-S3、NFR-I1（工具缺失时 SKIP/失败）

### Epic 3：Devshell 会话、文件能力与 shell 体验

用户以交互或脚本方式使用 devshell，完成受控文件操作、管道/重定向、待办子集、帮助与 TTY 补全，体验与领域数据文件约定一致。

**FRs covered:** FR14–FR19  
**NFR 相关:** NFR-P2（管道缓冲上限）

### Epic 4：Rust 工具链、VM 与工作区模式

用户在宿主沙箱或可选 VM（Unix γ / Windows β）中执行 `rustup`/`cargo`，在 VM 不可用时降级；在条件满足时使用 Mode P 使 guest 与宿主工程视图一致；会话元数据落在工作区约定路径。侧车协议保持 stdout 可解析、不被污染。

**FRs covered:** FR20–FR25  
**NFR 相关:** NFR-S2、NFR-I1、NFR-I2、NFR-R1

### Epic 5：机读 CLI、错误模型与 AI 集成

用户与自动化工具可通过 `--json`、稳定退出码、`--dry-run` 与 `init-ai` 以可预测方式集成，成功与失败均机读、可测试。

**FRs covered:** FR26–FR29  
**NFR 相关:** NFR-P1

### Epic 6：跨平台交付与发布

用户在 Linux、macOS、Windows（MSVC）上获得可编译的库与二进制；通过 crates.io 安装 `xtask-todo-lib` 及 `cargo devshell`；发布物与破坏性变更有 semver/CHANGELOG 可追溯。

**FRs covered:** FR30–FR31  
**NFR 相关:** NFR-S3、NFR-R2

### Epic 7：追溯、测试覆盖率与产品边界

维护者能在文档体系中追溯需求—设计—用例—验收 ID；运行测试与覆盖率工作流；系统不默认承诺 HTTP API、多租户、自动迁移等未列入 PRD 的能力。

**FRs covered:** FR32–FR34  
**NFR 相关:** NFR-S3、NFR-R2

### Epic 8：Devshell 历史命令回放（TTY）

用户在 TTY 交互会话中使用上下箭头浏览并复用本会话历史命令，减少重复输入，提高交互效率，同时保持“复用执行”与“手动输入执行”语义一致。

**FRs covered:** FR35  
**NFR 相关:** NFR-P1（交互效率）

---

## Epic 1：终端待办与数据生命周期（故事）

### Story 1.1：创建待办与校验失败不脏写

作为一名在终端管理任务的开发者，  
我希望用 CLI 添加待办并在校验失败时不产生无效或部分写入，  
以便自动化脚本与个人工作流都不会破坏 `.todo.json`。

**Acceptance Criteria:**

**Given** 工作区已有或可无 `.todo.json`  
**When** 我执行带合法参数的 `todo add`（或等价子命令）  
**Then** 新待办出现在列表中且文件保持 schema 合法  
**And** 当输入非法（日期、必填字段等）时命令非 0 退出且不写入非法记录（**FR1**，**NFR-S1**）

### Story 1.2：列出待办与空结果

作为一名开发者，  
我希望在无数据时看到可理解的空列表语义，  
以便区分「无待办」与「命令失败」。

**Acceptance Criteria:**

**Given** `.todo.json` 为空或无任何条目  
**When** 我执行 `todo list`（或等价）  
**Then** 输出明确表示空集（人类可读与 `--json` 一致）（**FR2**，与 Epic 5 契约对齐）

### Story 1.3：过滤、排序与列表浏览

作为一名开发者，  
我希望按状态、日期、标签等维度过滤与排序，  
以便在任务变多时仍能快速定位。

**Acceptance Criteria:**

**Given** 存在多条待办且带标签/日期等字段  
**When** 我使用文档已列的过滤与排序参数  
**Then** 结果顺序与过滤条件与 `docs/requirements.md` / `test-cases.md` 一致（**FR3**）

### Story 1.4：完成、删除与非法引用

作为一名开发者，  
我希望完成或删除待办，并在 id 不存在或非法时得到约定错误语义，  
以免误删或静默失败。

**Acceptance Criteria:**

**Given** 存在或不存在某 `TodoId`  
**When** 我执行 `complete` / `delete` 等子命令  
**Then** 成功路径更新状态；失败路径返回约定错误与退出码（**FR4**，**FR28** 在 Epic 5 细化）

### Story 1.5：查看详情与更新可选字段

作为一名开发者，  
我希望查看单条详情并更新描述、截止日期、优先级、重复规则等可选字段，  
以便维护任务上下文。

**Acceptance Criteria:**

**Given** 一条已存在的待办  
**When** 我执行 `show` / `update`（或等价）  
**Then** 字段读写与 `Store` / `.todo.json` 契约一致，非法组合被拒绝（**FR5**）

### Story 1.6：搜索与统计摘要

作为一名开发者，  
我希望按关键词搜索并查看统计摘要，  
以便快速评估工作量与范围。

**Acceptance Criteria:**

**Given** 多条待办含可搜索文本  
**When** 我执行搜索/统计子命令  
**Then** 结果与摘要字段与文档约定一致（**FR6**）

### Story 1.7：导入与导出

作为一名开发者，  
我希望按约定格式导出并自文件导入，且可选择替换策略，  
以便迁移与备份。

**Acceptance Criteria:**

**Given** 有效导出文件与可选冲突策略  
**When** 我执行 `export` / `import`  
**Then** 数据完整性与替换行为符合 `requirements` / `test-cases`（**FR7**）

### Story 1.8：重复任务

作为一名开发者，  
我希望使用重复规则、下一实例与 `--no-next` 等能力，  
以便管理周期性工作。

**Acceptance Criteria:**

**Given** 含 `RepeatRule` 的待办  
**When** 我完成实例或请求下一实例  
**Then** 行为与边界与 PRD / 设计文档一致（**FR8**）

---

## Epic 2：工作流编排、仓库与质量门禁（故事）

### Story 2.1：统一 xtask 开发者子命令

作为一名贡献者，  
我希望从单一入口调用 fmt、clippy、test、clean 等编排命令，  
以便本地与文档描述一致。

**Acceptance Criteria:**

**Given** 本仓库 workspace 已构建  
**When** 我运行 `cargo xtask` 下列出的开发者子命令  
**Then** 行为与 `xtask` 帮助及 `docs/` 一致（**FR9**，**NFR-S3**）

### Story 2.2：Git 辅助

作为一名贡献者，  
我希望使用暂存、带检查的提交等 Git 辅助子命令，  
以便减少手工拼命令错误。

**Acceptance Criteria:**

**Given** 仓库为 git 工作副本  
**When** 我使用 `cargo xtask git …`  documented 路径  
**Then** 操作与安全检查点与文档一致（**FR10**，**NFR-I1** 在工具缺失时可诊断）

### Story 2.3：发布辅助

作为一名维护者，  
我希望运行与 `publishing.md` 对齐的发布前检查辅助命令，  
以便降低发版失误。

**Acceptance Criteria:**

**Given** 发布清单在 `docs/publishing.md`  
**When** 我执行对应 xtask 发布辅助  
**Then** 步骤与失败信息与文档一致（**FR11**）

### Story 2.4：一键验收

作为一名维护者，  
我希望运行 `cargo xtask acceptance` 得到汇总报告并区分自动通过与 SKIP/手工项，  
以便合并前可签字。

**Acceptance Criteria:**

**Given** 文档化环境依赖与 SKIP 规则  
**When** 我运行 `cargo xtask acceptance`  
**Then** 退出码、报告片段与 `docs/acceptance.md` 一致（**FR12**，**NFR-I1**）

### Story 2.5：提交前检查对齐 CI

作为一名贡献者，  
我希望在合并前运行与 CI 对齐的 pre-commit（或等价路径），  
以便本地失败与 CI 一致。

**Acceptance Criteria:**

**Given** `.githooks/pre-commit` 与 `cargo xtask git pre-commit` 文档  
**When** 我在变更上运行 pre-commit  
**Then** 检查集与 CI 描述一致或可解释差异（**FR13**）

---

## Epic 3：Devshell 会话、文件能力与 shell 体验（故事）

### Story 3.1：交互式会话与脚本执行

作为一名开发者，  
我希望启动交互 devshell 或用 `.dsh` 脚本非交互执行，  
以便复现与自动化 devshell 工作流。

**Acceptance Criteria:**

**Given** 已安装 `cargo devshell`  
**When** 我进入 REPL 或 `devshell script.dsh`  
**Then** 会话生命周期与错误行为符合 `requirements`（**FR14**）

### Story 3.2：内置文件与目录操作

作为一名开发者，  
我希望使用白名单内置命令做导航、列目录、读写文件、建目录，  
以便在受控环境中操作工作区。

**Acceptance Criteria:**

**Given** 当前会话根与 VFS 规则  
**When** 我执行内置文件命令  
**Then** 仅允许白名单操作且与 VFS 一致（**FR15**，**NFR-S2**）

### Story 3.3：管道与重定向及缓冲上限

作为一名开发者，  
我希望用管道/重定向组合内置命令，并在超限时得到可见失败，  
以免静默 OOM。

**Acceptance Criteria:**

**Given** `PIPELINE_INTER_STAGE_MAX_BYTES` 等常量  
**When** 管道阶段超过上限  
**Then** 失败可见且不无限缓冲（**FR16**，**NFR-P2**）

### Story 3.4：Devshell 内 Todo 子集

作为一名开发者，  
我希望在 devshell 中使用待办子集且读写与 `.todo.json` 约定一致，  
以便与 `cargo xtask todo` 数据对齐。

**Acceptance Criteria:**

**Given** 共享 `todo_io` 路径规则  
**When** 我在 devshell 执行受支持 todo 子命令  
**Then** 与 xtask 侧数据文件一致（**FR17**）

### Story 3.5：帮助与 TTY 补全

作为一名开发者，  
我希望获得与内置命令一致的帮助，并在 TTY 下获得命令/路径补全，  
以便减少查阅成本（**UX-DR1–UX-DR3**）。

**Acceptance Criteria:**

**Given** TTY 与非 TTY 环境  
**When** 我请求 `--help` 或使用补全  
**Then** 文本与补全行为与 `requirements` / `design.md` 一致（**FR18**，**FR19**）

---

## Epic 4：Rust 工具链、VM 与工作区模式（故事）

### Story 4.1：宿主沙箱执行 rustup/cargo

作为一名开发者，  
我希望在未启用 VM 时于宿主沙箱路径执行 `rustup`/`cargo` 并正确回写，  
以便默认路径不依赖 Lima/Podman。

**Acceptance Criteria:**

**Given** `DEVSHELL_VM=off` 或等效  
**When** 我在 devshell 调用 `rustup`/`cargo`  
**Then** 导出—执行—回写语义与文档一致（**FR20**，**NFR-R1**）

### Story 4.2：Unix γ（Lima）路径

作为一名在 macOS/Linux 上的开发者，  
我希望在启用 VM 时通过 γ 后端在隔离环境执行工具链命令，  
以便与宿主隔离。

**Acceptance Criteria:**

**Given** 可选 `limactl` 与文档前置条件  
**When** 我启用 γ 路径  
**Then** 行为与 `devshell-vm-gamma.md` / 设计文档一致；缺失时 SKIP 或显式失败（**FR21**，**NFR-I1**）

### Story 4.3：Windows β 与侧车协议

作为一名 Windows 开发者，  
我希望在启用 VM 时通过 β 与 JSON 行侧车执行 `rustup`/`cargo`，  
且协议通道不被污染。

**Acceptance Criteria:**

**Given** Podman 与匹配版本侧车镜像  
**When** 宿主与侧车握手并收发一行一 JSON  
**Then** stdout 仅协议输出；解析失败可诊断（**FR22**，**NFR-S2**，**NFR-I2**）

### Story 4.4：关闭 VM 与宿主降级

作为一名开发者，  
我希望关闭 VM 或选择仅宿主路径时仍可使用核心 devshell 能力，  
以便环境不全时仍可工作。

**Acceptance Criteria:**

**Given** `DEVSHELL_VM=off` / `BACKEND=host` 等  
**When** 我执行不依赖 guest 的核心命令  
**Then** 行为可预测且无强制依赖 Lima/Podman（**FR23**，**NFR-R1**）

### Story 4.5：Mode P 与会话元数据

作为一名开发者，  
我希望在有效条件下使用 guest 为主工作区（Mode P），并把会话元数据落在工作区约定路径，  
以便与 guest 工程视图一致且可复盘。

**Acceptance Criteria:**

**Given** `requirements §1.1` 与 Mode 冲突降级规则  
**When** 我启用 Mode P 或写入会话 JSON  
**Then** 降级到 Mode S 的条件与路径与文档一致（**FR24**，**FR25**）

---

## Epic 5：机读 CLI、错误模型与 AI 集成（故事）

### Story 5.1：结构化 JSON 输出

作为一名集成方或脚本作者，  
我希望在支持的子命令上使用 `--json` 获得稳定成功/失败体，  
以便机器解析（**UX-DR3**）。

**Acceptance Criteria:**

**Given** 支持 `--json` 的子命令列表  
**When** 我分别以成功与失败路径调用  
**Then** 字段与示例与 `requirements` / 集成测试一致（**FR26**）

### Story 5.2：Dry-run 无写盘

作为一名开发者，  
我希望在修改类命令上使用 `--dry-run` 时不写入 `.todo.json` 等约定文件，  
以便安全预览。

**Acceptance Criteria:**

**Given** 带 `--dry-run` 的修改类命令  
**When** 我执行 dry-run  
**Then** 磁盘上的 todo 数据不变（**FR27**）

### Story 5.3：退出码约定

作为一名脚本作者，  
我希望退出码区分成功、一般错误、用法错误与数据错误，  
以便 CI 与 shell 逻辑分支。

**Acceptance Criteria:**

**Given** `requirements §6` 码表  
**When** 我触发各类错误与成功路径  
**Then** 进程退出码与文档一致（**FR28**）

### Story 5.4：init-ai 技能材料

作为一名使用 AI 助手的开发者，  
我希望运行 `todo init-ai` 生成约定技能/命令片段，  
以便团队共享调用方式。

**Acceptance Criteria:**

**Given** 文档化的 flag 与输出路径  
**When** 我执行 `init-ai`  
**Then** 输出结构与可消费性与 PRD / `requirements` 一致（**FR29**）

---

## Epic 6：跨平台交付与发布（故事）

### Story 6.1：跨平台可编译交付物

作为一名用户或 CI，  
我希望在 Linux、macOS、Windows（MSVC）上均能编译通过主库与相关二进制，  
以免平台漂移。

**Acceptance Criteria:**

**Given** 声明的目标 triple 与 CI/本地 matrix  
**When** 我执行 `cargo build` / `cargo check`（含 `x86_64-pc-windows-msvc`）  
**Then** 无意外 `cfg` 遗漏；与 **NF-5** / pre-commit 描述一致（**FR30**）

### Story 6.2：crates.io 与 cargo devshell

作为一名终端用户，  
我希望从 crates.io 安装 `xtask-todo-lib` 并获得 `cargo devshell` 入口，  
以便与发布策略一致。

**Acceptance Criteria:**

**Given** `cargo publish -p xtask-todo-lib --dry-run` 与 `publishing.md`  
**When** 我按文档安装  
**Then** 二进制入口与版本元数据正确；破坏性变更有 **CHANGELOG**（**FR31**，**NFR-R2**，**NFR-S3**）

---

## Epic 7：追溯、测试覆盖率与产品边界（故事）

### Story 7.1：需求—设计—用例—验收追溯

作为一名维护者，  
我希望在仓库文档体系中对照 FR、设计、用例 ID 与验收 ID，  
以便变更可审计。

**Acceptance Criteria:**

**Given** `docs/requirements.md`、`design.md`、`test-cases.md`、`acceptance.md`  
**When** 我追溯某能力或验收项  
**Then** 可找到对应链条或显式记为 SKIP/手工（**FR32**）

### Story 7.2：自动化测试与覆盖率工作流

作为一名维护者，  
我希望运行 `cargo test` 与 `cargo xtask coverage` 并满足阈值与排除规则，  
以便回归可量化。

**Acceptance Criteria:**

**Given** `xtask/src/coverage.rs` 与 `test-coverage.md`  
**When** 我在本仓库运行覆盖率命令  
**Then** 摘要与阈值行为与文档一致（**FR33**）

### Story 7.3：产品能力边界（不默认承诺）

作为一名产品/工程角色，  
我希望系统与文档不把 HTTP API、多租户、`.todo.json` 自动迁移等未列入 PRD 的能力当作默认承诺，  
以免范围 creep。

**Acceptance Criteria:**

**Given** PRD **FR34** 与对外 README  
**When** 审计对外描述与默认行为  
**Then** 无隐式网络服务/多租户/自动迁移承诺；若新增须走 FR 变更（**FR34**）

---

## Epic 8：Devshell 历史命令回放（故事）

### Story 8.1：TTY 上下箭头历史浏览与复用

作为一名频繁使用 `cargo devshell` 的开发者，  
我希望在交互会话中通过 **↑/↓** 浏览并复用已执行命令，  
以便快速重放常用命令而无需重复输入。

**Acceptance Criteria:**

**Given** 处于 TTY 交互会话且已执行至少两条命令  
**When** 我按 **↑** 或 **↓**  
**Then** 输入行按历史顺序切换为对应命令，不直接执行  
**And** 我按 Enter 后其解析与执行结果与手动键入该命令一致（**FR35**）

**Given** 非 TTY（脚本模式或重定向）  
**When** 执行 devshell  
**Then** 不依赖上下箭头历史交互能力，现有脚本行为不变（**FR35** 边界）

---

## Final Validation（Step 4）

### FR 覆盖校验

- **FR1–FR35**：均在「FR Coverage Map」与各 Epic 故事中通过编号或语义覆盖；无遗漏 FR。
- **NFR**：在「NFR → Epic 对照」与相关故事（管道缓冲、侧车 IPC、SKIP 等）中可追溯。
- **UX-DR1–UX-DR3**：由 Epic 3 Story 3.5、Epic 5 等故事覆盖。

### 架构与 Starter 约定

- 架构文档选定 **棕地 Cargo workspace**，**不**使用一次性 Web/CLI 脚手架生成器；因此 **Epic 1 Story 1** 为「创建待办与校验」类领域故事，**不**要求改为「从模板克隆空仓库」——与架构 **Starter Template Evaluation** 一致。

### 数据模型 / 大前置

- 无「单故事建全库表」模式；持久化为 `.todo.json` + `Store`，故事按能力纵向切分，符合棕地演进。

### Epic 间依赖（说明）

- Epic 按**用户价值**划分；棕地代码库中多域已并存。**排期与增强**仍可按 Epic 分批；故事中指向「与 Epic 5 契约对齐」等处表示**接口一致性**，不表示必须先实现 Epic 5 才能编译 Epic 1（实现上契约已共享）。

### 故事内顺序依赖

- 各 Epic 内故事按 1.1→1.2→… 递增能力；无「依赖未来故事编号」的表述。

**结论：** 本文档通过 Step 4 校验，可作为开发与排期输入。
