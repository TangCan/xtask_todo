---
story: 2-2-release-config-and-size-startup-optimization
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles: []
---

# ATDD Checklist — Story 2.2

## Testability assessment

这是 Cargo release profile 的测量驱动优化 Story。验收依赖同一基准命令产生的 before/after JSON、workspace 构建矩阵和既有 CLI 回归，不应把性能断言写成不稳定的硬编码绝对时间。

## Red-phase scenarios

- release profile 配置只影响 release，不改变 debug/test 构建。
- before/after JSON 使用同一 schema、目标排序、运行次数和实际 commit 元数据。
- 至少一个目标的 size 或冷启动 p50 有可重复改善；无收益候选不进入最终配置。
- release/debug build、nextest、doctest、clippy、rustdoc 和 help smoke 均通过。
- 优化文档记录 strip/LTO/codegen-units/opt-level 的候选、结果和调试取舍。

## Priority

构建兼容与无行为回归为 P0；实测收益和可追溯报告为 P0；跨平台数值比较为 P1。
