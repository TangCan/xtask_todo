# dev_shell 中展示 Todo 功能 — 设计说明

**日期**：2026-03-10  
**范围**：在 `examples/dev_shell` 中通过内置命令 `todo` 展示 xtask-todo 能力；文档与入口并存。  
**状态**：设计已获用户逐节通过（§1 依赖与数据、§2 内置命令形态与子命令集、§3 实现边界与文档）。

---

## 1. 设计目标与方案选择

- **展示形式**：文档（README） + dev_shell 内 **todo 内置命令**（方案 B）。
- **集成方式**：内置命令直接依赖并调用 `xtask-todo-lib`（path 依赖），不通过子进程调用 `cargo xtask todo`。
- **数据**：读写 **宿主当前工作目录** 下的 `.todo.json`，与在项目根运行 `cargo xtask todo` 时共用同一数据文件。

---

## 2. 依赖与数据（§1 已通过）

### 2.1 依赖

- 在 `examples/dev_shell/Cargo.toml` 中增加对 workspace 内 todo 库的 path 依赖，例如：
  ```toml
  [dependencies]
  todo = { path = "../../crates/todo" }
  ```
  若将 dev_shell 纳入 workspace，则路径按 workspace 布局调整（如 `path = "../crates/todo"`）。

### 2.2 数据路径

- `todo` 内置命令使用 **`std::env::current_dir()`** 下的 `.todo.json`。
- 不在 dev_shell 的虚拟 FS 中读写；数据始终在宿主目录，与 `cargo xtask todo` 一致。

### 2.3 文档约定

- README 中增加「Todo 待办」小节：说明在 shell 内输入 `todo` / `todo list` / `todo add "标题"` 等与在项目根执行 `cargo xtask todo` 共用同一 `.todo.json`，并给出简短示例会话（如先 `todo add "某任务"`，再 `todo list`）。

---

## 3. 内置命令形态与子命令集（§2 已通过）

### 3.1 命令名与无参行为

- REPL 中输入 **`todo`** 或 **`todo <子命令> [参数...]`**。
- 无参数时：等价于 **`todo list`**（或打印简短用法，二者择一，实现时定）。

### 3.2 首期子命令

| 子命令   | 说明           |
|----------|----------------|
| `list`   | 列出待办       |
| `add`    | 添加待办       |
| `show`   | 查看单条       |
| `update` | 更新单条       |
| `complete` | 标记完成     |
| `delete` | 删除           |
| `search` | 搜索           |
| `stats`  | 统计           |

- 暂不实现：`export`、`import`、`init-ai`；文档中说明「完整能力请使用 `cargo xtask todo`」。

### 3.3 参数解析

- **简单解析**：以空格分词；`add` 的标题取第一个参数或整行去掉 `todo add` 后的部分（支持引号时再去引号）。
- **list**：首期可不支持或仅支持 1～2 个常用选项（如 `--status`/`--sort`）；可选支持 `todo list --json` 输出 JSON 到 stdout。

---

## 4. 实现边界与文档（§3 已通过）

### 4.1 不实现项

- 不实现 `cargo xtask todo` 的：`--dry-run`、`init-ai`、export/import。
- 错误信息可简化，但需能区分：**成功 / 参数错误 / 数据错误**（返回码或等效方式），便于脚本与后续扩展。

### 4.2 文档

- **README**：「Todo 待办」小节 — 命令说明 + 示例会话（见 §2.3）。
- **本设计说明**：`docs/superpowers/specs/2026-03-10-dev-shell-todo-design.md`，记录方案 B 与依赖、数据、子命令、边界，供实现与后续修改参考。

---

## 5. 实现时要点摘要

1. **Cargo**：`examples/dev_shell/Cargo.toml` 增加 `todo = { path = "../../crates/todo" }`（或按 workspace 调整）。
2. **REPL/command**：在 builtin 分发中识别 `todo`，解析子命令与参数，调用 `todo` crate 的 API（如 `TodoList::load`/`create`/`list`/`complete` 等），将结果输出到 stdout；错误时输出错误信息并设置适当退出/返回码。
3. **README**：新增「Todo 待办」小节与示例会话。
4. **不写**：export/import、init-ai、--dry-run；保持与本节设计一致。

---

*设计三节均已通过；可据此进入 writing-plans 编写实现计划，再开始编码。*
