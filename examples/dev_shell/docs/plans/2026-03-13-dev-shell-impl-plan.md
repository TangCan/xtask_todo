# 开发用 Shell 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现跨平台（Linux / macOS / Windows）的开发用 Shell，使用 Rust，带虚拟 FS、.bin 持久化与按需只读导出；首版仅内置命令，不执行宿主进程。

**Architecture:** 分层：虚拟 FS（内存树 + 路径解析）→ 序列化层（.bin 读写）→ 命令层（内置命令 + 管道/重定向）→ REPL → CLI（main + 加载/保存 .bin）。参考设计文档：`docs/plans/2026-03-13-dev-shell-design.md`。

**Tech Stack:** Rust（2021 edition），标准库为主；可选 `tempfile` 用于 export-readonly 的临时目录；测试用 `cargo test`。

---

## 前置条件

- 已安装 Rust（`rustc --version`、`cargo --version` 可用）。
- 在项目根目录执行本计划（即 `dev_shell` 目录）。

---

## Task 1: 创建 Cargo 项目骨架

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: 创建 Cargo.toml**

在项目根目录执行：

```bash
cargo init
```

若已存在 `Cargo.toml`，则确保内容包含：

```toml
[package]
name = "dev_shell"
version = "0.1.0"
edition = "2021"

[dependencies]
# 暂无；后续可按需加 tempfile

[dev-dependencies]
# 暂无
```

**Step 2: 占位 main 与 lib**

`src/main.rs`：

```rust
fn main() {
    println!("dev_shell 0.1.0");
}
```

`src/lib.rs`：

```rust
// 占位，后续导出 vfs、serialization、command、repl 等
```

**Step 3: 验证**

```bash
cargo build
cargo run
```

Expected: 编译成功，输出 `dev_shell 0.1.0`。

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/lib.rs
git commit -m "chore: init Rust project skeleton"
```

---

## Task 2: 虚拟 FS — 路径归一化（TDD）

**Files:**
- Create: `src/vfs.rs`
- Create: `tests/vfs_tests.rs`

**Step 1: 写失败测试**

在 `tests/vfs_tests.rs` 中：

```rust
use dev_shell::vfs::normalize_path;

#[test]
fn normalize_path_unix_style() {
    assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
    assert_eq!(normalize_path("foo/bar"), "foo/bar");
}

#[test]
fn normalize_path_windows_backslash() {
    assert_eq!(normalize_path("foo\\bar"), "foo/bar");
    assert_eq!(normalize_path("C:\\foo\\bar"), "/foo/bar");
}
```

**Step 2: 运行测试确认失败**

```bash
cargo test normalize_path --no-run 2>&1
```

Expected: 编译失败，因 `dev_shell::vfs::normalize_path` 不存在。

**Step 3: 最小实现**

在 `src/lib.rs` 添加：

```rust
pub mod vfs;
```

在 `src/vfs.rs` 中：

```rust
/// 将用户输入路径归一化为 Unix 风格：\ -> /，Windows 盘符去掉或变为根
pub fn normalize_path(input: &str) -> String {
    let s = input.replace('\\', "/");
    let s = s.trim_start_matches(|c: char| c.is_ascii_alphabetic() && s.chars().nth(1) == Some(':'));
    let s = s.trim_start_matches(':');
    if s.is_empty() || s == "/" {
        return "/".to_string();
    }
    let s = if s.starts_with("/") { s } else { s };
    let mut parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
    let mut out: Vec<&str> = Vec::new();
    for p in parts {
        if p == ".." {
            out.pop();
        } else {
            out.push(p);
        }
    }
    if out.is_empty() {
        return "/".to_string();
    }
    "/".to_string() + &out.join("/")
}
```

修正：Windows 盘符处理需更稳妥。简化版（首版仅处理 `\` 和 `..`）：

```rust
pub fn normalize_path(input: &str) -> String {
    let s = input.replace('\\', "/");
    // 去掉 Windows 盘符前缀，如 C:
    let s = if let Some(rest) = s.strip_prefix(|c: char| c.is_ascii_alphabetic()) {
        rest.strip_prefix(':').unwrap_or(rest)
    } else {
        &s
    };
    let s = s.trim_start_matches('/');
    let mut out: Vec<&str> = Vec::new();
    for p in s.split('/') {
        match p {
            "" | "." => {}
            ".." => { out.pop(); }
            _ => out.push(p),
        }
    }
    if out.is_empty() {
        return "/".to_string();
    }
    "/".to_string() + &out.join("/")
}
```

**Step 4: 运行测试**

```bash
cargo test normalize_path
```

Expected: PASS（若 Windows 盘符用例与实现不一致，可先调测试或实现，保证通过）。

**Step 5: Commit**

```bash
git add src/vfs.rs src/lib.rs tests/vfs_tests.rs
git commit -m "feat(vfs): path normalization with tests"
```

---

## Task 3: 虚拟 FS — 节点与树类型定义

**Files:**
- Modify: `src/vfs.rs`

**Step 1: 定义类型（无测试新增，仅类型）**

在 `src/vfs.rs` 顶部增加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Dir { name: String, children: Vec<Node> },
    File { name: String, content: Vec<u8> },
}

impl Node {
    pub fn name(&self) -> &str {
        match self {
            Node::Dir { name, .. } => name,
            Node::File { name, .. } => name,
        }
    }
    pub fn is_dir(&self) -> bool { matches!(self, Node::Dir { .. }) }
    pub fn is_file(&self) -> bool { matches!(self, Node::File { .. }) }
}
```

