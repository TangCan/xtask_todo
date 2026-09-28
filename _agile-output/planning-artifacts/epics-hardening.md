---
project_name: xtask_todo
document_type: hardening-epics-and-stories
source_spec: ../specs/spec-xtask-todo-hardening/SPEC.md
companion: ../specs/spec-xtask-todo-hardening/quality-tooling.md
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
status: complete
---

# xtask_todo 工程化与质量完善规划

本文件是 `SPEC-xtask-todo-hardening` 的独立实现规划，不覆盖既有 `_agile-output/planning-artifacts/epics.md`。

## Requirements Inventory

### Capability Requirements

- CAP-1：升级测试基础设施，使用 cargo-nextest、cargo-llvm-cov、CLI 快照和数据层属性测试。
- CAP-2：建立 fmt、clippy、测试、覆盖率和供应链扫描的 CI 硬门禁。
- CAP-3：记录 release 基准并优化构建体积与启动开销。
- CAP-4：收敛错误处理、数据容错、版本化迁移和 dry-run 校验一致性。
- CAP-5：提供 shell 补全、长操作进度反馈和结构化日志。
- CAP-6：提供稳定、有文档、面向 Agent/外部工具的机器可读导出接口。

### Constraints

- 保持 cargo-xtask 纯 Rust dev 命令架构。
- 覆盖率使用跨平台 source-based 方案，不使用 Linux 专属 ptrace 方案。
- release 和性能调整必须先有实测基准。
- `.todo.json` 迁移必须向后兼容，不得丢失既有数据。
- 保持现有 `--json` 输出契约和退出码语义。
- 不新增待办业务功能，不引入富 TUI、独立任务运行器、MCP 或 HTTP API。

## Capability Coverage Map

CAP-1: Epic 1 - 测试基础设施与覆盖率
CAP-2: Epic 1 - CI 质量门禁与供应链
CAP-3: Epic 2 - Release 基准与优化
CAP-4: Epic 2 - 错误处理与数据容错
CAP-5: Epic 3 - CLI 交互与可观测性
CAP-6: Epic 3 - Agent/外部工具导出

## Planned Work

### Epic 1: 测试与合并质量信号

让维护者能稳定运行测试，并在 CI 中获得覆盖率、lint、格式和供应链安全门禁。

**覆盖：** CAP-1、CAP-2

### Epic 2: 可靠性与发布质量

让用户在数据损坏、错误输入和发布场景下获得可预测行为，并让发布产物更小、更快。

**覆盖：** CAP-3、CAP-4

### Epic 3: 开发者体验与 Agent 集成

让用户获得更好的 CLI 交互和稳定的机器可读接口。

**覆盖：** CAP-5、CAP-6

## Epic 1：测试与合并质量信号

### Story 1.1: 测试运行器与测试工具链升级

作为项目维护者，
我希望使用更快且隔离性更好的测试工具链，
以便测试结果成为可靠的质量信号。

**验收标准：**

- Given 工作区已有现有单元测试和集成测试，When 执行 `cargo nextest run`，Then 所有非 doctest 测试通过，既有测试断言、JSON 输出契约和退出码语义保持不变。
- Given 项目需要 CLI 快照和数据层测试，When 构建测试目标，Then xtask 可使用 trycmd、assert_cmd、assert_fs、insta、proptest，xtask-todo-lib 可使用 insta、proptest。
- Given nextest 不执行 doctest，When 执行 `cargo test --doc`，Then 所有 doctest 独立通过。
- Given 测试依赖临时目录或工作目录，When 通过 nextest 并行执行测试，Then 测试之间不会共享可变状态或产生竞态，现有 cwd/temp-dir 相关测试保持通过。

### Story 1.2: 跨平台覆盖率能力

作为项目维护者，
我希望使用 source-based 覆盖率工具生成各 crate 的覆盖率摘要，
以便在 Linux、macOS 和 Windows 上获得一致的质量信号。

