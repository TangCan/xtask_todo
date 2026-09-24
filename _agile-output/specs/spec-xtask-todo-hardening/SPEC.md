---
id: SPEC-xtask-todo-hardening
companions: ["quality-tooling.md"]
sources: ["../../../_bmad-output/planning-artifacts/research/technical-improve-xtask-todo-project-2026-09-24/research.md"]
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# xtask_todo 工程化与质量完善

## Why

xtask_todo 已完成 8 个 epic 的功能建设，但工程化底座薄弱：测试仍用默认 `cargo test`（慢、无覆盖率门禁），CI 缺少硬质量门禁与供应链扫描，release 未做体积/启动优化，错误处理与数据容错不够收敛（`.todo.json` 损坏会静默当空），且 `deferred-work.md` 积压了三条测试/一致性债务。外部技术研究发现，上述每一项都有成熟、低风险的工程化手段可补齐——这是当前投入产出比最高的完善方向，应在新增功能之前完成底座加固。

## Capabilities

- **CAP-1 测试基础设施升级**
  - **intent:** 用更快、隔离性更好、可产出覆盖率的测试工具链替换默认 `cargo test`，使测试成为硬质量信号。
  - **success:** `cargo nextest run` 在 CI 上通过且覆盖率可输出；`deferred-work.md` 中"list 过滤 E2E 覆盖不全"有批量化 CLI 快照测试关闭。

- **CAP-2 CI 质量门禁与供应链**
  - **intent:** 建立 lint + fmt + test + coverage + 供应链扫描的硬门禁，任何 lint 警告、格式问题、漏洞都阻断合并。
  - **success:** CI 中 `-D warnings` clippy、`cargo fmt --check`、覆盖率阈值、`cargo deny/audit` 全绿方可合并。

- **CAP-3 release 配置与体积优化**
  - **intent:** 优化 release 构建配置，减小二进制体积与启动开销。
  - **success:** 优化前先记录体积/冷启动基准，优化后有可测的下降（数字以实测为准）。

- **CAP-4 错误处理与数据容错收口**
  - **intent:** 库层强类型错误 + CLI 层上下文传播，数据文件版本化容错迁移，损坏数据不再静默当空。
  - **success:** `deferred-work.md` 中"损坏 JSON 静默当空"与"dry-run 校验不一致"有针对性测试关闭；错误消息可定位；退出码区分参数/业务错误。

- **CAP-5 开发者体验点缀**
  - **intent:** 提供 shell 补全子命令、批量操作进度反馈、结构化日志，提升交互体验。
  - **success:** `completions <shell>` 可生成 bash/zsh/fish 补全；导入/导出等长操作有进度显示；日志结构化且不破坏现有 JSON 输出。

- **CAP-6 面向 agent 的可机读导出**
  - **intent:** 在现有结构化 JSON 输出基础上，提供稳定、可机读（趋向 agent/MCP 友好）的导出接口，作为差异化能力。
  - **success:** 导出的 schema 有文档且稳定，可被外部程序/agent 无歧义消费。

## Constraints

- 保持 `cargo-xtask` 纯 Rust dev 命令架构，不切换到 just/cargo-make 等外部任务运行器（研究已确证其为现代主流实践）。
- 测试与覆盖率工具须跨平台（Linux/Windows/Lima）精确，选 source-based 覆盖率方案，不用 Linux 限定（ptrace）方案。
- release/性能相关的体积与速度数字多为二手来源（置信 medium），任何配置变更必须先实测基准，不得直接采用报告数字。
- 数据文件（`.todo.json`）迁移须向后兼容已有数据，采用版本化 + 缺省回退策略，不得因升级丢数据。
- 结构化 JSON 输出契约保持不变，升级日志/输出手段不得破坏既有 `--json` 消费方。

## Non-goals

- 不新增待办业务功能（本次仅工程化加固）。
- 不引入富 TUI（ratatui）重写交互界面。
- 不切换数据存储格式（todo.txt 仅作 import/export 兼容，不作主格式）。
- 不引入独立任务运行器（just/cargo-make/ninja）。

## Success signal

一次可演示：在 CI 上 `cargo nextest run` + 覆盖率硬门禁 + clippy `-D warnings` + `cargo deny check` 全绿，且 `deferred-work.md` 三条债务逐条有对应测试/校验关闭；release 二进制体积与冷启动相对基准有实测下降。

## Assumptions

- 本迭代方向定为"质量与可靠性优先于新功能"，依据研究结论的优先级排序。
- 覆盖率目标阈值无行业共识，先按"充分覆盖核心路径"设定，具体百分比记为待确认项。

## Open Questions

- 覆盖率硬门禁的具体阈值（如 80% 行覆盖起步？）需确认。
- 是否/何时引入 MCP 导出（研究报告标记为 lead，需再深研）。
- 补全是否包含 `unstable` 动态补全（clap_complete 该 feature 尚未转正）。