**Step 2: 验证**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/vfs.rs
git commit -m "feat(vfs): Node enum Dir/File"
```

---

## Task 4: 虚拟 FS — Vfs 结构体与根目录

**Files:**
- Modify: `src/vfs.rs`

**Step 1: 添加 Vfs 与构造函数**

```rust
pub struct Vfs {
    root: Node,
    cwd: String, // 绝对路径，如 "/" 或 "/foo/bar"
}

impl Vfs {
    pub fn new() -> Self {
        Vfs {
            root: Node::Dir { name: "".to_string(), children: vec![] },
            cwd: "/".to_string(),
        }
    }
    pub fn cwd(&self) -> &str { &self.cwd }
    pub fn root(&self) -> &Node { &self.root }
}
```

**Step 2: 验证**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/vfs.rs
git commit -m "feat(vfs): Vfs struct and root"
```

---

## Task 5: 虚拟 FS — 解析绝对路径得到节点（TDD）

**Files:**
- Modify: `src/vfs.rs`
- Modify: `tests/vfs_tests.rs`

**Step 1: 写失败测试**

在 `tests/vfs_tests.rs` 中增加：

```rust
use dev_shell::vfs::{Vfs, Node};

#[test]
fn resolve_absolute_path_root() {
    let vfs = Vfs::new();
    let n = vfs.resolve_absolute("/").unwrap();
    assert!(n.is_dir());
}

#[test]
fn resolve_absolute_path_missing_returns_err() {
    let vfs = Vfs::new();
    assert!(vfs.resolve_absolute("/foo").is_err());
}
```

**Step 2: 运行测试**

```bash
cargo test resolve_absolute
```

Expected: 失败（方法不存在或未实现）。

**Step 3: 实现 resolve_absolute**

在 `impl Vfs` 中：

```rust
/// 根据绝对路径（已归一化）解析到节点引用；只读
pub fn resolve_absolute(&self, path: &str) -> Result<&Node, ()> {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return Ok(&self.root);
    }
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let mut current = &self.root;
    for seg in segments {
        let Node::Dir { children, .. } = current else { return Err(()) };
        let next = children.iter().find(|c| c.name() == seg).ok_or(())?;
        current = next;
    }
    Ok(current)
}
```

注意：返回 `&Node` 会与后续“可变操作”冲突，首版可改为返回路径对应的节点是否存在的 bool，或返回 clone。更简单：先实现 `resolve_absolute` 返回 `Option<()>` 仅表示存在，或返回 `Result<Node, ()>` 用 clone。为减少 clone，可先实现“存在性检查”和“列出子节点”，命令层再基于“路径 + 列举”工作。此处为简化，改为返回 `Result<bool, ()>` 表示是否为目录（用于 ls/mkdir 等），或直接返回 `Option<&Node>`。Rust 中从根遍历并返回引用会受生命周期和后续 mut 限制，建议：**返回 `Result<Node, ()>`（clone）** 便于首版实现。

