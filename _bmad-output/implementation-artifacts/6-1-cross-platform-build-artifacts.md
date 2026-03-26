# Story 6.1：跨平台可编译交付物

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名用户或 CI，  
我希望在 **Linux、macOS、Windows（MSVC）** 上均能**编译通过**主库与相关二进制，  
以免平台漂移（**FR30**）。

## 映射需求

- **FR30**：在声明的目标平台上获取**可编译**的库与二进制交付物（以产品与 **`requirements`** 声明为准）。
- **NF-5** / **D8**（**`docs/acceptance.md`**）：**`xtask-todo-lib`** 对 **`x86_64-pc-windows-msvc`** 可 **`cargo check`**（交叉编译自检），与 **pre-commit**、**§7.2** 一致。

## Acceptance Criteria

1. **Given** **`rustup target add x86_64-pc-windows-msvc`** 已安装  
   **When** 执行 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**  
   **Then** 成功 **exit 0**；**无**仅宿主目标下能通过、MSVC 下失败的 **`cfg`/API 遗漏**（**FR30**，**NF-5**）。

2. **Given** 工作区默认目标（Linux/macOS 本地）  
   **When** **`cargo build` / `cargo test` / `cargo clippy --all-targets`**（策略以 **`Cargo.toml`** 与 **`.githooks/pre-commit`** 为准）  
   **Then** 与 **CI / pre-commit** 描述一致；**不**引入**仅**单一平台可用的依赖或 **`cfg`** 组合而未文档化（**FR30**）。

3. **Given** **`.githooks/pre-commit`**（**`design.md` §1.6**、**`requirements` §4**）  
   **When** 完整执行 hook  
   **Then** 含 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`** 作为最后一步（与当前脚本一致）；若变更检查范围，须同步 **NF-5** 与 **`docs/acceptance.md` D8**（**FR30**）。

4. **Given** **`cargo xtask acceptance`**（或 **`docs/acceptance.md`** 自动化矩阵）  
   **When** 运行 **NF-5 / D8** 相关项  
   **Then** 行为与 **`docs/acceptance-report.md`** 或 **SKIP** 规则一致（**无 target 时 SKIP** 等）（**NF-5**）。

5. **棕地**：交叉编译门槛已在 **`requirements §1.2 / §7.2`**、**`crates/todo/README.md`**（Windows）说明；本故事以 **核对 AC、补 CI 缺口、修复 MSVC 回归** 为主，**不**在本故事中完成 **6.2** 的 **crates.io 发布** 流程。

6. **回归**：本地 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**、**`cargo test`**、**`cargo clippy --all-targets -- -D warnings`**（或等价 pre-commit 子集）通过；记录若环境缺失的 **SKIP**。

## Tasks / Subtasks

- [ ] **矩阵**：**`xtask-todo-lib`** 特性（**`default` / `beta-vm`** 等）× **宿主 triple** × **`x86_64-pc-windows-msvc`** — 明确 **`cargo check`** 命令列表（**最小**）。
- [ ] **xtask 包**：若产品要求 **Windows 上编译 `xtask` 二进制**，在 AC 中单独列出 **`cargo check -p xtask --target …`**（否则在文档中说明「发布主库为 `xtask-todo-lib`」）。
- [ ] **文档**：**`README`** / **`design.md`** 与 **pre-commit** 三步是否一致；**`acceptance-report`** 是否反映 **NF-5**。
- [ ] **验证**：上述 **`cargo check`** + **`cargo test -p xtask-todo-lib`**（及全工作区策略）。

## Dev Notes

### 棕地现状（摘录）

| 区域 | 说明 |
|------|------|
| Pre-commit | **`.githooks/pre-commit`** — fmt、clippy、doc、test、**MSVC `cargo check`** |
| 验收 | **`docs/acceptance.md`** — **NF-5**、**D8**；**`xtask/src/acceptance/`** |
| 库 | **`crates/todo`** — **`cfg(unix)` / Windows** 分支（**`vm`** 等） |

### 架构合规（摘录）

- **覆盖率工具**（**`test-coverage.md`**）**不**替代 MSVC **`cargo check`**；二者职责分离。

### 前序故事

- **Epic 2**（pre-commit/CI 对齐）与 **Epic 4**（VM **`cfg`**）变更若影响 MSVC，须在本故事回归中验证。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 6 Story 6.1]
- [Source: `docs/requirements.md` — §1.2、§4、§7.2]
- [Source: `docs/acceptance.md` — NF-5、D8]
- [Source: `docs/test-cases.md` — TC-NF-5、TC-X-GIT-2]
- [Source: `.githooks/pre-commit`]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
