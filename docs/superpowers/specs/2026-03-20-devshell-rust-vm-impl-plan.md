# devshell Rust 工具链（轻量隔离环境）— 实现计划

**设计依据**：`docs/superpowers/specs/2026-03-20-devshell-rust-vm-design.md`

---

## 前置条件

- 在 `crates/todo` 下开发；devshell 位于 `src/devshell/`，现有 VFS（`vfs/`）、`command.rs`（`run_builtin_core`）、`repl`。
- VFS 已有 `copy_tree_to_host`（导出到宿主机目录）；需新增「从宿主机目录同步回 VFS」或复用/扩展现有能力。
- 首期仅实现轻量隔离（fd-only on Linux，随机路径 + 0700 回退）；Docker/Podman 后端为可选扩展，不列入本计划。

---

## Task 1：VFS 导出到临时目录

**目标**：将 VFS 以当前 cwd 为根的子树导出到宿主机临时目录，目录唯一且权限 0700。

**步骤**：

1. 在 `devshell` 下新增模块（如 `sandbox.rs` 或 `isolate.rs`），提供「创建临时目录」：在 `std::env::temp_dir()` 或 `$TMPDIR` 下创建 `devshell_<uuid>`（如 `uuid::Uuid::new_v4()` 或 `std::time` + 随机），权限 `0o700`（仅当前用户）。
2. 复用或调用 VFS 的 `copy_tree_to_host(vfs_path, host_dir)`：以 `vfs.cwd()` 为 `vfs_path`，将整棵子树写入临时目录。
3. 返回临时目录的路径（`PathBuf`）供后续使用；若导出失败（创建目录失败、写文件失败），返回 `Err` 并清理已创建目录。

**验证**：单元测试：构造含若干文件/子目录的 VFS，设置 cwd，导出后断言临时目录下存在对应文件与目录结构。

**文件**：`crates/todo/src/devshell/sandbox.rs`（或 `isolate.rs`）、或扩展现有 `vfs` 的导出接口。

---

## Task 2：隔离层（Linux fd-only / 非 Linux 回退）

**目标**：Linux 上临时目录创建后 unlink，仅保留 fd；非 Linux 保留路径。提供「在导出目录中执行子进程」的接口。

**步骤**：

1. **Linux**：导出完成后，对临时目录调用 `std::fs::remove_dir` 的**父级下的该目录名**（即 unlink 目录条目）；父进程通过 `std::fs::File::open` 在 unlink 前打开目录（或使用 `openat` 等，依 std 能力），保留 `File`（即 fd）。子进程 cwd 通过 `/proc/self/fd/<n>`（n 为 fd 编号）传入，或使用 `rustix`/`nix` 等库的 `fexecve`/`openat` 类 API 将 fd 作为 cwd；若 std 的 `Command::current_dir` 只接受路径，则 Linux 首期可回退为「不 unlink、仅随机路径 + 0700」，在文档中注明「严格 fd-only 需后续依赖 os 扩展」。
2. **非 Linux**：不 unlink；保留 `PathBuf`，子进程使用 `Command::current_dir(&path)`。
3. 抽象：提供 `run_in_export_dir(path: Option<&Path>, fd: Option<&File>, program: &str, args: &[String], stdin/stdout/stderr)`：若 fd 可用且平台支持则以 fd 为 cwd，否则用 path；执行 `program`、`args`，透传 stdio。

**验证**：Linux：创建目录后 unlink，断言路径不再可访问、fd 仍可读写目录内容。非 Linux：断言子进程 cwd 为导出目录。

**文件**：`sandbox.rs`（或 `isolate.rs`）；可选依赖 `rustix`/`nix` 用于 fd-as-cwd（若 std 无法满足）。

**说明**：若首期在 Linux 上实现「fd 作为 cwd」成本高，可统一采用「随机路径 + 0700」，在实现计划与文档中注明「fd-only 为后续增强」。

---

## Task 3：从临时目录同步回 VFS

**目标**：将导出目录中的变更（相对导出根的新增、修改、删除）写回 VFS 对应子树。

**步骤**：

1. 遍历临时目录（通过保留的 path 或 fd）；对每个文件：计算相对导出根的路径，在 VFS 中对应路径写入内容（若为新增则创建父目录与文件，若为修改则覆盖）。
2. 删除：遍历 VFS 当前子树，若宿主机导出目录中已不存在该相对路径的节点，则从 VFS 中删除（或实现「差分」：先列出导出目录全部相对路径，再与 VFS 子树对比，删除 VFS 中有而导出目录没有的）。
3. 实现时注意：导出目录可能含 `target/`、`.git` 等大或无关目录；设计约定是否全部同步回 VFS 或排除部分目录（如 `target/`）；首期可全部同步，后续可加排除列表。

**验证**：导出空 VFS 到临时目录，在临时目录中创建文件、子目录；同步回 VFS 后断言 VFS 中存在对应节点与内容。

**文件**：`sandbox.rs` 或 `vfs` 扩展（如 `load_tree_from_host`）；与 Task 1 的导出形成对称。

---

## Task 4：查找宿主二进制与执行流程

**目标**：在 PATH 中查找 `rustup`、`cargo`（及可选 `rustc`）；实现「导出 → 执行 → 同步 → 清理」完整流程。

**步骤**：

