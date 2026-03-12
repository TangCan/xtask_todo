# 设计说明 (Design)

本文档描述 xtask_todo 的技术架构、数据流与接口设计，与 [requirements.md](./requirements.md) 中的用户故事与验收标准对应。

---

## 1. 技术架构

### 1.1 总体结构

项目为 Cargo workspace，根目录仅做工作区配置，业务与工具拆分到独立 crate：

```
┌─────────────────────────────────────────────────────────────────┐
│                     xtask_todo (workspace)                        │
├─────────────────────────────────────────────────────────────────┤
│  crates/todo          │  xtask                                   │
│  领域库 + 存储抽象     │  CLI 入口，调用 todo 或执行构建/脚本      │
└─────────────────────────────────────────────────────────────────┘
```

- **crates/todo**：待办领域逻辑与对外 API，不依赖 xtask。
- **xtask**：通过 `cargo xtask` 调用的二进制，封装「运行主程序、构建、发布」等脚本式任务，可依赖 `todo` 做集成或演示。

### 1.2 技术选型

| 层级       | 选型        | 说明 |
|------------|-------------|------|
| 语言       | Rust 2021   | 与 requirements 一致 |
| 工作区解析 | resolver = "2" | 统一依赖版本 |
| xtask CLI  | argh        | 子命令与参数解析，零配置 |
| 入口       | .cargo/config.toml alias | `cargo xtask` → `cargo run -p xtask --` |

### 1.3 Todo 库分层（crates/todo）

```
┌──────────────────────────────────────┐
│  Public API (调用方：CLI / 其他 crate) │
│  TodoList, create, list, complete, …  │
├──────────────────────────────────────┤
│  Domain (领域模型与规则)               │
│  Todo, TodoId, 状态、校验              │
├──────────────────────────────────────┤
│  Storage (存储抽象，可替换实现)         │
│  Store trait → InMemoryStore / 后续   │
└──────────────────────────────────────┘
```

- **Public API**：对外暴露的类型与函数，满足 requirements 中的创建/列表/完成/删除。
- **Domain**：待办实体、ID、状态及不依赖存储的校验（如标题非空）。
- **Storage**：以 trait 抽象「增删改查」，默认内存实现，便于测试与后续扩展持久化。

### 1.4 Xtask 角色

- 作为**单一入口**：`cargo xtask run`、`cargo xtask build` 等，不依赖系统 shell 或外部脚本。
- **不承载领域逻辑**：业务在 `todo` 中；xtask 只做编排（如调用 `todo` API、执行 `cargo build`、写文件等）。

---

## 2. 数据流

### 2.1 待办操作数据流（Todo 领域）

调用方（CLI 或测试）通过 todo 的 Public API 操作，API 使用领域模型并委托给 Store 实现。

```mermaid
flowchart LR
    subgraph Caller["调用方"]
        CLI[CLI / 测试]
    end
    subgraph API["todo Public API"]
        create[create]
        list[list]
        complete[complete]
        delete[delete]
    end
    subgraph Domain["领域层"]
        Todo[Todo]
        Rules[校验规则]
    end
    subgraph Store["存储"]
        Mem[(InMemoryStore)]
    end
    CLI --> create
    CLI --> list
    CLI --> complete
    CLI --> delete
    create --> Rules
    complete --> Rules
    delete --> Rules
    create --> Todo
    list --> Mem
    create --> Mem
    complete --> Mem
    delete --> Mem
```

- **创建**：输入标题 → 校验（非空等）→ 构造 `Todo` → 写入 Store → 返回 id。
- **列表**：无入参 → 从 Store 读取 → 按约定排序（如创建时间）→ 返回列表。
- **完成/删除**：输入 id → 校验存在性及权限 → 更新/移除 Store → 返回结果或错误。

### 2.2 Xtask 调用链

开发者执行 `cargo xtask <子命令>` 时，由 Cargo 根据 alias 调用 xtask 二进制，再按子命令分发。

```mermaid
sequenceDiagram
    participant Dev as 开发者
    participant Cargo as cargo
    participant Xtask as xtask 二进制
    participant Todo as crates/todo
    participant System as 系统/文件/cargo

    Dev->>Cargo: cargo xtask run
    Cargo->>Xtask: run -p xtask -- run
    Xtask->>Xtask: 解析子命令 (argh)
    alt run
        Xtask->>System: 执行主程序 (如 cargo run -p todo-cli)
        System-->>Xtask: 退出码
    else 其他子命令 (如 codegen)
        Xtask->>Todo: 可选：调用 todo API
        Xtask->>System: 脚本逻辑
        System-->>Xtask: 结果
    end
    Xtask-->>Cargo: exit code
    Cargo-->>Dev: 输出与退出码
```

