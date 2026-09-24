# Story 7.2：自动化测试与覆盖率工作流

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名维护者，  
我希望运行 **`cargo test`** 与 **`cargo xtask coverage`** 并满足**阈值与排除规则**（以文档与实现为准），  
以便回归可量化（**FR33**）。

## 映射需求

- **FR33**：维护者可运行自动化测试与覆盖率工作流（以 **`xtask`** 实现与排除规则为准）。

## Acceptance Criteria

1. **Given** **`docs/test-coverage.md`**（目标 **各 crate ≥95%**、**`cargo-tarpaulin`**、**`--test-threads=1`** 说明）  
   **When** 在仓库根执行 **`cargo xtask coverage`**（需已 **`cargo install cargo-tarpaulin`**）  
   **Then** 对每个包（**`xtask-todo-lib`**、**`xtask`**）运行 **`coverage.rs`** 中配置的 **`tarpaulin`** 参数；输出 Markdown 风格摘要表；任一包无法解析覆盖率时提示安装并 **error**（与 **`cmd_coverage`** 一致）（**FR33**）。

2. **Given** **`xtask/src/coverage.rs`** 中的 **`--exclude-files`** 列表  
   **When** 与 **`test-coverage.md`**「说明」各节对照  
   **Then** **语义一致**：文档指向**单一事实来源**（**「精确列表见 `coverage.rs`」**）；若代码增删排除项，**同步**文档中**概括性**列表或脚注（**FR33**）。

3. **Given** **全量测试**  
   **When** **`cargo test`**（工作区）与 **`pre-commit`** 策略  
   **Then** **`test-coverage.md`** 中关于 **cwd 竞态**、**`--test-threads=1`** 的约束仍成立；**不**声称 **tarpaulin** 替代 **MSVC `cargo check`**（**§注意** 已说明）（**FR33**）。

4. **Given** **阈值 ≥95%**  
   **When** 核对 **`cmd_coverage`** 实现  
   **Then** 当前实现**打印**各包百分比，**不**在代码内硬编码 **95% 失败门禁**（若以文档为「目标」而非 CI 强制，保持现状并在 **`test-coverage.md`** 或本故事**显式**说明）；若产品要求**硬门禁**，在本故事内**新增**失败条件并更新文档（**可选扩展**，避免静默改变 CI）。

5. **棕地**：**`coverage.rs`** 含 **`#[cfg(test)]`**（**`parse_coverage_percentage`**、fake **`CARGO`** 等）；**`XTASK_COVERAGE_TEST_FAKE*`** 仅用于测试。本故事以 **核对 AC、文档与排除列表同步** 为主。

6. **回归**：**`cargo test -p xtask coverage::`**（或 **`cargo test -p xtask`**）、在已安装 **tarpaulin** 时 **`cargo xtask coverage`**（或记录环境 **SKIP**）；**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **对照表**：从 **`coverage.rs`** 提取 **`xtask-todo-lib`** / **`xtask`** 两组 **`exclude-files`**，与 **`test-coverage.md`** 段落逐条核对。
- [x] **95%**：确认团队期望是「人工读摘要」还是「CI 失败」；若仅前者，在文档加**一句**「实现不强制阈值」。
- [x] **beta-vm / devshell-vm**：**`test-coverage.md`** 中 **`cargo test --features beta-vm`**、**`devshell-vm`** 段落与 **`test-cases.md`** 引用仍正确。
- [x] **验证**：**`cargo test -p xtask`**；有条件时 **`cargo xtask coverage`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 命令 | **`xtask/src/coverage.rs`** — **`cmd_coverage`**、**`run_tarpaulin`** |
| 文档 | **`docs/test-coverage.md`** — 目标 ≥95%、排除说明、与 pre-commit 关系 |
| 测试 | **`coverage.rs` `mod tests`** — 假 **`CARGO`** 脚本路径 |

### 架构合规（摘录）

- **覆盖率**与 **Windows MSVC 交叉编译**职责分离（**`test-coverage.md` §注意**）。

### 前序故事

- **7.1**：需求—用例追溯；本故事关注**可量化回归**工具链。
- **6.1**：**NF-5**；勿与 tarpaulin 混淆。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 7 Story 7.2]
- [Source: `docs/test-coverage.md`]
- [Source: `xtask/src/coverage.rs`]
- [Source: `docs/test-cases.md` — 与覆盖率/VM 相关 TC]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask`
- `cargo xtask coverage`
- `cargo clippy -p xtask --all-targets -- -D warnings`

### Completion Notes List

- 对照 `xtask/src/coverage.rs` 与 `docs/test-coverage.md`：文档继续采用“概括性说明 + 精确列表以 `coverage.rs` 为准”的单一事实来源策略，语义一致。
- 在 `docs/test-coverage.md` 补充阈值说明：当前 `cargo xtask coverage` 仅输出各 crate 覆盖率摘要，不在实现中硬编码 `95%` 失败门禁；`95%` 维持为团队目标值。
- 复核 `beta-vm/devshell-vm` 说明与 `docs/test-cases.md` 的 VM 用例引用（含 `TC-D-VM-4/7`）一致，且文档未将 tarpaulin 与 MSVC 交叉编译门禁混淆。
- 执行回归命令通过，`cargo xtask coverage` 摘要输出：`xtask-todo-lib 95.26%`、`xtask 96.00%`。

### File List

- `xtask/src/coverage.rs`（`exclude-files` / `cmd_coverage` 单一事实来源）
- `docs/test-coverage.md`
- `docs/test-cases.md`（VM 用例引用核对）
- `_bmad-output/implementation-artifacts/7-2-test-coverage-workflow.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings

- [x] [Review][Patch] 审阅路径清单不完整 — `File List` 未列出 **`coverage.rs`**（AC2 对照核心）及 **`test-cases.md`**（Completion Notes 中 VM 引用核对）；已补全上表路径（本轮审查）。

## Change Log

- **2026-03-26**：BMad 并行审查通过；补全 **File List**；**Status → done**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
