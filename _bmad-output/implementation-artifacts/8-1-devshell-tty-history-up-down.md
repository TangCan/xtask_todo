# Story 8.1：TTY 上下箭头历史浏览与复用

Status: done

## Story

作为一名频繁使用 `cargo devshell` 的开发者，  
我希望在交互会话中通过 **↑/↓** 浏览并复用已执行命令，  
以便快速重放常用命令而无需重复输入（**FR35**）。

## 映射需求

- **FR35**：TTY 交互会话支持通过 **↑/↓** 浏览并复用当前会话命令历史；复用执行语义与手动输入一致。
- **关联 NFR**：**NFR-P1**（交互体验需保持可用、响应不退化）。
- **边界**：非 TTY / 脚本执行路径行为不变，不引入新的非交互副作用。

## Acceptance Criteria

1. **Given** 进入 `cargo devshell` 的 TTY 交互会话并已执行多条命令  
   **When** 按 **↑/↓**  
   **Then** 输入行在历史命令间切换（不自动执行），回显顺序符合最近历史语义（**FR35**）。

2. **Given** 通过 **↑/↓** 选中历史命令  
   **When** 按 Enter  
   **Then** 解析与执行结果与手动键入该命令一致（同样的 parser/dispatcher 路径）（**FR35**）。

3. **Given** 非 TTY 或脚本模式（`.dsh`）  
   **When** 执行 devshell  
   **Then** 不依赖上下箭头历史交互能力，现有脚本行为与退出码不变（**FR35** 边界）。

4. **Given** 当前已有 TTY 补全（FR19）  
   **When** 新增历史能力后进行回归  
   **Then** 命令/路径补全不回退，二者可共存。

## Tasks / Subtasks

- [x] 在 `crates/todo/src/devshell/repl` 侧确认/实现历史记录启用策略（仅 TTY 会话）。
- [x] 保证历史回放走现有命令解析与分派链路，不新增旁路执行逻辑。
- [x] 增加/更新测试：
  - [x] TTY 历史浏览与复用（通过 `should_add_history_entry` 规则测试覆盖空行/重复行过滤与新增条件）。
  - [x] 非 TTY / 脚本路径不受影响（沿用既有 non-TTY/脚本测试并全量回归）。
  - [x] FR19 补全回归 smoke（全量 `xtask-todo-lib` 测试包含 completion/repl 用例）。
- [x] 文档最小更新（如 `docs/development-guide.md` 或相关 devshell 文档）说明历史能力与边界。

## Dev Notes

### 代码定位建议

- `crates/todo/src/devshell/repl.rs`
- `crates/todo/src/devshell/completion/*`
- `crates/todo/src/devshell/tests/*`

### 设计约束

- 历史记录默认以**会话内内存**为主；是否持久化到会话文件不在本故事强制范围。
- 不改变 `.todo.json`、`--json`、退出码契约。
- 不新增网络/VM 依赖。

### 测试建议

- 优先沿用现有 devshell 测试风格（`crates/todo/src/devshell/tests/`）。
- 回归命令：
  - `cargo test -p xtask-todo-lib -- --test-threads=1`

## 参考资料

- [Source: `_bmad-output/planning-artifacts/prd.md` — FR35]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — Devshell FR14–FR19、FR35 映射]
- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 8 Story 8.1]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo fmt --all`
- `cargo test -p xtask-todo-lib -- --test-threads=1`

### Completion Notes List

- 在 `run_tty` 中接入 `rustyline` 历史入栈：成功读取的非空命令会加入会话历史，连续重复命令去重，避免无意义重复条目。
- 历史命令复用后仍走既有 `process_line -> parser -> execute_pipeline`，未引入旁路执行。
- 新增 `should_add_history_entry` 及单元测试，覆盖空行/重复行过滤规则。
- 补充 `docs/development-guide.md`：说明 `cargo devshell` 在 TTY 下支持 ↑/↓ 历史浏览，非 TTY 不依赖该能力。

### File List

- `crates/todo/src/devshell/repl/mod.rs`
- `crates/todo/src/devshell/repl/tests.rs`
- `docs/development-guide.md`
- `_bmad-output/implementation-artifacts/8-1-devshell-tty-history-up-down.md`

### Review Findings

- [x] 审查通过：未发现会阻塞合并的缺陷；历史回放沿用既有解析执行链路，且非 TTY 路径未改变。

## Change Log

- **2026-03-26**：Story 8.1 实现完成并通过回归；状态从 `review` 更新为 `done`。
