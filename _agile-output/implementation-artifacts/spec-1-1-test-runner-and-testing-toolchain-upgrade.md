---
title: '测试运行器与测试工具链升级'
type: 'chore'
created: '2026-09-28'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '36d25e3fad63312943124d2f994936108806fe4d'
context:
  - '/nvme_data2/richard/xtask_todo/_agile-output/implementation-artifacts/epic-1-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/project-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/specs/spec-xtask-todo-hardening/SPEC.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前 workspace 依赖默认 `cargo test`，测试隔离和执行反馈不足，且缺少为 CLI 快照和数据层属性测试准备的工具依赖。后续覆盖率和 CI 门禁无法在可靠的测试执行基础上落地。

**Approach:** 为 `xtask` 和 `xtask-todo-lib` 添加测试专用依赖，采用 cargo-nextest 运行非 doctest 测试，并保留 `cargo test --doc` 作为独立 doctest 检查。检查并修复已存在的 cwd、临时目录和环境变量隔离问题，但不改变业务契约。

## Boundaries & Constraints

**Always:**
- 保持 Rust 2021、workspace resolver 2 和现有 crate 边界。
- 不改变 `--json` 输出、退出码语义、`.todo.json` 数据格式或既有测试断言。
- 测试依赖只进入对应 crate 的 `[dev-dependencies]`。
- `cargo nextest run` 覆盖非 doctest workspace 测试；`cargo test --doc` 单独覆盖 doctest。
- 保留并验证 cwd 恢复、临时目录清理、环境变量清理和必要的测试锁。

**Never:**
- 不实现 cargo-llvm-cov、覆盖率阈值或完整 CI 门禁；这些属于后续 Story。
- 不删除或弱化现有测试，不以全局串行化掩盖共享状态问题。
- 不引入 just、cargo-make、富 TUI 或新的待办业务能力。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| MISSING_TOOL | cargo-nextest 未安装 | 实现文档提供明确安装/运行前置说明 | 不伪造测试通过；命令失败可定位 |
| DEV_DEPENDENCY_RESOLUTION | 新增 dev-dependencies | workspace 可解析并编译所有测试目标 | 依赖冲突或 feature 错误必须显式失败 |
| PARALLEL_TESTS | cwd/temp-dir/env 测试并行执行 | 测试结果稳定，无状态污染 | 竞态必须修复根因，不关闭测试 |
| DOCTEST_SEPARATION | nextest 不执行 doctest | `cargo test --doc` 独立通过 | doctest 失败阻断验证 |

</frozen-after-approval>

## Code Map

- `xtask/Cargo.toml` -- 添加 CLI 快照/断言/属性测试 dev-dependencies；不要改变运行时依赖。
- `crates/todo/Cargo.toml` -- 添加数据层 `insta`/`proptest` dev-dependencies；保持默认 `beta-vm` feature。
- `xtask/tests/` -- 现有真实二进制集成测试和 list 测试；保留并复用其测试辅助设施。
- `xtask/src/tests/` -- cwd 锁、临时工作区和命令测试模式；重点检查并行 nextest 下的隔离。
- `crates/todo/src/list/tests.rs`、`store.rs`、`model.rs` -- 后续 Story 的快照/属性测试目标，本 Story 不添加业务断言。
- `.github/workflows/ci.yml` -- 当前仍使用 `cargo test -- --test-threads=1`；本 Story 只记录未来 CI 切换所需的执行契约，不在本 Story 修改门禁。

## Tasks & Acceptance