**验收标准：**

- Given 工作区已安装 `cargo-llvm-cov` 和 `llvm-tools-preview`，When 执行 `cargo xtask coverage`，Then 使用 llvm-cov 运行覆盖率，输出各 crate 的覆盖率摘要，且不再调用 cargo-tarpaulin。
- Given 现有 coverage 命令配置了文件排除规则，When 执行覆盖率命令，Then 排除规则通过 `--ignore-filename-regex` 等价传递，VM、Lima、REPL 胶水代码的既有排除意图保持不变。
- Given 本机缺少 `llvm-tools-preview`，When 执行 `cargo xtask coverage`，Then 命令失败并明确提示安装方式，不产生误导性的覆盖率结果。
- Given coverage 命令存在 fake/fail 测试钩子，When 执行 coverage 单元测试，Then llvm-cov 输出格式和失败路径均被覆盖，原有命令错误传播语义保持不变。

### Story 1.3: CLI 快照与数据层属性测试

作为项目维护者，
我希望用批量快照和属性测试覆盖 CLI 与数据层边界，
以便及时发现过滤、排序和序列化回归。

**验收标准：**

- Given 工作区存在多条包含不同状态、优先级、标签和日期的待办，When 运行 trycmd 快照套件，Then 覆盖 `--status`、`--priority`、`--tags`、`--due-before`、`--due-after` 和 `--sort`，每个过滤维度至少包含正向和负向场景，并通过真实 todo 二进制执行。
- Given 已有 `list_json.rs` 等集成测试，When 新增 CLI 快照，Then 新快照覆盖不同场景，不重复已有测试断言，且不改变既有 JSON 输出契约。
- Given 数据层执行过滤、排序和序列化往返，When 使用 insta 和 proptest 测试，Then 正常输入、边界输入和随机有效输入均满足既定不变量，序列化再反序列化不会丢失合法字段，属性测试失败时保留可复现的失败输入。
- Given list 命令收到非法状态、非法日期或其他参数，When 执行命令，Then 返回参数错误退出码 2，且 `.todo.json` 内容保持不变。
- 完成后，`deferred-work.md` 中关于 list 过滤 E2E 覆盖不足的债务应有对应测试证据。

### Story 1.4: CI 硬质量门禁

作为项目维护者，
我希望 CI 自动执行测试、格式、lint、覆盖率和供应链检查，
以便低质量或存在已知漏洞的变更无法合并。

**验收标准：**

- Given CI 执行测试流程，When 运行工作区测试，Then 安装并执行 `cargo nextest run`，单独执行 `cargo test --doc`，两者失败时均阻断 CI。
- Given Rust 源码或格式发生问题，When CI 执行质量检查，Then `cargo fmt --check` 失败会阻断 CI，clippy 使用 `-D warnings`，任意警告会阻断 CI。
- Given 覆盖率门禁已配置阈值，When 覆盖率低于阈值，Then CI 失败，覆盖率报告作为 CI 工件保留，阈值来源和调整方式有文档说明。
- Given 依赖存在许可证、来源、重复版本或 RustSec 风险，When CI 执行 `cargo deny check` 和 `cargo audit`，Then 任一检查失败会阻断 CI，检查结果可在 CI 日志中定位。
- Given CI 在 Linux、macOS 或 Windows 上运行，When 执行上述质量门禁，Then 命令使用跨平台可用的配置，不依赖 Linux 专属 ptrace 覆盖率方案。

## Epic 2：可靠性与发布质量

### Story 2.1: Release 基准与可重复测量

作为项目维护者，
我希望在修改 release 配置前记录二进制体积和冷启动基准，
以便后续优化结果有客观依据。

**验收标准：**

