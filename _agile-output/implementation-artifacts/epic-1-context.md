# Epic 1 Context: 测试与合并质量信号

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

让维护者能稳定、快速且可重复地运行 workspace 测试，并把测试、格式、lint、覆盖率和供应链检查转化为可靠的合并质量信号。Epic 1 优先建立测试基础设施，后续 Story 在此基础上扩展覆盖率和 CI 门禁。

## Stories

- Story 1.1: 测试运行器与测试工具链升级
- Story 1.2: 跨平台覆盖率能力
- Story 1.3: CLI 快照与数据层属性测试
- Story 1.4: CI 硬质量门禁

## Requirements & Constraints

- 使用 cargo-nextest 运行非 doctest 测试；doctest 由 `cargo test --doc` 单独运行。
- 覆盖率使用跨平台 source-based 工具链，不使用 Linux 专属 ptrace 方案。
- CI 应逐步形成 fmt、clippy `-D warnings`、测试、覆盖率和供应链扫描硬门禁。
- 不删除既有测试断言，不改变 `--json` 输出契约、退出码语义或 `.todo.json` 格式。
- 保持 cargo-xtask 纯 Rust 架构，不引入 just/cargo-make。
- 测试必须可重复、隔离，并兼容 Linux、macOS、Windows MSVC 的 workspace 目标。

## Technical Decisions

- workspace 使用 Rust 2021、resolver 2；各成员 crate 管理自身 lint 配置。
- `xtask` 的 CLI/集成测试位于 `xtask/tests/` 和 `xtask/src/tests/`；领域数据层测试位于 `crates/todo/src/`。
- 测试工具作为对应 crate 的 dev-dependencies，不进入运行时依赖。
- 现有 cwd 锁、工作目录恢复、临时目录和环境变量清理是隔离边界，除非测试证据证明需要调整，否则保留。
- 后续 CLI 快照优先复用真实二进制和现有测试 fixtures，避免重复已有 `list_json.rs` 断言。

## Cross-Story Dependencies

- Story 1.1 是基础设施前置，但必须本身完成且可验证；Story 1.2 使用新的测试执行方式验证 coverage，Story 1.3 使用新增测试依赖，Story 1.4 将命令接入 CI。
- Story 1.1 不应提前实现 coverage、覆盖率阈值或完整 CI 门禁。
