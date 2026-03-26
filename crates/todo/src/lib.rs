//! todo - workspace library
//!
//! Todo domain: create, list, complete, delete items with in-memory or pluggable storage.
//! Also includes the devshell REPL/VFS logic used by the `cargo-devshell` binary (for test coverage).

pub mod devshell;
mod error;
mod id;
mod list;
mod model;
mod priority;
mod repeat;
mod store;

pub use error::TodoError;
pub use id::TodoId;
pub use list::TodoList;
pub use model::{ListFilter, ListOptions, ListSort, Todo, TodoPatch};
pub use priority::Priority;
pub use repeat::RepeatRule;
pub use store::{InMemoryStore, Store};

#[cfg(test)]
mod tests;

/// Serialize tests that use `std::env::set_current_dir` (process-global).
#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, OnceLock, PoisonError};

    pub fn cwd_mutex() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Serialize `DEVSHELL_WORKSPACE_ROOT` with [`crate::devshell::vm::export_devshell_workspace_root_env`]
    /// and `session_store` tests (parallel `cargo test`).
    pub fn devshell_workspace_env_mutex() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Serialize VM-related environment mutation in tests (`DEVSHELL_VM*`, `PATH`, etc.).
    pub fn vm_env_mutex() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
