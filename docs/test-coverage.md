# 测试覆盖率（Test Coverage）

覆盖率工具为 [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)。目标与 **[requirements.md](./requirements.md)**、**[design.md](./design.md)** 一致：可测代码尽量覆盖；与需求追溯见 **[test-cases.md](./test-cases.md)**。

## 目标：各 crate ≥95%

| Crate | 目标 | 常用命令 |
|--------|------|----------|
| **xtask-todo-lib** | **≥95%** | `cargo xtask coverage`（与 `xtask/src/coverage.rs` 中 `--exclude-files` 一致） |
| **xtask** | **≥95%** | `cargo xtask coverage` 或 `cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" --exclude-files "crates/todo/*" -- --test-threads=1 --include-ignored` |

**说明**

- **xtask-todo-lib**：排除项用于聚焦可稳定测的库代码（`cargo-devshell` 入口、REPL、脚本、VM/Lima、宿主 sandbox、`host_text`、补全等）；核心 todo/VFS/parser/sandbox 与 devshell 集成测试覆盖其余部分；精确列表见 **`xtask/src/coverage.rs`**。
- **β / `guest_fs`**：`cargo test -p xtask-todo-lib --features beta-vm`；**`crates/devshell-vm`**：`cargo test -p devshell-vm`。
- **xtask**：**`main.rs`** 仅薄入口，排除后分母反映 **`lib.rs`** 可测逻辑；集成测试 **`xtask_run_exits_success`** 验证二进制入口。

## 运行

```bash
cargo install cargo-tarpaulin   # 一次性

cargo xtask coverage            # 推荐：工作区摘要

cargo tarpaulin -p xtask-todo-lib
cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" -- --test-threads=1
cargo tarpaulin --exclude-files "xtask/src/main.rs" -- --test-threads=1
```

## 注意

- 会改 **cwd** 的 xtask 测试请 **`--test-threads=1`**，避免竞态。
- **`xtask::run()`** 经 **`argh::from_env()`**，主要由集成测试覆盖。
