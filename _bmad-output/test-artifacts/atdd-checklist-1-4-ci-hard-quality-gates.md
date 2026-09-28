---
story: 1-4-ci-hard-quality-gates
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles: []
---

# ATDD Checklist — Story 1.4

## Testability assessment

这是 CI 配置与供应链门禁 Story，主要可观测面是 workflow YAML、门禁命令和生成的覆盖率工件，而非运行时业务 API。测试以静态 YAML 检查、本地等价命令和配置工具为主，避免把 GitHub runner 状态硬编码进 Rust 测试。

## Red-phase scenarios

- CI 定义 Linux、macOS、Windows 三类 runner，并在每个平台运行 nextest 与独立 doctest。
- 格式、clippy、覆盖率阈值、cargo-deny 与 cargo-audit 任一失败都使对应 job 失败。
- 覆盖率命令产生可上传的 JSON/摘要工件，并使用文档声明的阈值。
- 供应链配置允许项目当前依赖通过检查，同时保留许可证、来源和 RustSec 失败策略。
- workflow 不依赖 tarpaulin、Linux-only ptrace 或仅在 bash 可用的路径拼接。

## Priority

CI 门禁场景为 P0；无 API、浏览器或用户交互测试。
