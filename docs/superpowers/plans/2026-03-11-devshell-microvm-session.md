# Devshell microVM 会话（γ → β）Implementation Plan

> **状态（历史文档）：** 下文 **checkbox** 为 **2026-03 前后** 分步追踪快照；**主线实现与后续 Mode P / guest-primary / β 侧车** 已并入 **`[2026-03-20-devshell-guest-primary-workspace.md](./2026-03-20-devshell-guest-primary-workspace.md)`** 与 **`[requirements.md](../../requirements.md)`**、**`[test-cases.md](../../test-cases.md)`**。未勾选项 **不**表示当前仓库仍待办；以 **guest-primary 计划**、**requirements** 与 **`cargo xtask acceptance`** 为准。  
> 规格背景仍可读：**`[2026-03-11-devshell-microvm-session-design.md](../specs/2026-03-11-devshell-microvm-session-design.md)`**、**IPC 草案** **`[2026-03-11-devshell-vm-ipc-draft.md](../specs/2026-03-11-devshell-vm-ipc-draft.md)`**。

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional **session-scoped VM execution** for `rustup`/`cargo` per `docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md`, starting with **γ (external CLI orchestration)** and later **β (sidecar `devshell-vm` + IPC)**; default **`DEVSHELL_VM=off`** keeps current host-directory `sandbox` behavior.

**Architecture:** Introduce a **`VmExecutionSession`** abstraction: when VM mode is on, the REPL holds (or lazily starts) a session that owns a **host staging directory** + **γ backend** (e.g. `limactl shell` with a named instance) or **β IPC client**. **Sync engine** implements §3.2: full push at session ready, incremental push before / pull after each rust command, full pull on exit. **Host sandbox** (`sandbox.rs`) remains the implementation for `DEVSHELL_VM=off` and for fallback.

**Tech Stack:** Rust 2021, existing `xtask-todo-lib` devshell modules; γ uses **subprocess** to Lima / platform CLI (configurable); β adds **new crate** `devshell-vm` + **JSON-lines or JSON-RPC** over Unix socket / named pipe (detailed in β chunk). Spec reference: `@docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md`

**Note (UX vs spec wording):** Spec says VM starts when REPL starts; this plan uses **lazy VM start on first `rustup`/`cargo`** unless `DEVSHELL_VM_EAGER=1`, to avoid taxing `pwd`/`help`-only sessions. Document the env var in user docs.

**Tech Stack:** `serde_json` (β protocol), optional `which` or keep manual PATH; no new hard dependency for γ beyond documenting external CLIs.

---

## Chunk 0: File map (read before tasks)

| Path | Responsibility |
|------|----------------|
| `crates/todo/src/devshell/mod.rs` | `pub mod vm;` export |
| `crates/todo/src/devshell/vm/mod.rs` | `VmMode`, `VmExecutionSession` trait, re-exports |
| `crates/todo/src/devshell/vm/config.rs` | `DEVSHELL_VM`, `DEVSHELL_VM_BACKEND`, paths, eager flag |
| `crates/todo/src/devshell/vm/sync.rs` | Push/pull full + incremental (pure + tests) |
| `crates/todo/src/devshell/vm/session_host.rs` | **Null / host-only** session (delegates to existing `run_rust_tool`) |
| `crates/todo/src/devshell/vm/session_gamma.rs` | γ: shell out to `limactl`/`ssh` template (Linux first) |
| `crates/todo/src/devshell/repl.rs` | Session drop on exit; thread session into `ExecContext` or global slot |
| `crates/todo/src/devshell/command/dispatch.rs` | `run_rust_tool_builtin` calls VM session or `sandbox::run_rust_tool` |
| `crates/todo/src/devshell/sandbox.rs` | Unchanged public API for host path; may extract shared `export`/`sync` helpers if deduping |
| `docs/design.md`, `docs/requirements.md`, `docs/dev-container.md` | User-facing VM vs host |
| `docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md` | Normative design |

---

## Chunk 1: γ — Config, sync primitives, host fallback wiring

### Task 1.1: VM config module

**Files:**
- Create: `crates/todo/src/devshell/vm/config.rs`
- Modify: `crates/todo/src/devshell/mod.rs` (add `mod vm; pub mod vm` or `pub use vm::...`)

- [ ] **Step 1: Add `VmConfig` struct** with `enabled: bool` from `DEVSHELL_VM` (`on`/`1`/`true` vs off), `backend: String` from `DEVSHELL_VM_BACKEND` (default `auto`), `eager: bool` from `DEVSHELL_VM_EAGER`, `lima_instance` from `DEVSHELL_VM_LIMA_INSTANCE` (default `devshell-rust`).

```rust
// config.rs — shape only
pub struct VmConfig {
    pub enabled: bool,
    pub backend: String,
    pub eager_start: bool,
    pub lima_instance: String,
}
impl VmConfig {
    pub fn from_env() -> Self { /* std::env::var */ }
}
```

