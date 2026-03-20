//! Workspace abstraction: Mode S ([`MemoryVfsBackend`]) vs Mode P ([`GuestPrimaryBackend`]).
//!
//! See `docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md` §4.

#![allow(clippy::pedantic, clippy::nursery)]

mod backend;
#[cfg(unix)]
mod guest_export;
mod io;

pub use backend::{
    logical_path_to_guest, GuestPrimaryBackend, MemoryVfsBackend, WorkspaceBackend,
    WorkspaceBackendError,
};
#[cfg(unix)]
pub use guest_export::guest_export_readonly_to_vfs;
#[cfg(unix)]
pub use io::logical_to_guest_abs;
pub use io::{read_logical_file_bytes, read_logical_file_bytes_rc, WorkspaceReadError};
