---
title: '测试基础设施升级'
type: 'chore'
created: '2026-09-24'
status: 'in-progress'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '36d25e3fad63312943124d2f994936108806fe4d'
context:
  - '_agile-output/specs/spec-xtask-todo-hardening/SPEC.md'
  - '_agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 测试仍用默认 `cargo test`（慢、无隔离、无内置覆盖率），现有 `coverage` 子命令用 cargo-tarpaulin（ptrace，Linux 限定），而 SPEC 约束要求 source-based 跨平台方案；`deferred-work.md` 第三条"list 过滤 E2E 覆盖不全"仍积压，缺少快照/属性测试工具箱。

**Approach:** 用 cargo-nextest 替换 `cargo test` 作测试运行器，把 `coverage` 子命令从 tarpaulin 重写为 cargo-llvm-cov；引入 trycmd/assert_cmd+assert_fs/insta+proptest，用 trycmd 文件驱动快照补齐 list 六维过滤矩阵并关闭债务。

## Boundaries & Constraints

**Always:**
- 保持 cargo-xtask 纯 Rust dev 命令架构；覆盖率用 source-based 方案（llvm-cov），不用 Linux 限定（ptrace）方案。
- 不破坏既有 `--json` 输出契约与退出码语义（2=参数、3=业务）。
- 不删除既有集成/单元测试断言；只补充，不回退既有覆盖。

**Never:**
- 不新增待办业务功能。
- 不引入 just/cargo-make 等外部任务运行器。
- 不在本 story 设置覆盖率硬门禁阈值（`--fail-under-lines` 属 story 2，本 story 只产出覆盖率为可输出能力）。
- 不切换数据存储格式（`.todo.json` 保持）。

</frozen-after-approval>

## Code Map

- `xtask/src/coverage.rs` -- 现用 cargo-tarpaulin 跑覆盖率（`run_tarpaulin` 调 `cargo tarpaulin -p <pkg> --out Stdout` + 解析 `X.XX% coverage` + 打印 per-crate 表），保留 `XTASK_COVERAGE_TEST_FAKE`/`XTASK_COVERAGE_TEST_FAKE_FAIL` 测试钩子。重写为 `cargo llvm-cov`（`--ignore-filename-regex` 承载原 `--exclude-files` 语义），保留 per-crate 摘要表与排除集哲学（注释里记录的 VM/Lima/REPL 胶水排除意图）。
- `xtask/src/lib.rs` -- `XtaskSub::Coverage` 分发到 `coverage::cmd_coverage`，签名不变，无需改（仅定位）。
- `xtask/Cargo.toml` -- 无 dev-dependencies，新增 trycmd / assert_cmd / assert_fs / insta / proptest。
- `crates/todo/Cargo.toml` -- 无 dev-dependencies，新增 insta / proptest（数据层）。
- `.github/workflows/ci.yml` -- Test 步骤现为 `cargo test -- --test-threads=1`，换为安装 cargo-nextest 后 `cargo nextest run` + `cargo test --doc`。
- `xtask/tests/common/mod.rs` -- `xtask_bin()`/`todo_bin()` 用 `env!("CARGO_BIN_EXE_*")` 调真实二进制；`todo_list/` 已有 `list_json.rs` 等 Command 风格集成测试（复用，不删）。
- `xtask/tests/todo_list/` -- 新增 trycmd 文件驱动快照目录（`.trycmd`/`.toml` fixture + `.stdout`），覆盖 list 六维过滤矩阵。
- `xtask/src/tests/todo/todo_cmd/list_options.rs` -- 现有单测模式（`cmd_todo(todo_args(...))` + `cwd_test_lock()` + `RestoreCwd` + temp dir），作为新快照用例参照。
- `crates/todo/src/list/mod.rs` / `crates/todo/src/list/tests.rs` -- 数据层过滤/排序逻辑，insta 快照 + proptest 属性测试目标。
- `crates/todo/src/store.rs` / `crates/todo/src/model.rs` -- 持久化与数据模型，proptest 序列化往返 + insta 快照目标。

## Tasks & Acceptance

**Execution:**
- [ ] `xtask/Cargo.toml` -- 新增 trycmd / assert_cmd / assert_fs / insta / proptest 到 `[dev-dependencies]` -- 为 CLI 快照与数据层测试提供工具。
- [ ] `crates/todo/Cargo.toml` -- 新增 insta / proptest 到 `[dev-dependencies]` -- 数据层快照/属性测试。
- [ ] `xtask/src/coverage.rs` -- 用 `cargo llvm-cov` 重写覆盖率的运行与解析，原 per-crate 排除集映射到 `--ignore-filename-regex` -- 满足 source-based 跨平台约束。
- [ ] `xtask/src/coverage.rs`（test 模块）-- 更新 fake 钩子测试为 llvm-cov 输出格式 -- 保持 `cmd_coverage` 可测。
- [ ] `xtask/tests/todo_list/` -- 新增 trycmd 文件驱动快照用例，覆盖 `--status`/`--priority`/`--tags`/`--due-before`/`--due-after`/`--sort` 六维正/负矩阵 -- 关闭 deferred-work 第三条债务。
- [ ] `.github/workflows/ci.yml` -- Test 步骤换 `cargo nextest run` + `cargo test --doc`，并安装 cargo-nextest -- 使 nextest 在 CI 上通过。

**Acceptance Criteria:**
- Given 工作区含编译后的 `todo` 二进制，when 运行 trycmd 快照套件，then list 六维过滤各有一组正/负快照通过，且不与既有 `list_json.rs` 断言重复。
- Given `cargo nextest run`，when 在本地与 CI 执行，then 全部测试通过（doctest 由 `cargo test --doc` 单独通过）。
- Given `cargo xtask coverage`，when 运行，then 输出 per-crate 覆盖率摘要表且基于 llvm-cov（非 tarpaulin），且 `llvm-tools-preview` 缺失时给出可操作安装提示。
- Given 污染输入（如非法 `--status`、`--due-before` 非日期），when 调 list，then 退出码 2 且 `.todo.json` 不变（既有集成测试继续通过）。

## Implementation Notes

<!-- Append-only during implementation. Leave empty at planning time. -->

## Design Notes

- nextest 每测试一进程，使 `--test-threads=1` 不再必要；现有 `cwd_test_lock`/`RestoreCwd` 是进程内互斥，切到 nextest 后各测试独立进程、独立 temp dir，竞态消失。实现后须全量跑一遍确认无 flake（尤其 cwd/temp-dir 相关测试）。
- trycmd 快照用 `$CARGO_BIN_EXE_todo` 跑真实二进制，fixture 内以 `[EXE]` 占位符指代；需要先 seed `.todo.json`，快照比较 stdout/stderr。

## Verification

**Commands:**
- `cargo nextest run` -- 期望：全部测试通过。
- `cargo test --doc` -- 期望：doctest 全部通过。
- `cargo run -p xtask -- coverage` -- 期望：输出 per-crate 覆盖率表（llvm-cov）。
- `cargo clippy --all-targets -- -D warnings` -- 期望：无告警。
- `cargo fmt -- --check` -- 期望：无格式漂移。