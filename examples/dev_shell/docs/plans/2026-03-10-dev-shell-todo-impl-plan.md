# dev_shell Todo 内置命令 — 实现计划

> **目标**：在 dev_shell 中通过内置命令 `todo` 展示 xtask-todo 能力；与 `cargo xtask todo` 共用宿主当前目录下的 `.todo.json`。  
> **设计依据**：`docs/superpowers/specs/2026-03-10-dev-shell-todo-design.md`（根目录下为 `../../../docs/superpowers/specs/2026-03-10-dev-shell-todo-design.md`）。

**前置条件**：已安装 Rust。dev_shell 当前**未**纳入 workspace，因此须在 **`examples/dev_shell`** 目录下执行 `cargo build` / `cargo run`；或在仓库根执行 `cargo build --manifest-path examples/dev_shell/Cargo.toml`。path 依赖为 `../../crates/todo`。

---

## Task 1: 添加依赖

**文件**：`examples/dev_shell/Cargo.toml`

**步骤**：在 `[dependencies]` 中增加：

```toml
todo = { path = "../../crates/todo" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

保留已有 `rustyline`。若 dev_shell 被纳入 workspace，则 `path` 按 workspace 布局调整（例如 `path = "../crates/todo"`）。

**验证**：

```bash
cd /home/ars/richard/2026/pvm_2/xtask_todo/examples/dev_shell
cargo build
```

或从仓库根：`cargo build --manifest-path examples/dev_shell/Cargo.toml`。Expected: 编译成功（dev_shell 与 path 依赖 crates/todo 均能编译）。

---

## Task 2: 实现 .todo.json 读写模块

**目的**：与 xtask 使用同一 JSON 格式，使 dev_shell 与 `cargo xtask todo` 共用同一 `.todo.json`。仅依赖 `crates/todo`，不依赖 xtask。

**文件**：新建 `examples/dev_shell/src/todo_io.rs`

**步骤**：

1. **定义 DTO**（与 xtask `xtask/src/todo/io.rs` 中 `TodoDto` 一致）：
   - 字段：`id`, `title`, `completed`, `created_at_secs`, `completed_at_secs`, `description`, `due_date`, `priority`, `tags`, `repeat_rule`, `repeat_until`, `repeat_count`；可选字段用 `#[serde(default)]`。
2. **路径**：`todo_file() -> Result<PathBuf, ...>`：`std::env::current_dir()?.join(".todo.json")`。
3. **加载**：`load_todos() -> Result<Vec<Todo>, ...>`：若文件不存在返回 `Ok(Vec::new())`；否则读字符串、`serde_json::from_str` 得到 `Vec<TodoDto>`，逐项转为 `Todo`（`TodoId::from_raw`, `SystemTime::UNIX_EPOCH + Duration::from_secs(...)`，`Priority::from_str`/`RepeatRule::from_str` 等），无效项可跳过。
4. **保存**：`save_todos(list: &TodoList<InMemoryStore>) -> Result<(), ...>`：将 `list.list()` 转为 `Vec<TodoDto>`（`id.as_u64()`, `created_at.duration_since(UNIX_EPOCH).unwrap().as_secs()` 等），`serde_json::to_string_pretty` 后写入 `todo_file()` 路径。
5. **从 Vec<Todo> 构建 TodoList**：使用 `TodoList::with_store(InMemoryStore::from_todos(todos))`（见 `crates/todo/src/store.rs`）。

**在 `lib.rs` 中**：声明 `pub mod todo_io;`（并 `pub use` 若需要）。

**验证**：

```bash
cd /home/ars/richard/2026/pvm_2/xtask_todo/examples/dev_shell && cargo build
```

Expected: 编译通过。可选：在 `tests/` 中写一个简单测试，在临时目录创建 `.todo.json`，调用 `load_todos`/`save_todos` 做一次往返（本任务可不强制，后续任务会通过手工测试验证）。

---

## Task 3: 在 command 中实现 `todo` 内置命令

**文件**：`examples/dev_shell/src/command.rs`

**步骤**：

1. **依赖**：在文件顶部 `use`：`crate::todo_io::{load_todos, save_todos}`，以及 `todo::{TodoList, InMemoryStore, TodoId, TodoPatch}` 等（按实际调用补全）。`TodoList` 与 `InMemoryStore` 来自 `todo` crate。
2. **错误**：在 `BuiltinError` 中增加变体，例如 `TodoLoadFailed`、`TodoSaveFailed`、`TodoArgError`（参数错误）、`TodoDataError`（如 id 不存在）；或在现有 `UnknownCommand` 之外用少量新变体区分「加载/保存失败」与「库返回的 InvalidInput/NotFound」。
3. **分支**：在 `run_builtin_core` 的 `match name` 中增加 `"todo"`：
   - 若 `argv.len() < 2`：视作 `todo list`（即子命令默认为 `list`）。
   - 否则子命令为 `argv[1].as_str()`，参数为 `argv[2..]`。
