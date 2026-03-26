# 开发与运维（棕地摘要）

**来源：** `README.md`、`docs/requirements.md` §7、`.github/workflows/ci.yml`、`.githooks/pre-commit` 的 Quick Scan 汇总。

## 前置条件

- **Rust** toolchain（`rustup`）；仓库使用 **edition 2021**（见各 `Cargo.toml`）。  
- 可选：Windows 交叉检查需安装 **`x86_64-pc-windows-msvc`** target（pre-commit / acceptance 中可能 SKIP）。  
- **GitHub CLI `gh`**：仅在使用 `cargo xtask gh log` / GHCR 相关命令时需要。

## 常用命令

```bash
# 格式化 / 静态检查 / 测试（与 CI 精神一致，细节以 xtask 与 CI 为准）
cargo xtask fmt
cargo xtask clippy
cargo test --workspace
# 或按仓库 README 使用 cargo xtask test 等封装

# 本地提交前钩子（与 CI 对齐说明见 requirements §7.2）
cargo xtask git pre-commit
# 或：git config core.hooksPath .githooks 后 git commit 触发

# 一键验收（生成报告，见 acceptance.md）
cargo xtask acceptance
```

## CI（`.github/workflows/ci.yml`）

典型步骤：`cargo fmt --check` → `cargo build` → `cargo test` → `cargo clippy` → `cargo doc`（含 `RUSTDOCFLAGS=-D warnings` 等，以仓库内 workflow 为准）。**MSVC 交叉检查**主要出现在本地 pre-commit/acceptance，而非所有 CI job 重复。

## 发布

- 权威流程：**`docs/publishing.md`**。  
- 辅助命令：**`cargo xtask publish`**（支持 `--dry-run` 等，见实现与文档）。

## 风险与注意事项

- **临时目录**：仓库根下若存在 `xtask_*_fail_*` 等目录，多为测试/失败产物，**不宜**当作正式源码树依赖。  
- **Lint**：根 `Cargo.toml` 说明虚拟 workspace 不在根设 `[lints]`；各 crate 自行定义 clippy 策略。
- **Devshell TTY 历史**：交互式 `cargo devshell` 支持用 **↑/↓** 浏览当前会话历史命令；脚本/非 TTY 模式不依赖此能力。
