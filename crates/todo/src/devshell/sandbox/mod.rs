//! Sandbox: export VFS to a temp dir for isolated execution (e.g. rustup/cargo), then sync back.
//!
//! ## Isolation (no Podman/Docker)
//!
//! Devshell **does not** invoke `podman`, `docker`, or any OCI runtime. Flow: export VFS subtree to a
//! unique host temp dir (`0o700` on Unix) → run `cargo` / `rustup` from the host `PATH` with `cwd` set
//! to the export root → sync back → remove the temp dir.
//!
//! **Linux optional mount namespace** — set **`DEVSHELL_RUST_MOUNT_NAMESPACE=1`** (or `true` / `yes`) so the
//! child process calls `unshare(CLONE_NEWNS)` and makes the mount tree private (`MS_REC | MS_PRIVATE`)
//! before `exec`. That gives a **separate mount namespace** (kernel feature via libc), similar in spirit
//! to container mount isolation but **without** a container engine. It does **not** hide the host
//! filesystem from the child; a full root jail would need additional work (e.g. `pivot_root`).
//!
//! On non-Linux platforms the env var is ignored.
//!
//! ## Unix execute bit on `target/` binaries
//!
//! VFS sync uses [`std::fs::write`], which creates files without the execute bit. After a round-trip,
//! `target/debug/foo` is often **0644** while still a valid ELF. `cargo run` may skip rebuild and then
//! **execve** fails with **EACCES (Permission denied)**. Before running `cargo`/`rustup`, we walk
//! `target/` and set **0755** on files that look like **ELF** objects.

mod elf;
mod error;
mod export;
mod paths;
mod run;
mod sync;

#[cfg(target_os = "linux")]
mod linux_mount;

#[cfg(test)]
mod tests;

pub use error::SandboxError;
pub use export::export_vfs_to_temp_dir;
pub use paths::{devshell_export_parent_dir, find_in_path, ENV_EXPORT_BASE};
pub use run::{run_in_export_dir, run_rust_tool};
pub use sync::sync_host_dir_to_vfs;

pub(crate) use elf::restore_execute_bits_for_build_artifacts;
pub(crate) use sync::host_export_root;
