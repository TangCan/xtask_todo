# Tab 补全（命令 + 路径）实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 dev_shell 中增加交互式 Tab 补全：按 Tab 时补全命令名与虚拟 FS 路径/文件名；交互模式用 rustyline，非交互模式保持 read_line。

**Architecture:** 增加 completion 模块：解析当前行与光标位置得到「当前 token + 是命令还是路径」；命令补全用固定列表过滤前缀；路径补全用 Vfs 的 resolve_to_absolute + list_dir + 前缀匹配。REPL 在 TTY 时用 rustyline::Editor + Helper（实现 Completer），非 TTY 时仍用 stdin.read_line()。Helper 每轮由 REPL 更新「cwd + 路径候选快照」避免长期借用 Vfs。

**Tech Stack:** Rust 2021，rustyline（约 17.x），标准库；可选 is-terminal 或 rustyline 自带的 TTY 检测。

**参考设计:** `docs/plans/2026-03-13-tab-completion-design.md`

---

## 前置条件

- 项目根目录为 `dev_shell`，已有 `src/repl.rs`、`src/command.rs`、`src/vfs.rs`、`src/main.rs`。
- 设计文档已阅读，明确「命令位置」与「路径位置」的判定规则。

---

## Task 1: 添加 rustyline 依赖

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加依赖**

在 `[dependencies]` 中增加：

```toml
rustyline = "17"
```

若 17 不可用，用 `cargo search rustyline` 查当前稳定版后替换版本号。

**Step 2: 验证**

```bash
cargo build
```

Expected: 编译通过。

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add rustyline dependency"
```

---

## Task 2: 补全上下文解析 — 分词与位置类型（TDD）

**Files:**
- Create: `src/completion.rs`
- Modify: `src/lib.rs`
- Create: `tests/completion_tests.rs`

**Step 1: 定义类型与失败测试**

在 `src/completion.rs` 中定义：

```rust
/// 当前输入位置是命令名还是路径（用于选择补全源）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,
    Path,
}

/// 从 (line, pos) 解析出的补全上下文：当前词的前缀，以及是命令还是路径
#[derive(Debug)]
pub struct CompletionContext {
    pub prefix: String,
    pub kind: CompletionKind,
    pub start: usize, // 当前词在 line 中的起始位置（用于 rustyline 的 replace start）
}
```

在 `tests/completion_tests.rs` 中写测试（先写测试再实现）：

```rust
use dev_shell::completion::completion_context;
use dev_shell::completion::CompletionKind;

#[test]
fn context_at_line_start_is_command() {
    let ctx = completion_context("hel", 3).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Command);
    assert_eq!(ctx.prefix, "hel");
    assert_eq!(ctx.start, 0);
}

#[test]
fn context_after_pipe_is_command() {
    let ctx = completion_context("ls | pw", 7).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Command);
    assert_eq!(ctx.prefix, "pw");
}

