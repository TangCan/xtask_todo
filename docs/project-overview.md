# 项目概览（棕地文档索引）

**生成：** Document Project 工作流（初次 Quick Scan）  
**仓库：** xtask-todo（Rust workspace）

## 目的与边界

本地待办 CLI/库：数据默认存放在当前目录 **`.todo.json`**；库以 **`xtask-todo-lib`** 发布至 crates.io。产品边界见 **`requirements.md` §2**（非托管 HTTP API、非多租户等默认不承诺）。

## 技术栈摘要

| 类别 | 技术 |
|------|------|
| 语言 | Rust（workspace，`resolver = "2"`） |
| 核心 crate | `crates/todo` → 包名 **xtask-todo-lib**（devshell、VFS、VM 会话、todo 领域） |
| 维护工具 | `xtask`（`cargo xtask`：fmt、clippy、test、publish、acceptance、git 等） |
| VM/侧车 | `crates/devshell-vm`（Windows β 侧车）；γ/β/Lima 等见 `docs/devshell-vm-*.md` |

## 仓库形态

- **单 workspace**，多成员：`crates/todo`、`crates/devshell-vm`、`xtask`。  
- 文档与 BMad 工件：`_bmad-output/`、`docs/` 下已有大量需求与设计真源。

## 延伸阅读

- [主索引](./index.md)  
- [源码树说明](./source-tree-analysis.md)  
- [开发与 CI](./development-guide.md)  
- 架构决策（规划工件）：[`../_bmad-output/planning-artifacts/architecture.md`](../_bmad-output/planning-artifacts/architecture.md)
