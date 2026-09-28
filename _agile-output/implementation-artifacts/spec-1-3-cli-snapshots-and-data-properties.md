---
title: 'CLI 快照与数据层属性测试'
type: 'feature'
created: '2026-09-28'
status: 'done'
route: 'dispatch'
baseline_commit: 'a040c79'
context:
  - '/nvme_data2/richard/xtask_todo/_agile-output/implementation-artifacts/epic-1-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/project-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md'
---

<frozen-after-approval reason="story intent derived from approved hardening Epic 1">

## Intent

Add real-binary CLI snapshots and data-layer property tests for list filtering,
sorting, and stable serialization boundaries without changing product behavior.

## Boundaries & Constraints

- Preserve JSON output, exit code 2 for invalid list parameters, and the
  `.todo.json` format.
- Avoid duplicating the existing `list_json.rs` assertions.
- Keep generated tests deterministic and isolated under nextest.

</frozen-after-approval>

## Acceptance Criteria

- trycmd covers six list dimensions with positive and no-match cases.
- proptest checks list filter invariants; insta covers stable representative output.
- Invalid list inputs do not mutate an existing data file.

## Implementation Notes

- 通过固定 `.in/.out` 文件系统快照隔离 trycmd 场景；测试只增加验证，不修改产品实现。
- 属性测试覆盖状态、标签、优先级和日期边界；已有数据文件的非法日期不变性由 CLI 集成测试验证。

## Review Findings
