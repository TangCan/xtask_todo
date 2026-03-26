# Story 6.2：crates.io 与 cargo devshell

Status: review

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名终端用户，  
我希望从 **crates.io** 安装 **`xtask-todo-lib`** 并获得 **`cargo devshell`** 入口，  
以便与**发布策略**一致（**FR31**）。

## 映射需求

- **FR31**：通过包注册表安装主库（含 **`cargo devshell`** 入口，以发布策略为准）。
- **NFR-S3**：依赖 crates.io 与语义化版本；发布流程须能**复现**构建与检查（pre-commit / acceptance）。
- **NFR-R2**：破坏性变更须在**主版本**与 **CHANGELOG**（或等价发布说明）中可追溯。

## Acceptance Criteria

1. **Given** **`crates/todo/Cargo.toml`**（包名 **`xtask-todo-lib`**）含 **`[lib]`** 与 **`[[bin]] name = "cargo-devshell"`**  
   **When** 在仓库根执行 **`cargo publish -p xtask-todo-lib --dry-run`**（可加 **`--registry crates-io`**）  
   **Then** 预检通过；打包内容包含库与 **`cargo-devshell`** 二进制入口说明与 **`docs/publishing.md`** 一致（**FR31**，**NFR-S3**）。

2. **Given** **`cargo install xtask-todo-lib`**（版本以 **`Cargo.toml`** / **README** 最低版本说明为准，见 **`crates/todo/README.md`**）  
   **When** 安装完成且 **`PATH`** 含 **`~/.cargo/bin`**  
   **Then** 可执行 **`cargo devshell`**（或文档所载等价调用）；**`--version`/`-V`** 或 crate 版本与 **crates.io** 元数据一致（以实现为准）（**FR31**）。

3. **Given** **`default-features`**（**`beta-vm`** 等，见 **`Cargo.toml` `[features]`**）  
   **When** 用户选择 **`default-features = false`**  
   **Then** 行为与 **`README.md`「Devshell VM on Windows」** 一致；**不**在发布物中留下未文档化的隐式依赖（**FR31**）。

4. **Given** **语义化版本** bump 与破坏性变更  
   **When** 发布新版本  
   **Then** 存在**可追溯**的变更记录：**根目录 `CHANGELOG.md`**、**GitHub Release 说明** 或 **`publishing.md` 指向的等价流程** 至少一种；**NFR-R2** 满足（若当前仓库**无** `CHANGELOG.md`，本故事可**新增最小** `CHANGELOG.md` 或明确「仅以 Release 为准」并写入 **`publishing.md`**）。

5. **棕地**：**`cargo xtask publish`**（**`docs/publishing.md` §2.4**）为团队一键流程；本故事**不**替代维护者手动 **`cargo publish`**，但 AC 须与 **`publishing.md`**、**`xtask/src/publish.rs`**（若存在）行为对齐。

6. **回归**：**`cargo publish -p xtask-todo-lib --dry-run`**、**`cargo test -p xtask-todo-lib`**（及 **`6.1`** 的 MSVC **`cargo check`** 若仍适用）通过；**不**在本故事中执行真实 **`cargo publish`**（除非维护者显式执行）。

## Tasks / Subtasks

- [x] **预检**：**`cargo publish -p xtask-todo-lib --dry-run`**，记录警告/排除文件列表。
- [x] **安装演练**：干净环境 **`cargo install xtask-todo-lib --version <workspace>`**（或 **`--path crates/todo`**）验证 **`cargo devshell --help`**。
- [x] **文档**：**`crates/todo/README.md`** 与 **`docs/publishing.md`** 中 **crate 名 / 二进制名 / `cargo install` 段落** 一致。
- [x] **CHANGELOG / Release**：按 **NFR-R2** 选定单一事实来源并补链接。
- [x] **验证**：**`cargo publish --dry-run`**、**`cargo test -p xtask-todo-lib`**。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 包与二进制 | **`crates/todo/Cargo.toml`** — **`xtask-todo-lib`**、**`cargo-devshell`** |
| 发布文档 | **`docs/publishing.md`** |
| 用户说明 | **`crates/todo/README.md`** — **`cargo install`**、Windows / **`beta-vm`** |

### 架构合规（摘录）

- **`xtask`**、**`crates/devshell-vm`** 为 **`publish = false`**（**`publishing.md`**）；**不**误发布工作区私有 crate。

### 前序故事

- **6.1**：MSVC **`cargo check`**；发布前须仍通过，避免 **crates.io** 用户与交叉编译矩阵脱节。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 6 Story 6.2]
- [Source: `docs/publishing.md`]
- [Source: `crates/todo/Cargo.toml`]
- [Source: `crates/todo/README.md`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo publish -p xtask-todo-lib --dry-run --registry crates-io`
- `cargo install --path crates/todo --root /tmp/xtask_todo_install_default --force`
- `printf 'exit\n' | PATH="/tmp/xtask_todo_install_default/bin:$PATH" cargo devshell`
- `cargo install --path crates/todo --root /tmp/xtask_todo_install_nodefault --no-default-features --force`
- `printf 'exit\n' | PATH="/tmp/xtask_todo_install_nodefault/bin:$PATH" cargo devshell`
- `cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`
- `cargo test -p xtask-todo-lib -- --test-threads=1`

### Completion Notes List

- `cargo publish -p xtask-todo-lib --dry-run --registry crates-io` 预检通过；打包 `99 files`，并给出预期提示 `xtask-todo-lib@0.1.29 already exists`（版本已存在）与 dry-run 中止上传，符合 AC1/AC6（未执行真实发布）。
- 安装演练完成：分别在隔离 root（`/tmp/xtask_todo_install_default` 与 `/tmp/xtask_todo_install_nodefault`）执行 `cargo install --path crates/todo`（默认特性）与 `--no-default-features`；均成功安装 `cargo-devshell`，并通过 `printf 'exit\n' | cargo devshell` 验证 `cargo devshell` 入口可调用。
- 文档一致性对齐：保留 `crates/todo/README.md` 既有说明，并修正 `docs/publishing.md` 6.3 中与实现不一致的旧描述（此前误写“默认不含 beta-vm”）；现已明确默认包含 `beta-vm`，以及如何 `--no-default-features` 关闭。
- NFR-R2 追溯源已选定并写入：在 `docs/publishing.md` 增加“GitHub Release 说明为单一事实来源”（若未来引入 `CHANGELOG.md` 再切换规则），满足“可追溯至少一种”要求。
- 回归验证通过：`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc` 与 `cargo test -p xtask-todo-lib -- --test-threads=1` 全绿；与 6.1 的跨平台门禁保持一致。

### File List

- `docs/publishing.md`
- `_bmad-output/implementation-artifacts/6-2-crates-io-cargo-devshell.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
