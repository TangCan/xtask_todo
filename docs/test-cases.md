# 测试用例 (Test Cases)

本文档列出与 [requirements.md](./requirements.md) 和 [acceptance.md](./acceptance.md) 对应的测试用例，便于需求→用例→验证的追溯与回归。

**结构说明**：每条用例包含用例 ID、需求/验收引用、描述、步骤、验证方式、预期结果。

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
