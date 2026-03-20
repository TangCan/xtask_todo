# 验收文档（Acceptance）

对 [requirements.md](./requirements.md) 中的能力与验收标准做**可勾选**验证；发布或迭代结束时可作为签字依据。详细用例与自动化映射见 [test-cases.md](./test-cases.md)；覆盖率目标见 [test-coverage.md](./test-coverage.md)。

---

## 1. 验收说明

- **范围**：以 **requirements.md** 当前版本为准。
- **方式**：自动化（测试名 / 命令）或手工；CI 通过的测试可标「由 CI 执行」。
- **结果**：✅ 通过 / ❌ 未通过 / ⏸ 跳过（注明原因）。

---

## 2. Todo 领域（requirements §3）

以下 **US-T\*** 编号便于与 [test-cases.md](./test-cases.md) 对照；语义以 **requirements §3** 为准。

### 2.1 US-T1 创建

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| T1-1 | 有效标题创建成功并得 id | `TodoList::create` / CLI `add` | 返回 id，列表含该项 | |
| T1-2 | 空或非法标题拒绝 | 单元或 `cargo xtask todo add ""` | `Err` 或退出码 **2**，列表无新增 | |

### 2.2 US-T2 列表

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| T2-1 | 无数据时为空列表 | `list` | 空或提示一致 | |
| T2-2 | 多条顺序与过滤排序符合约定 | `list` + 选项 | 与 **requirements §3.2** 一致 | |

### 2.3 US-T3 / US-T4 完成与删除

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| T3-1 | 完成后项为已完成 | `complete` → `list` | `completed` 为 true | |
| T3-2 | 不存在 id 完成 → 错误 | `complete` 无效 id | **退出码 3** 或 `NotFound` | |
| T4-1 | 删除后项消失 | `delete` → `list` | 无该 id | |
| T4-2 | 不存在 id 删除 | 与设计一致（Err 或幂等） | 数据一致 | |

### 2.4 US-T5～T6 时间与展示

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| T5-1 | 创建/完成时间字段 | 模型与 `show` | 含 `created_at` / `completed_at` | |
| T5-2 | 列表人类可读含相对时间与用时 | `todo list` | 符合 **format** 约定 | |
| T6-1 | TTY 下超 **7 天**未完成项视觉区分 | 终端 `list` | 有着色 | |
| T6-2 | 非 TTY 无 ANSI | `list \| cat` | 无转义序列 | |

### 2.5 US-T7～T13 扩展能力

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| T7 | `show` 有效/无效 id | CLI | 详情或错误与非 0 退出 | |
| T8 | `update` 与 `--clear-repeat-rule` | CLI | 持久化一致 | |
| T9 | 可选字段与 `list` 过滤/排序；非法日期 **退出码 2** | CLI | 与 **§3.2** 一致 | |
| T10 | `search` 命中与空结果 | CLI | 符合预期 | |
| T11 | `stats` 含总数/未完成/已完成 | CLI | 输出含统计 | |
| T12 | `export`/`import` JSON·CSV；`import --replace` | CLI + 文件 | 数据一致 | |
| T13 | 重复规则、`complete --no-next`、终止条件 | CLI + 领域测试 | 与 **requirements §3.1** 一致 | |

---

## 3. Xtask 与 AI（requirements §4、§6）

### 3.1 子命令与帮助

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| X1-1 | `cargo xtask --help` 含子命令 | 项目根执行 | 列出 `run`、`todo` 等 | |
| X1-2 | 已实现子命令可执行且退出码合理 | 抽样 | 成功 **0**，失败非 0 | |
| X2-1 | `cargo xtask run` 按约定执行 | 集成测试 / 手工 | 无 panic，退出码合理 | |
| X3-1 | 新子命令在 `XtaskSub` 注册即可 | 代码结构 | 无需改 cargo 别名 | |
| X4-1 | `todo add/list/complete/delete` 持久化 **`.todo.json`** | CLI | 重启后数据一致 | |
| X4-2 | `cargo xtask todo --help` 含子命令与全局 `--json`、`--dry-run` | `--help` | 文档与实现一致 | |

### 3.2 US-A1～A4

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| A1 | `--json` 可解析 | 各子命令 | 成功/失败结构约定 | |
| A2 | 退出码 **0/1/2/3** | 成功、缺参、不存在 id | 与 **requirements §6** 一致 | |
| A3 | `init-ai` 生成技能/命令文件 | `todo init-ai` | 目标目录有文件 | |
| A4 | `--dry-run` 不写 **`.todo.json`**、不改列表 | 修改类命令 | 文件与内存列表不变 | |

---

## 4. Devshell（requirements §1.1、§5）

| # | 验收标准 | 验证方式 | 预期结果 | 结果 |
|---|----------|----------|----------|------|
| D1 | REPL / `-f` 脚本可启动；非法参数非 0 退出 | 集成测试 / 手工 | 与 **§5.3** 一致 | |
| D2 | VFS：`cd`/`ls`/`cat`/管道/重定向 | `devshell` 测试 | 通过 | |
| D3 | 内置 `todo` 子集与 **`.todo.json`** 一致 | `run_todo` 等 | 通过 | |
| D4 | Tab 补全 **`CompletionType::List`**；路径保留前缀 | 单元测试 | 通过 | |
| D5 | `rustup`/`cargo` 经 sandbox 或 VM（视 env） | `sandbox` / 手工 | 与 **design §2.5** 一致 | |
| D6 | **Mode P**（可选）：`DEVSHELL_VM_WORKSPACE_MODE=guest` 且 VM 可用时，工程树与 guest 挂载一致 | 需 Lima/β 环境 | 与 **requirements §5.1** 一致 | |
| D7 | 会话元数据路径与 **§1.1** 一致（工作区内 JSON，非宿主 cwd 旁规范文件） | 代码评审 / 将来契约测试 | 与需求一致 | |

---

## 5. 非功能（requirements §7）

| # | 项目 | 验证方式 | 预期结果 | 结果 |
|---|------|----------|----------|------|
| NF-1 | Workspace 含 `crates/todo`、`xtask` | `Cargo.toml` | members 正确 | |
| NF-2 | `cargo xtask` 通过 **`.cargo/config.toml`** | 未全局安装 xtask 插件 | 可执行 | |
| NF-3 | 主版本 CLI 稳定性 | CHANGELOG / 评审 | 破坏性变更有说明 | |
| NF-4 | `--help` 与 README 一致 | 文档 | 同步 | |

---

## 6. 验收汇总

| 类别 | 说明 | 通过 | 未通过 | 跳过 |
|------|------|------|--------|------|
| §3 Todo | §2 各表 | | | |
| §4 xtask | §3.1 | | | |
| §6 AI | §3.2 | | | |
| §5 Devshell | §4 | | | |
| §7 非功能 | §5 | | | |

**结论**：☑ 全部通过，可发布 / □ 部分通过 / □ 不通过。

**签字**：________________ **日期**：________________

---

## 7. 文档维护

- 需求变更时同步 [requirements.md](./requirements.md)、本文档与 [test-cases.md](./test-cases.md)。
- 实现与验收冲突时：按产品决策改代码或改文档。
