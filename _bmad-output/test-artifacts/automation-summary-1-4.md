---
story: 1-4-ci-hard-quality-gates
date: 2026-09-28
---

# 自动化测试总结 — Story 1.4

## 覆盖范围

- `.github/workflows/ci.yml` 覆盖 Linux/macOS/Windows 与 amd64/arm64 六种 runner 组合。
- CI 显式执行格式、workspace build、全特性 nextest、全特性 doctest、rustdoc、clippy、两个 crate 的 llvm-cov 95% 行阈值、coverage JSON 工件、cargo-deny 和 cargo-audit。
- `deny.toml` 固定许可证、来源、依赖重复版本和安全公告策略；workspace path 依赖的版本限制保留为已记录例外。

## 本地验证结果

- 两条 `cargo llvm-cov --fail-under-lines 95` 命令通过。
- `cargo deny check`、`cargo audit`、`cargo fmt --all -- --check` 和 CI YAML 六矩阵静态校验通过。
- 既有 Story 1.3 全量 nextest/doctest/clippy 回归已通过；本 Story 仅调整 CI/供应链配置和文档，不改变运行时逻辑。
