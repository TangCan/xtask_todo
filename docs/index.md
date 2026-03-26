# 项目文档索引（AI 检索入口）

**类型：** Rust workspace（库 **xtask-todo-lib** + **xtask** 工具 + **devshell-vm**）  
**主语言：** Rust  
**架构：** 本地 CLI / devshell / 可选 VM 后端；无内置多租户 HTTP API（见 `requirements.md` §2）。

**生成：** Document Project（`initial_scan` / `quick`），`project-scan-report.json` 记录步骤。

---

## 本次生成的棕地文档

| 文档 | 说明 |
|------|------|
| [项目概览](./project-overview.md) | 目的、技术栈、边界、延伸阅读 |
| [源码树导读](./source-tree-analysis.md) | 顶层目录与关键入口 |
| [开发与运维](./development-guide.md) | 构建、测试、CI、发布、注意事项 |

---

## 既有权威文档（仓库内）

需求与验收：

- [需求规格](./requirements.md)
- [设计](./design.md)
- [验收标准](./acceptance.md)
- [测试用例 ID](./test-cases.md)
- [发布流程](./publishing.md)

Devshell / VM：

- [devshell-vm-gamma](./devshell-vm-gamma.md)
- [devshell-vm-windows](./devshell-vm-windows.md)
- [devshell-vm-oci-release](./devshell-vm-oci-release.md)

### 规划工件（BMad）

- [架构（architecture.md）](../_bmad-output/planning-artifacts/architecture.md)
- Epics / PRD 等：见 `../_bmad-output/planning-artifacts/`

---

## Quick Scan 未单独生成的项

以下类型在 **library/cli** 的 Quick Scan 策略下**不生成**独立文件（无 REST API 目录、无 ORM  schema 文档）：

- API Contracts（不适用：非 HTTP 产品）
- Data Models（不适用：`.todo.json` 领域模型见 `requirements` §3 与 `xtask-todo-lib` 源码）

若需更深文档，可重新运行 Document Project 并选择 **Deep / Exhaustive** 或 **Deep-dive** 指定目录。

---

## 快速开始（给 AI / 新贡献者）

1. 读 [项目概览](./project-overview.md) 与 [requirements.md](./requirements.md) §1–§2。  
2. 改 CLI/行为：从 `xtask/src/todo/`、`crates/todo/src/devshell/` 入口跟踪。  
3. 合并前：运行 `cargo xtask git pre-commit` 或 CI 等价命令（见 [development-guide](./development-guide.md)）。
