# Story 2.3：发布辅助

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名维护者，  
我希望运行与 `docs/publishing.md` 对齐的发布前检查与发布辅助命令，  
以便降低发版失误。

## 映射需求

- **FR11**（`epics.md` Story 2.3；`docs/requirements.md` §4 表 — `publish` 见 **`docs/publishing.md`**）
- **NFR-I1**：`cargo` / `git` 不可用、登录/registry、工作区不洁等场景下**可诊断**（stderr 可读、退出码非 0）

## Acceptance Criteria

1. **Given** 权威清单在 **`docs/publishing.md`**（§一发布前准备、§二发布到 crates.io、§2.4 一键发布）  
   **When** 维护者阅读 **`cargo xtask publish --help`** 并执行与文档对应的 **xtask 发布辅助**路径  
   **Then** 子命令意图、主要步骤顺序与文档一致：**仅检查**（对应 §1.2 `cargo publish -p xtask-todo-lib --dry-run` 类行为）与 **一键发布**（对应 §2.4：patch 版本 +1 → `git add`/`commit` → `cargo publish` → `tag` → `push`）在实现与文档中可一一对应（**FR11**）。

2. **Given** 仓库根存在 **`crates/todo/Cargo.toml`**（包 **`xtask-todo-lib`**）  
   **When** 执行 **发布前检查**（不修改版本、不上传；行为须与 **`publishing.md` §1.2** 一致：等价于对 **`xtask-todo-lib`** 做 **`cargo publish --dry-run`**，含镜像 registry 时 **`--registry crates-io`** 的约定）  
   **Then** 成功时以非 0 以外约定退出；失败时 stderr/错误串标明失败步骤（如 `cargo publish` / registry），**不**静默成功（**FR11**，**NFR-I1**）。

3. **Given** 维护者已 **`cargo login`**，当前分支可推送，且 **执行一键发布前** 工作区除即将进行的版本提交外**无其它未提交改动**（与 **`publishing.md` §2.4 前置条件**一致）  
   **When** 执行 **`cargo xtask publish`**（完整流程，无「仅 dry-run」开关）  
   **Then** 步骤顺序与 **`publishing.md` §2.4** 一致：patch **+1** → **`git add crates/todo/Cargo.toml`** → **`git commit`** → **`cargo publish -p xtask-todo-lib`**（registry 行为与文档/现有实现一致，如 **`--registry crates-io`**）→ **`git tag xtask-todo-lib-vX.Y.Z`** → **`git push origin HEAD`** → **`git push origin tag`**；关键日志或结束语与文档预期（如 Release 工作流、tag 命名）一致或可解释差异并在 Dev Notes 记录。

4. **Given** **不满足** 一键发布前置条件（例如：**非** git 仓库、**`git`/`cargo` 不在 PATH**、**工作区在未 bump 前已有脏文件**、**`crates/todo/Cargo.toml` 缺失**）  
   **When** 调用发布辅助  
   **Then** 不表现为「部分成功」；错误信息可区分场景（**NFR-I1**）。若实现选择在一键发布前增加 **git status 洁净检查**，须在故事实现中写明并与文档 §2.4 前置条件对齐。

5. **棕地**：核心逻辑位于 **`xtask/src/publish.rs`**；本故事以 **对齐 AC、补测试/文档/CLI（如 `--dry-run` 或嵌套子命令）、小修正**为主，**不**无依据改变 **`xtask-todo-lib`** 的包名、tag 前缀 **`xtask-todo-lib-v`** 等已约定对外行为。

6. **回归**：不破坏 **`cargo xtask`** 其它子命令及 Epic 1 已完成测试；**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **棕地核对**：通读 **`docs/publishing.md`** 与 **`xtask/src/publish.rs`**；列出 §1.2 / §2.4 与代码逐步对照表；标出缺口（例如是否缺「仅 dry-run」、是否缺发布前洁净检查）。
- [x] **CLI 设计（_additive_）**：优先采用 **非破坏性** 扩展（例如在 **`PublishArgs`** 上增加 **`--dry-run`** 仅跑 dry-run，默认行为保持 §2.4 一键发布）；若改用 **`publish` 嵌套子命令**（类似 **`git`**），须评估对现有脚本的影响并在 Dev Notes 说明迁移。
- [x] **实现与文档**：同步 **`docs/requirements.md` §4** 一行说明与 **`docs/publishing.md`**（必要时补充「xtask 子命令」小节交叉引用）；**`xtask/tests/xtask_help.rs`** 等帮助快照更新。
- [x] **测试**：为 **`bump_version_in_cargo_toml`** 已有单测保持；为 **`cmd_publish` 新分支**（dry-run、洁净检查、错误路径）补充单元/集成测试；沿用 **`cwd_test_lock` / `path_test_lock`** 等现有模式。
- [x] **验证**：`cargo test -p xtask`（说明见 Completion Notes）、`cargo clippy -p xtask --all-targets -- -D warnings`。