- Given 当前 workspace 可构建 release 二进制，When 执行基准采集命令，Then 记录每个目标二进制的文件大小，记录统一环境下的冷启动指标，并记录 Rust/toolchain、目标平台和构建参数。
- Given 基准数据已生成，When 在相同环境重复采集，Then 输出格式稳定，测量结果包含足以解释差异的元数据，基准文件存放在约定的工程文档或工件目录。
- Given 后续 release 配置发生变化，When 比较优化前后的结果，Then 能明确显示体积和冷启动指标的变化，不使用研究报告中的二手数字替代实测结果。

### Story 2.2: Release 配置与体积/启动优化

作为项目维护者，
我希望优化 release 构建配置，
以便获得更小的二进制和更好的启动表现。

**验收标准：**

- Given Story 2.1 已生成稳定的 release 基准，When 调整 release profile，Then 优先评估 strip、LTO、codegen-units 和 opt-level 等配置，每项调整均有明确变更记录。
- Given release 配置完成调整，When 在与基准相同的环境构建并测量，Then 二进制体积或冷启动指标至少有可测改善，报告优化前后实际数值，若某项配置没有收益则不保留无效调整。
- Given release 构建用于正式发布，When 执行 workspace release build，Then 所有目标仍可成功构建，debug 构建、测试构建和开发工作流不受影响。
- Given release 二进制发生运行时错误，When 需要排查问题，Then 保留约定级别的调试信息或符号策略，并文档说明 strip/LTO 等配置对诊断能力的影响。

### Story 2.3: 强类型错误与 CLI 错误上下文

作为 CLI 和库的使用者，
我希望错误信息具有明确类型、上下文和退出码，
以便快速定位问题并让自动化程序可靠处理失败。

**验收标准：**

- Given 库层发生可分类的领域或存储错误，When 错误返回给调用方，Then 使用可枚举的强类型错误表达，并区分输入、数据、存储和业务失败。
- Given CLI 调用库层或外部命令失败，When 错误向用户传播，Then 添加命令、路径或操作上下文，不丢失底层错误原因，人类可读输出不泄漏调试堆栈。
- Given 用户传入非法参数或参数组合，When CLI 执行失败，Then 返回退出码 2，并输出可定位的参数错误。
- Given 数据损坏、存储失败或业务操作失败，When CLI 执行失败，Then 返回退出码 3，并输出可定位的业务/数据错误。
- Given 命令使用 `--json`，When 成功或失败，Then 输出保持既有 JSON 契约，错误输出不混入非 JSON 协议内容。

### Story 2.4: 数据容错、版本化迁移与 dry-run 一致性

作为待办数据使用者，
我希望数据文件损坏或格式升级时得到明确、可恢复的行为，
以便避免数据被静默丢失或误判为空。

**验收标准：**

- Given `.todo.json` 不存在，When CLI 读取待办数据，Then 按既有约定将其视为空数据集，且不创建意外文件或写入默认数据。
- Given `.todo.json` 存在但 JSON 损坏或结构不可解析，When CLI 读取待办数据，Then 命令失败并返回业务/数据错误退出码 3，不将损坏数据静默当作空列表，错误信息包含文件路径和解析原因。
- Given 旧版本 `.todo.json` 缺少新增字段或使用旧版本格式，When CLI 读取数据，Then 通过默认值或显式迁移兼容读取，已有待办内容不丢失，迁移后的数据仍可被当前版本保存和读取。
- Given 数据文件标记了不支持的未来版本，When CLI 尝试读取，Then 命令失败并明确提示版本不受支持，且不覆盖原始数据文件。
- Given 用户执行修改类命令的 `--dry-run`，When 参数或数据校验失败，Then dry-run 与正常执行使用相同的校验规则，返回相同的参数/业务错误语义，且不修改 `.todo.json`。
- Given 用户执行成功的修改类 `--dry-run`，When 命令完成，Then 展示将要执行的变更或结构化结果，且 `.todo.json`、相关备份和迁移文件均不发生写入。

## Epic 3：开发者体验与 Agent 集成

### Story 3.1: 跨平台 Shell 补全

