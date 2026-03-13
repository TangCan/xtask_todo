# 测试用例 (Test Cases)

本文档列出与 [requirements.md](./requirements.md) 和 [acceptance.md](./acceptance.md) 对应的测试用例，便于需求→用例→验证的追溯与回归。

**结构说明**：每条用例包含用例 ID、需求/验收引用、描述、步骤、验证方式、预期结果。

**实现状态**：上述需求均已实现。TC-T* 对应 `crates/todo` 单元/集成测试；TC-X*、TC-A* 及部分 TC-T* 对应 `xtask` 内 `tests/`（如 `tests/todo/`、`tests/run.rs`、`tests/clippy.rs`、`tests/git.rs`）。pre-commit 或 CI 中通过 `cargo test` 执行回归。

---

## 1. Todo 领域

### US-T1：创建待办

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T1-1 | US-T1 / T1-1 | 有效标题创建成功 | 对非空标题调用 `TodoList::create`；再调用 `list()` | 单元/集成测试 | 返回 `Ok(TodoId)`，列表中有一条 title 与 id 一致 |
| TC-T1-2 | US-T1 / T1-2 | 空标题或非法输入拒绝 | 对空字符串（及约定非法输入）调用 `create`；再调用 `list()` | 单元/集成测试 | 返回 `Err(TodoError::InvalidInput)`（或等价），列表条数不变 |

### US-T2：列出待办

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T2-1 | US-T2 / T2-1 | 空列表 | 新建 `TodoList` 后调用 `list()` | 单元/集成测试 | 返回空列表（如 `Vec::new()`） |
| TC-T2-2 | US-T2 / T2-2 | 列表内容与顺序 | 依次 `create` 若干条，再 `list()` | 单元/集成测试 | 列表长度与 create 次数一致，顺序按创建时间（或设计约定） |

### US-T3：完成待办

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T3-1 | US-T3 / T3-1 | 存在 id 完成成功 | `create` → `complete(id)` → `list()` 或 `get(id)` | 单元/集成测试 | 该项 `completed == true` |
| TC-T3-2 | US-T3 / T3-2 | 不存在 id 返回错误 | `create` 一条，对不存在的 `TodoId` 调用 `complete`；再 `list()` | 单元/集成测试 | 返回 `Err(TodoError::NotFound)`，原有待办状态不变 |

### US-T4：删除待办

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T4-1 | US-T4 / T4-1 | 存在 id 删除成功 | `create` → 记下 id → `delete(id)` → `list()` | 单元/集成测试 | 列表无该项，按 id 查询为 None 或等价 |
| TC-T4-2 | US-T4 / T4-2 | 不存在 id 的约定行为 | 对不存在的 id 调用 `delete`；再 `list()` | 单元/集成测试 | 符合 design 约定（Err 或幂等 Ok），列表条数不变 |

### US-T5：时间戳与完成时间

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T5-1 | US-T5 | 创建时记录 created_at，完成时记录 completed_at | `create` 后检查 `list()[0].created_at`；`complete(id)` 后检查 `list()[0].completed_at.is_some()` | 单元测试 | 创建后有创建时间；完成后有完成时间 |
| TC-T5-2 | US-T5 | 列表展示创建/完成时间与用时 | `cargo xtask todo list` 查看输出 | 手工/CI | 每行含「创建 X」「完成 Y」「用时 Z」（已完成项） |

### US-T6：长时间未完成提醒

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T6-1 | US-T6 | TTY 下超 7 天未完成项着色 | 在 TTY 下执行 `cargo xtask todo list`，存在超 7 天未完成项 | 手工 | 该行以不同颜色（如黄色）显示 |
| TC-T6-2 | US-T6 | 非 TTY 不输出颜色 | `cargo xtask todo list \| cat` 或重定向到文件 | 手工 | 输出无 ANSI 转义码 |

### US-T7：查看单条任务（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T7-1 | US-T7 | 有效 id 返回完整字段 | 对已存在 id 调用 `todo show <id>`（人类可读含描述、截止、优先级、标签、重复规则与结束条件；`--json` 含全部字段） | 单元/集成/CLI | 输出该任务 id、标题、创建时间、完成时间、状态及可选字段 |
| TC-T7-2 | US-T7 | 不存在 id 返回错误 | 对不存在的 id 调用 show | 单元/集成/CLI | 明确错误信息，退出码非 0 |