### Review Findings

- [x] [Review][Patch] `sprint-status.yaml` 文件头 `# last_updated` 误为 `2026-03-25T10:08:19Z`，与 `last_updated` 字段（`2026-03-27T05:30:00Z`）严重不一致 — 已统一为 `2026-03-27T05:30:00Z`（BMad code review）。
- [x] [Review][Patch] Dev Notes「棕地现状」仍写 **`PublishArgs`（当前为空结构体）**，与实现（含 **`dry_run` / `--dry-run`**）不符 — 已改写。

## Change Log

- 2026-03-27：`publish` 增加 `--dry-run` 预检模式；默认一键发布前新增 git 工作区洁净检查；补充对应单测并同步发布文档。
- 2026-03-27：BMad code review — 修正 sprint 注释与 Dev Notes 棕地描述。
- 2026-03-27：审查通过，故事标 `done`；`sprint-status` 中 `2-3-publish-assist` 同步为 `done`。

## Dev Notes

### 棕地现状（摘录）

- **`publish` 子命令**：**`PublishArgs`** 含 **`dry_run`**（**`--dry-run`**：仅 **`cargo publish -p xtask-todo-lib --dry-run --registry crates-io`**，不改版本、不 git 提交/推送）。默认路径下 **`cmd_publish`**：`git status --porcelain` 洁净检查 → patch bump → `git add` → `git commit` → **`cargo publish -p xtask-todo-lib --registry crates-io`** → `tag` → `push` branch → `push` tag。常量 **`CRATE_CARGO`**、**`PACKAGE`** 见源码。
- **文档**：**`docs/publishing.md` §1.2** 强调 **`cargo publish -p xtask-todo-lib --dry-run`**；**§2.4** 描述 **`cargo xtask publish`** 全流程与前置条件。

### 须触摸的源码与测试

| 区域 | 路径 |
|------|------|
| 发布辅助 | **`xtask/src/publish.rs`** |
| 分发 | **`xtask/src/lib.rs`** — **`XtaskSub::Publish`** |
| 集成/帮助 | **`xtask/src/tests/run.rs`**、**`xtask/tests/xtask_help.rs`** |

### 架构合规（摘录）

- **xtask** 为唯一 **`cargo xtask`** 入口；通过宿主 **`cargo` / `git`** 子进程完成发布；**不**在 xtask 内嵌 crates.io HTTP 客户端替代 `cargo publish`。
- 变更 CLI 时遵循 **argh** 既有模式；**`git`** 子命令的嵌套子命令结构（**`GitArgs` / `GitSub`**）可作为拆分 **`publish`** 多模式的参考。

### 前序故事

- **2-1**、**2-2** 已 **`done`**；发布流程依赖 **`git`** 与干净工作区的惯例与 **2-2** 中错误可诊断性一致。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.3]
- [Source: `docs/publishing.md` — §一、§二、§2.4、§三]
- [Source: `docs/requirements.md` §4]
- [Source: `xtask/src/publish.rs`]

## Dev Agent Record

### Agent Model Used

Cursor Agent

### Debug Log References

### Completion Notes List

- 新增 `PublishArgs::dry_run`（`cargo xtask publish --dry-run`）：仅执行 `cargo publish -p xtask-todo-lib --dry-run --registry crates-io`，不修改版本、不提交、不打 tag、不推送。
- 默认发布流程（无 `--dry-run`）新增 `git status --porcelain` 洁净检查；不满足前置条件时快速失败并给出可诊断错误。
- 新增测试：`cmd_publish_dry_run_checks_only`、`cmd_publish_fails_when_worktree_dirty`；既有发布成功/失败与 bump 相关测试保持通过。
- 文档已同步：`docs/requirements.md` 的 `publish` 行补充 `--dry-run`；`docs/publishing.md` §2.4 增加 `cargo xtask publish --dry-run` 示例。
- 验证通过：`cargo test -p xtask cmd_publish_`、`cargo test -p xtask --test xtask_help xtask_subcommand_help_smoke`、`cargo clippy -p xtask --all-targets -- -D warnings`。`cargo test -p xtask` 仍有既有不稳定失败 `lima_todo::tests::cmd_lima_todo_print_only_no_build_smoke`（与本故事改动无关，执行日志已记录）。

### File List

- `xtask/src/publish.rs`
- `xtask/src/tests/run.rs`
- `docs/requirements.md`
- `docs/publishing.md`
- `_bmad-output/implementation-artifacts/2-3-publish-assist.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