- 所有「业务数据」都经由 `todo` 的 API；xtask 只做调用与进程/文件级操作。

### 2.3 状态与存储

- **crates/todo**：默认 `InMemoryStore`，进程内无持久化；可通过 `Store` 抽象扩展。
- **cargo xtask todo**：通过 xtask 将待办持久化到项目根目录的 **`.todo.json`**（JSON，含 id、title、completed、created_at_secs、completed_at_secs）；xtask 启动时加载、操作后写回，与 `TodoList` + `InMemoryStore::from_todos` 配合使用。

### 2.4 列表展示与时间

- **列表展示**（如 `cargo xtask todo list`）：每条展示创建时间（相对如「Xm/Xh/Xd ago」）；已完成项另展示完成时间与**用时**（完成时间 − 创建时间）。
- **长时间未完成提醒**：当输出为 TTY 时，创建超过约定阈值（默认 7 天）且未完成的任务以不同颜色（如 ANSI 黄色）展示；非 TTY 时不输出颜色码。

---

## 3. 接口

### 3.1 Todo 库公开 API（crates/todo）

以下为面向调用方的类型与函数级接口（具体签名以代码为准，此处约定语义与错误处理）。

#### 3.1.1 类型

| 类型        | 说明 |
|-------------|------|
| `TodoId`    | 待办唯一标识，对外不透明（如 `uuid` 或 `NonZeroU64`）。 |
| `Todo`      | 单条待办：`id`, `title`, `completed: bool`, `created_at`, `completed_at: Option<SystemTime>`。扩展（US-T9、US-T13）：可选 `description`, `due_date`, `priority`, `tags`, `repeat_rule`。 |
| `TodoList`  | 门面：持有 Store，提供 `create` / `list` / `complete` / `delete`。 |

#### 3.1.2 行为接口（函数语义）

| 操作 | 签名语义 | 返回值 | 错误 |
|------|----------|--------|------|
| 创建 | `create(&mut self, title: impl AsRef<str>)` | `Result<TodoId, TodoError>` | 标题为空或违反校验规则 |
| 列表 | `list(&self) -> Vec<Todo>` 或 `list(&self, filter?)` | 按创建时间排序的列表 | - |
| 完成 | `complete(&mut self, id: TodoId)` | `Result<(), TodoError>` | id 不存在 |
| 删除 | `delete(&mut self, id: TodoId)` | `Result<(), TodoError>` | id 不存在（可选：幂等返回 Ok） |

**扩展（US-T7～US-T13、US-A1～US-A4，已实现）**：

| 操作 | 签名语义 | 说明 |
|------|----------|------|
| 查看单条 | `get(&self, id: TodoId) -> Option<Todo>`；CLI `todo show <id>` | US-T7：返回单条完整信息，不存在为 None 或 Err |
| 更新 | `update(&mut self, id: TodoId, patch)`；CLI `todo update <id> <title>` | US-T8：修改标题、描述、截止日期、优先级、标签、重复规则等 |
| 列表过滤/排序 | `list_with_options(&self, options: &ListOptions)` | US-T9：按状态、优先级、标签、截止日期过滤与排序 |
| 搜索 | `search(&self, keyword: &str) -> Vec<Todo>`；CLI `todo search <keyword>` | US-T10：在标题、描述、标签中匹配 |
| 统计 | `stats(&self) -> (total, incomplete, complete)`；CLI `todo stats` | US-T11：总数、未完成数、已完成数 |
| 导出/导入 | CLI `todo export <file>` / `todo import <file> [--replace]`；库 `add_todo` 等 | US-T12：JSON 格式，merge 或 replace 策略 |
| 重复规则 | `Todo.repeat_rule: Option<RepeatRule>`；`complete(id, no_next)`；CLI `complete --no-next` | US-T13：daily/weekly/monthly/yearly/weekdays/custom(n)；完成时自动生成下一实例 |
| JSON 输出 | 全局 `--json`，成功 `{ "status":"success", "data": ... }`，失败含 error 与 code | US-A1 |
| 退出码 | 0 成功，1 一般错误，2 参数错误，3 数据错误（todo 子命令） | US-A2 |
| init-ai | `todo init-ai [--for-tool cursor] [--output <dir>]`，默认 `.cursor/commands/` | US-A3 |
| dry-run | 全局 `--dry-run`，add/update/complete/delete 不写 `.todo.json` | US-A4 |

#### 3.1.3 错误类型

- `TodoError`（或等价枚举）：至少包含 `InvalidInput`（如空标题）、`NotFound(TodoId)`，便于调用方与测试断言。

#### 3.1.4 存储抽象（内部接口）

