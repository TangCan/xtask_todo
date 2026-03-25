# Story 3.2：内置文件与目录操作

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望使用**白名单内置命令**做导航、列目录、读写文件、建目录，  
以便在受控环境中操作工作区。

## 映射需求

- **FR15**（`epics.md` Story 3.2；`docs/requirements.md` **§5.4** — `pwd` / `cd` / `ls` / `mkdir` / `cat` / `touch` / `echo` 等）
- **NFR-S2**：**非**任意宿主 shell；内置命令经 **`run_builtin`** / **`Vfs`** 白名单路径（与 **§5.8**、侧车 IPC 约束一致）

## Acceptance Criteria

1. **Given** 当前 **logical cwd** 与 **`Vfs`** 状态（**Mode S / Mode P** 以 **`SessionHolder`** 与 **`docs/requirements.md` §5** 为准）  
   **When** 执行 **`pwd` / `cd` / `ls` / `mkdir`**（参数与路径以 **`help`** 与实现一致）  
   **Then** 仅影响 **会话工作区视图**（VFS 与/或 VM 映射），行为与 **`docs/requirements.md` §5.4** 表「导航与列出」一致；非法路径或越权访问返回**可读错误**，不崩溃（**FR15**）。

2. **Given** 同上会话上下文  
   **When** 执行 **`cat` / `touch` / `echo`**（含重定向时以 **`command/dispatch`** 与 **`workspace`** 辅助为准）  
   **Then** 读写与 **`docs/requirements.md` §5.4**「文件与输出」一致；**不**通过隐式 shell 执行任意字符串（**FR15**，**NFR-S2**）。

3. **Given** 用户输入**非**白名单命令名（或拼写错误）  
   **When** 进入分派逻辑  
   **Then** 明确失败（未知命令 / 用法），**不**回退到宿主 **`sh -c`** / 任意解释器（**NFR-S2**）。

4. **Given** **`builtin_impl`** / **`types::BuiltinError`** 已定义错误类型  
   **When** 文件/目录操作失败  
   **Then** 错误信息可映射到 REPL/脚本的可观察输出（stderr 或命令失败语义），与 **`design.md`** 错误处理约定一致（**FR15**）。

5. **棕地**：实现位于 **`crates/todo/src/devshell/command/`**（**`dispatch/builtin_impl.rs`**、**`dispatch/workspace.rs`**、**`dispatch/mod.rs`**）、**`vfs/`**；本故事以 **核对 AC、补单测、边界修正**为主。**管道阶段字节上限**（**`PIPELINE_INTER_STAGE_MAX_BYTES`**）的专项验收属 **Epic 3 Story 3.3**，本故事仅**避免**与之矛盾的实现。

6. **回归**：**`cargo test -p xtask-todo-lib`**（含 **`devshell`** / **`command`** 相关）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：阅读 **`dispatch/builtin_impl.rs`** 与 **`requirements.md` §5.4** 表；列出 **白名单命令** 与参数矩阵；标出与 **`help`** 文案差异。
- [ ] **VFS**：核对 **`cd`/`ls`** 在 **Mode S / Mode P** 下路径解析（**`workspace`/`vfs`**）；缺测试则补 **`devshell` 测试**或记手工矩阵。
- [ ] **安全**：确认无**非白名单**命令路径；grep **`Command::new("sh")`** 等仅出现在 **`rustup`/`cargo`/`source`** 等**已文档化**分支，而非 **`cat`/`ls`** 等。
- [ ] **验证**：`cargo test -p xtask-todo-lib`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 路径 |
|------|------|
| 内置分派 | **`crates/todo/src/devshell/command/dispatch/builtin_impl.rs`** — **`run_builtin_core`** |
| 重定向与 workspace 文件读写 | **`dispatch/mod.rs`**、**`dispatch/workspace.rs`** |
| VFS | **`crates/todo/src/devshell/vfs/`** |

### 架构合规（摘录）

- **内置命令**经 **`run_builtin`** 进入 **`Vfs`**；**禁止**将用户任意字符串交给宿主 shell 执行（**NFR-S2**）。

### 前序故事

- **3-1**（交互/脚本）：提供 REPL/脚本入口；本故事聚焦 **§5.4** 文件类内置命令。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.2]
- [Source: `docs/requirements.md` — §5.4、§5.8（与 NFR-S2 交叉）]
- [Source: `docs/design.md` — devshell / VFS / 错误]
- [Source: `crates/todo/src/devshell/command/`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
