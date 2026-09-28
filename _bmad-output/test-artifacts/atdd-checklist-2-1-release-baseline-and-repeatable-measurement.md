---
story: 2-1-release-baseline-and-repeatable-measurement
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles: []
---

# ATDD Checklist — Story 2.1

## Testability assessment

这是 release 测量工具 Story。验收重点是稳定 JSON、元数据完整性、统计计算和失败传播；测试应使用临时输出和短运行次数，实际提交的 baseline 则通过同一命令生成。

## Red-phase scenarios

- 运行基准命令能发现并排序 workspace release binary targets。
- 输出包含 schema 版本、commit、工具链、平台、profile、运行次数和每个目标的 size/startup 统计。
- 重复运行时目标顺序、字段结构和统计定义稳定。
- 非法次数、不可写输出和目标启动失败返回非零且不留下半成品。
- 测试使用临时路径，不修改仓库 baseline 或 `.todo.json`。

## Priority

基准 schema 和失败安全为 P0；实际冷启动测量为 P1；无 API、浏览器测试。
