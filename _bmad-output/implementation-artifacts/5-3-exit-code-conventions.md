# Story 5.3：退出码约定

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名脚本作者，  
我希望**退出码**区分成功、一般错误、用法/参数错误与数据错误，  
以便 CI 与 shell 分支（**US-A2**）。

## 映射需求

- **FR28**：约定退出码区分成功、一般错误、参数错误、数据错误（以 **`requirements` §6 `US-A2`** 为准；见下表）。
- **UX-DR3**：**`--json`** 下 **`error.code`** 与进程退出码一致（**5.1**）。

| 码 | 含义（摘要） |
|----|----------------|
| **0** | 成功 |
| **1** | 一般错误（I/O、未预期等） |
| **2** | 参数/用法错误（如非法 id **0**、空标题） |
| **3** | 数据错误（如不存在 id、领域拒绝） |

## Acceptance Criteria

1. **Given** **`TodoCliError`** 映射（**`xtask/src/todo/error.rs`** **`EXIT_GENERAL` / `EXIT_PARAMETER` / `EXIT_DATA`**）  
   **When** **`cargo xtask todo`** 或独立 **`todo`** 二进制退出  
   **Then** 进程退出码与 **`TodoCliError::exit_code()`** 一致；**`lib.rs`** 中 **`RunFailure { code }`** 与 **`bin/todo.rs`** 的 **`std::process::exit`** 使用同一套数值（**FR28**）。

2. **Given** **`--json`** 且失败  
   **When** 打印 **`print_json_error`**  
   **Then** 载荷中 **`error.code`** 与进程退出码相同（与 **`xtask/tests/complete_delete/mod.rs`** TC-A2-2 / TC-A2-3 一致）（**FR28**）。

3. **Given** **`requirements §3.1`** 摘要 — 非法标题 → **2**；非法/不存在 id → **3**  
   **When** 核对 **`dispatch.rs`** / **`cmd/parse.rs`** 等将失败归类为 **`Parameter` vs `Data` vs `General`**  
   **Then** 与 **`TodoCliError` 变体**一致；**`General`** 仅用于 **`Box<dyn Error>`** 类（I/O、导入解析等），**不**用于可预见的用户输入错误（**FR28**）。

4. **棕地**：大量集成测试已覆盖 **`complete`/`delete`** 的 **2/3**；本故事以 **全子命令码表核对、补漏测、文档与 §6 对齐** 为主，**不**改变已发布约定除非有明确产品决策。

5. **回归**：**`cargo test -p xtask`**（含 **`complete_delete`**、**`show_update`** 等）、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **矩阵**：**子命令** × **失败原因** → **期望码**（对照 **`error.rs`** + **`dispatch.rs`**）；记录与 **`requirements §3.1`** 的差异。
- [x] **json 一致性**：抽查 **`--json`** 失败路径，**`error.code`** == **`status.code()`**（已有测试可复用模式）。
- [x] **文档**：若 **`requirements §6`** 表过简，可在 **`§3.2`** 脚注或 **`docs/design.md`** 增加「**`TodoCliError` 映射**」**一句**引用 **`error.rs`**。
- [x] **验证**：**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 常量与枚举 | **`xtask/src/todo/error.rs`** — **`EXIT_*`**、**`TodoCliError`** |
| 进程退出 | **`xtask/src/lib.rs`**（**`XtaskSub::Todo`**）、**`xtask/src/bin/todo.rs`** |
| 集成测试 | **`xtask/tests/complete_delete/mod.rs`**、**`show_update/mod.rs`** 等 |

### 架构合规（摘录）

- **`argh`** 解析失败**可能**在 **`xtask`** 层产生**非** todo 子命令的退出码；本故事**范围**为 **`todo` 子命令/独立 `todo`** 的业务逻辑退出码。

### 前序故事

- **5.1**：JSON **`error.code`**；本故事保证 **进程级** 与 **载荷** 一致。
- **5.2**：**`--dry-run`** 下错误路径仍按 **`TodoCliError`** 分类。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 Story 5.3]
- [Source: `docs/requirements.md` — §3.1、§6 **US-A2**]
- [Source: `xtask/src/todo/error.rs`]
- [Source: `xtask/src/lib.rs` — `XtaskSub::Todo`]
- [Source: `xtask/tests/complete_delete/mod.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask todo_bin_exit_code_mapping_uses_general_parameter_data_contract`
- `cargo test -p xtask && cargo clippy -p xtask --all-targets -- -D warnings`

### Completion Notes List

- 已核对退出码主链路：`TodoCliError::exit_code()`（`error.rs`）→ `xtask/src/lib.rs` 中 `RunFailure.code` → `xtask/src/bin/todo.rs` 中 `std::process::exit(e.exit_code())`，两入口共用同一 1/2/3 映射（FR28）。
- 已核对失败分类：`Parameter` 用于可预期输入问题（空标题、`id=0`、非法参数）；`Data` 用于领域数据不存在（如不存在 id）；`General` 保留 I/O/导入解析等一般错误，符合 AC3 与 `requirements §3.1`。
- 新增集成测试 `todo_bin_exit_code_mapping_uses_general_parameter_data_contract`，在独立 `todo` 二进制下验证 `--json` 失败载荷 `error.code` 与进程退出码一致：参数错误=2、数据错误=3、一般错误=1。
- 在 `docs/requirements.md` §3.2 补充一句 `TodoCliError` 与 `EXIT_*` 的映射来源，明确文档锚点并避免与实现漂移。

### File List

- `xtask/tests/todo_list/basic.rs`
- `docs/requirements.md`
- `_bmad-output/implementation-artifacts/5-3-exit-code-conventions.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | 新测 **`todo_bin_exit_code_mapping_uses_general_parameter_data_contract`** 在独立 **`todo`** 下串联 **空标题 → 2**、**不存在 id → 3**、**缺失导入文件 → 1**，且断言 **`stdout` JSON `error.code`** 与 **`status.code()`** 一致，覆盖 **AC1/AC2** 与 **FR28**。 |
| **Edge Case Hunter** | **`requirements.md` §3.2** 脚注指向 **`TodoCliError` / `EXIT_*`**（**`error.rs`**），降低文档与实现漂移风险（**AC4**）。 |
| **Acceptance Auditor** | **`dispatch.rs`** 中 **`list.create`** 空标题 → **`Parameter`**、**`todo not found`** → **`Data`**，与测试期望一致；**AC5** 复验 **`cargo test -p xtask`**、**`clippy`** 通过。 |

**待办项**：无。勿将 **`crates/todo/.dev_shell.bin`** 等非本故事变更一并提交。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — **5-3-exit-code-conventions** 与 sprint 同步为 **done**；**`last_updated`** **`2026-03-26T05:42:00Z`**；复验 **`cargo test -p xtask`**、**clippy**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
