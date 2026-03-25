# Story 2.2：Git 辅助

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名贡献者，  
我希望使用暂存、带检查的提交等 Git 辅助子命令，  
以便减少手工拼命令错误。

## 映射需求

- **FR10**（`epics.md` Story 2.2；`docs/requirements.md` §4 表 — `git add` / `git pre-commit` / `git commit`）
- **NFR-I1**：工具缺失或环境不满足时**可诊断**（错误信息可读、退出码非 0 且可理解）

## Acceptance Criteria

1. **Given** 当前目录为 **git 工作副本**（含 `.git`）  
   **When** 使用 `cargo xtask git add`（或文档列出的暂存路径）  
   **Then** 行为与 `xtask/src/git.rs` / `--help` 一致；非仓库时失败信息明确（**FR10**）。

2. **Given** 仓库根存在 **`.githooks/pre-commit`**（或文档说明的等价物）  
   **When** 执行 `cargo xtask git pre-commit`  
   **Then** 与 **`docs/requirements.md` §4** 描述一致：执行钩子脚本中的检查集（fmt、clippy、rustdoc、`.rs` 行数、test、Windows cross-check 等，以实现为准）；失败时非 0 退出。

3. **Given** 同上  
   **When** 执行 `cargo xtask git commit -m "…"`（或故事实现支持的参数形式）  
   **Then** 在约定流程下完成提交或按实现返回错误；与 **`docs/requirements.md` §4** 及 `git.rs` 帮助文案一致。

4. **Given** **非** git 仓库或 **`git` 不在 PATH**（可选场景）  
   **When** 调用上述子命令  
   **Then** 不静默成功；错误信息或退出码可区分「非仓库」「钩子缺失」「外部命令失败」等（**NFR-I1**）。

5. **棕地**：实现位于 **`xtask/src/git.rs`**；本故事以**核对 AC、补测试或文档、小修正**为主，**不**无依据重写 Git 集成策略。

6. **回归**：不破坏 **`cargo xtask`** 其他子命令及 Epic 1 已完成的 todo 集成测试。

## Tasks / Subtasks

- [ ] **棕地核对**：阅读 `xtask/src/git.rs` 全文；对照 `docs/requirements.md` §4、`docs/design.md`（若引用 pre-commit）；列出 `git add` / `pre-commit` / `commit` 参数与行为矩阵。
- [ ] **帮助与错误路径**：运行 `cargo xtask git --help`、各子命令 `--help`；在非仓库临时目录试跑，记录错误信息是否满足 AC4。
- [ ] **测试（若缺）**：为 `cmd_git` 或关键分支补 **单元/集成** 测试（可 mock 或条件跳过），避免破坏无 git 的 CI 时标注 `#[ignore]` 或 feature gate。
- [ ] **验证命令**：`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`。

## Change Log

（实现时填写）

## Dev Notes

### 棕地现状

- **`git` 子命令**：`GitArgs` / `GitSub`（`add`、`pre-commit`、`commit`）；`run_pre_commit_checks` 调用 **`.githooks/pre-commit`**（`sh`）。

### 须触摸的源码位置

| 区域 | 路径 |
|------|------|
| Git 辅助 | `xtask/src/git.rs` |
| 分发 | `xtask/src/lib.rs` — `XtaskSub::Git` |
| 文档 | `docs/requirements.md` §4 |

### 架构合规（摘录）

- **xtask** 通过 **宿主 `git` / `sh`** 调用钩子；**不**内嵌 libgit2（除非架构变更）。

### 测试与追溯

- 以 **`docs/requirements.md` §4** 与 **`epics.md` Story 2.2** 为锚点。

### 前序故事

- **2-1**（统一 xtask 开发者子命令）若仍为 **`review`**，可与本故事并行；注意 **`git`** 与 **`run`/`fmt`** 等文档交叉引用一致。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.2]
- [Source: `docs/requirements.md` §4]
- [Source: `xtask/src/git.rs`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

（实现时填写）

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
