# Story 2.4：一键验收

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名维护者，  
我希望运行 **`cargo xtask acceptance`** 得到汇总报告并区分自动通过与 SKIP/手工项，  
以便合并前可签字。

## 映射需求

- **FR12**（`epics.md` Story 2.4；`docs/requirements.md` §4 表 — **`acceptance`** 与 **`docs/acceptance.md`**）
- **NFR-I1**：`cargo`/`rustup` 缺失、target 未安装等场景下行为**可解释**（报告中 **SKIP** 带原因、stderr 摘要；非静默误判为通过）

## Acceptance Criteria

1. **Given** **`docs/acceptance.md`** 已文档化环境依赖、**§8** 自动化摘要表、**退出码**规则及「本命令不自动执行」项  
   **When** 在仓库根执行 **`cargo xtask acceptance`**（及 **`--stdout-only`**、**`-o <path>`** 变体）  
   **Then** 行为与 **`acceptance.md` §8** 一致：**默认** 写入 **`docs/acceptance-report.md`**（除非 **`--stdout-only`**）；报告含 **自动化结果表**（**PASS / FAIL / SKIP**）、**需人工/环境的验收项** 表、与 **`acceptance.md`** 交叉引用的覆盖说明与 **结论** 段落（**FR12**）。

2. **Given** 自动化检查集（**NF-1、NF-2、NF-6** 文件检查；三 crate 的 **`cargo test`**；**NF-5/D8** MSVC **`cargo check`** 等，以 **`xtask/src/acceptance/checks.rs`** 为准）  
   **When** 对照 **`docs/acceptance.md` §8** 表格「本命令会执行（摘要）」  
   **Then** 每条摘要行在实现中有对应检查或**已记录**的合理差异（若调整检查顺序/ID，须同步 **§8** 与 **`requirements.md` §4**）（**FR12**）。

3. **Given** **`acceptance.md` §8** 规定：**全部自动化通过（含 SKIP）→ 退出码 0**；**任一 FAIL → 退出码 1**  
   **When** 运行 **`cargo xtask acceptance`**  
   **Then** **`xtask` 进程退出码**符合上述规则；**SKIP**（如未安装 **`x86_64-pc-windows-msvc`** target）**不**导致非 0（**FR12**）。

4. **Given** **`acceptance.md` §8** 列出需在报告中体现为「不自动执行」的项（如 **T6-1/T6-2、NF-3、NF-4、D5、D6、D9、X3-1** 等）  
   **When** 查看 **`docs/acceptance-report.md`**（或由 **`--stdout-only`** 打印的等价内容）**§2 需人工或环境**  
   **Then** 与文档列出的 **SKIP/手工** 意图一致：读者能区分 **自动化结果** 与 **须人工/环境**；若 **`report::manual_skip_rows`** 与 **§8** 列举项有缺漏（例如 **D9**），本故事实现中**补齐或**在 Dev Notes 说明为何省略（**FR12**）。

5. **棕地**：实现位于 **`xtask/src/acceptance/`**（**`mod.rs`**、**`checks.rs`**、**`report.rs`**、**`tests.rs`**）；本故事以 **对齐 AC、文档与实现、补测试、小修正**为主，**不**将需 **Lima/Podman 全链路** 的项改为在本命令内自动执行（仍归 **手工/SKIP**）。

6. **回归**：不破坏其它 **`cargo xtask`** 子命令；**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：通读 **`docs/acceptance.md` §1、§8** 与 **`xtask/src/acceptance/*.rs`**；做 **§8 表 ↔ `run_all_checks`** 对照清单；记录 **manual_skip_rows** 与 **§8「不自动执行」** 列表差异。
- [ ] **退出码与报告**：确认 **`cmd_acceptance`** 仅在存在 **Fail** 时返回错误；验证 **`--stdout-only`** 不写文件、仍遵守退出码；必要时为 **`RunFailure.code`** 与外层 **`main`** 行为补集成说明（当前为 **1**）。
- [ ] **文档同步**：若增删检查项或人工表项，更新 **`docs/acceptance.md` §8**、**`docs/requirements.md` §4** 单行描述；**`xtask/tests/xtask_help.rs`** 若需快照则更新。
- [ ] **测试**：沿用 **`acceptance/tests.rs`** 对 **`build_report`**、文件检查的覆盖；对 **`cmd_acceptance`** 关键分支可追加测试（注意 **`cargo test` 嵌套** 耗时，优先单元测试与桩）。
- [ ] **验证**：`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`；在干净工作区试跑 **`cargo xtask acceptance --stdout-only`** 抽样目检报告结构。

## Dev Notes

### 棕地现状（摘录）

- **`AcceptanceArgs`**：**`-o` / `--output`**、**`--stdout-only`**（**argh** 开关）。
- **`run_all_checks`**：顺序为 **NF-1** → **NF-2** → **NF-6** → 三 crate **`cargo test`**（**`--test-threads=1`**）→ **NF-5/D8** MSVC（未安装 target 则 **Skip**）。
- **`build_report`**：Markdown **§1** 自动化表、失败详情代码块、**§2** 人工表、**§3** 覆盖说明、**§4** 结论。

### 须触摸的源码与测试

| 区域 | 路径 |
|------|------|
| 验收子命令 | **`xtask/src/acceptance/mod.rs`** |
| 检查实现 | **`xtask/src/acceptance/checks.rs`** |
| 报告 | **`xtask/src/acceptance/report.rs`** |
| 单元测试 | **`xtask/src/acceptance/tests.rs`** |
| 分发与退出码 | **`xtask/src/lib.rs`** — **`XtaskSub::Acceptance`**（**`RunFailure { code: 1 }`**） |

### 架构合规（摘录）

- 验收通过 **宿主 `cargo` / `rustup`** 子进程完成构建与测试；**不**在 xtask 内嵌测试运行器替代 **`cargo test`**。
- 变更对外验收语义时同步 **CHANGELOG / requirements** 约定（若适用）。

### 前序故事

- **2-3**（发布辅助）已 **`done`**；合并前流水线常与 **acceptance 报告**并列使用，避免在故事中引入与 **`publishing.md`** 冲突的假设。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.4]
- [Source: `docs/acceptance.md` — §1、§8]
- [Source: `docs/requirements.md` §4]
- [Source: `xtask/src/acceptance/`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
