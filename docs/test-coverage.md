# 测试覆盖率（Test Coverage）

覆盖率工具为 [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)，使用 Rust 的 source-based instrumentation。目标与 **[requirements.md](./requirements.md)**、**[design.md](./design.md)** 一致：可测代码尽量覆盖；与需求追溯见 **[test-cases.md](./test-cases.md)**。

## CI 门禁：各核心 crate ≥95%

| Crate | 目标 | 常用命令 |
|--------|------|----------|
| **xtask-todo-lib** | **≥95%** | CI 使用 `cargo llvm-cov --fail-under-lines 95`，排除意图与 `xtask/src/coverage.rs` 一致 |
| **xtask** | **≥95%** | CI 使用 `cargo llvm-cov --fail-under-lines 95`，排除意图与 `xtask/src/coverage.rs` 一致 |

`cargo xtask coverage` 仍用于本地生成两个 crate 的摘要；CI 工作流中的 `cargo llvm-cov` 在 Linux/macOS 上是硬门禁，任一核心 crate 低于 95% 或命令失败都会阻断对应平台 job。Windows 继续执行构建、测试、文档、Clippy、依赖和安全检查，但跳过覆盖率命令：当前 Windows runner 的 llvm-cov profile 生成不稳定，且 xtask 中的 Unix 专属测试路径会使跨平台汇总值失真。CI 同时将可生成的 `coverage/*.json` 作为机器可读工件上传。

### 阈值调整流程

95% 是当前团队基线。提高阈值或修改排除项时，必须同步更新 `.github/workflows/ci.yml`、本页说明以及 `xtask/src/coverage.rs` 的排除意图，并记录覆盖率变化原因。降低阈值需要维护者明确批准，并记录临时期限和恢复计划；不能只修改 CI 中的数字。

**说明**

- **xtask-todo-lib**：排除项用于聚焦可稳定测的库代码（`cargo-devshell` 入口、REPL、脚本、VM/Lima、宿主 sandbox、`host_text`、**`completion/*`**、**`workspace/*`**、**`command/dispatch/{builtin_impl,workspace}.rs`**、**`vfs/tree.rs`**、**`session_store.rs`** 等）；核心 todo/VFS/parser/sandbox 与 devshell 集成测试覆盖其余部分；精确列表见 **`xtask/src/coverage.rs`**。
- **β / `guest_fs`**：`cargo test -p xtask-todo-lib --features beta-vm`；**`crates/devshell-vm`**：`cargo test -p devshell-vm`（覆盖 **`exec`**、**`exec_timeout`**（**TC-D-VM-7**）、**`guest_fs`**、**`--devshell-vm-test-fail`**、TCP 子进程集成等；**Windows + Podman** 全链路不在 llvm-cov 摘要内，见 **[test-cases.md](./test-cases.md) TC-D-VM-4**）。
- **xtask**：**`main.rs`**、**`bin/todo.rs`**、**`lib.rs`** 为薄入口/顶层分发器（具体命令逻辑由各模块单测覆盖，`todo` 逻辑在 **`todo::run_standalone`** 中有单测）；**`lima_todo/*`**、**`gh.rs`**、**`ghcr.rs`**（HTTP 层）、**`acceptance/*`**（嵌套 `cargo test`）以及 **`release_baseline.rs`**（启动已构建二进制并采集宿主机相关耗时）在 llvm-cov 分母中排除，但仓库内仍有对应 **`#[cfg(test)]`**（如 **`ghcr::tests`** 解析 JSON、**`acceptance::tests`** 生成报告、**`lima_todo::tests`** 含 `cmd_lima_todo` smoke、release baseline 的纯函数/报告测试）。与 **`crates/todo/*`** 一并排除后，摘要 ≥95%；精确列表见 **`xtask/src/coverage.rs`**。

## 运行

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked

cargo xtask coverage            # 推荐：工作区摘要

cargo llvm-cov -p xtask-todo-lib --text
cargo llvm-cov -p xtask --text   # 若需与 CI 摘要一致，请使用 `cargo xtask coverage` 中的排除项
cargo llvm-cov --text --ignore-filename-regex 'xtask/src/main\.rs' -- --test-threads=1
```

CI 在 Linux 和 macOS 上运行 source-based llvm-cov，分别生成并上传 `coverage/xtask-todo-lib.json` 与 `coverage/xtask.json`；Windows 使用其余质量门禁，不运行当前不稳定的 llvm-cov coverage gate。不使用 cargo-tarpaulin 或 Linux-only ptrace 方案。

## 注意

- 会改 **cwd** 的 xtask 测试请 **`--test-threads=1`**，避免竞态。
- **`xtask::run()`** 经 **`argh::from_env()`**，主要由集成测试覆盖。
- **Pre-commit / Windows 交叉编译**：**`cargo xtask coverage`** 与 llvm-cov **不**替代 **`.githooks/pre-commit`** 中的 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**；后者用于保证 **MSVC** 目标可编译，见 **[requirements.md](./requirements.md) §7.2**、**[test-cases.md](./test-cases.md) TC-X-GIT-2 / TC-NF-5**。