#[test]
fn context_after_cd_is_path() {
    let ctx = completion_context("cd /fo", 6).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Path);
    assert_eq!(ctx.prefix, "/fo");
}
```

**Step 2: 运行测试确认失败**

```bash
cargo test completion_context
```

Expected: 编译失败或 test 失败（函数未实现或返回错误）。

**Step 3: 实现 completion_context**

在 `src/completion.rs` 中实现 `pub fn completion_context(line: &str, pos: usize) -> Option<CompletionContext>`：

- 取 `line[..pos]`，按空格和 `|`、`<`、`>`、`2>` 分词（注意 `2>` 为一个 token）；得到 token 列表与每个 token 的 start 索引。
- 找出「光标所在」的 token（即 pos 落在的 token），其内容为 prefix，start 为 ReplacementStart。
- 判断 kind：若该 token 是行首 token（索引 0）或是紧接在 `|` 后的 token，则 `CompletionKind::Command`；否则若前一个 token 是 `cd`、`ls`、`cat`、`mkdir`、`touch`、`export-readonly`、`export_readonly` 或 `>`、`2>`、`<`，则 `CompletionKind::Path`；否则默认 `Path`（设计文档约定未知上下文做路径补全）。
- 返回 `Some(CompletionContext { prefix, kind, start })`。若 line 为空或 pos 越界可返回 `None`。

**Step 4: 运行测试**

```bash
cargo test completion_context
```

**Step 5: Commit**

```bash
git add src/completion.rs src/lib.rs tests/completion_tests.rs
git commit -m "feat(completion): completion context parsing (command vs path)"
```

---

## Task 3: 命令补全候选列表

**Files:**
- Modify: `src/completion.rs`

**Step 1: 定义命令名常量**

在 `completion.rs` 中定义静态列表（与 command.rs 中内置命令一致）：

```rust
const BUILTIN_COMMANDS: &[&str] = &[
    "pwd", "cd", "ls", "mkdir", "cat", "touch", "echo",
    "save", "export-readonly", "export_readonly", "exit", "quit", "help",
];
```

**Step 2: 实现命令补全**

```rust
pub fn complete_commands(prefix: &str) -> Vec<String> {
    let prefix = prefix.to_lowercase();
    BUILTIN_COMMANDS
        .iter()
        .filter(|c| c.to_lowercase().starts_with(prefix.as_str()))
        .map(|s| (*s).to_string())
        .collect()
}
```

**Step 3: 单元测试（可选）**

在 `tests/completion_tests.rs` 中：`complete_commands("he")` 应包含 `"help"`；`complete_commands("")` 返回所有命令。

**Step 4: Commit**

```bash
git add src/completion.rs tests/completion_tests.rs
git commit -m "feat(completion): command completion candidates"
```

---

## Task 4: 路径补全候选列表（依赖 Vfs 快照）

**Files:**
- Modify: `src/completion.rs`

**Step 1: 路径补全接口**

补全时不能长期持有 `&Vfs`，因此路径补全接受「已解析好的上下文」：父目录的 list_dir 结果由调用方传入（REPL 每轮在调用 readline 前用 vfs 填好）。定义：

```rust
/// 路径补全：prefix 为当前输入的路径片段（可含 /）；parent_entries 为父目录下列出的名字（或当前目录下的名字）
/// 返回以 prefix 最后一段为前缀的候选；若 prefix 含 /，则只匹配 last segment
pub fn complete_path(prefix: &str, parent_entries: &[(String, bool)]) -> Vec<String> {
    // parent_entries: (name, is_dir). 若 prefix 为空，返回所有；否则取 prefix 最后一段做前缀匹配。
    // 若唯一且为目录，候选可带 "/" 后缀（由调用方或 Helper 层加）
}
```

实现逻辑：若 prefix 包含 `/`，则 last_segment = prefix.rsplit('/').next().unwrap_or(prefix)；否则 last_segment = prefix。用 last_segment 对 parent_entries 的 name 做 starts_with 过滤；若候选唯一且 is_dir，可 push name + "/"。

**Step 2: 调用方如何提供 parent_entries**

在 REPL 层（或 Helper 内）每轮根据 vfs.cwd() 与当前 completion_context 的 prefix 解析父路径，调用 vfs.list_dir(父路径)，再对每个子节点判断是文件还是目录（需 Vfs 能「列出带类型的条目」或仅名字）。若 Vfs 当前只有 list_dir(path) -> Vec<String>，则先只返回名字，不区分目录/文件；后续可在 Vfs 增加 list_dir_with_type 或用 resolve_absolute 逐项判断。首版可简化为：parent_entries 仅为 `Vec<String>`，complete_path 只做前缀过滤，不追加 `/`。

**Step 3: 简化版 complete_path**

```rust
pub fn complete_path(prefix: &str, parent_names: &[String]) -> Vec<String> {
    let last = prefix.rsplit('/').next().unwrap_or(prefix);
    parent_names
        .iter()
        .filter(|n| n.starts_with(last))
        .cloned()
        .collect()
}
```

若 prefix 为空，则 last 为空，starts_with("") 对所有 name 为 true，即返回全部。符合预期。

**Step 4: 测试**

在 completion_tests 中：parent_names = ["foo", "foobar", "bar"]，complete_path("fo", parent_names) => ["foo", "foobar"]；complete_path("foo/", ["a", "b"]) => ["a", "b"]（或仅 ["a","b"] 不追加 /，首版均可）。

**Step 5: Commit**

```bash
git add src/completion.rs tests/completion_tests.rs
git commit -m "feat(completion): path completion from parent dir names"
```

---

## Task 5: Helper 与 Completer 实现（rustyline）

**Files:**
- Modify: `src/completion.rs`
- Modify: `src/lib.rs`

**Step 1: 查阅 rustyline 版本**

运行 `cargo doc --open -p rustyline` 或查 docs.rs，确认当前项目使用的 rustyline 中 `Completer`、`Helper`、`Candidate`、`Context` 的签名。以下按 rustyline 17 风格写；若不同则按实际 API 调整。

**Step 2: 定义 Helper 与上下文快照**

Helper 需要只读的「当前 cwd」和「当前路径候选列表」；由 REPL 每轮更新，避免持有 &mut Vfs。例如：

```rust
use std::sync::Mutex;

