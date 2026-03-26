# Story 5.1：结构化 JSON 输出

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名集成方或脚本作者，  
我希望在**支持的子命令**上使用 **`--json`** 获得**稳定**的成功与失败载荷，  
以便机器解析（**UX-DR3**）。

## 映射需求

- **FR26**：在支持的子命令上，**`--json`** 下成功/失败均为可解析的结构化输出。
- **UX-DR3**：人机与 **`--json`** 两种模式下错误与成功均可理解或可机读（与 **FR26–FR28** 一致；**5.2/5.3** 分别覆盖 **`--dry-run`** 与退出码）。

## Acceptance Criteria

1. **Given** **`cargo xtask todo`**（及独立 **`todo`** 二进制）的全局开关 **`--json`**（见 **`xtask/src/todo/args.rs` `TodoArgs::json`**）  
   **When** 子命令在**成功**路径执行  
   **Then** **stdout** 为单行 JSON，且包含统一 **`status: "success"`** 与 **`data`** 载荷（见 **`error.rs` `TodoJsonSuccess`**、各 **`handle_*`** 中的 **`print_json_success`**）（**FR26**）。

2. **Given** 同上且 **`--json`**  
   **When** 子命令在**失败**路径结束（验证失败、数据不存在、I/O 等）  
   **Then** **stdout** 为单行 JSON，**`status: "error"`**，**`error.code`** 与 **`error.message`** 与 **`TodoCliError::exit_code`** 一致（**`TodoCliError` 与 `EXIT_*` 常量**（**`error.rs`**）在 **5.3** 可与 **`requirements §6`** 码表再对齐；本故事至少与 **`error.rs` `TodoJsonError`** 一致）（**FR26**）。

3. **Given** **`requirements §3.2`** 与 **`docs/requirements.md` §5（非功能）** — **`--json` 成功/失败均为可解析 JSON**  
   **When** 核对下列子命令的 **`data` 形状**（含 `list` 空列表的 **`empty`/`message`**，见 **`todo_list_json_payload`**）  
   **Then** 与 **`xtask/src/todo/cmd/dispatch.rs`** 及现有 **`xtask/tests/`**（如 **`complete_delete`**、**`import_export`**、**`json_dry_init`**）一致；**`init-ai`** 在 **`--json`** 下成功体含 **`generated: true`**（见 **`dispatch.rs`** 分支）（**FR26**）。

4. **棕地**：核心类型与打印在 **`xtask/src/todo/error.rs`**；**`xtask/src/lib.rs`** 在 **`todo` 子命令失败且 `json`** 时调用 **`print_json_error`**。本故事以 **核对 AC、补测试缺口、文档化「支持的子命令 + `data` 字段表」** 为主，**避免**重复造 JSON 封装。

5. **回归**：**`cargo test -p xtask`**（含 **`todo`** 集成测试）、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **清单**：从 **`TodoSub`**（**`args.rs`**）列出所有子命令，标注哪些 **已实现** **`--json`** 成功/失败路径（对照 **`dispatch.rs`**）。
- [x] **载荷表**：为每个子命令写 **`data` 字段**（或指向 **`serde_json::json!`** / **`todo_to_json`** 的单一事实来源）；**`list`/`search`** 与 **`todo_list_json_payload`** 对齐。
- [x] **缺口**：若某子命令 **`--json`** 仅部分路径有 JSON 或与人机模式不一致，补测试或实现（**最小化**变更）。
- [x] **文档**：在 **`docs/requirements.md` §3.2** 或 **`crates/todo/README.md`** 增加**简短** JSON 示例（成功/失败各一条）或链接到 **`xtask/tests`** 中的 fixture 行为。
- [x] **验证**：**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 统一成功/失败 | **`xtask/src/todo/error.rs`** — **`TodoJsonSuccess`**、**`TodoJsonError`**、**`print_json_success`** / **`print_json_error`** |
| 分发 | **`xtask/src/todo/cmd/dispatch.rs`** — 各 **`handle_*`** 的 **`json`** 分支 |
| 入口 | **`xtask/src/lib.rs`**（**`XtaskSub::Todo`**）、**`xtask/src/bin/todo.rs`**（独立 `todo`） |

### 架构合规（摘录）

- **`--json`** 下错误信息**仍**经 **`print_json_error`** 到 **stdout**（与 **`lib.rs`** / **`todo.rs`** 一致）；人类可读 stderr 由 **`lib.rs`** 在 **`todo` 失败**时是否仍打印需与实现核对（**不**在本故事中改变 **5.3** 退出码语义）。

### 前序故事

- **Epic 1–4**：领域与 devshell 能力已就绪；本故事聚焦 **`cargo xtask todo`** 机读契约。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 Story 5.1]
- [Source: `docs/requirements.md` — §3.2、§6 US-A1、§7 错误]
- [Source: `xtask/src/todo/error.rs`]
- [Source: `xtask/src/todo/cmd/dispatch.rs`]
- [Source: `xtask/tests/` — `complete_delete`、`todo_list`、`import_export` 等]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask xtask_todo_init_ai_json_success_has_generated_true`
- `cargo test -p xtask todo_bin_json_list_empty_matches_contract`
- `cargo test -p xtask todo_bin_json_show_invalid_id_outputs_error_contract`
- `cargo test -p xtask && cargo clippy -p xtask --all-targets -- -D warnings`

### Completion Notes List

- 已核对 `TodoSub` 全量子命令（`add/list/show/update/complete/delete/search/stats/export/import/init-ai`）在 `dispatch.rs` 均有 `--json` 成功路径；失败路径统一由 `xtask/src/lib.rs` 与 `xtask/src/bin/todo.rs` 在 `json=true` 时通过 `print_json_error` 输出 `status:error`。
- 补齐合同测试缺口：新增 `xtask todo --json init-ai` 成功载荷断言（`data.generated=true`），并新增独立 `todo` 二进制的 `--json list` 成功与 `--json show 0` 失败合同断言，覆盖 AC1/AC2 对两条入口（`cargo xtask todo` + `todo`）的一致性要求。
- 为避免并行测试下全局 `current_dir` 竞争导致 `cargo metadata` 偶发失败，在 `xtask/src/lima_todo/tests.rs` 的 smoke 测试接入 `cwd_test_lock()`；随后全量 `cargo test -p xtask` 稳定通过。
- 在 `docs/requirements.md` §3.2 增加 `--json` 成功载荷字段表（逐子命令）与统一失败载荷格式说明（`status:error + error.code/message`），将数据形状文档化为单一对外约定。

### File List

- `xtask/tests/todo_list/basic.rs`
- `xtask/src/lima_todo/tests.rs`
- `docs/requirements.md`
- `_bmad-output/implementation-artifacts/5-1-structured-json-output.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
