# 开发与运维（棕地摘要）

**来源：** `README.md`、`docs/requirements.md` §7、`.github/workflows/ci.yml`、`.githooks/pre-commit` 的 Quick Scan 汇总。

## 前置条件

- **Rust** toolchain（`rustup`）；仓库使用 **edition 2021**（见各 `Cargo.toml`）。  
- 可选：Windows 交叉检查需安装 **`x86_64-pc-windows-msvc`** target（pre-commit / acceptance 中可能 SKIP）。  
- **GitHub CLI `gh`**：仅在使用 `cargo xtask gh log` / GHCR 相关命令时需要。
- **cargo-nextest**：运行非 doctest workspace 测试时需要；安装命令为 `cargo install cargo-nextest --locked`。doctest 仍使用 `cargo test --doc`。

## 常用命令

```bash
# 格式化 / 静态检查 / 测试（与 CI 精神一致，细节以 xtask 与 CI 为准）
cargo xtask fmt
cargo xtask clippy
cargo nextest run
cargo test --doc
# 或按仓库 README 使用 cargo xtask test 等封装

# 本地提交前钩子（与 CI 对齐说明见 requirements §7.2）
cargo xtask git pre-commit
# 或：git config core.hooksPath .githooks 后 git commit 触发

# 一键验收（生成报告，见 acceptance.md）
cargo xtask acceptance
```

## CI（`.github/workflows/ci.yml`）

CI 在 Linux/macOS/Windows 的 amd64 与 arm64 runner 上执行同一组硬门禁：`cargo fmt --all -- --check`、workspace build、`cargo nextest run --workspace --all-features`、独立的 `cargo test --doc --workspace --all-features`、`RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、两个核心 crate 的 llvm-cov 95% 行覆盖率阈值、`cargo deny check` 与 `cargo audit`。工具版本固定在 workflow 的 install-action 步骤；Rust stable 按官方 runner 工具链维护。nextest 不包含 doctest，因此 doctest 步骤不可删除或合并。

覆盖率 JSON 工件按 runner 上传，便于机器消费和比较。覆盖率阈值、排除意图与调整流程见 [test-coverage.md](test-coverage.md)；供应链策略见仓库根目录的 [`deny.toml`](../deny.toml)。本地 pre-commit/acceptance 仍负责 MSVC 目标交叉检查，CI 不依赖本地 hook。

## 发布

- 权威流程：**`docs/publishing.md`**。  
- 辅助命令：**`cargo xtask publish`**（支持 `--dry-run` 等，见实现与文档）。

## 风险与注意事项

- **临时目录**：仓库根下若存在 `xtask_*_fail_*` 等目录，多为测试/失败产物，**不宜**当作正式源码树依赖。  
- **Lint**：根 `Cargo.toml` 说明虚拟 workspace 不在根设 `[lints]`；各 crate 自行定义 clippy 策略。
- **Devshell TTY 历史**：交互式 `cargo devshell` 支持用 **↑/↓** 浏览当前会话历史命令；脚本/非 TTY 模式不依赖此能力。
