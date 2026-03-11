# Clippy 分析与修正说明

本文档记录在开启 `clippy::pedantic` 与 `clippy::nursery` 后对项目进行的 Clippy 检查、问题分析与对应修正。

---

## 1. 检查方式

- **命令**：`cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery`
- **范围**：workspace 下所有 target（`crates/todo` 库与测试、`xtask` 二进制）。
- **目标**：在更严格的 lint 级别下零告警，便于后续在 CI 中采用 `-D warnings` 或统一启用部分 pedantic/nursery 规则。

---

## 2. 问题与修正概览

| 位置 | Lint | 问题简述 | 修正方式 |
|------|------|----------|----------|
| `crates/todo` | 见下表 | 文档、const、Self、match | 见各小节 |
| `xtask` | 见下表 | format 内联、无意义 Result 包装 | 见各小节 |

---

## 3. crates/todo 修正详情

### 3.1 store.rs

| Lint | 说明 | 修正 |
|------|------|------|
| **doc_markdown** | 文档中的标识符（如 `created_at`）未用反引号，不利于与代码一致、可点击。 | 将 `by created_at` 改为 `` by `created_at` ``。 |

```diff
- /// Returns all todos in creation order (by created_at).
+ /// Returns all todos in creation order (by `created_at`).
```

### 3.2 lib.rs

| Lint | 说明 | 修正 |
|------|------|------|
| **missing_const_for_fn** | 纯计算、无 I/O 的函数可标为 `const fn`，便于在常量上下文使用。 | `TodoId::as_u64`、`TodoList::with_store` 改为 `const fn`。 |
| **use_self** | 在 impl 内重复写类型名（如 `TodoError::InvalidInput`）可简写为 `Self::...`。 | `Display for TodoError` 的 match 中改为 `Self::InvalidInput`、`Self::NotFound`。 |
| **match_wildcard_for_single_variants** | 仅剩一个变体时用 `_` 会隐藏未来新增变体，不利于可维护性。 | 测试中 `_ => panic!("expected NotFound")` 改为显式 `TodoError::InvalidInput => panic!(...)`。 |

**代码变更摘要：**

- `pub fn as_u64` → `pub const fn as_u64`
- `pub fn with_store(store: S)` → `pub const fn with_store(store: S)`
- `TodoError::InvalidInput` / `TodoError::NotFound` → `Self::InvalidInput` / `Self::NotFound`
- 两处测试：`_ => panic!(...)` → `TodoError::InvalidInput => panic!(...)`

---

## 4. xtask 修正详情

### 4.1 main.rs

| Lint | 说明 | 修正 |
|------|------|------|
| **uninlined_format_args** | 单变量可直接写在格式串中（`"{x}"`），更简洁且符合当前风格。 | 所有 `format!("...", x)` / `println!("...", x)` 改为内联形式 `format!("...{x}...")`。 |
| **unnecessary_wraps** | 函数仅返回 `Ok(())`、无错误路径，返回 `Result` 无实际意义。 | `cmd_run` 改为返回 `()`，在 `run()` 的 `XtaskSub::Run` 分支中调用后写 `Ok(())`。 |

**涉及位置与修改：**

- `eprintln!("error: {}", e)` → `eprintln!("error: {e}")`
- `format!("{}s", s)` → `format!("{s}s")`
- `format!("  用时 {}", s)` → `format!("  用时 {s}")`
- `format!("  创建 {}  完成 {}{}", ...)` → `format!("  创建 {created}  完成 {completed}{took}")`
- `format!("  创建 {}", created)` → `format!("  创建 {created}")`
- `println!("\x1b[33m{}\x1b[0m", line)` → `println!("\x1b[33m{line}\x1b[0m")`
- `println!("{}", line)` → `println!("{line}")`
- `println!("Completed [{}]", id)` → `println!("Completed [{id}]")`
- `println!("Deleted [{}]", id)` → `println!("Deleted [{id}]")`
- `fn cmd_run(...) -> Result<(), ...>` → `fn cmd_run(...)`，`run()` 中 `XtaskSub::Run(args) => { cmd_run(args); Ok(()) }`

---

## 5. 验证

- **Clippy**：`cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery` 通过，无告警。
- **测试**：`cargo test` 全部通过（含 `crates/todo` 的 8 个单元测试）。

---

## 6. 后续建议

- **已实施**：CI（`.github/workflows/ci.yml`）已固定使用 `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings`，告警视为错误。
- 若需放宽某条规则，可在对应 crate 或模块顶部使用 `#[allow(clippy::lint_name)]`，并在本文档中注明原因。
- 可与 `docs/improvements-and-rust-style.md` 中的「启用 Clippy 并修复告警」建议对照，将 CI 与本地检查流程固化下来。
