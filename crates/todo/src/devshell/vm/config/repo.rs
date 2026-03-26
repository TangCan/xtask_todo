//! Discover `xtask_todo` repo root via `containers/devshell-vm/Containerfile` (Windows β).
#![allow(clippy::redundant_pub_crate)]

#[cfg(feature = "beta-vm")]
fn devshell_repo_root_walk(mut dir: std::path::PathBuf) -> Option<std::path::PathBuf> {
    loop {
        let cf = dir.join("containers/devshell-vm/Containerfile");
        if cf.is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Walk parents from [`std::env::current_dir`] looking for `containers/devshell-vm/Containerfile` (`xtask_todo` repo).
#[cfg_attr(not(windows), allow(dead_code))]
#[cfg(feature = "beta-vm")]
pub(crate) fn devshell_repo_root_with_containerfile() -> Option<std::path::PathBuf> {
    let dir = std::env::current_dir().ok()?;
    devshell_repo_root_walk(dir)
}

/// Same as [`devshell_repo_root_with_containerfile`] but starting from `start` (e.g. workspace parent).
#[cfg_attr(not(windows), allow(dead_code))]
#[cfg(feature = "beta-vm")]
pub(crate) fn devshell_repo_root_from_path(start: &std::path::Path) -> Option<std::path::PathBuf> {
    devshell_repo_root_walk(start.to_path_buf())
}
