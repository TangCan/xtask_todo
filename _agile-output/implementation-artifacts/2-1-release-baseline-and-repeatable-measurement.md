---
title: 'Release 基准与可重复测量'
story_key: '2-1-release-基准与可重复测量'
epic: 2
story: 1
status: done
created: '2026-09-28'
source: '../planning-artifacts/epics-hardening.md'
context:
  - '../implementation-artifacts/epic-1-context.md'
  - '../implementation-artifacts/1-4-ci-hard-quality-gates.md'
  - '../specs/spec-xtask-todo-hardening/quality-tooling.md'
  - '../project-context.md'
---

# Story 2.1: Release 基准与可重复测量

## Story

作为项目维护者，
我希望在修改 release 配置前记录二进制体积和冷启动基准，
以便后续优化结果有客观依据。

## Acceptance Criteria

1. 能用一个仓库内命令构建 release 目标并测量每个目标二进制的文件大小与冷启动耗时；输出包含 Rust/Cargo 版本、目标平台、commit、构建参数和测量次数。
2. 基准输出使用稳定、机器可读的 JSON schema；相同环境重复运行时字段顺序/目标排序稳定，并记录足以解释数值差异的元数据。
3. 基准文件存放在约定的文档/工件目录，明确区分实测结果与后续优化结果；不使用外部研究报告中的二手数字。
4. 测量命令不会修改源代码、Cargo 配置或用户数据；失败的构建、缺失目标或无法启动的二进制会返回非零并说明目标。

## Tasks / Subtasks

- [x] 增加 `cargo xtask release-baseline` 命令，支持输出路径、运行次数和 release 构建参数。
- [x] 发现并构建 workspace release binary targets，记录字节大小和每次冷启动耗时，并计算 min/p50/p95。
- [x] 记录 schema 版本、UTC 时间、git commit、rustc/cargo 版本、host/target、profile、命令参数和目标清单。
- [x] 添加稳定 JSON schema/文档与一份当前环境的实测 baseline 工件，说明不可跨平台直接比较。
- [x] 添加单元/集成测试覆盖排序、schema、失败传播和输出路径隔离，并完成 fmt、clippy、nextest、doctest 回归。

## Developer Context

### Scope and Boundaries

- 这是测量与记录工具，不调整 release profile，不宣称优化收益；Story 2.2 才能修改 strip/LTO/codegen-units/opt-level。
- 冷启动指标应通过独立子进程执行目标 binary（例如 `--help`），并明确命令、次数和统计方法；禁止在测试中调用真实长时间业务操作。
- 默认输出可以指向 `docs/benchmarks/release-baseline.json`，但命令必须允许临时路径，避免测试污染已提交基准。
- 不修改 `.todo.json`、用户 HOME 或 CI 供应链配置。

### Existing Patterns to Preserve

- `xtask` 使用纯 Rust dev 命令架构，子命令在 `xtask/src/` 中实现并由现有参数解析入口 dispatch。
- workspace 目标包含 `todo`、`cargo-devshell`、`devshell-vm` 与 `xtask`；目标发现应来自 `cargo metadata` 或 Cargo target metadata，不复制易过期清单。
- JSON 输出遵循现有 `serde_json` 依赖和稳定字段命名；文件写入采用临时路径/原子替换或明确失败，不覆盖已有结果。

### Likely Files

- `xtask/src/benchmark.rs` 或等价的新模块
- `xtask/src/main.rs`/dispatch 参数入口
- `xtask/tests/` 基准命令测试
- `docs/benchmarks/release-baseline.json`、`docs/release-benchmark.md`

### Testing Requirements

- `cargo xtask release-baseline --help`
- 使用临时输出路径运行一次小次数基准并解析 JSON。
- 覆盖空/非法次数、不可写输出和子进程失败路径。
- `cargo fmt --all -- --check`
- `cargo nextest run`
- `cargo test --doc --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Previous Story Intelligence

- Story 1.4 已将质量工具版本和平台矩阵固定在 CI；本 Story 的元数据应记录实际 runner/host，而不是假设所有平台一致。
- 项目已有 `xtask` 的命令错误/退出码测试模式，应复用临时目录和 `assert_cmd`，避免并行测试共享 cwd。

## Completion Notes

基于 Epic 2 要求、当前 xtask 纯 Rust 架构和 CI 工具链约束创建；本故事只建立可复现测量基线，不提前实施 release 优化。
