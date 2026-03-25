# Story 3.3：管道与重定向及缓冲上限

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望用**管道**/**重定向**组合内置命令，并在**超限时得到可见失败**，  
以免静默 OOM。

## 映射需求

- **FR16**（`epics.md` Story 3.3；`docs/requirements.md` **§5.5** — 管道 `|`、重定向 `<` / `>` / `2>`）
- **NFR-P2**：管道**非末段**阶段 stdout 在宿主内存缓冲，上限为 **`PIPELINE_INTER_STAGE_MAX_BYTES`**（当前 **16 MiB**，见实现）；超限须**失败可见**，不无限增长（PRD / **`docs/devshell-vm-gamma.md`** 管道摘要）

## Acceptance Criteria

1. **Given** 已解析的 **`Pipeline`**（至少两段命令，由 **`parser`** 产生）  
   **When** 经 **`execute_pipeline`** 执行，且**非最后一段**命令产生超过 **`PIPELINE_INTER_STAGE_MAX_BYTES`** 字节的 stdout  
   **Then** 返回 **`BuiltinError::PipelineInterStageBufferExceeded`**（含 **`limit`/`actual`**），管线**中止**；**不**继续后续阶段（**FR16**，**NFR-P2**）。

2. **Given** 单段命令或管道**末段** stdout 直接写入终端/文件（不经中间整段缓冲）  
   **When** 输出任意大（受宿主/终端其它限制除外）  
   **Then** **不**错误套用「段间 16 MiB」限制末段直连输出（行为与 **`dispatch/mod.rs` `execute_pipeline`** 注释一致）（**FR16**）。

3. **Given** **`SimpleCommand`** 上附加 **`stdin`/`stdout`/`stderr` 重定向**（**`<`、`**`>`**、**`2>`**）  
   **When** 经 **`run_builtin_with_streams`** / **`run_builtin`** 执行  
   **Then** 重定向目标经 **`workspace_read_file` / `workspace_write_file`** 解析到 **VFS/工作区**，失败时错误可读（**FR16**）。

4. **Given** 管道或重定向失败（含超限、路径非法、读失败）  
   **When** REPL（**`repl.rs`**）或脚本（**`script/exec.rs`**）调用 **`execute_pipeline`**  
   **Then** 错误传播到用户可见输出（stderr 或等价），**不**静默成功（**NFR-P2**）。

5. **棕地**：核心逻辑在 **`crates/todo/src/devshell/command/dispatch/mod.rs`**（**`execute_pipeline`**、**`check_pipeline_inter_stage_size`**、**`pipeline_limit_tests`**）、**`command/types.rs`**（**`BuiltinError`**）；本故事以 **核对 AC、补端到端/集成测试、文档常量交叉引用**为主，**不**无依据上调/下调 **16 MiB**（若变更须同步 **`docs/devshell-vm-gamma.md`**、**`requirements`/PRD** 与 **CHANGELOG**）。

6. **回归**：**`cargo test -p xtask-todo-lib`**、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：对照 **`docs/requirements.md` §5.5** 与 **`execute_pipeline`** 实现；确认 **`|`** 与重定向语义在 **`parser`** + **`dispatch`** 中的分工。
- [ ] **边界测试**：保留 **`pipeline_limit_tests`**；若缺**端到端**「管道产生 >16MiB 中间输出」用例，评估**性能/CI 时间**后添加**生成固定大小**的集成测试或**文档化手工步骤**。
- [ ] **文档**：确认 **`docs/devshell-vm-gamma.md`**（管道 §8.2 引用）、**`command` 模块导出** **`PIPELINE_INTER_STAGE_MAX_BYTES`** 的公开说明与 **`cargo doc`** 可见性。
- [ ] **验证**：`cargo test -p xtask-todo-lib devshell`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 常量 | **`PIPELINE_INTER_STAGE_MAX_BYTES`** = **16 * 1024 * 1024`**（**`dispatch/mod.rs`**） |
| 管线 | **`execute_pipeline`**：非末段 **`next_buffer`** 整段缓冲 → **`check_pipeline_inter_stage_size`** |
| 错误 | **`BuiltinError::PipelineInterStageBufferExceeded`**（**`types.rs`**） |
| 调用链 | **`repl.rs`**、**`script/exec.rs`** → **`execute_pipeline`** |

### 架构合规（摘录）

- 管道为**进程内**阶段缓冲，**非** Unix 真 pipe；上限为 **NFR-P2** 显式约束。

### 前序故事

- **3-2**（内置文件操作）：单命令与重定向；本故事聚焦 **多段管道** 与 **段间上限**。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.3]
- [Source: `docs/requirements.md` — §5.5]
- [Source: `docs/devshell-vm-gamma.md` — 管道与 `PIPELINE_INTER_STAGE_MAX_BYTES`]
- [Source: `crates/todo/src/devshell/command/dispatch/mod.rs`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