pub struct CompletionState {
    pub cwd: String,
    pub path_candidates: Vec<String>, // 当前补全路径时使用的父目录下的名字列表
}

pub struct DevShellHelper {
    state: Mutex<CompletionState>,
}
```

或不用 Mutex，改为 REPL 每轮创建新的 Helper 并传入 state（若 rustyline 的 set_helper 接受每次替换）。更简单做法：Helper 持有一个 `CompletionState` 结构，REPL 在每次 readline 前调用 `helper.update_state(vfs)`，在 update_state 里用 vfs 的 cwd 和当前行（若需要）填 path_candidates。但「当前行」在 readline 之前未知，所以 path_candidates 只能在 Tab 时现场用 vfs 计算——因此要么 Helper 持有 `*const Vfs` 或 `&Vfs`（只读），在 complete 回调里临时用 vfs.list_dir；要么 REPL 每轮不更新 path_candidates，只更新 cwd，路径补全时在 complete 里根据 prefix 和 cwd 现场调用「某个」Vfs。rustyline 的 Completer 是 `&self`，不能拿到 `&mut Vfs`。所以只能：

- 方案 A：Helper 持有 `&Vfs`（不可变引用）。在 REPL 中，Editor 和 Helper 与 vfs 共存：`Editor` 借入时不能同时借 `vfs` 为 mut，所以 run 的签名要改成分两阶段：先创建 Editor+Helper（Helper 持有 &Vfs），然后 loop 里 readline 不需要 &mut vfs，但 execute_pipeline 需要 &mut vfs——因此 readline 与 execute 不能同时持有 vfs。流程改为：loop { let line = editor.readline(...)?; 然后 drop(line) 后用 vfs 执行 pipeline }。这样 Helper 可以持有 `&Vfs`（在创建 Editor 时传入），因为 readline 期间不会调用 execute_pipeline。但 Editor::new() 时我们需要传入 helper，helper 需要 &Vfs；而 run(vfs: &mut Vfs, ...) 里 vfs 是 mut，不能同时借给 Helper 不可变。所以要么 run 签名改为 run(vfs: &Vfs, ...) 只读 vfs 用于补全，执行时再通过内部可变性（RefCell）或别的方式拿到 mut；要么 Helper 不持 vfs，只持 CompletionState（cwd + path_candidates），path_candidates 在「每次 readline 之前」由 REPL 根据「上一行的内容」或「默认空」填好——这样路径补全只能基于「上一轮」的状态，不理想。  
更稳妥：**Helper 持有 `Rc<RefCell<Vfs>>` 或 `Arc<Mutex<Vfs>>`**，在 complete 回调里临时 borrow 只读，调用 list_dir。这样 REPL 在 loop 里先 readline（Helper 内部 borrow vfs 只读），readline 返回后 REPL 再 borrow vfs 为 mut 执行 pipeline。需要把 vfs 包进 Rc<RefCell<Vfs>>，main 和 repl 都改用该类型。

为减少大改，**首版采用**：Helper 只做**命令补全**，不依赖 Vfs；路径补全留空或返回空列表。后续再引入 Rc<RefCell<Vfs>> 或类似做路径补全。这样 Task 5 只实现「命令补全 + rustyline Helper」，路径补全在 Task 6 用 RefCell 或 Arc<Mutex<Vfs>> 再接上。

**Step 2（续）：仅命令补全的 Helper**

- 定义 `DevShellHelper`（首版无字段或仅占位）。实现 `Completer`：`Candidate` 使用 rustyline 提供的类型（如 `String`）；`complete(&self, line, pos, ctx)` 内调 `completion_context(line, pos)`，若 `kind == Command` 则 `complete_commands(&ctx.prefix)` 并返回 `(ctx.start, candidates)`，否则返回空。
- 若 rustyline 要求 Helper 同时实现 `Hinter`、`Highlighter`、`Validator`，为它们提供空实现或默认行为（不提示、不高亮、不校验）。
- 在 `src/lib.rs` 中增加 `pub mod completion`。

**Step 3: 运行 build**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add src/completion.rs src/lib.rs
git commit -m "feat(completion): rustyline Helper with command completion only"
```

