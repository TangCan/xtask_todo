---
title: '强类型错误与 CLI 错误上下文'
story_key: '2-3-强类型错误与-cli-错误上下文'
epic: 2
story: 3
status: review
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../implementation-artifacts/1-3-cli-snapshots-and-data-properties.md'
  - '../implementation-artifacts/2-2-release-config-and-size-startup-optimization.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 2.3: 强类型错误与 CLI 错误上下文

## Story

作为 CLI 和库的使用者，我希望错误信息具有明确类型、上下文和退出码，以便快速定位问题并让自动化程序可靠处理失败。

## Acceptance Criteria

1. 库层可分类错误使用可枚举强类型，至少区分输入、数据、存储和业务/未找到错误，并保留底层原因。
2. CLI 传播库层或外部命令失败时添加操作/路径/命令上下文，不丢失根因，不泄漏调试堆栈。
3. 非法参数返回退出码 2；数据损坏、存储失败或业务失败返回退出码 3；一般外部失败保持退出码 1。
4. `--json` 成功与失败均只输出既有 JSON 契约，不混入普通文本；人类模式保持可读上下文。
5. 现有成功行为、快照、数据格式和 dry-run 语义不回归。

## Tasks / Subtasks

- [x] 设计并实现 todo 库的分类错误类型与 source 链接，替换字符串/裸 boxed errors 的关键边界。
- [x] 为文件读取、解析、保存、导入和领域操作补充路径/操作上下文映射。
- [x] 统一 CLI 错误到退出码和 JSON body 的映射，确保 stdout/stderr 协议稳定。
- [x] 添加错误显示、source、退出码、JSON purity 和失败路径测试/快照。
- [x] 完成 fmt、nextest、doctest、clippy、rustdoc 和现有 CLI 回归。

## Developer Context

- 保持现有 `TodoCliError` 退出码：1 general、2 parameter、3 data；只补足类型和上下文。
- 不改变 `TodoJsonError {status,error:{code,message}}` 字段结构；错误上下文应进入 message。
- 不把 parse error、I/O error 静默当作空数据；Story 2.4 将进一步处理版本迁移和恢复策略。
- 优先从 `crates/todo/src/error.rs`、`xtask/src/todo/error.rs`、`xtask/src/todo/io.rs` 和 dispatch 边界着手。
- 错误测试必须使用临时目录和固定输入，不依赖 HOME、Lima、VM 或网络。

## Testing Requirements

- 分类错误 display/source 和领域错误映射。
- 缺失/损坏/不可写路径的退出码、上下文和 JSON purity。
- 非法参数退出码 2；not found/业务失败退出码 3；一般外部失败退出码 1。
- `cargo nextest run --workspace --all-features`、`cargo test --doc --workspace --all-features`、clippy、rustdoc、fmt。
