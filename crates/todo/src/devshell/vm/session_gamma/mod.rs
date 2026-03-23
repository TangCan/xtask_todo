//! γ backend: [Lima](https://github.com/lima-vm/lima) via `limactl start` / `limactl shell`.
//!
//! The host directory [`GammaSession::workspace_parent`] must be mounted in the guest at
//! [`GammaSession::guest_mount`] (default `/workspace`). By default [`helpers::workspace_parent_for_instance`]
//! follows the **Cargo workspace root** from `cargo metadata` (unless overridden); see `docs/devshell-vm-gamma.md`.

#![allow(clippy::pedantic, clippy::nursery)]

mod env;
mod helpers;
mod session;

#[cfg(test)]
mod tests;

pub use env::ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT;
pub use env::{
    ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL, ENV_DEVSHELL_VM_AUTO_BUILD_TODO_GUEST,
    ENV_DEVSHELL_VM_AUTO_TODO_PATH, ENV_DEVSHELL_VM_GUEST_HOST_DIR,
    ENV_DEVSHELL_VM_GUEST_TODO_HINT, ENV_DEVSHELL_VM_GUEST_WORKSPACE, ENV_DEVSHELL_VM_LIMACTL,
    ENV_DEVSHELL_VM_STOP_ON_EXIT, ENV_DEVSHELL_VM_WORKSPACE_PARENT,
};
pub use helpers::workspace_parent_for_instance;
pub use session::GammaSession;