---

## Task 6: 路径补全接入 Vfs（可选：Rc<RefCell<Vfs>>）

**Files:**
- Modify: `src/completion.rs`
- Modify: `src/repl.rs`（见 Task 7）
- Modify: `src/main.rs`

**Step 1: 引入可共享 Vfs**

在 main 中创建 `let vfs = Rc::new(RefCell::new(vfs));`，repl::run 改为接受 `Rc<RefCell<Vfs>>`；在 run 内创建 Helper 时传入 `vfs.clone()`。Helper 持有 `Rc<RefCell<Vfs>>`，在 complete 中若 kind == Path，则 `vfs.borrow().list_dir(父路径)` 得到 parent_names，再 `complete_path(prefix, &parent_names)`。父路径由 prefix 解析：若 prefix 含 `/`，则父路径 = prefix 的 dirname（用 rfind('/') 截断）；否则父路径 = "."（当前目录）。用 vfs.borrow().resolve_to_absolute(父路径) 得到绝对路径，再 list_dir(该绝对路径)。list_dir 可能失败，返回空列表即可。

**Step 2: 测试**

手动运行，在 `cd /foo<Tab>` 下应有路径补全（若存在 /foo 下子节点）。

**Step 3: Commit**

```bash
git add src/completion.rs src/repl.rs src/main.rs
git commit -m "feat(completion): path completion via Rc<RefCell<Vfs>>"
```

---

## Task 7: REPL 分支 — 交互用 Editor，非交互用 read_line

**Files:**
- Modify: `src/repl.rs`
- Modify: `src/main.rs`

**Step 1: 判断是否 TTY**

使用 `rustyline::config::Config::default()` 或 `Editor::new()` 时，rustyline 通常会在非 TTY 下自动退化；或使用 `atty::is(Stream::Stdin)` / `is-terminal` crate 判断。若为 false，则 REPL 使用原有 `stdin.read_line(&mut line)` 逻辑；若为 true，则创建 `Editor::new()`，`editor.set_helper(Some(DevShellHelper::new(vfs))`（若 Task 6 完成则传 Rc<RefCell<Vfs>>），loop 内 `let prompt = format!("{} $ ", vfs.cwd());`，`match editor.readline(&prompt)`，`Ok(Some(line))` 则 trim 后走 parse + execute；`Ok(None)` 为 EOF 退出；`Err(e)` 写 stderr 并 continue 或 return。

**Step 2: 签名与兼容**

保留 `run(vfs, stdin, stdout, stderr)` 签名；vfs 改为 `Rc<RefCell<Vfs>>`（main 中先 `let vfs = Rc::new(RefCell::new(vfs))` 再传入）。内部：若 stdin 为 TTY，使用 Editor + Helper(vfs.clone())，不用 stdin 读行；否则用 `stdin.read_line()` 保持管道/脚本兼容。

**Step 3: 验证**

- `echo "exit" | cargo run`：应直接退出，无 Tab 交互。
- `cargo run` 在终端中：输入 `he<Tab>` 应补全为 `help`。

**Step 4: Commit**

```bash
git add src/repl.rs src/main.rs
git commit -m "feat(repl): use rustyline in TTY, read_line in pipe"
```

---

## Task 8: README 与测试收尾

**Files:**
- Modify: `README.md`
- Optional: `tests/completion_tests.rs` 补充路径补全用例

**Step 1: README**

在「Built-in commands」或「Features」中增加一句：支持 **Tab 补全**：交互式下按 Tab 可补全命令名与路径/文件名。

**Step 2: 全量测试**

```bash
cargo test
```

确保原有测试仍通过；completion_tests 全部通过。

**Step 3: Commit**

```bash
git add README.md tests/completion_tests.rs
git commit -m "docs: README mention Tab completion; completion tests"
```

---

## 执行方式说明

完成本计划后，可选择：

1. **Subagent-Driven（本会话）** — 每项任务派子代理执行，任务间代码评审。
2. **独立会话** — 新会话中使用 executing-plans 按任务批量执行并检查点评审。

请回复希望采用的执行方式（1 或 2）。
