# Story 5.4：init-ai 技能材料

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名使用 AI 助手的开发者，  
我希望运行 **`todo init-ai`** 生成**约定**的技能/命令片段，  
以便团队共享调用方式（**US-A3**）。

## 映射需求

- **FR29**：可生成面向外部工具（含 AI 助手）的初始化/技能材料（以 **`init-ai`** 约定为准）。
- **US-A3**（**`requirements` §6**）：**`init-ai`** 可生成工具用命令/技能文件。

## Acceptance Criteria

1. **Given** **`TodoInitAiArgs`**（**`xtask/src/todo/args.rs`**）— **`--for-tool`**、**`--output`**（默认输出目录文档化为 **`.cursor/commands`**，相对 **`cwd`**）  
   **When** 执行 **`cargo xtask todo init-ai`**（或独立 **`todo init-ai`**）  
   **Then** 在目标目录生成 **`todo.md`**（或文档约定的主文件名），且 **`std::fs::create_dir_all`** 成功；与 **`run_init_ai`**（**`xtask/src/todo/init_ai.rs`**）一致（**FR29**）。

2. **Given** 生成文件内容（当前为内嵌 Markdown 模板字符串）  
   **When** 与 **`cargo xtask todo --help`** / 子命令 **`--help`** 及 **`requirements §3.2`** 对照  
   **Then** 子命令列表、**`--json`** 说明、**`--dry-run`** 说明、退出码摘要与主线实现**无矛盾**；若 CLI 已增新 flag/子命令，**`init_ai.rs`** 模板须同步或故事内记录**刻意滞后**（**FR29**）。

3. **Given** **`--json`** 全局 flag（**`dispatch.rs`** **`InitAi` 分支先于 `load_todos`**）  
   **When** **`todo --json init-ai`** 成功  
   **Then** **stdout** 含 **`{"status":"success","data":{"generated":true}}`**（或等价字段）；人机模式打印 **`Generated init-ai skill file.`**（**FR26** 与 **FR29** 交叉）。

4. **Given** **`--for-tool`**  
   **When** 当前实现中 **`run_init_ai(_for_tool, …)`** 未分支  
   **Then** 行为为 **保留** 或 **文档化**（例如「预留 cursor」）；**不**在本故事中静默引入破坏性路径变更，除非有明确产品说明。

5. **棕地**：**`xtask/src/tests/todo/todo_cmd/json_dry_init.rs`** **`cmd_todo_init_ai_generates_file`** 已覆盖生成与内容片段；本故事以 **核对 AC、补测试（如默认路径、`**`--json`** 成功体）**、**与 `requirements` / PRD 对齐** 为主。

6. **回归**：**`cargo test -p xtask`**（含 **`json_dry_init`**）、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [x] **路径**：验证默认 **`cwd/.cursor/commands/todo.md`** 与 **`--output`** 覆盖；**Windows** 路径分隔与权限（**最小**手工或已有 CI）。
- [x] **内容审计**：将 **`init_ai.rs`** 模板与 **`args.rs`** 子命令逐行对照；更新模板或 **`requirements §3.2`** 脚注。
- [x] **`--json`**：若缺 **`init-ai` + `--json`** 单元测试，按 **`dispatch`** 行为补测。
- [x] **`for_tool`**：决定实现占位或帮助文案，避免用户以为会生成多工具不同文件。
- [x] **验证**：**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 入口 | **`dispatch.rs`** — **`TodoSub::InitAi`** 先于 **`load_todos`** |
| 实现 | **`xtask/src/todo/init_ai.rs`** — **`run_init_ai`**、固定 **`todo.md`** 内容 |
| 测试 | **`json_dry_init.rs`**、**`cmd_todo_init_ai_generates_file`** |

### 架构合规（摘录）

- **Devshell** 内 **`todo`** **不**暴露 **`init-ai`**（**`requirements` §5.4**）；本故事不改变该限制。

### 前序故事

- **5.1**：**`--json`** 形状；**5.3**：退出码；**init-ai** 成功路径应与之兼容。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 Story 5.4]
- [Source: `docs/requirements.md` — §3.2 `init-ai`、§6 **US-A3**]
- [Source: `xtask/src/todo/init_ai.rs`]
- [Source: `xtask/src/todo/cmd/dispatch.rs` — `InitAi` 分支]
- [Source: `xtask/src/tests/todo/todo_cmd/json_dry_init.rs`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo test -p xtask todo_bin_init_ai_with_output_and_for_tool_generates_todo_md`
- `cargo test -p xtask && cargo clippy -p xtask --all-targets -- -D warnings`

### Completion Notes List

- 已核对 `init-ai` 路径行为：默认输出 `cwd/.cursor/commands/todo.md`，`--output` 可覆盖；`dispatch.rs` 中 `InitAi` 分支先于 `load_todos` 执行，符合 AC1。
- 已完成模板审计：`init_ai.rs` 子命令清单与 `args.rs::TodoSub` 对齐（含 `init-ai`、`--json`、`--dry-run`、退出码摘要）；并在模板新增 Notes 段，明确 `--for-tool` 当前为预留参数。
- 已补独立二进制端到端测试 `todo_bin_init_ai_with_output_and_for_tool_generates_todo_md`，验证 `todo init-ai --for-tool other --output <dir>` 可生成 `todo.md`，并包含保留参数提示，覆盖 AC1/AC4。
- 现有 `json_dry_init` 测试已覆盖 `init-ai + --json` 成功路径（`{"status":"success","data":{"generated":true}}`），与 AC3 一致。
- `requirements.md §3.2` 已补一句：`init-ai` 默认路径与 `--for-tool` 预留语义，避免多工具分支误解。

### File List

- `xtask/src/todo/init_ai.rs`
- `xtask/tests/todo_list/basic.rs`
- `docs/requirements.md`
- `_bmad-output/implementation-artifacts/5-4-init-ai-skill-materials.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings（BMad 分层审查 · 2026-03-26）

| 层 | 结论 |
|----|------|
| **Blind Hunter** | **`init_ai.rs`** 模板增加 **Notes**：**`--for-tool` 预留**；**`requirements.md`** 同步默认路径与预留语义；**`todo_bin_init_ai_with_output_and_for_tool_generates_todo_md`** 验证 **`todo.md`** 生成及文案含 **`currently reserved`**，覆盖 **AC1/AC4**。 |
| **Edge Case Hunter** | 子命令清单与 **5.1/5.3** 叙述一致（**JSON / dry-run / 退出码**）；**AC3** 仍由 **`json_dry_init`** 覆盖 **`--json` 成功体**（Completion Notes 已核对）。 |
| **Acceptance Auditor** | **AC6**：**`cargo test -p xtask`**、**`clippy -p xtask -D warnings`** 已通过。 |

**待办项**：无。勿提交 **`crates/todo/.dev_shell.bin`** 等非本故事产物。

## Change Log

- **2026-03-26**：BMad 代码审查通过 — **5-4-init-ai-skill-materials** 与 **Epic 5** 标为 **done**；sprint **`last_updated`** 与 **`# last_updated`** 为 **`2026-03-26T06:05:00Z`**（以仓库为准）；复验 **`cargo test -p xtask`**、**clippy**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
