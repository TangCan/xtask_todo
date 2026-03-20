//! Command and path completion candidates (built-ins, VFS / guest FS listing).

use std::cell::RefCell;
use std::rc::Rc;

use super::super::vfs::Vfs;
use super::super::vm::SessionHolder;

#[cfg(unix)]
use super::super::vm::GuestFsOps;
#[cfg(unix)]
use crate::devshell::workspace::logical_path_to_guest;

/// Built-in command names for tab completion (must match command.rs).
const BUILTIN_COMMANDS: &[&str] = &[
    "pwd",
    "cd",
    "ls",
    "mkdir",
    "rustup",
    "cargo",
    "cat",
    "touch",
    "echo",
    "save",
    "export-readonly",
    "export_readonly",
    "exit",
    "quit",
    "help",
    "todo",
];

/// Command completion: case-insensitive prefix match. Returns matching command names.
#[must_use]
pub fn complete_commands(prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    BUILTIN_COMMANDS
        .iter()
        .filter(|c| c.to_lowercase().starts_with(prefix_lower.as_str()))
        .map(|s| (*s).to_string())
        .collect()
}

/// Split path being completed into `(dir_prefix, basename_prefix)`.
///
/// `dir_prefix` ends with `/` (or is empty); it is preserved in candidates so readline replaces
/// the whole token (e.g. `src/` → `src/main.rs`, not `main.rs` alone).
fn split_dir_and_basename_prefix(prefix: &str) -> (&str, &str) {
    prefix.rfind('/').map_or(("", prefix), |idx| {
        let dir = &prefix[..=idx];
        let rest = &prefix[idx + 1..];
        (dir, rest)
    })
}

/// Path completion: prefix may contain slashes; only the basename segment is matched.
///
/// Returned strings are **full token replacements** (include any directory part before the last
/// `/`), so rustyline's replace-from-`start` keeps paths like `src/main.rs` correct.
/// `parent_names` are basenames in the resolved parent directory. Empty basename prefix returns all.
#[must_use]
pub fn complete_path(prefix: &str, parent_names: &[String]) -> Vec<String> {
    let (dir_prefix, basename_prefix) = split_dir_and_basename_prefix(prefix);
    parent_names
        .iter()
        .filter(|n| n.starts_with(basename_prefix))
        .map(|n| format!("{dir_prefix}{n}"))
        .collect()
}

/// List directory entry names for path tab completion: **Mode P** (γ or β) uses [`GuestFsOps`] when
/// guest-primary is active; otherwise [`Vfs::list_dir`].
#[must_use]
pub fn list_dir_names_for_completion(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
    abs_parent: &str,
) -> Vec<String> {
    #[cfg(unix)]
    {
        let cwd = vfs.borrow().cwd().to_string();
        let mut session = vm_session.borrow_mut();
        if let Some((ops, mount)) = session.guest_primary_fs_ops_mut() {
            if let Ok(guest_path) = logical_path_to_guest(&mount, &cwd, abs_parent) {
                if let Ok(names) = GuestFsOps::list_dir(ops, &guest_path) {
                    return names;
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = vm_session;
    }
    vfs.borrow().list_dir(abs_parent).unwrap_or_default()
}