### US-T8：更新任务（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T8-1 | US-T8 | 有效 id 与合法修改更新成功 | update(id, 新标题及可选 --description/--due-date/--priority/--tags/--repeat-rule/--repeat-until/--repeat-count) 后 list/show | 单元/集成/CLI | 该任务内容更新，持久化一致 |
| TC-T8-2 | US-T8 | 不存在 id 或非法参数返回错误 | 对不存在 id 或非法参数调用 update | 单元/集成 | 返回错误，其他任务不变 |

### US-T9：任务可选属性（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T9-1 | US-T9 | add/update 支持描述、截止、优先级、标签（CLI：--description、--due-date、--priority、--tags） | add/update 带可选参数，list/show 查看 | 单元/集成/CLI | 字段正确存储与展示 |
| TC-T9-2 | US-T9 | list 支持按状态/优先级/标签/截止过滤与排序 | CLI `todo list --status incomplete`、`--priority high`、`--tags a,b`、`--due-before 2025-12-31`、`--sort due-date` 等 | 单元/集成/CLI | 结果集与顺序符合选项 |

### US-T10：搜索任务（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T10-1 | US-T10 | 关键词匹配返回列表 | search(keyword)，存在匹配任务 | 单元/集成 | 返回包含匹配项的列表 |
| TC-T10-2 | US-T10 | 无匹配返回空列表 | search(不存在的关键词) | 单元/集成 | 返回空列表 |

### US-T11：统计信息（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T11-1 | US-T11 | stats 输出总数、未完成、已完成 | 创建若干条并完成部分后调用 stats | 单元/集成/CLI | 输出含总任务数、未完成数、已完成数 |

### US-T12：导入与导出（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T12-1 | US-T12 | export 写出文件（格式由扩展名或 --format json|csv 决定） | export(file.csv) 或 export(file.json)、export(file, --format csv)，检查文件内容 | 单元/集成/CLI | 文件格式正确（JSON 或 CSV） |
| TC-T12-2 | US-T12 | import 读入并合并/覆盖（支持 .json/.csv 按扩展名识别） | import(file.json) 或 import(file.csv) 后 list | 单元/集成/CLI | 任务列表与文件一致或按约定合并 |

### US-T13：定期重复任务（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-T13-1 | US-T13 | 带重复规则任务完成时生成下一实例 | 创建 daily 重复任务，complete(id)，再 list | 单元/集成 | 出现新任务且截止日期符合规则 |
| TC-T13-2 | US-T13 | --no-next 仅完成不生成下一笔 | complete(id --no-next)，再 list | 单元/集成/CLI | 原任务完成，无新实例 |
| TC-T13-3 | US-T13 | 重复规则支持 2d/3w 简写及 custom:N | 使用 repeat_rule "2d" 或 "3w" 添加/更新任务，完成时下一实例截止日正确 | 单元/集成 | 2d→每 2 天，3w→每 21 天，next_due 正确 |
| TC-T13-4 | US-T13 | 结束条件 repeat_count/repeat_until | repeat_count=1 完成时不生成下一笔；repeat_until 早于 next_due 时不生成 | 单元/集成 | 符合结束条件时无新实例 |
| TC-T13-5 | US-T13 | show/update 展示与修改重复规则 | show(id) 含 repeat_rule；update 修改规则（含可选字段）或使用 `--clear-repeat-rule` 取消规则 | 单元/集成/CLI | 规则正确展示与持久化；--clear-repeat-rule 后 repeat_rule 为 None |
| TC-T13-6 | US-T13 | add/update 非法 repeat_count 返回参数错误 | add 带 `--repeat-count not_a_number` | CLI | 退出码 2，错误信息含 invalid repeat_count |
| TC-T13-7 | US-T13 | add 支持 --repeat-until/--repeat-count 端到端 | 命令行 `todo add "标题" --repeat-rule weekly --repeat-until 2026-12-31 --repeat-count 3`，再 list | 集成 | 任务创建成功，list 可见该任务 |

### 日期格式校验（CLI）

| ID   | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|------|------|----------|----------|
| TC-DATE-1 | due_date/repeat_until 非 YYYY-MM-DD 返回参数错误 | add 带 `--due-date not-a-date` 或 `--repeat-until 2026/01/01` | CLI | 退出码 2，错误信息含 invalid due_date / invalid repeat_until |
| TC-DATE-2 | list 的 due_before/due_after 非 YYYY-MM-DD 返回参数错误 | list 带 `--due-before 2026/01/01` | CLI | 退出码 2，错误信息含 invalid due_before / invalid due_after |

