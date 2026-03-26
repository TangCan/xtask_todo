//! β client: JSON-lines over **Unix socket** (Unix), **TCP**, or **Windows:** **`podman machine ssh`**
//! stdio to `devshell-vm --serve-stdio` (default; see `docs/devshell-vm-windows.md`).
//!
//! Build: `cargo build -p xtask-todo-lib --features beta-vm`. Env: `DEVSHELL_VM_SOCKET`.
//! - **Unix:** path to socket, or `tcp:127.0.0.1:9847` / `tcp://127.0.0.1:9847`
//! - **Windows:** `stdio` (default) or `tcp:HOST:PORT`

mod ipc;
mod session;
#[cfg(test)]
mod tests;

pub use session::BetaSession;
