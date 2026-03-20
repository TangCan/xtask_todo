# Rust toolchain sandbox (no container engine)

`cargo-devshell` runs **`rustup`** and **`cargo`** by exporting the current VFS subtree to a **host directory** (unique subfolder per run, `0o700` on Unix), executing the real binaries from your **`PATH`** with `cwd` set to that export, then **syncing** host changes back into the VFS and deleting the export folder.

**Export location:** defaults to **`~/.cache/cargo-devshell-exports`** (or **`XDG_CACHE_HOME`**, or Windows **`%LOCALAPPDATA%/cargo-devshell-exports`**) instead of **`/tmp`**, because many Linux systems mount the temp filesystem with **`noexec`**, which breaks **`cargo run`**. Set **`DEVSHELL_EXPORT_BASE`** to override.

**Unix execute bit:** syncing the VFS back uses plain file writes, so **`target/debug/*` binaries lose `+x`** on the next export. Before running **`cargo`/`rustup`**, devshell walks **`target/`** and sets **`0755`** on files that start with the **ELF** magic bytes so **`cargo run`** can exec them even when Cargo skips rebuild.

**There is no `podman run`, `docker run`, or other OCI runtime** in this flow. Isolation is:

- A dedicated tree per invocation (plus optional Linux mount namespace; see below).
- The same export → run → sync contract as before.

## Optional Linux mount namespace

On **Linux only**, set:

| Variable | Effect |
|----------|--------|
| `DEVSHELL_RUST_MOUNT_NAMESPACE` | If `1`, `true`, or `yes`: before `exec`, the child calls `unshare(CLONE_NEWNS)` and applies `mount(..., MS_REC \| MS_PRIVATE)` on `/`. This creates a **private mount namespace** for the `cargo`/`rustup` process (kernel APIs via **libc**, not Podman/Docker). |

This does **not** provide a full filesystem jail (the child still sees the host tree). It limits **mount propagation** from the child’s mounts to the parent namespace, which is a small slice of what container runtimes do.

On macOS, Windows, and other non-Linux targets, this variable is ignored.

## VM session (γ / β) — default for `cargo devshell` on Unix

On **Linux and macOS**, for the **`cargo devshell` binary** (non-`cfg(test)` build), **`DEVSHELL_VM` unset means VM mode is on**, and **`DEVSHELL_VM_BACKEND` unset defaults to `lima`**, so `rustup`/`cargo` run inside **Lima** while the library syncs a host **staging directory** with the VFS. You need **`limactl`** on `PATH` and a Lima instance that mounts that directory at guest **`/workspace`** (or override with `DEVSHELL_VM_GUEST_WORKSPACE`). **Opt out:** **`DEVSHELL_VM=off`** or **`DEVSHELL_VM_BACKEND=host`** / **`auto`**. **`cargo test`** for this crate still defaults to the host sandbox. Details: **[devshell-vm-gamma.md](./devshell-vm-gamma.md)**.

The **`devshell-vm`** binary (workspace crate, currently a **stub**) is the planned **β** sidecar; IPC message shapes are described in **[superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md](./superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md)**.

## Related code

- `crates/todo/src/devshell/sandbox.rs` — `export_vfs_to_temp_dir`, `run_in_export_dir`, `run_rust_tool`, `sync_host_dir_to_vfs`
- `crates/todo/src/devshell/vm/` — optional `SessionHolder` (host vs Lima γ)

A root **`Dockerfile`** in the repo (if present) is only for **your own** image builds if you use Docker elsewhere; devshell does not call it.