**Execution:**
- [x] `xtask/Cargo.toml` -- 添加 `trycmd`、`assert_cmd`、`assert_fs`、`insta`、`proptest` dev-dependencies -- 为后续 CLI 快照和断言测试提供工具。
- [x] `crates/todo/Cargo.toml` -- 添加 `insta`、`proptest` dev-dependencies -- 为后续数据层快照和属性测试提供工具。
- [x] `xtask/src/tests/` 与相关测试辅助文件 -- 检查 cwd、临时目录和环境变量隔离 -- 确保 nextest 并行执行不共享可变状态。
- [x] `Cargo.lock` -- 记录依赖解析结果 -- 保持可重复构建。
- [x] 测试文档或 Story Implementation Notes -- 记录 nextest/doctest 分工和本地前置条件 -- 避免后续 CI 误用。

**Acceptance Criteria:**
- Given workspace 存在现有单元测试和集成测试，when 执行 `cargo nextest run`，then 所有非 doctest 测试通过，既有断言、JSON 输出和退出码语义保持不变。
- Given 测试目标需要新工具，when 执行 workspace test-target compile，then `xtask` 可解析 trycmd/assert_cmd/assert_fs/insta/proptest，`xtask-todo-lib` 可解析 insta/proptest。
- Given nextest 不执行 doctest，when 执行 `cargo test --doc`，then 所有 doctest 独立通过。
- Given 测试使用 cwd、临时目录或环境变量，when nextest 并行运行，then 不产生竞态、状态污染或 flaky 结果。

## Implementation Notes

- 官方 cargo-nextest 文档确认 `cargo nextest run` 是非 doctest 测试运行入口；doctest 必须单独执行。
- trycmd 适合批量 CLI snapshot，但其 fixture 和 list 六维矩阵留给 Story 1.3。
- Story 1.2 才迁移 `xtask/src/coverage.rs` 到 cargo-llvm-cov；本 Story 不碰 coverage 实现。
- 已添加测试专用依赖：xtask 使用 trycmd 1.2、assert_cmd 2.2、assert_fs 1.1、insta 1.48、proptest 1.11；todo 库使用 insta 1.48、proptest 1.11。
- 现有 cwd 锁、RestoreCwd、临时目录和环境变量清理均已检查，未发现需要为依赖接入而修改的隔离缺口。
- cargo-nextest 是 CI/本地工具，不作为运行时或 crate dev-dependency 写入 Cargo.toml。

## Design Notes

- 依赖版本应与当前稳定 Rust/现有 workspace 兼容；优先使用 Cargo 解析出的稳定版本，不手工锁定不必要的 transitive 版本。
- 如果现有测试在 nextest 下失败，修复具体共享状态或清理问题；不要简单把整个 suite 改为串行。

## Verification

**Commands:**
- `cargo check --workspace --all-targets` -- expected: all test-only dependencies resolve and test targets compile.
- `cargo nextest run` -- expected: all non-doctest workspace tests pass.
- `cargo test --doc` -- expected: all doctests pass independently.
- `cargo fmt -- --check` -- expected: no formatting drift.
- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings` -- expected: no warnings.

### Review Findings

- [x] [Review][Defer] CI 仍未切换到 nextest [`.github/workflows/ci.yml:31-32`] — deferred: 这是 Story 1.4「CI 硬质量门禁」的明确范围，当前 Story 仅建立本地工具链。
- [x] [Review][Reject] Cargo.lock 未出现在 diff [`.gitignore:2`] — rejected: 仓库既有策略明确忽略 Cargo.lock；本次 `cargo check` 已验证依赖可解析，强行纳入会改变项目既有版本控制约定。
- [x] [Review][Patch] 补充 cargo-nextest 安装和本地运行前置 [docs/development-guide.md] — fixed: 增加 `cargo install cargo-nextest --locked`、`cargo nextest run` 和独立 `cargo test --doc` 说明。
- [x] [Review][Patch] 统一 Story 完成状态和验证证据 [1-1-test-runner-and-testing-toolchain-upgrade.md] — fixed: 标记已完成任务并记录 471 个 nextest 测试通过、1 个跳过及 doctest 结果。

Rejected:

- false — Cargo.lock 缺失并非当前变更造成的问题；`.gitignore` 的既有策略要求不提交该文件。
