# 进一步改进建议

在 [improvements-and-rust-style.md](./improvements-and-rust-style.md) 与既有工程实践（CI、pre-commit、单文件行数约定等）基础上，对项目做一次分析并给出后续可选的改进方向。已落实项见下文「当前状态」，未落实或可加强项见「改进建议」。

---

## 一、当前状态概览

### 已落实

| 类别 | 内容 |
|------|------|
| **代码质量** | rustfmt.toml（edition、max_width）；pre-commit 与 CI 跑 `cargo fmt -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`；crates/todo 与 xtask 的 `[lints.clippy] all = "warn"`；API 文档含 `# Errors` 与反引号；`#[must_use]` 已加。 |
| **工程** | 根目录 README（用途、todo 用法、测试、Git hooks、文档入口）；.github/workflows/ci.yml（fmt / build / test / clippy / doc）；.githooks/pre-commit（fmt 检查、.rs 500 行限制、clippy、test）；依赖版本已固定（argh、serde、serde_json）。 |
| **xtask** | 子命令：run、fmt、clippy、coverage、git（add/commit）、todo（add/list/complete/delete）；`cargo xtask fmt` 等价于 `cargo fmt`；coverage 流式输出与 per-crate 汇总。 |
| **测试** | crates/todo 单元测试覆盖 US-T*；xtask 多模块单元测试（含 cwd 互斥、fake coverage、fmt/clippy/git/todo）；集成测试 `xtask_run_exits_success`；tarpaulin 实跑测试已 `#[ignore]` 避免与 binary 测试冲突。 |

### 与文档的差异

- **CI Clippy**：CI 使用 `-W clippy::pedantic -W clippy::nursery -D warnings`，pre-commit 仅 `-D warnings`。若希望本地与 CI 完全一致，可将 pre-commit 改为与 CI 相同参数，或在 CI 中去掉 pedantic/nursery（二选一即可）。
- **openspec/project.md**：未写 pre-commit、单文件 500 行约定、xtask 子命令列表；若希望「单一事实来源」，可在 project.md 的 Code Style / Testing Strategy 中补一句说明。

---

## 二、改进建议（按优先级）

### 高优先级

| 建议 | 说明 |
|------|------|
| **统一 CI 与 pre-commit 的 Clippy 参数** | 要么 pre-commit 也加 `-W clippy::pedantic -W clippy::nursery`，要么 CI 去掉这两项，使本地与 CI 行为一致，避免「本地过、CI 红」或反过来。 |
| **README 补充 xtask 子命令** | 在 README 中列出 `cargo xtask fmt`、`cargo xtask clippy`、`cargo xtask coverage`、`cargo xtask git add/commit`，并简要说明用途（如：fmt 格式化、clippy 检查、coverage 覆盖率、git 暂存与提交），便于新成员上手。 |

### 中优先级

| 建议 | 说明 |
|------|------|
| **crates/todo 集成测试** | 在 `crates/todo/tests/` 增加 1～2 个集成测试：例如从 JSON 反序列化 → `InMemoryStore::from_todos` → 再序列化或通过 `TodoList` 做一次完整流程，覆盖与 xtask 共用的 DTO 与存储路径（与 improvements-and-rust-style 中「集成测试」一致）。 |
| **集成测试覆盖 xtask todo** | 在 `xtask/tests/` 中增加用例：在临时目录下执行 `xtask todo add "x"`、`xtask todo list`，断言退出码与 stdout 含预期内容；可选再测 `complete`/`delete`。不依赖真实 .todo.json，用临时目录即可。 |
| **更新 openspec/project.md** | 在 Code Style 中注明：提交前需通过 pre-commit（或等价于 fmt + clippy + test）；单文件 .rs 不超过 500 行；在 Testing Strategy 或单独小节中列出 xtask 子命令及「CI 跑 cargo test、可选跑 xtask 端到端」等。 |

### 低优先级

| 建议 | 说明 |
|------|------|
| **错误处理与错误类型** | 为 `TodoError` 实现 `From<…>` 或引入 `thiserror`/`anyhow`，便于 xtask 中统一错误输出与日志；当前 `Box<dyn Error>` 已可用，此为可维护性增强。 |
| **.todo.json 路径约定** | 若需多工作区隔离，可约定「项目根」为含 `Cargo.toml` 的目录，在此目录下读写 .todo.json，避免误写其他目录；当前 `current_dir()` 对单仓库已足够。 |
| **CI 可选步骤** | 可选在 CI 中增加：`cargo audit`（安全审计）、`cargo test --no-fail-fast`（完整测试列表）、或定期 `cargo update` 并跑 test 以发现依赖升级问题。 |
| **CHANGELOG 或 CONTRIBUTING** | 若多人协作，可增加 CHANGELOG.md（按版本记录变更）或 CONTRIBUTING.md（如何跑测试、提 PR、代码风格与 pre-commit 说明）。 |

---

## 三、实施顺序建议

1. **先做**：统一 Clippy 参数（pre-commit 与 CI）、README 补充 xtask 子命令。改动小、收益明确。
2. **随后**：crates/todo 集成测试、xtask 集成测试（todo add/list），并更新 project.md。
3. **按需**：错误类型增强、.todo.json 路径约定、CI 可选步骤、CHANGELOG/CONTRIBUTING。

---

## 四、与现有文档的关系

- **improvements-and-rust-style.md**：仍为 Rust 风格与历史改进建议的参考；其中「单文件行数」「pre-commit」「CI」等已落实，未落实项（如集成测试、错误处理）与本文档「改进建议」一致。
- **requirements / design / acceptance / test-cases**：功能与验收不变；本文档不改变需求范围，仅建议工程与可维护性上的增强。
- **openspec/project.md**：建议在此补充「pre-commit、单文件行数、xtask 子命令与 CI 测试策略」的简要描述，使规范与实现一致。

上述建议均为可选；实施时可按优先级与人力逐步推进，并同步更新相关 doc。