修改实现为：

```rust
pub fn resolve_absolute(&self, path: &str) -> Result<Node, ()> {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return Ok(self.root.clone());
    }
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let mut current = &self.root;
    for seg in segments {
        let Node::Dir { children, .. } = current else { return Err(()) };
        let next = children.iter().find(|c| c.name() == seg).ok_or(())?;
        current = next;
    }
    Ok(current.clone())
}
```

**Step 4: 运行测试**

```bash
cargo test resolve_absolute
```

**Step 5: Commit**

```bash
git add src/vfs.rs tests/vfs_tests.rs
git commit -m "feat(vfs): resolve_absolute path lookup"
```

---

## Task 6: 虚拟 FS — 解析相对路径为绝对路径

**Files:**
- Modify: `src/vfs.rs`

**Step 1: 实现 resolve_to_absolute**

```rust
/// 将任意路径（相对或绝对）归一化并解析为绝对路径字符串
pub fn resolve_to_absolute(&self, path: &str) -> String {
    let path = normalize_path(path);
    if path.starts_with('/') && path != "/" {
        return path;
    }
    if path == "/" {
        return self.cwd.clone();
    }
    let base = self.cwd.trim_end_matches('/');
    let p = path.trim_start_matches('/');
    normalize_path(&format!("{}/{}", base, p))
}
```

**Step 2: 测试（可选在 vfs_tests 中加一条）**

```bash
cargo test
```

**Step 3: Commit**

```bash
git add src/vfs.rs
git commit -m "feat(vfs): resolve_to_absolute for cwd-relative paths"
```

---

## Task 7: 虚拟 FS — 创建目录 mkdir（TDD）

**Files:**
- Modify: `src/vfs.rs`
- Modify: `tests/vfs_tests.rs`

**Step 1: 失败测试**

```rust
#[test]
fn mkdir_creates_path() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo/bar").unwrap();
    let n = vfs.resolve_absolute("/foo/bar").unwrap();
    assert!(n.is_dir());
}
```

**Step 2: 运行**

```bash
cargo test mkdir
```

Expected: 失败。

**Step 3: 实现 mkdir**

在 `Vfs` 上需要可变地遍历到父节点并插入子目录。实现方式：在 `Vfs` 上增加 `fn mkdir(&mut self, path: &str) -> Result<(), ()>`，通过 `resolve_parent_mut` 找到父目录（可变引用），再 push 新 Dir。由于 Rust 中从 root 一路取 `&mut` 较繁琐，可写辅助方法 `get_node_mut_at_path(&mut self, path) -> Option<&mut Node>` 返回路径指向的节点（若为目录则可用于添加子节点）；mkdir 逻辑为：解析父路径（path 的 dirname），取父节点为 Dir 则在其 children 中 push 新 Node::Dir（名字为 path 的 basename），若已存在同名则返回 Err 或 Ok 视需求而定。

简化：实现 `mkdir_all` 风格，路径中缺失的父目录一并创建。

```rust
pub fn mkdir(&mut self, path: &str) -> Result<(), ()> {
    let abs = self.resolve_to_absolute(path);
    let segments: Vec<&str> = abs.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok(());
    }
    let mut current = &mut self.root;
    for (i, seg) in segments.iter().enumerate() {
        let Node::Dir { children, .. } = current else { return Err(()) };
        let exists = children.iter_mut().find(|c| c.name() == *seg);
        current = match exists {
            Some(n) => n,
            None => {
                children.push(Node::Dir { name: (*seg).to_string(), children: vec![] });
                children.last_mut().unwrap()
            }
        };
    }
    Ok(())
}
```

**Step 4: 运行测试**

```bash
cargo test mkdir
```

**Step 5: Commit**

```bash
git add src/vfs.rs tests/vfs_tests.rs
git commit -m "feat(vfs): mkdir and mkdir_all style"
```

---

## Task 8: 虚拟 FS — 写文件 write_file（TDD）

**Files:**
- Modify: `src/vfs.rs`
- Modify: `tests/vfs_tests.rs`

**Step 1: 失败测试**

```rust
#[test]
fn write_file_creates_file() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    vfs.write_file("/foo/f", b"hello").unwrap();
    let n = vfs.resolve_absolute("/foo/f").unwrap();
    match &n { Node::File { content, .. } => assert_eq!(content.as_slice(), b"hello"), _ => panic!() }
}
```

