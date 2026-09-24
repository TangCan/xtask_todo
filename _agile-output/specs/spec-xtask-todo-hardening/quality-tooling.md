# quality-tooling — 工具选型目录（HOW）

> 本文承载 SPEC 各能力的具体实现选型（工具库/构建开关/CI 配置），对应 Spec Law 规则 2：内核只说 WHAT，HOW 落在此处。来源见 `sources:`（研究报告），置信度标注沿用其结论；D3 性能数字统一置信 `medium`，须先实测。

## CAP-1 测试基础设施

| 用途 | 选型 | 说明 |
|---|---|---|
| 测试运行器 | `cargo-nextest` | 每测试一进程、用例级并行、泄漏检测、失败重试；不跑 doctest，配 `cargo nextest run && cargo test --doc` |
| 覆盖率 | `cargo-llvm-cov` | source-based、最准、跨平台、支持分支；需 `llvm-tools-preview`；`--branch` 目前 unstable |
| CLI 批量快照 | `trycmd` / `snapbox` | `.stdout/.stderr` 快照 + `[EXE]` 占位符；"少量精养用 assert_cmd、批量放养用 trycmd" |
| CLI 定制断言 | `assert_cmd` + `assert_fs` | 核心路径（如 dry-run 校验一致性）精确测试 |
| 数据层 | `insta`（快照）+ `proptest`（属性） | proptest `failure_persistence` 把失败输入写回归文件下次优先重放 |

## CAP-2 CI 门禁与供应链

| 门 | 命令 / 配置 | 说明 |
|---|---|---|
| lint | `RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features` | 任何 lint 阻断（官方模式） |
| 格式 | `cargo fmt --check` | 格式漂移阻断 |
| 覆盖率 | `cargo llvm-cov --fail-under-lines <阈值>` | 阈值见 SPEC Open Questions |
| 供应链 | `cargo deny check` + `cargo audit` | 前者管许可证/禁用/重复 crate/来源，后者管 RustSec 漏洞；双跑 |
| 矩阵 | linux/macos/windows × amd64/arm64 | 与项目跨平台定位一致 |

## CAP-3 release 配置

基于 Cargo release 默认基线（`opt-level=3, lto=false, codegen-units=16, panic=unwind, strip=none`）逐项调整，性价比排序：

| 开关 | 收益 | 代价 | 置信 |
|---|---|---|---|
| `strip = "symbols"` | 体积 −50%~−70% | 丢 crash 符号（可用 `debug="line-tables-only"` 折中） | medium |
| `lto = true`（或 thin） | 体积 −10%~−20% | 编译慢 2~5× | medium |
| `codegen-units = 1` | 再 −5%~−10% | 编译慢 1.5~2× | medium |
| `opt-level = "s"` | 体积/启动折中 | 略慢于 "3" | medium |

分析入口：`cargo-bloat`（按函数/按 crate 看 `.text` 段，定位 serde/regex 等未用 feature）。**约束：数字二手，先建基准再改。**

## CAP-4 错误处理与数据容错

| 关注点 | 选型 / 模式 |
|---|---|
| 库层错误 | `thiserror`（可枚举强类型） |
| 应用/CLI 层 | `anyhow`（传播 + `context()` 堆叠），`bail!`/`ensure!` 早退 |
| 退出码 | 参数/用法错误 2、业务失败 3，勿一律 1、勿失败退 0；`exitcode` crate 语义 |
| 可观测性 | `tracing` + `tracing-subscriber`（`EnvFilter`/RUST_LOG、JSON 输出、`tracing-appender` 滚动）；优先结构化字段，不破坏既有 `--json` |
| 数据迁移 | `#[serde(default)]` 缺字段回退 + `#[serde(tag="version")]` 版本化枚举迁移 |

## CAP-5 开发者体验

| 用途 | 选型 | 说明 |
|---|---|---|
| 补全 | `clap_complete` / `clap_complete_command` | 内置 `completions <shell>` 子命令；动态补全在当前版本为 `unstable` feature |
| 进度/反馈 | `indicatif`（+ `tracing-indicatif`） | stderr、限频刷新，避免与日志抢屏 |
| 颜色 | `anstream`（clap 默认后端） | 跨平台自动降级 ANSI（含 Windows） |
| 交互提示（如需要） | `inquire` 或 `dialoguer` | 内建 Select/MultiSelect/Confirm/补全 Text |

## CAP-6 生态 / 差异化

| 项 | 结论 |
|---|---|
| 架构 | `cargo-xtask` 纯 Rust dev 命令是已验证主流模式（rust-analyzer/tokio/egui 采用），保持不变 |
| import/export | `todo-txt` crate 可作 todo.txt 导入导出直接依赖 |
| 差异化 | "本地-first + 面向 LLM/agent 的可机读导出"（对标 Todoist CLI 的 JSON/MCP 趋势）；是否引入 MCP 待再深研 |