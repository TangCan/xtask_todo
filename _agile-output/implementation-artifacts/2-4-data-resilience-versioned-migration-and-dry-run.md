---
title: '数据容错、版本化迁移与 dry-run 一致性'
story_key: '2-4-数据容错-版本化迁移与-dry-run-一致性'
epic: 2
story: 4
status: done
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../implementation-artifacts/1-3-cli-snapshots-and-data-properties.md'
  - '../implementation-artifacts/2-3-typed-errors-and-cli-context.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 2.4: 数据容错、版本化迁移与 dry-run 一致性

## Story

作为待办数据使用者，我希望数据文件损坏或格式升级时得到明确、可恢复的行为，以便避免数据被静默丢失或误判为空。

## Acceptance Criteria

1. 不存在 `.todo.json` 仍按既有约定视为空数据集，不创建意外文件。
2. JSON 损坏或结构不可解析时返回数据错误码 3，消息包含文件路径和解析原因，不静默当作空列表。
3. 旧版本/缺少新增字段可通过默认值或显式迁移读取，已有待办不丢失；未来不支持版本拒绝读取且不覆盖原文件。
4. 成功修改类 `--dry-run` 展示将执行的变化，不写 `.todo.json`、备份或迁移文件；校验失败与正常执行使用相同错误语义。
5. 迁移/保存采用临时文件或备份保护，失败时原始文件可恢复；既有 JSON 字段和 CLI 成功协议保持兼容。

## Tasks / Subtasks

- [x] 定义数据 schema/version envelope 与向后兼容读取规则，保留 legacy array 读取。
- [x] 将 JSON parse/结构错误改为带路径上下文的数据错误码 3，覆盖 todo、import、export 相关路径。
- [x] 添加未来版本拒绝、旧版本默认值/迁移和原文件保护逻辑。
- [x] 统一 dry-run 与正常执行的解析/参数/业务校验，确保成功 dry-run 不产生任何持久化副作用。
- [x] 添加损坏、迁移、版本和 dry-run 回归测试，并完成 fmt、nextest、doctest、clippy、rustdoc。

## Developer Context

- 不改变 `TodoJsonError {status,error:{code,message}}` 结构或退出码约定；错误码 3 表示数据/业务失败。
- 保持现有 legacy `.todo.json` 数组可读；新 envelope 应显式版本且不覆盖用户原文件直到成功保存。
- 使用临时目录、cwd guard 和文件快照验证“失败不写入”；不依赖 HOME、Lima、网络。
- Story 2.3 已提供 `ContextError`/错误分类，复用而不是重新拼接错误字符串。

## Testing Requirements

- 缺失文件为空、损坏 JSON、错误结构、未来版本、legacy 数组和新 envelope。
- 成功/失败 dry-run 的文件 hash、备份和迁移产物不变。
- `cargo nextest run --workspace --all-features`、doctest、clippy、rustdoc、fmt。