**Step 2: 实现 write_file**

```rust
pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), ()> {
    let abs = self.resolve_to_absolute(path);
    let segments: Vec<&str> = abs.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(());
    }
    let (parent_path, name) = segments.split_at(segments.len() - 1);
    let name = name[0];
    let mut current = &mut self.root;
    for seg in parent_path {
        let Node::Dir { children, .. } = current else { return Err(()) };
        current = children.iter_mut().find(|c| c.name() == *seg).ok_or(())?;
    }
    let Node::Dir { children, .. } = current else { return Err(()) };
    if let Some(existing) = children.iter_mut().find(|c| c.name() == name) {
        *existing = Node::File { name: name.to_string(), content: content.to_vec() };
    } else {
        children.push(Node::File { name: name.to_string(), content: content.to_vec() });
    }
    Ok(())
}
```

**Step 3: 运行测试**

```bash
cargo test write_file
```

**Step 4: Commit**

```bash
git add src/vfs.rs tests/vfs_tests.rs
git commit -m "feat(vfs): write_file"
```

---

## Task 9: 虚拟 FS — 读文件 read_file、列目录 list_dir

**Files:**
- Modify: `src/vfs.rs`

**Step 1: 实现 read_file 和 list_dir**

```rust
pub fn read_file(&self, path: &str) -> Result<Vec<u8>, ()> {
    let n = self.resolve_absolute(&self.resolve_to_absolute(path))?;
    match n {
        Node::File { content, .. } => Ok(content.clone()),
        _ => Err(()),
    }
}

pub fn list_dir(&self, path: &str) -> Result<Vec<String>, ()> {
    let n = self.resolve_absolute(&self.resolve_to_absolute(path))?;
    match n {
        Node::Dir { children, .. } => Ok(children.iter().map(|c| c.name().to_string()).collect()),
        _ => Err(()),
    }
}
```

**Step 2: 实现 cd**

```rust
pub fn set_cwd(&mut self, path: &str) -> Result<(), ()> {
    let abs = self.resolve_to_absolute(path);
    let n = self.resolve_absolute(&abs)?;
    if !n.is_dir() {
        return Err(());
    }
    self.cwd = if abs == "/" { "/".to_string() } else { abs };
    Ok(())
}
```

**Step 3: 验证**

```bash
cargo build
cargo test
```

**Step 4: Commit**

```bash
git add src/vfs.rs
git commit -m "feat(vfs): read_file, list_dir, set_cwd"
```

---

## Task 10: 虚拟 FS — touch（创建空文件）

**Files:**
- Modify: `src/vfs.rs`

**Step 1: 实现 touch**

```rust
pub fn touch(&mut self, path: &str) -> Result<(), ()> {
    self.write_file(path, &[])
}
```

**Step 2: 验证**

```bash
cargo test
```

**Step 3: Commit**

```bash
git add src/vfs.rs
git commit -m "feat(vfs): touch"
```

---

## Task 11: .bin 序列化 — 格式与序列化（TDD）

**Files:**
- Create: `src/serialization.rs`
- Modify: `src/lib.rs`
- Create: `tests/serialization_tests.rs`

**Step 1: 定义格式**

魔数 `DEVS`（4 字节）+ 版本 u8（1 字节）+ 根节点递归序列化。节点：1 字节类型（0=Dir, 1=File）+ 名字长度 u16 + 名字 UTF-8 + 若 Dir 则子节点个数 u32 + 递归；若 File 则内容长度 u64 + 内容。

**Step 2: 失败测试**

```rust
use dev_shell::vfs::{Vfs, Node};
use dev_shell::serialization;

#[test]
fn roundtrip_empty_vfs() {
    let vfs = Vfs::new();
    let bytes = serialization::serialize(&vfs).unwrap();
    assert!(bytes.starts_with(b"DEVS"));
    let loaded = serialization::deserialize(&bytes).unwrap();
    assert!(loaded.resolve_absolute("/").is_ok());
}
```

**Step 3: 实现 serialize/deserialize**