1. 通过 `which` 或 `std::env::var_os("PATH")` + 遍历 PATH 查找 `rustup`、`cargo`；未找到则返回错误（不导出）。
2. 实现 `run_rust_tool(vfs, cwd_vfs_path, program, args, stdin, stdout, stderr)`（或拆成 rustup/cargo 两个入口）：  
   (a) 导出 VFS 子树（cwd_vfs_path）到临时目录（Task 1）；  
   (b) 在隔离层中启动子进程，cwd 为导出目录，执行 `program` + `args`，透传 stdin/stdout/stderr（Task 2）；  
   (c) 子进程结束后，将导出目录同步回 VFS（Task 3）；  
   (d) 关闭 fd（若有）、删除临时目录（若有路径）；无论 (b)(c) 成功与否都执行 (d)。
3. 子进程退出码非零时，`run_rust_tool` 返回 `Err`（或等效），以便脚本 `set -e` 能终止。

**验证**：集成测试：VFS 中创建含 `Cargo.toml` 的目录并 cd 进去，调用 `run_rust_tool` 执行 `cargo build`（若 CI 有 cargo）；断言同步后 VFS 中存在 `target/` 或预期产物；或 mock 子进程仅 touch 一个文件，断言同步后 VFS 中有该文件。

**文件**：`sandbox.rs`（或 `isolate.rs`）、`command.rs` 中调用。

---

## Task 5：内置命令 `rustup`、`cargo`

**目标**：在 REPL 与脚本中可使用 `rustup`、`cargo` 内置，参数原样转发。

**步骤**：

1. 在 `run_builtin_core` 中增加分支：`"rustup"`、`"cargo"`。取 `argv[0]` 为命令名，`argv[1..]` 为参数列表。
2. 调用 Task 4 的 `run_rust_tool(vfs, vfs.cwd(), "rustup"/"cargo", &argv[1..], stdin, stdout, stderr)`。
3. 重定向：若当前 `SimpleCommand` 有 stdin/stdout/stderr 重定向，在调用 `run_rust_tool` 前设置好（与现有 `run_builtin_with_streams` 一致，子进程继承或使用传入的 stream）。
4. 可选：增加 `rustc` 分支，同上。

**验证**：REPL 或脚本中执行 `cargo --version`（或 `rustup --version`），断言 stdout 包含版本信息；执行 `cargo new foo` 后断言 VFS 中有 `foo/` 及 `Cargo.toml`。

**文件**：`crates/todo/src/devshell/command.rs`。

---

## Task 6：错误处理与清理

**目标**：导出失败、二进制未找到、子进程失败、同步失败时统一报错；单次调用结束前必做清理。

**步骤**：

1. 定义 `BuiltinError` 新变体（如 `RustupNotFound`、`CargoNotFound`、`ExportFailed`、`SyncBackFailed`）；在 `run_rust_tool` 与内置分支中返回相应错误，stderr 写入提示（如 "rustup not found in PATH"）。
2. 使用 `Drop` 或显式 `defer` 风格：将「关闭 fd、删除临时目录」封装为守卫或始终在返回路径调用的函数，确保子进程被 kill 或 panic 时也尽量清理（如 `std::panic::catch_unwind` 后仍执行清理，或文档约定「panic 时可能残留临时目录」）。
3. 同步失败时：可选保留临时目录路径到 stderr 便于调试；实现时约定是否部分写回。

**验证**：测试「PATH 中无 rustup 时调用 rustup」返回错误且不创建临时目录；测试「导出后子进程失败」仍执行清理（临时目录被删除或 fd 关闭）。

**文件**：`command.rs`（BuiltinError）、`sandbox.rs`（清理逻辑）。

---

## Task 7：测试与文档

**目标**：覆盖导出、同步、内置调用的单元/集成测试；README 增加 Rust 工具链小节。

**步骤**：

1. 单元测试：导出空 VFS / 含文件 VFS 到临时目录，断言目录结构与内容；从临时目录同步回 VFS，断言 VFS 内容。
2. 集成测试：devshell 内置 `cargo --version` 或 `rustup --version` 能输出；可选：`cargo new` 后检查 VFS（若 CI 有 cargo）。
3. Linux 隔离测试（可选）：验证 unlink 后路径不可访问（若实现了 fd-only）。
4. README（或 devshell 文档）增加「Rust 工具链」小节：说明 `rustup`/`cargo` 内置、隔离方式（fd-only / 随机路径）、导出与同步语义、与 Docker/Podman 的差异及可选容器后端；示例：`cd my_project` → `cargo build`。

**文件**：`devshell/sandbox.rs` 的 `#[cfg(test)]`、`devshell/tests/` 下新增或现有集成测试、`README.md`。

---

## 实现顺序与依赖

| 顺序 | 任务               | 依赖     |
|------|--------------------|----------|
| 1    | Task 1 VFS 导出    | 无       |
| 2    | Task 2 隔离层      | Task 1   |
| 3    | Task 3 同步回 VFS  | 无（可与 1 并行设计） |
| 4    | Task 4 执行流程    | Task 1, 2, 3 |
| 5    | Task 5 内置命令    | Task 4   |
| 6    | Task 6 错误与清理  | Task 4, 5 |
| 7    | Task 7 测试与文档  | 全部     |

建议顺序：Task 1 → Task 3（导出与同步对称，可先打通）→ Task 2 → Task 4 → Task 5 → Task 6 → Task 7。若 Task 2 的 fd-only 首期简化为「仅随机路径」，则 Task 2 与 Task 4 可更快落地。

---

*计划编写完成；可按任务顺序实现，每步通过测试与验证后再进行下一任务。Docker/Podman 后端为可选扩展，不包含在本计划内。*
