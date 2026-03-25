# Story 2.5：提交前检查对齐 CI

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名贡献者，  
我希望在合并前运行与 CI 对齐的 pre-commit（或等价路径），  
以便本地失败与 CI 一致。

## 映射需求

- **FR13**（`epics.md` Story 2.5；`docs/requirements.md` §4 表、`§7.2` — **`.githooks/pre-commit`**、**`cargo xtask git pre-commit`** 与 CI 描述）
- **NFR-I1**：**`rustup` target**、**`cargo`** 等缺失时错误可读（与 **§7.2** 一致），**不**静默成功

## Acceptance Criteria

1. **Given** 权威脚本 **`/.githooks/pre-commit`** 与 **`cargo xtask git pre-commit`**（见 **`xtask/src/git.rs`** — 调用 **`sh`** 执行该脚本）  
   **When** 对照 **`docs/requirements.md` §4** 表中对该钩子的逐项描述（**fmt**、暂存 **`.rs` 行数**、**clippy**、**rustdoc**、**test**、**Windows MSVC `cargo check`**）  
   **Then** 脚本内容与文档一致，或**差异已写入** **`requirements` / `design` / 本文 Dev Notes** 并说明理由（**FR13**）。

2. **Given** 仓库 **`.github/workflows/ci.yml`**（**`cargo fmt`**、**`cargo build`**、**`cargo test`**、**`cargo clippy`**、**`cargo doc --no-deps`**）  
   **When** 制作 **pre-commit ↔ CI** 对照表（命令、顺序、环境变量如 **`RUSTDOCFLAGS`**、是否含 **`x86_64-pc-windows-msvc`**）  
   **Then** 对每一项 **仅本地**、**仅 CI** 或**双方**有的步骤标注清楚；若存在**故意**差异（例如本地 **pre-commit** 更严），须在 **`docs/requirements.md` §7.2** 或 **`ci.yml` 注释** 中写清「为何本地多/少某步」（**FR13**）。

3. **Given** 贡献者按文档启用 **`git config core.hooksPath .githooks`** 或使用 **`cargo xtask git pre-commit`**  
   **When** 在含变更的工作区上运行  
   **Then** 失败时退出码非 0，且输出可定位失败阶段（fmt / clippy / doc / test / msvc 等），与脚本 **`set -e`** 行为一致（**FR13**，**NFR-I1**）。

4. **Given** **`rustup target list --installed`** 不含 **`x86_64-pc-windows-msvc`**（**§7.2** 已说明）  
   **When** 运行 **pre-commit** 至 MSVC 一步  
   **Then** 行为与 **§7.2** 及 **`cargo xtask acceptance`** 中 **NF-6/NF-5** 叙述一致（本地须安装 target；错误信息可理解）（**NFR-I1**）。

5. **棕地**：本故事以 **核对与文档/CI 对齐、小范围脚本或工作流补丁**为主；**不**无依据重写 **git** 子命令架构。若调整 **CI** 以贴近 **pre-commit**（例如为 **`cargo doc`** 统一 **`RUSTDOCFLAGS=-D warnings`**），须评估 **Actions** 时长与缓存策略。

6. **回归**：**`cargo xtask git pre-commit`** 与现有集成测试不破坏；**`cargo test -p xtask`**、**`cargo clippy -p xtask --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：精读 **`.githooks/pre-commit`**、**`.github/workflows/ci.yml`**、**`docs/requirements.md` §4、§7.2**；输出 **Markdown 对照表**（可附于故事 Completion Notes 或 **`docs/`** 若团队希望单独成文）。
- [ ] **差距处理**：对每个差异选择 **(a)** 改 **CI**、**(b)** 改 **pre-commit**、或 **(c)** 仅文档化；实施所选项并更新 **`requirements`** / **workflow 注释**。
- [ ] **验收引用**：若 **NF-6**（**`acceptance` 对 pre-commit 脚本**的静态检查）与脚本变更相关，同步 **`xtask/src/acceptance/checks.rs`** 中的关键字检测或说明。
- [ ] **验证**：本地试跑 **`cargo xtask git pre-commit`**（或 **`sh .githooks/pre-commit`**）；**`cargo test -p xtask`**、**clippy**。

## Dev Notes

### 棕地现状（摘录）

| 工件 | 路径 / 说明 |
|------|-------------|
| Git 钩子脚本 | **`.githooks/pre-commit`**（**`sh`**；**`MAX_RS_LINES=500`** → **fmt** → **clippy** → **`RUSTDOCFLAGS=-D warnings` `cargo doc --no-deps`** → **`cargo test -- --test-threads=1`** → **MSVC `cargo check`**） |
| xtask 封装 | **`xtask/src/git.rs`** — **`GitSub::PreCommit`** 执行上述脚本 |
| CI | **`.github/workflows/ci.yml`** — 顺序为 **fmt → build → test → clippy → doc**；**doc** 步骤当前**未**设置 **`RUSTDOCFLAGS=-D warnings`**；**无**单独 **MSVC** 步骤（与 **pre-commit** 可能不一致，须在 AC2 中裁决） |

### 须触摸的常见路径

| 区域 | 路径 |
|------|------|
| 钩子 | **`.githooks/pre-commit`** |
| CI | **`.github/workflows/ci.yml`** |
| Git 辅助 | **`xtask/src/git.rs`** |
| 需求/验收 | **`docs/requirements.md`**；若动 **NF-6** 语义则 **`xtask/src/acceptance/checks.rs`** |

### 架构合规（摘录）

- **Pre-commit** 继续以 **shell 脚本**为单一真源；**`git pre-commit`** **不**在 Rust 内复制一整套 clippy 参数（除非产品决定迁移）。

### 前序故事

- **2-2**（Git 辅助）已 **`done`**：**`git pre-commit`** 路径已建立。  
- **2-4**（一键验收）已 **`done`**：**NF-6** 与钩子内容交叉验证时可引用。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.5]
- [Source: `docs/requirements.md` — §4 表、`§7.2`]
- [Source: `.githooks/pre-commit`]
- [Source: `.github/workflows/ci.yml`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