在 `src/serialization.rs` 中实现（或用现成库如 bincode；为减少依赖，首版可用手写简单二进制格式）：先写魔数+版本，再递归写 root，同时写 cwd 字符串长度和内容。反序列化时校验魔数版本，再递归读回树和 cwd。

（具体字节布局略，实现时保证 roundtrip 测试通过。）

**Step 4: 运行测试并提交**

```bash
cargo test roundtrip
git add src/serialization.rs src/lib.rs tests/serialization_tests.rs
git commit -m "feat(serialization): .bin format serialize/deserialize"
```

---

## Task 12: .bin — 与 Vfs 的 load/save 接口

**Files:**
- Modify: `src/vfs.rs` 或 `src/serialization.rs`

**Step 1: 在 Vfs 或 serialization 提供 load/save**

例如：

```rust
// serialization.rs
pub fn save_to_file(vfs: &Vfs, path: &std::path::Path) -> std::io::Result<()> {
    let bytes = serialize(vfs).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, bytes)
}
pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Vfs> {
    let bytes = std::fs::read(path)?;
    deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
```

**Step 2: 验证**

```bash
cargo test
```

**Step 3: Commit**

```bash
git add src/serialization.rs
git commit -m "feat(serialization): save_to_file / load_from_file"
```

---

## Task 13: 解析器 — 行拆分为命令与重定向（TDD）

**Files:**
- Create: `src/parser.rs`
- Modify: `src/lib.rs`
- Create: `tests/parser_tests.rs`

**Step 1: 定义简单 AST**

例如：一条管线由多个「简单命令」组成；简单命令 = 可执行名 + 参数列表 + 可选 stdin/stdout/stderr 重定向。

```rust
pub struct Redirect { pub fd: u8; pub path: String; }
pub struct SimpleCommand { pub argv: Vec<String>; pub redirects: Vec<Redirect>; }
pub struct Pipeline { pub commands: Vec<SimpleCommand>; }
```

**Step 2: 失败测试**

```rust
use dev_shell::parser::parse_line;

#[test]
fn parse_simple() {
    let p = parse_line("echo hello").unwrap();
    assert_eq!(p.commands.len(), 1);
    assert_eq!(p.commands[0].argv, ["echo", "hello"]);
}

#[test]
fn parse_redirect_out() {
    let p = parse_line("echo hi > out").unwrap();
    assert_eq!(p.commands[0].redirects.len(), 1);
    assert_eq!(p.commands[0].redirects[0].path, "out");
}
```

**Step 3: 实现 parse_line**

按空格分词，识别 `>`, `2>`, `<` 为重定向；`|` 为管道分隔符。输出 `Pipeline { commands: vec![SimpleCommand { argv, redirects }, ...] }`。

**Step 4: 运行测试并提交**

```bash
cargo test parse
git add src/parser.rs src/lib.rs tests/parser_tests.rs
git commit -m "feat(parser): parse line to pipeline with redirects"
```

---

## Task 14: 命令执行层 — 内置命令分发（TDD）

**Files:**
- Create: `src/command.rs`
- Modify: `src/lib.rs`

**Step 1: 定义执行环境**

```rust
pub struct ExecContext<'a> {
    pub vfs: &'a mut Vfs,
    pub stdin: &'a mut dyn std::io::Read,
    pub stdout: &'a mut dyn std::io::Write,
    pub stderr: &'a mut dyn std::io::Write,
}
```

**Step 2: 实现 pwd, cd, ls, mkdir, cat, touch, echo**

每个命令从 `ExecContext` 读 vfs 和 stdin/stdout/stderr，根据 argv 和 redirects 执行。例如 `pwd` 写 `ctx.vfs.cwd()` 到 stdout；`cat` 从 vfs 读文件写到 stdout；重定向在调用命令前由上层打开 vfs 文件并替换 stdout/stderr/stdin。

**Step 3: 测试**

在 `tests/command_tests.rs` 中为 pwd、cd、ls、mkdir、cat、touch、echo 各写至少一条测试（给定 Vfs 和 argv，断言 stdout 或 vfs 状态）。

**Step 4: Commit**

```bash
git add src/command.rs src/lib.rs tests/command_tests.rs
git commit -m "feat(command): builtin pwd, cd, ls, mkdir, cat, touch, echo"
```

---

## Task 15: 管道执行

**Files:**
- Modify: `src/command.rs`