- [ ] **Step 2: Unit test** `from_env` parses `DEVSHELL_VM=on` / missing → disabled.

Run: `cargo test -p xtask-todo-lib vm::config::tests::...`

- [ ] **Step 3: Commit** `feat(devshell): add VM config from environment`

---

### Task 1.2: Sync engine (full + incremental)

**Files:**
- Create: `crates/todo/src/devshell/vm/sync.rs`
- Uses: existing `Vfs`, `sandbox::sync_host_dir_to_vfs`, `vfs.copy_tree_to_host` patterns from `crates/todo/src/devshell/sandbox.rs`

- [ ] **Step 1: Implement `push_full(vfs, vfs_root, host_dir)`** — delegate to `vfs.copy_tree_to_host(vfs_root, host_dir)` (clear `host_dir` subtree first or use fresh subdir per session; **spec**: single workspace dir reused → **empty `host_dir` contents** before full push except tool mounts are outside this tree).

Document: workspace host path is **only** project files; `RUSTUP_HOME` mount is handled by γ backend, not copied into workspace.

- [ ] **Step 2: Implement `pull_full(host_dir, vfs_root, vfs)`** — reuse `sandbox::sync_host_dir_to_vfs` logic or call it (refactor `sync_host_dir_to_vfs` to `pub(crate)` if needed).

- [ ] **Step 3: Implement `push_incremental`** — MVP: compare **relative paths + file size + mtime** from vfs walk vs host; write changed files only. **Tests:** VFS with two files, mutate one in memory, incremental push updates one host file.

- [ ] **Step 4: Implement `pull_incremental`** — MVP: walk `host_dir`, for each file if mtime newer than session watermark **or** always pull files under `target/`; simplest correct approach for γ: **walk all files under `host_dir`**, `read` + `vfs.write_file` / mkdir (same as sync but scoped). Optimize later.

- [ ] **Step 5: Run** `cargo test -p xtask-todo-lib vm::sync::` and `cargo clippy -p xtask-todo-lib -- -D warnings`

- [ ] **Step 6: Commit** `feat(devshell): vm workspace push/pull helpers`

---

### Task 1.3: `VmExecutionSession` trait + host session

**Files:**
- Create: `crates/todo/src/devshell/vm/mod.rs`
- Create: `crates/todo/src/devshell/vm/session_host.rs`

- [ ] **Step 1: Define trait**

```rust
pub trait VmExecutionSession {
    fn ensure_ready(&mut self, vfs: &Vfs, vfs_cwd: &str) -> Result<(), VmError>;
    fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<std::process::ExitStatus, VmError>;
    fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError>;
}
```

- [ ] **Step 2: `HostSandboxSession`** implements trait by calling existing `sandbox::run_rust_tool` (no VM). `ensure_ready` no-op; `shutdown` no-op.

- [ ] **Step 3: Unit test** Host session runs `true` via `run_rust_tool` path (reuse sandbox test pattern).

- [ ] **Step 4: Commit** `feat(devshell): VmExecutionSession + host implementation`

---

### Task 1.4: Thread session through REPL + dispatch

**Files:**
- Modify: `crates/todo/src/devshell/repl.rs`
- Modify: `crates/todo/src/devshell/command/dispatch.rs`
- Modify: `crates/todo/src/devshell/command/types.rs` (if `ExecContext` needs `vm_session: Option<&mut dyn VmExecutionSession>` — prefer **`Rc<RefCell<Option<Box<dyn VmExecutionSession>>>>`** on REPL side to avoid GAT pain; or enum `SessionHolder { Host, Gamma(Box<...>) }`)

- [ ] **Step 1: Add `SessionHolder` enum** in `vm/mod.rs`: `Host`, `Gamma(GammaSession)` (stub struct for Task 2).

- [ ] **Step 2: `repl::run`** creates `SessionHolder::Host` when `!VmConfig::from_env().enabled`; when enabled, start with `Host` still until γ implemented, or stub `Gamma` that returns error "not implemented" — **prefer** keep **Host** until Task 2.1 implements Gamma.

- [ ] **Step 3: Pass `&mut SessionHolder` into `execute_pipeline` / `ExecContext`** — extend `ExecContext` with optional `vm: Option<&mut SessionHolder>`.

- [ ] **Step 4: `run_rust_tool_builtin`** branches: if VM enabled and session is Gamma, call session trait; else `sandbox::run_rust_tool`.

- [ ] **Step 5: On REPL exit** (`StepResult::Exit` / EOF), call `session.shutdown(&mut vfs, cwd)` when VM mode.

- [ ] **Step 6: Tests** `DEVSHELL_VM=off` existing tests unchanged; add one test `DEVSHELL_VM=on` with backend `host` alias if needed.

- [ ] **Step 7: Commit** `feat(devshell): plumb VM session holder through REPL`

---

## Chunk 2: γ — Lima (Linux) backend + docs

### Task 2.1: Gamma session skeleton (subprocess)

