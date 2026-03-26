---
project_name: xtask_todo
user_name: Richard
date: '2026-03-25'
sections_completed:
  - technology_stack
  - language_rules
  - framework_rules
  - testing_rules
  - quality_rules
  - workflow_rules
  - anti_patterns
status: complete
rule_count: 28
optimized_for_llm: true
---

# 面向 AI 代理的项目上下文

_本文件汇总实现代码时**必须遵守**的规则与模式，侧重仓库中**不显而易见**、易被忽略的细节。_

---

## 技术栈与版本

| 层级 | 说明 |
|------|------|
| **语言** | Rust **edition 2021**（根与各 crate 一致）；无根目录 `rust-toolchain.toml`，以本地 `rustup` 默认 toolchain 为准。 |
| **Workspace** | `resolver = "2"`；成员：`crates/todo`（`xtask-todo-lib`）、`crates/devshell-vm`、`xtask`。 |
| **库版本（摘录）** | `serde` / `serde_json` 1.x；`chrono` 0.4（`default-features = false`）；`rustyline` 17；`argh` 0.1；`ureq` 2.12；`semver` 1；`serde_yaml` 0.9；`csv` 1；`base64` 0.22（β/可选）。 |
| **特性** | `xtask-todo-lib` 默认启用 **`beta-vm`**（Windows 侧车等）；不需要时可 `default-features = false`。 |
| **跨平台** | Windows 为 **MSVC** 一等目标；pre-commit 含 `x86_64-pc-windows-msvc` 的 `cargo check`。 |
| **编排与集成** | 宿主 Git/GitHub/发布逻辑放在 **`xtask`**，不侵入可发布的领域库语义。 |

---

## 关键实现规则

### 语言（Rust）相关

- **Lint 策略**：虚拟 workspace **不能在根 `Cargo.toml` 设 `[lints]`**；各成员 crate 在自身 `Cargo.toml` 中设 `[lints.clippy]`（例如 `all = "warn"`）。本地/CI 实际更严：见下文 pre-commit。
- **条件编译**：Unix / Windows / `beta-vm` 等路径用 `cfg` 明确区分，避免隐式假设 Linux。
- **错误与契约**：面向 CLI/可编程接口时，保持 **`--json`、退出码、序列化错误体** 与 PRD/文档一致；devshell-vm 侧 **stdout 仅 JSON 行**（一行一对象），勿混入非协议输出。
- **依赖**：新增依赖时对齐 workspace 现有版本区间；注意 `chrono` 已关闭默认特性。

### 框架 / 架构相关

- **分层**：Todo **领域与存储**在 `xtask-todo-lib`；**任务编排**（`fmt`/`clippy`/git/publish/acceptance）在 `xtask`；**β 侧车进程**在 `devshell-vm`。不要把宿主专用逻辑塞进可复用库的核心路径。
- **OpenSpec**：涉及**新能力、破坏性变更、架构/性能/安全类大改**或需求不清时，必须先阅读并遵循仓库根 **`openspec/AGENTS.md`**（及 `openspec/project.md`），按变更流程做 proposal / validate；**纯 bug 恢复、排版、非破坏性依赖升级**等可按该文档「可跳过 proposal」的情形处理。
- **根目录 `AGENTS.md`**：含 OpenSpec 与技能表说明；规划类任务勿忽略其中的 OpenSpec 区块。

### 测试相关

- Pre-commit 中 **`cargo test` 使用 `--test-threads=1`**；新增测试时注意并发假设，避免依赖全局可变共享状态。
- 集成与单元边界以现有 `xtask`/`crates` 测试布局为准；大型场景优先可重复、可本地运行的方式。

### 代码质量与风格

- **格式化**：`cargo fmt`，提交前必须通过 `cargo fmt -- --check`。
- **Clippy**：`cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings`（与 `.githooks/pre-commit` 一致）。
- **文档**：`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`；公共 API 变更时更新 rustdoc。
- **文件体量**：pre-commit 对**暂存**的 `.rs` 文件检查 **单文件不超过 500 行**（`MAX_RS_LINES`）；超限应拆分模块/文件。

### 开发流程与仓库约定

- **Pre-commit**：启用方式：`git config core.hooksPath .githooks`，或直接运行 `cargo xtask git pre-commit`（与 CI 精神对齐，细节以 `docs/requirements.md` §7 与实现为准）。
- **常用入口**：`cargo xtask fmt`、`cargo xtask clippy`、`cargo test --workspace`、`cargo xtask acceptance`；发布流程以 **`docs/publishing.md`** 为准，辅助命令 **`cargo xtask publish`**。
- **CI**：通常包含 `fmt --check`、`build`、`test`、`clippy`、`doc`（含 `-D warnings` 类约束）；具体以 `.github/workflows/*.yml` 为准。
- **文档 ID / 追溯**：需求与实现对照时遵循 PRD/文档中的 ID 约定，重大行为变更需可追溯（如 CHANGELOG）。

### 切勿遗漏（反模式与边界）

- 不要在未读 OpenSpec 流程的情况下做大范围「静默实现」Spec 级变更。
- 不要假设 **Lima/Podman/侧车** 一定存在：主路径须可降级（见架构/NFR）。
- **β/IPC**：勿向侧车 stdin/stdout 写入非协议文本；解析侧严格按 **JSON 行** 协议版本处理。
- 忽略 **Windows MSVC** 交叉检查易导致仅 Unix 通过的代码合并。
- 仓库根下若出现 `xtask_*_fail_*` 等目录，多为测试产物，**不要**当作稳定源码依赖。

---

## 使用说明

**对 AI 代理：**

- 在实现功能或改动行为前阅读本文件与相关 `docs/`、`openspec/` 权威说明。
- 规则冲突时以**更严格**或**发布/契约文档**为准。
- 技术栈或团队约定变化时，应同步更新本文件。

**对维护者：**

- 保持正文精简，只保留对代理**非显而易见**的约束。
- 依赖升级或流程变更后更新「技术栈」与「流程」小节。
- 可定期删除已变成常识的条目。

最后更新：2026-03-25
