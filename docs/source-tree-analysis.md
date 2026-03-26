# 源码树导读（Quick Scan）

**说明：** 以下为仓库**主路径**注释，便于 AI/新人定位；非全量文件清单。

```
xtask_todo/                          # workspace 根
├── Cargo.toml                       # workspace 成员：crates/todo, crates/devshell-vm, xtask
├── README.md                        # 使用说明：cargo xtask todo、cargo-devshell、边界
├── AGENTS.md                        # OpenSpec / 技能表（AI 助手）
├── .github/workflows/               # CI（ci.yml）、发布（release.yml）
├── .githooks/pre-commit             # 本地与 xtask git pre-commit 对齐的检查脚本
├── crates/
│   ├── todo/                        # xtask-todo-lib：领域、devshell、vm、sandbox、bin cargo-devshell / todo
│   └── devshell-vm/                 # Windows 侧车二进制（β VM 协议）
├── xtask/                           # 维护用 xtask：todo 子命令、acceptance、publish、git 等
├── docs/                            # 需求、设计、验收、devshell VM 文档（本索引所在）
├── _bmad-output/                    # BMad：planning-artifacts、implementation-artifacts、sprint-status
├── containers/                      # 与 devshell-vm 镜像/OCI 相关
└── openspec/                        # OpenSpec 变更（若启用）
```

## 关键入口

| 入口 | 路径 |
|------|------|
| `cargo xtask` | `xtask/src/main.rs` → `xtask::run()` |
| `todo` CLI（独立二进制） | `crates/todo/src/bin/todo.rs` |
| `cargo-devshell` | `crates/todo/src/bin/cargo_devshell/` |
| Todo 子命令分发 | `xtask/src/todo/cmd/dispatch.rs` |
| Devshell REPL | `crates/todo/src/devshell/repl.rs`、`mod.rs` |

## 测试布局（概念）

- **`xtask/tests/`**：集成测试（todo、acceptance 相关等）  
- **`crates/todo/src/.../tests/`**：库内与 devshell 测  
- 详见各 crate 内 `tests/` 与 `docs/test-cases.md`
