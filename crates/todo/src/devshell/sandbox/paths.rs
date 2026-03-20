//! Export base directory resolution and `PATH` lookup.

use std::path::PathBuf;

/// Override parent directory for sandbox exports (`devshell_*` folders). Trims whitespace; empty ignores.
pub const ENV_EXPORT_BASE: &str = "DEVSHELL_EXPORT_BASE";

/// Parent directory for per-run `devshell_*` export folders.
///
/// Many Linux systems mount [`std::env::temp_dir`] (often `/tmp`) with **`noexec`**, so `cargo run`
/// can build but fails to **execute** `target/debug/...` with **Permission denied (EACCES)**. This
/// defaults to a user cache path that is normally executable.
///
/// Resolution order:
/// 1. **`DEVSHELL_EXPORT_BASE`** if set and non-empty.
/// 2. Unix: **`XDG_CACHE_HOME`/cargo-devshell-exports**, else **`HOME`/.cache/cargo-devshell-exports**.
/// 3. Windows: **`LOCALAPPDATA`/cargo-devshell-exports**.
/// 4. Fall back to [`std::env::temp_dir`].
#[must_use]
pub fn devshell_export_parent_dir() -> PathBuf {
    if let Ok(p) = std::env::var(ENV_EXPORT_BASE) {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    #[cfg(unix)]
    {
        if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
            let xdg = xdg.trim();
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("cargo-devshell-exports");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let home = home.trim();
            if !home.is_empty() {
                return PathBuf::from(home)
                    .join(".cache")
                    .join("cargo-devshell-exports");
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let local = local.trim();
            if !local.is_empty() {
                return PathBuf::from(local).join("cargo-devshell-exports");
            }
        }
    }
    std::env::temp_dir()
}

/// Search for `program` in PATH. Returns the first absolute path where the executable exists.
#[must_use]
pub fn find_in_path(program: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    let ext = if cfg!(windows) { ".exe" } else { "" };
    for part in std::env::split_paths(&path_env) {
        let candidate = part.join(format!("{program}{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