- `Store` trait：提供 `insert`, `get`, `list`, `update`, `remove` 等，由 `InMemoryStore` 实现；后续可增加 `FileStore`、`SqlStore` 等而不改 Public API。

### 3.2 Xtask CLI 接口

| 子命令 | 说明 | 参数（当前/预留） |
|--------|------|-------------------|
| `run` | 运行主程序或默认「运行」行为 | 无 |
| `todo` | 待办管理（数据在 `.todo.json`） | 子命令见下 |
| （预留）`build` | 构建产物 | 可选 `--release` |
| （预留）`release` | 发布流程 | 可选版本/目标 |

**todo 子命令（已实现）**：`add "标题"`、`list`、`show <id>`、`update <id> <title>`、`complete <id> [--no-next]`、`delete <id>`、`search <keyword>`、`stats`、`export <file>`、`import <file> [--replace]`、`init-ai [--for-tool cursor] [--output <dir>]`。list 输出含创建/完成时间与用时；TTY 下对超过阈值未完成项着色。全局选项：`--json`（统一 JSON 输出）、`--dry-run`（修改类命令不写 `.todo.json`）。

- 入口：`cargo xtask [--] <子命令> [子命令参数]`。
- 帮助：`cargo xtask --help`、`cargo xtask todo --help`。
- 退出码（US-A2）：0 成功，1 一般错误，2 参数错误，3 数据错误（如 id 不存在）；仅 todo 子命令区分 2/3，其余子命令失败为 1。

### 3.3 与需求的对应关系

| 需求 | 设计对应 |
|------|----------|
| US-T1 创建待办 | `TodoList::create`，校验在 Domain，Store 持久化 |
| US-T2 列出待办 | `TodoList::list`，Store 提供列表并排序 |
| US-T3 完成待办 | `TodoList::complete`，Store 更新状态 |
| US-T4 删除待办 | `TodoList::delete`，Store 移除 |
| US-X1 通过 cargo xtask 执行 | .cargo/config.toml alias + xtask 二进制 |
| US-X2 xtask run | xtask 子命令 `run`，内部执行主程序 |
| US-X3 扩展子命令 | 在 xtask 中新增 argh 子命令与对应 handler |
| US-T5 时间戳与完成时间 | Todo 的 created_at / completed_at；list 展示创建/完成/用时 |
| US-T6 长时间未完成提醒 | list 在 TTY 下对超阈值未完成项着色 |
| US-X4 cargo xtask todo | xtask 子命令 todo add/list/complete/delete，.todo.json 持久化 |
| US-T7 查看单条 | `TodoList::get(id)` 或 Store::get；CLI `todo show <id>`（扩展） |
| US-T8 更新任务 | `TodoList::update(id, patch)`；CLI `todo update <id>`（扩展） |
| US-T9 任务可选属性 | Todo 扩展 description、due_date、priority、tags；add/update 与 list 过滤/排序（扩展） |
| US-T10 搜索 | `TodoList::search(keyword)`；CLI `todo search <keyword>`（扩展） |
| US-T11 统计 | `TodoList::stats()`；CLI `todo stats`（扩展） |
| US-T12 导入导出 | 序列化/反序列化 + 文件 I/O；CLI `todo export|import <file>`（扩展） |
| US-T13 定期重复任务 | Todo 的 repeat_rule；complete 时可选生成下一实例；CLI `--no-next`、show/update 展示与修改规则（扩展） |
| US-A1 JSON 输出 | 各子命令 `--json`，统一 JSON 结构（扩展） |
| US-A2 标准退出码 | 0/1/2/3 约定（扩展） |
| US-A3 init-ai | `todo init-ai --for <tool>`，生成技能文件到目标目录（扩展） |
| US-A4 dry-run | 修改类命令 `--dry-run`，不写存储（扩展） |

---

## 4. 扩展与维护

- **新增 todo 能力**：在 Domain 与 Public API 增加方法或类型，必要时扩展 `Store` trait 与现有实现；扩展需求见 US-T7～US-T13、US-A1～US-A4（见 3.1.2 与 3.3）。
- **新增 xtask 子命令**：在 `xtask/src/main.rs` 中增加子命令枚举与实现，保持 `cargo xtask --help` 更新；扩展子命令 show/update/search/stats/export/import/init-ai 及全局选项 --json、--dry-run 见 3.2。
- **持久化**：新增实现 `Store` 的 crate，在构造 `TodoList` 时注入，不改变本文档中的 Public API 与数据流图。
- **重复任务**：实现 US-T13 时需在 Domain 增加 `RepeatRule`（如 type + interval + until），Store 需支持按规则生成下一实例并写入；CLI complete 支持 `--no-next`。

文档与实现不一致时，以代码为准并同步更新本文档。