### US-A1：结构化 JSON 输出（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-A1-1 | US-A1 | --json 输出合法 JSON | 各子命令加 --json，解析输出 | 单元/集成/CLI | 合法 JSON，成功含 status+data，失败含 status+error |

### US-A2：标准退出码（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-A2-1 | US-A2 | 成功 0、参数错误 2、数据错误 3 | 执行成功/缺参/id 不存在等场景，查退出码 | CLI | 0 / 2 / 3 符合约定 |

### US-A3：AI 技能生成（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-A3-1 | US-A3 | init-ai 生成技能文件 | `cargo xtask todo init-ai` 或 `--for-tool cursor`、`--output <dir>`，默认 `.cursor/commands/` | CLI | 约定目录下生成 `todo.md`，含子命令说明与 --json/退出码/dry-run 说明 |

### US-A4：模拟执行（扩展）

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-A4-1 | US-A4 | --dry-run 不写数据且不修改内存 | add/update/complete/delete 带 --dry-run，再 list 或查文件 | 单元/集成/CLI | 输出拟执行操作，.todo.json 未变更，列表条数/内容不变 |

---

## 2. Xtask 工作流

### US-X1：通过 cargo xtask 执行任务

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-X1-1 | US-X1 / X1-1 | 帮助输出 | 在项目根执行 `cargo xtask --help` | 手工/CI | 输出用法说明及子命令列表（如 `run`） |
| TC-X1-2 | US-X1 / X1-2 | 子命令退出码 | 执行 `cargo xtask run`（及已实现子命令）；成功与失败场景 | 手工/CI | 成功退出码 0，失败非 0 |

### US-X2：运行主程序

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-X2-1 | US-X2 / X2-1 | run 执行成功 | 在项目根执行 `cargo xtask run` | 手工/CI | 进程正常执行并退出，无 panic |
| TC-X2-2 | US-X2 / X2-2 | 依赖未满足时错误 | 在依赖未满足环境下执行 `cargo xtask run`（若当前无此类依赖可标 N/A） | 手工/CI | stderr 有明确错误，退出码非 0 |

### US-X3：扩展新的 xtask 子命令

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-X3-1 | US-X3 / X3-1 | 新子命令可执行 | 在 xtask 中新增子命令（如 `build`）后执行 `cargo xtask <新命令>` | 手工/CI | 新命令被正确解析并执行 |
| TC-X3-2 | US-X3 / X3-2 | 帮助中展示新命令 | 执行 `cargo xtask --help` | 手工/CI | 帮助文本中包含新子命令及描述 |

### US-X4：cargo xtask todo 任务管理

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-X4-1 | US-X4 | add 新增并持久化 | `cargo xtask todo add "标题"`，再 `cargo xtask todo list` | 手工/CI | 新项出现；重启后 list 仍可见 |
| TC-X4-2 | US-X4 | list/complete/delete 生效并持久化 | `cargo xtask todo list`、`complete <id>`、`delete <id>`，再次 list | 手工/CI | 列表正确；complete/delete 后数据写回 `.todo.json` |
| TC-X4-3 | US-X4 | 数据文件位置 | 查看项目根目录 | 手工 | 存在 `.todo.json`（或文档约定的路径） |

---

## 3. 非功能与约束

| ID   | 需求/验收 | 描述 | 步骤 | 验证方式 | 预期结果 |
|------|-----------|------|------|----------|----------|
| TC-NF-1 | NF-1 | Workspace 结构 | 查看根 `Cargo.toml` 的 `[workspace] members` | 手工 | 存在 `crates/todo`、`xtask` |
| TC-NF-2 | NF-2 | cargo xtask 无需全局安装 | 未安装 cargo-xtask 时执行 `cargo xtask --help` | 手工 | 命令可用 |

---

## 4. Maintenance（维护）

- 新增或变更 [requirements.md](./requirements.md) 中的用户故事与验收标准时，须在本文档中补充或更新对应测试用例（至少一条验收标准对应一条用例），或显式注明暂不覆盖（如 N/A / 后续补充）。
- 验收文档 [acceptance.md](./acceptance.md) 中的验收项与本文档用例的对应关系应保持可追溯（通过需求/验收引用列）。
