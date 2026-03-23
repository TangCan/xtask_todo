# 测试覆盖率（Test Coverage）

覆盖率工具为 [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)。目标与 **[requirements.md](./requirements.md)**、**[design.md](./design.md)** 一致：可测代码尽量覆盖；与需求追溯见 **[test-cases.md](./test-cases.md)**。

## 目标：各 crate ≥95%

| Crate | 目标 | 常用命令 |
|--------|------|----------|
| **xtask-todo-lib** | **≥95%** | `cargo xtask coverage`（与 `xtask/src/coverage.rs` 中 `--exclude-files` 一致） |
| **xtask** | **≥95%** | `cargo xtask coverage`（与 `xtask/src/coverage.rs` 中 xtask 的 `--exclude-files` 一致） |

**说明**

- **xtask-todo-lib**：排除项用于聚焦可稳定测的库代码（`cargo-devshell` 入口、REPL、脚本、VM/Lima、宿主 sandbox、`host_text`、**`completion/*`**、**`workspace/*`**、**`command/dispatch/{builtin_impl,workspace}.rs`**、**`vfs/tree.rs`**、**`session_store.rs`** 等）；核心 todo/VFS/parser/sandbox 与 devshell 集成测试覆盖其余部分；精确列表见 **`xtask/src/coverage.rs`**。
- **β / `guest_fs`**：`cargo test -p xtask-todo-lib --features beta-vm`；**`crates/devshell-vm`**：`cargo test -p devshell-vm`。
- **xtask**：**`main.rs`**、**`bin/todo.rs`** 为薄入口；**`lima_todo/*`** 依赖 Lima/VM；**`gh.rs`** 依赖 `gh` CLI；**`acceptance.rs`** 为验收/文档检查辅助（多在单独流程中跑）；与 **`crates/todo/*`** 一并排除后，报告率反映 **`lib`** 中可稳定单测的部分；精确列表见 **`xtask/src/coverage.rs`**。

## 运行

```bash
cargo install cargo-tarpaulin   # 一次性

cargo xtask coverage            # 推荐：工作区摘要

cargo tarpaulin -p xtask-todo-lib
cargo tarpaulin -p xtask   # 若需与 CI 摘要一致，请使用 `cargo xtask coverage` 中的排除项
cargo tarpaulin --exclude-files "xtask/src/main.rs" -- --test-threads=1
```

## 注意

- 会改 **cwd** 的 xtask 测试请 **`--test-threads=1`**，避免竞态。
- **`xtask::run()`** 经 **`argh::from_env()`**，主要由集成测试覆盖。
- **Pre-commit / Windows 交叉编译**：**`cargo xtask coverage`** 与 **tarpaulin** **不**替代 **`.githooks/pre-commit`** 中的 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**；后者用于保证 **MSVC** 目标可编译，见 **[requirements.md](./requirements.md) §7.2**、**[test-cases.md](./test-cases.md) TC-X-GIT-2 / TC-NF-5**。