4. **子命令实现**（每个子命令内：`load_todos()?` 得到 `Vec<Todo>`，`InMemoryStore::from_todos` + `TodoList::with_store` 得到 `list`，执行操作，需要时 `save_todos(&list)?`，输出到 `stdout`/`stderr`）：
   - **list**：`list.list()`（或 `list_with_options` 若需要），逐行打印（格式可简化：`id. title [done]`）；可选：若参数含 `--json` 则输出 JSON 到 stdout。
   - **add**：标题 = `argv[2..].join(" ")` 或首个参数；trim 后若空返回参数错误；`list.create(title)?`；`save_todos(&list)?`；打印已创建 id。
   - **show**：`argv[2]` 解析为 id（u64）；`TodoId::from_raw(id)`；`list.get(id).ok_or(TodoDataError)`；打印该条信息。
   - **update**：id + 可选 title（或仅 title）；`list.update_title(id, title)?` 或 `list.update(id, patch)?`（简单实现可只支持改 title）；`save_todos(&list)?`。
   - **complete**：id；`list.complete(id, false)?`；`save_todos(&list)?`。
   - **delete**：id；`list.delete(id)?`；`save_todos(&list)?`。
   - **search**：keyword = `argv[2..].join(" ")`；`list.search(&keyword)`；打印结果列表。
   - **stats**：`list.stats()`；打印 (open, completed, total) 或类似摘要。
5. **库错误映射**：`TodoError::InvalidInput` -> 参数错误；库内「id 不存在」类错误 -> 数据错误；写 stderr 并返回对应 `BuiltinError`。
6. **无参 `todo`**：已约定为 `todo list`，无需额外分支。

**验证**：

从 `examples/dev_shell` 目录运行（或从仓库根 `cargo run --manifest-path examples/dev_shell/Cargo.toml`；当前工作目录将用于 `.todo.json`）：

```bash
cd /home/ars/richard/2026/pvm_2/xtask_todo/examples/dev_shell
cargo run
```

在 REPL 中执行：

- `todo`
- `todo add "first task"`
- `todo list`
- `todo complete 1`
- `todo list`
- `todo stats`
- `exit`
```

再用 `cargo xtask todo list` 查看，应看到同一份数据（含 "first task" 且 id 1 为已完成）。

---

## Task 4: 更新 help 与 README

**文件**：`examples/dev_shell/src/command.rs`、`examples/dev_shell/README.md`

**步骤**：

1. **help**：在 `run_builtin_core` 的 `"help"` 分支中增加一行（或数行），例如：
   - `todo [list|add|show|update|complete|delete|search|stats] ...` — 待办列表（与项目根 `cargo xtask todo` 共用 `.todo.json`）。
2. **README**：在「Built-in commands」表格后或「Usage example」前增加 **「Todo 待办」** 小节：
   - 说明：在 shell 内输入 `todo`、`todo list`、`todo add "标题"` 等，与在项目根执行 `cargo xtask todo` 共用同一 `.todo.json`（宿主当前工作目录）。
   - 示例会话：先 `todo add "某任务"`，再 `todo list`，再 `todo complete 1`，再 `todo list` 或 `todo stats`。
   - 注明：完整能力（如 export/import）请使用 `cargo xtask todo`。

**验证**：在 REPL 输入 `help` 应包含 todo；README 在本地预览或阅读时语义完整、无错别字。

---

## Task 5: 可选 — 列表输出格式与 list --json

**范围**：设计允许「可选支持 `todo list --json`」。若首期不实现，可跳过本任务。

**步骤**：在 `todo list` 分支中，若 `argv` 含 `--json`，则将 `list.list()` 序列化为 JSON 写入 stdout（格式可与 xtask 一致或简化）；否则保持人类可读的逐行输出。

**验证**：`todo list --json` 输出合法 JSON；`todo list` 仍为可读文本。

---

## 实现顺序与检查点

| 顺序 | 任务 | 检查点 |
|------|------|--------|
| 1 | Task 1 依赖 | 在 `examples/dev_shell` 下 `cargo build` 通过 |
| 2 | Task 2 todo_io | 编译通过；可与 Task 3 联调 |
| 3 | Task 3 todo 内置 | REPL 中 todo/list/add/complete 与 xtask 共用 .todo.json |
| 4 | Task 4 help + README | help 含 todo；README 有「Todo 待办」与示例 |
| 5 | Task 5（可选） | `todo list --json` 可用 |

不实现：export、import、init-ai、--dry-run；与设计说明一致。

---

*计划编写完成；可按 Task 1 → 2 → 3 → 4（→ 5）顺序实现。*
