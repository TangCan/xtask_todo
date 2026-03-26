# Story 4.4：关闭 VM 与宿主降级

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名开发者，  
我希望**关闭 VM**或选择**仅宿主路径**时仍可使用核心 devshell 能力，  
以便环境不全时仍可工作。

## 映射需求

- **FR23**（`epics.md` Story 4.4；PRD — VM 关闭 / 仅宿主仍可用）
- **NFR-R1**：**无 VM / Mode S** 主路径**不依赖** Lima/Podman（与 **4.1** 一致）

## Acceptance Criteria

1. **Given** **`VmConfig::from_env()`** 且 **`DEVSHELL_VM`** 为 **falsy**（**`off`/`0`/`false`** 等，见 **`config/mod.rs` `vm_enabled_from_env`**）  
   **When** **`SessionHolder::try_from_config`**  
   **Then** 返回 **`SessionHolder::Host`**（**`HostSandboxSession`**），**不**尝试连接 γ/β（**FR23**，**NFR-R1**）。

2. **Given** **`DEVSHELL_VM`** 开启但 **`DEVSHELL_VM_BACKEND=host`**（或 **`auto`/`空`** 导致 **`VmConfig::use_host_sandbox()`** 为真，见 **`config/mod.rs`）  
   **When** **`try_from_config`**  
   **Then** 同样返回 **`Host`**；**`rustup`/`cargo`** 走 **`sandbox::run_rust_tool`**（**4.1**）（**FR23**）。

3. **Given** **`try_session_rc`** 失败（例如 Unix 上 **`lima`** 选中但 **`limactl`** 不可用）  
   **When** 入口使用 **`try_session_rc_or_host`**（**`vm/mod.rs`**）  
   **Then** stderr 提示后回退 **`SessionHolder::Host`**，REPL/脚本**可继续**；行为与 **`docs/devshell-vm-gamma.md`**「VM 未启用或回退」一致（**FR23**，**NFR-R1**）。

4. **Given** 核心 devshell 能力（**内置命令**、**`todo`**、**脚本**）在 **`Host`** 下  
   **When** 无 Lima/Podman  
   **Then** **不**因 VM 缺失而**直接退出进程**（除非调用方显式把 `try_session_rc` 失败当致命错误）；与 **`devshell/mod.rs`** 各分支（`try_session_rc` vs `try_session_rc_or_host`）一致（**FR23**）。

5. **棕地**：逻辑集中在 **`vm/config/mod.rs`**（**`VmConfig`**、**`use_host_sandbox`**）、**`vm/mod.rs`**（**`try_from_config`**、**`try_session_rc_or_host`**）及 **`devshell/mod.rs`** 入口；本故事以 **核对 AC、补测试或文档、澄清 `auto`/`host` 语义**为主，**不**在本故事中实现 **4.5** 的 **Mode P** 会话 JSON（仅交叉引用降级规则）。

6. **回归**：**`cargo test -p xtask-todo-lib`**（**`vm::config`** 测试）、**`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`** 通过。

## Tasks / Subtasks

- [ ] **棕地核对**：画 **环境变量 → `VmConfig` → `SessionHolder` 变体** 真值表（含 **`DEVSHELL_VM` 未设置** 时默认 **enabled=true** 的行为）。
- [ ] **入口**：确认 **`run_main_from_args`** / **`run_with`** 在 Windows 与 Unix 上对 **`try_session_rc`** 失败的处理是否一致；必要时统一为 **`try_session_rc_or_host`** 或文档说明差异。
- [ ] **文档**：若 **`auto`** 与 README 描述不一致，同步 **`docs/devshell-vm-gamma.md`** 或 **`crates/todo/README.md`**。
- [ ] **验证**：`cargo test -p xtask-todo-lib vm`、`cargo clippy -p xtask-todo-lib --all-targets -- -D warnings`。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| 配置 | **`crates/todo/src/devshell/vm/config/mod.rs`** — **`VmConfig`**、**`use_host_sandbox`** |
| 会话构造 | **`vm/mod.rs`** — **`SessionHolder::try_from_config`**、**`try_session_rc_or_host`** |
| 宿主执行 | **`session_host.rs`** + **`sandbox::run_rust_tool`** |

### 架构合规（摘录）

- **降级**须**显式**（日志/stderr），避免静默切换到 **`Host`** 却未写入 **`DEVSHELL_WORKSPACE_ROOT`** 等环境时产生歧义（以实现为准）。

### 前序故事

- **4-1**：宿主 **`rustup`/`cargo`** 语义；本故事确保**在 VM 关/失败时**仍落到该路径。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 Story 4.4]
- [Source: `docs/requirements.md` — §1.1、§5.8]
- [Source: `docs/devshell-vm-gamma.md` — 简单心智模型 §5–6]
- [Source: `crates/todo/src/devshell/vm/mod.rs`]
- [Source: `crates/todo/src/devshell/vm/config/mod.rs`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