作为 CLI 用户，
我希望为常用 shell 生成补全脚本，
以便更快发现和输入可用命令、参数及选项。

**验收标准：**

- Given CLI 支持 bash、zsh 和 fish，When 执行 `completions <shell>`，Then 输出对应 shell 的完整补全脚本，且不向 stdout 混入日志、进度或诊断文本。
- Given 用户传入不支持的 shell 名称，When 执行 `completions <shell>`，Then 返回参数错误退出码 2，并明确列出支持的 shell。
- Given 用户通过重定向保存补全脚本，When 将脚本加载到目标 shell，Then CLI 顶层命令、子命令和静态选项可补全，补全内容与当前 CLI 命令定义一致。
- Given CLI 使用 `--json` 或其他结构化输出选项，When 运行非 completions 命令，Then 新增补全功能不改变既有 JSON 输出和退出码契约。
- Given 当前版本不支持动态补全，When 生成补全脚本，Then 不启用不稳定的动态补全能力，并在文档明确说明支持范围。

### Story 3.2: 长操作反馈与结构化日志

作为 CLI 用户和维护者，
我希望长时间操作有清晰进度、日志可查询，
以便了解执行状态并排查问题，同时不破坏机器可读输出。

**验收标准：**

- Given 导入、导出或其他耗时操作需要处理多条数据，When 在交互式终端执行，Then 在 stderr 显示限频更新的进度反馈，进度包含已处理数量或可解释的阶段信息，且不明显拖慢操作。
- Given 同一操作的 stdout 被程序消费，When 命令运行，Then stdout 只包含约定的人类输出或 JSON 输出，进度和诊断信息不污染 stdout。
- Given 用户启用结构化日志，When 执行命令，Then 日志包含时间、级别、操作和必要上下文字段，并可通过环境变量或既有配置控制日志级别。
- Given 命令使用 `--json`，When 执行成功或失败，Then stdout 仍是合法且稳定的 JSON，结构化日志默认不混入 JSON stdout。
- Given 操作失败或被中断，When 进度反馈结束，Then 输出不会伪造成功状态，返回既有约定退出码，日志包含足够上下文定位失败阶段。

### Story 3.3: 稳定的 Agent/外部工具导出接口

作为 Agent 或外部自动化工具的开发者，
我希望获得有文档、稳定且可验证的机器可读导出接口，
以便无需解析人类可读文本即可消费待办数据。

**验收标准：**

- Given 用户请求机器可读导出，When 执行导出命令，Then 输出稳定的 JSON 结构，schema 明确记录字段、类型、必填性和语义，且输出不包含非 JSON 文本。
- Given 导出结果被外部程序解析，When 数据包含空值、特殊字符、Unicode、标签、日期或重复任务字段，Then JSON 仍合法，字段语义与现有 `--json` 契约一致，且不发生字段静默丢失或类型漂移。
- Given 当前 schema 需要演进，When 增加兼容字段或调整输出，Then 提供 schema 版本或等价的版本识别机制，向后兼容已有消费者，并对破坏性变更提供明确迁移说明。
- Given 外部程序需要验证输出，When 查看项目文档或 schema 文件，Then 能找到机器可读接口的示例、字段说明和版本策略，且示例可被实际解析。
- Given 导出命令执行失败，When 数据不可读或参数非法，Then 返回既有约定退出码，错误输出遵循既有 JSON 错误契约，且不承诺本 Story 之外的 MCP 服务或 HTTP API。

## Implementation Notes

- Epic 1 是质量基础，可先于其他 Epic 实施。
- Epic 2 的 Story 2.2 依赖 Story 2.1 的基准数据；Story 2.3 和 2.4 可独立实现。
- Epic 3 建立在现有 CLI 和 `--json` 契约上，不引入 MCP 或 HTTP API。
- 所有跨平台行为必须覆盖 Windows MSVC 与 Unix 主路径；不假定 Lima、Podman 或 VM 一定存在。
