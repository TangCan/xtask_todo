# 头脑风暴：将 xtask-todo-lib 覆盖率提升到 95% 以上

## 现状

- 包内既有**库**（list、store、repeat 等）也有 **cargo-devshell** 二进制。
- `cargo test -p xtask-todo-lib` 只运行**库的测试**；测试二进制只链接库，**不会执行** binary 的代码。
- 因此 binary 下所有代码覆盖率为 0%，整体约 21.77%。

## 方案对比

| 方案 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 排除 binary** | `--exclude-files "crates/todo/src/bin/*"` | 实现简单，库可达 95%+ | 已明确不允许忽略源码 |
| **B. 集成测试启动 binary** | 在 tests/ 里 `Command` 启动 cargo-devshell，传 stdin 断言 stdout | 不挪代码 | Tarpaulin 通常只统计当前进程，子进程覆盖可能不计入 |
| **C. 将 binary 逻辑迁入 lib** | 把 parser/serialization/vfs/command/repl 等迁到 `src/devshell/`，binary 只保留薄封装并调用 `devshell::run_main()` | 同份代码被测试二进制链接并执行，覆盖率可靠 | 需要一次重构和导入修正 |

## 选定方案：C（逻辑迁入 lib）

- 不排除任何源码，只是把“同一份源码”放到 lib 下，由测试二进制链接并执行。
- 在 lib 内为 parser、vfs、serialization、command、repl 等加单元/集成式测试，即可把整包覆盖率拉到 95%+。

## 实施步骤

1. 在 lib 中新增 `devshell` 模块，将 `src/bin/cargo_devshell/` 下除 `main.rs` 外的模块迁入 `src/devshell/`，并统一为 `super::` / `crate::` 导入。
2. 在 devshell 中实现 `run_main()`，把当前 main 的“解析参数 → 加载 VFS → 运行 REPL”逻辑放进去；binary 的 main 只调用 `xtask_todo_lib::devshell::run_main()`。
3. 为 parser、vfs、serialization、command、repl 增加单元测试（含错误分支）；用 mock  stdin/stdout 测试 REPL 与命令执行路径。
4. 运行 `cargo tarpaulin -p xtask-todo-lib` 验证整体覆盖率。

**当前结果**：已将覆盖率从 21.77% 提升到 **82.94%**（devshell 逻辑迁入 lib 后由 83 个单元/集成测试覆盖）。剩余未覆盖主要集中在：repl 的 TTY 分支（依赖 rustyline 交互）、completion 的 Completer 实现、main 的 usage 错误路径等，若要冲 95% 需再补大量分支测试或对部分难以测试的代码做结构化/ mock。