**Files:**
- Create: `crates/todo/src/devshell/vm/session_gamma.rs`
- Add: `docs/devshell-vm-gamma.md` (prerequisites: install Lima, example `lima.yaml` snippet)

- [ ] **Step 1: `GammaSession` fields** — `host_workspace: PathBuf`, `lima_instance: String`, `config: VmConfig`.

- [ ] **Step 2: `ensure_ready`** — `std::process::Command` run `limactl start <instance>` (idempotent), then **rsync or copy** initial push: actually spec says data on **guest disk** — γ shortcut: use Lima **mounted workspace** from host directory = `host_workspace`; guest sees mount at `/workspace`. So **no separate guest disk file** in γ: **host directory is synced to mount**. Aligns with "guest /workspace" being the mount point of host export. Document deviation: γ uses **bind mount** of host staging dir instead of qcow2 image; β moves to real disk image.

- [ ] **Step 3: `run_rust_tool`** — `limactl shell <instance> -- cwd=/workspace -- cargo ...` with env forwarding minimal set (`PATH` inside guest). Mount host `RUSTUP_HOME`/`CARGO_HOME` via Lima `mounts:` in YAML (document user must edit template once).

- [ ] **Step 4: Before command:** `push_incremental`; **after:** `pull_incremental`; use `host_workspace` as staging.

- [ ] **Step 5: `shutdown`** — `limactl stop <instance>` (optional `delete` behind flag).

- [ ] **Step 6: Integration test** — `#[ignore]` + comment: requires Lima; or mock `Command` with test hook (hard) — **use `#[cfg(feature = "vm-gamma-test")]`** optional; default **manual checklist** in doc.

- [ ] **Step 7: Commit** `feat(devshell): gamma Lima session (Linux)`

---

### Task 2.2: macOS / Windows γ notes (documentation-only task)

- [ ] **Step 1:** Extend `docs/devshell-vm-gamma.md` with **Multipass** / **Lima on Mac** / **WSL2** command equivalents; no code requirement for parity in same PR.

- [ ] **Step 2: Commit** `docs(devshell): gamma backend per-OS notes`

---

## Chunk 3: β — Sidecar outline (no full code in first PR)

### Task 3.1: IPC protocol draft (spec-only in repo)

**Files:**
- Create: `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`

- [ ] **Step 1:** Define JSON-lines messages: `{"op":"handshake","version":1}` / `session_start` / `push` / `exec` / `pull` / `shutdown` with base64 or path references for bulk file transfer (prefer **paths on shared staging dir** + `op: sync_request` to avoid huge payloads).

- [ ] **Step 2: Commit** `docs: devshell-vm IPC draft`

---

### Task 3.2: New crate scaffold `devshell-vm`

**Files:**
- Create: `crates/devshell-vm/Cargo.toml`, `crates/devshell-vm/src/main.rs` (stub loop)

- [ ] **Step 1:** Add workspace member in root `Cargo.toml` if workspace exists; else crate inside `crates/` only.

- [ ] **Step 2:** `main` prints `devshell-vm 0.0.0 stub` and exit 0.

- [ ] **Step 3: Commit** `chore: add devshell-vm crate stub`

---

### Task 3.3: Replace γ subprocess with IPC client (future milestone)

- [ ] **Step 1:** Implement `session_beta.rs` implementing `VmExecutionSession` using Unix socket client.

- [ ] **Step 2:** Feature-flag `beta-vm` in `xtask-todo-lib`.

- [ ] **Step 3: Commit** (separate release train)

---

## Chunk 4: Docs & requirements

### Task 4.1: User-facing documentation

**Files:**
- Modify: `docs/design.md` §2.5
- Modify: `docs/requirements.md` (rustup/cargo row)
- Modify: `README.md` Rust toolchain bullet
- Modify: `docs/dev-container.md`

- [ ] **Step 1:** Document `DEVSHELL_VM`, `DEVSHELL_VM_BACKEND`, Lima template, **γ uses host-mounted workspace**, β will differ.

- [ ] **Step 2: Commit** `docs: document VM session mode`

---

## Chunk 5: Error handling & cargo failure + pull

Per design §6: **after guest non-zero exit, still run `pull_incremental`** (or full pull) so `target/` partial results return to VFS unless pull itself fails.

- [ ] **Step 1:** In `VmExecutionSession::run_rust_tool` impls, structure:

```rust
let status = inner_exec(...);
let _ = pull_incremental(...); // log error but still return status
status
```

- [ ] **Step 2: Test** with mock: force pull error, assert stderr contains expected.

- [ ] **Step 3: Commit** `fix(devshell): pull workspace after failed cargo in VM mode`

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-03-11-devshell-microvm-session.md`. Ready to execute?**

- Prefer **Chunk 1** first (all tests green before Lima).
- Run **plan-document-reviewer** (superpowers) on each chunk if available.
- After γ works on one Linux machine, expand CI matrix manually.

**Suggested first command for implementer:**

```bash
cargo test -p xtask-todo-lib
```

Then implement Task 1.1 → 1.4 in order.
