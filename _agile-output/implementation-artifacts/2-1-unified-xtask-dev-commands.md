# Story 2.1：统一 xtask 开发者子命令

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名贡献者，  
我希望从单一入口调用 fmt、clippy、test、clean 等编排命令，  
以便本地与文档描述一致。

## 映射需求

- **FR9**（`epics.md` Story 2.1；`docs/requirements.md` §4「其他 `cargo xtask` 子命令」）
- **NFR-S3**（可维护性：入口与文档一致，见 `epics.md` / `architecture.md` 摘要）

## Acceptance Criteria

1. **Given** 本仓库 workspace 已能构建  
   **When** 执行 `cargo xtask --help`（或等价）  
   **Then** 列出子命令（含 `run`、`fmt`、`clippy`、`coverage`、`clean`、`todo`、`git`、`publish`、`acceptance` 等，以 **`xtask` 实际 `XtaskSub` 枚举为准**），与帮助文案可读、无崩溃。

2. **Given** 文档化开发者子命令（见 **`docs/requirements.md` §4** 表）  
   **When** 在干净工作区或约定环境下调用各子命令（至少覆盖 **smoke**：能解析参数并进入实现分支，不要求 CI 全绿）  
   **Then** 行为与 `docs/` 描述**无未文档化的破坏性差异**；若实现与文档不一致，以本故事**最小修订文档**或**故事内记录「已知差异」**（二选一，需与维护者一致）。

3. **棕地**：`xtask` 入口为 `xtask::run()`（`xtask/src/lib.rs` / `main.rs`）；子命令分发在 `cmd_*` 模块。本故事**不**要求重写子命令实现，**除非** AC 与验收测试发现明确缺口。

4. **回归**：不破坏 **`cargo xtask todo`** 及 Epic 1 相关集成测试（`--test integration` 等）。

## Tasks / Subtasks

- [x] **棕地核对**：枚举 `XtaskSub` 与 `docs/requirements.md` §4、`docs/tasks.md`（若有）中的 xtask 列表；记录缺漏。
- [x] **帮助与入口**：运行 `cargo xtask --help`、典型 `cargo xtask <sub> --help`；确认无 panic、文案与代码一致。
- [x] **文档最小修订（若需）**：仅当 AC2 发现不一致时，更新 `docs/requirements.md` §4 或相关 README 一句；**不**扩大范围到 Epic 2/3 其他故事。
- [x] **验证命令**：`cargo test -p xtask --test integration`（项目约定子集）、`cargo clippy -p xtask --all-targets -- -D warnings`。

### Review Findings

- [x] [Review][Patch] `sprint-status.yaml` 文件头 `# last_updated`（`03:00`）与 `last_updated` 字段（`03:35`）不一致 — 已统一为 `2026-03-27T03:35:00Z`（BMad code review）。
- [x] [Review][Patch] Dev Notes「前序上下文」仍写 Epic 1 / 1-8 可能为 `review`，与当前 sprint（Epic 1 `done`、1-8 `done`）不符 — 已改写为与现状一致。

## Change Log

- 2026-03-27：新增 `xtask` 顶层/子命令 `--help` smoke 测试；同步 `docs/requirements.md` §4 以包含实现中的 `ghcr`、`lima-todo`。
- 2026-03-27：BMad code review — 同步 sprint 注释与 `last_updated`；更新故事内前序上下文表述。
- 2026-03-27：审查通过，故事标 `done`；`sprint-status` 中 `2-1-unified-xtask-dev-commands` 同步为 `done`。

## Dev Notes

### 棕地现状

- **入口**：`xtask/src/main.rs` → `xtask::run()`。
- **子命令**：`xtask/src/lib.rs` 中 `XtaskSub` 及 `cmd_run`、`cmd_fmt`、`cmd_clippy` 等（以仓库为准）。

### 须触摸的源码位置（若需补测或文档）

| 区域 | 路径 |
|------|------|
| 分发与 CLI | `xtask/src/lib.rs`、各 `xtask/src/*.rs` 子模块 |
| 文档 | `docs/requirements.md` §4、`README.md`（若引用 xtask） |

### 架构合规（摘录）

- **xtask** 为 `publish = false` 工具 crate；与 **`xtask-todo-lib`** 边界见 `architecture.md`。

### 测试与追溯

- 以 **`docs/requirements.md` §4** 与 **`epics.md` Story 2.1** 为验收锚点；无单独 `TC-*` ID 时以 **AC 表** 为准。

### 前序上下文

- Epic 1（含 **`1-8-recurring-tasks`**）已 **`done`**；本故事在 **Epic 2** 内跟踪，与 Epic 1 回归测试（`--test integration`）解耦但需保持通过。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.1]
- [Source: `docs/requirements.md` §4]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — xtask 职责]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

### Completion Notes List

- `XtaskSub` 已包含：`acceptance`、`run`、`clean`、`clippy`、`coverage`、`fmt`、`gh`、`ghcr`、`git`、`publish`、`lima-todo`、`todo`。
- 新增集成测试 `xtask/tests/xtask_help.rs`，覆盖顶层 `--help` 子命令列表与所有子命令 `--help` smoke。
- 发现并修复文档差异：`docs/requirements.md` §4 之前缺少 `ghcr` 与 `lima-todo`，已做最小补充，无代码行为变更。
- 回归验证通过：`cargo test -p xtask --test integration` 与 `cargo clippy -p xtask --all-targets -- -D warnings`。

### File List

- `xtask/tests/xtask_help.rs`
- `xtask/tests/integration.rs`
- `docs/requirements.md`
- `_bmad-output/implementation-artifacts/2-1-unified-xtask-dev-commands.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