**Step 1: 实现 execute_pipeline**

对 `Pipeline` 中每个 `SimpleCommand`：若有重定向，则从 vfs 打开对应文件作为 stdin/stdout/stderr；管道则用内存缓冲区（或 `std::process::Command` 不适用，因不调宿主进程）——即用 `Vec<u8>` 做缓冲区，前一个命令的 stdout 写入 buffer，后一个命令的 stdin 从 buffer 读。顺序执行各简单命令，传递缓冲区。

**Step 2: 测试**

`echo a | cat` 应使最终 stdout 为 `a\n`。

**Step 3: Commit**

```bash
git add src/command.rs
git commit -m "feat(command): pipeline execution"
```

---

## Task 16: export-readonly 内置命令

**Files:**
- Modify: `src/command.rs`
- Modify: `src/vfs.rs`（若需要递归复制到宿主目录的辅助函数）

**Step 1: 实现 export_readonly**

- 使用 `std::env::temp_dir()` 创建子目录（如 `dev_shell_export_XXXX`）。
- 递归将虚拟 FS 中指定路径下的目录和文件复制到该临时目录（仅写一次，只读语义对“复制”而言即不再从该目录读回）。
- 将临时目录的绝对路径写入 stdout，供用户使用。

**Step 2: 测试**

在测试中调用 export_readonly，检查临时目录存在且包含预期文件内容。

**Step 3: Commit**

```bash
git add src/command.rs src/vfs.rs
git commit -m "feat(command): export-readonly to host temp dir"
```

---

## Task 17: save / exit 与 REPL 集成

**Files:**
- Create: `src/repl.rs`
- Modify: `src/lib.rs`

**Step 1: 实现 REPL 循环**

- 读取一行（或从脚本文件读）。
- 调用 `parse_line` 得到 `Pipeline`。
- 调用 `execute_pipeline`（传入当前 `Vfs` 和 stdin/stdout/stderr）。
- 内置命令 `save [path]` 在 command 层调用 serialization::save_to_file；`exit`/`quit` 返回“退出”标志，main 退出循环。

**Step 2: 可选：退出前提示保存**

若设计文档要求“可选提示保存”，在收到 exit 时若未保存则打印提示。

**Step 3: Commit**

```bash
git add src/repl.rs src/lib.rs
git commit -m "feat(repl): loop with parse and execute, save/exit"
```

---

## Task 18: main — 加载 .bin、运行 REPL、保存

**Files:**
- Modify: `src/main.rs`

**Step 1: CLI 参数**

- 无参数：从空 Vfs 启动；或从当前目录下默认 `.dev_shell.bin` 加载（若存在）。
- 一个参数：视为 .bin 路径，加载后启动；若文件不存在则从空 Vfs 启动并提示。

**Step 2: 启动 REPL**

调用 `repl::run(&mut vfs, &mut stdin, &mut stdout, &mut stderr)`，直到 exit。

**Step 3: 退出时可选保存**

若用户通过 `save` 已指定路径，退出时可按该路径再保存一次；或仅依赖用户显式 `save` 不自动保存。

**Step 4: 验证**

```bash
cargo run
# 输入 pwd、mkdir /foo、save test.bin、exit
# 再次 cargo run -- test.bin，输入 ls /，应看到 foo
```

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): load .bin, run REPL, optional save on exit"
```

---

## Task 19: 跨平台与文档

**Files:**
- Modify: `README.md`（若存在）或 Create: `README.md`
- 可选: `.github/workflows/ci.yml` 用于 Linux/macOS/Windows 的 `cargo test`

**Step 1: README**

说明：项目为开发用 Shell，跨平台，虚拟 FS，.bin 持久化，内置命令列表，使用示例（启动、save、export-readonly）。

**Step 2: CI（可选）**

```yaml
# 示例：matrix strategy 下 run: cargo test
```

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: README and usage"
```

---

## 执行方式说明

完成本计划后，可选择：

1. **Subagent-Driven（本会话）** — 每项任务派一个子代理执行，任务间做代码评审（使用 @skill-subagent-driven-development）。
2. **独立会话** — 在新会话中打开本计划，使用 @skill-executing-plans 按任务批量执行并做检查点评审。

请回复希望采用的执行方式（1 或 2）。
