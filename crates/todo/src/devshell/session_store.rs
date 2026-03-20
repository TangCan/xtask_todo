//! Session persistence for devshell.
//!
//! - **Workspace session (preferred):** [`ENV_DEVSHELL_WORKSPACE_ROOT`] — metadata at
//!   **`$DEVSHELL_WORKSPACE_ROOT/.cargo-devshell/session.json`** (see `docs/requirements.md` §1.1).
//!   On Unix, [`crate::devshell::vm::export_devshell_workspace_root_env`] sets this before REPL.
//! - **Mode S:** legacy [`.dev_shell.bin`] via [`crate::devshell::serialization`] when applicable.
//! - **Mode P (guest-primary):** no legacy bin on exit; JSON holds `logical_cwd` only.
//! - **Fallback** (no `DEVSHELL_WORKSPACE_ROOT`): **`./.cargo-devshell/session.json`** under
//!   [`std::env::current_dir`] when it succeeds (local dev / tests).
//! - **Legacy (migration only):** **`{stem}.session.json`** beside the bin path (e.g.
//!   `.dev_shell.bin` → `.dev_shell.session.json`); still **read** on load, no longer preferred for new saves.
//! - **Load order:** workspace env path → cwd workspace path → legacy beside `bin_path`.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Environment variable: host directory aligned with guest workspace (Lima γ).
pub const ENV_DEVSHELL_WORKSPACE_ROOT: &str = "DEVSHELL_WORKSPACE_ROOT";

/// JSON `format` field for [`GuestPrimarySessionV1`].
pub const FORMAT_DEVSHELL_SESSION_V1: &str = "devshell_session_v1";

/// Serialize / deserialize guest-primary session file under [`ENV_DEVSHELL_WORKSPACE_ROOT`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestPrimarySessionV1 {
    pub format: String,
    /// Logical cwd (absolute Unix-style path) restored on startup.
    pub logical_cwd: String,
    /// Wall time when saved (milliseconds since UNIX epoch).
    pub saved_at_unix_ms: u64,
}

impl GuestPrimarySessionV1 {
    fn new(logical_cwd: String) -> Self {
        let saved_at_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(0));
        Self {
            format: FORMAT_DEVSHELL_SESSION_V1.to_string(),
            logical_cwd,
            saved_at_unix_ms,
        }
    }
}

/// Companion path for guest-primary metadata next to a legacy `.bin` snapshot path.
///
/// Example: `.dev_shell.bin` → `.dev_shell.session.json` (replaces the last extension).
#[must_use]
pub fn session_path_for_bin(bin_path: &Path) -> PathBuf {
    bin_path.with_extension("session.json")
}

/// Session file under **`$DEVSHELL_WORKSPACE_ROOT/.cargo-devshell/session.json`** when the env var is set.
#[must_use]
pub fn workspace_session_metadata_path() -> Option<PathBuf> {
    std::env::var(ENV_DEVSHELL_WORKSPACE_ROOT)
        .ok()
        .and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(
                    PathBuf::from(t)
                        .join(".cargo-devshell")
                        .join("session.json"),
                )
            }
        })
}

/// Same layout as [`workspace_session_metadata_path`], but rooted at [`std::env::current_dir`].
///
/// Used when `DEVSHELL_WORKSPACE_ROOT` is unset (e.g. plain `cargo-devshell` without VM env).
#[must_use]
pub fn cwd_session_metadata_path() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .map(|p| p.join(".cargo-devshell").join("session.json"))
}

/// Resolved path to write guest-primary session JSON: workspace env → cwd `.cargo-devshell/` → legacy beside `bin_path`.
#[must_use]
pub fn session_metadata_path(bin_path: &Path) -> PathBuf {
    workspace_session_metadata_path()
        .or_else(cwd_session_metadata_path)
        .unwrap_or_else(|| session_path_for_bin(bin_path))
}

fn load_one_guest_primary(p: &Path) -> io::Result<Option<GuestPrimarySessionV1>> {
    if !p.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(p)?;
    let v: GuestPrimarySessionV1 = serde_json::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid guest-primary session JSON {}: {e}", p.display()),
        )
    })?;
    if v.format != FORMAT_DEVSHELL_SESSION_V1 {
        return Ok(None);
    }
    Ok(Some(v))
}

/// Save guest-primary session metadata (JSON). Does **not** write `.dev_shell.bin`.
///
/// # Errors
/// I/O errors from writing the JSON file or creating parent directories.
pub fn save_guest_primary(bin_path: &Path, logical_cwd: &str) -> io::Result<()> {
    let meta = GuestPrimarySessionV1::new(logical_cwd.to_string());
    let text = serde_json::to_string_pretty(&meta).map_err(|e| io::Error::other(e.to_string()))?;
    let path = session_metadata_path(bin_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)
}

/// Load guest-primary metadata: workspace env path, then cwd `.cargo-devshell/session.json`, then legacy beside `bin_path`.
///
/// Returns `Ok(None)` if no file exists or format is unrecognized.
///
/// # Errors
/// I/O errors, or invalid JSON (wrapped as `InvalidData`).
pub fn load_guest_primary(bin_path: &Path) -> io::Result<Option<GuestPrimarySessionV1>> {
    if let Some(ref ws) = workspace_session_metadata_path() {
        match load_one_guest_primary(ws) {
            Ok(Some(m)) => return Ok(Some(m)),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }
    if let Some(ref cwd_meta) = cwd_session_metadata_path() {
        match load_one_guest_primary(cwd_meta) {
            Ok(Some(m)) => return Ok(Some(m)),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }
    load_one_guest_primary(&session_path_for_bin(bin_path))
}

/// On guest-primary startup: if a v1 session file exists, reset VFS to empty and restore `logical_cwd`
/// (creating directories as needed so [`crate::devshell::vfs::Vfs::set_cwd`] succeeds).
///
/// If no session file exists, leaves `vfs` unchanged (e.g. legacy `.dev_shell.bin` load for cwd only).
///
/// # Errors
/// I/O from [`load_guest_primary`], or `InvalidData` if `logical_cwd` cannot be created or set.
pub fn apply_guest_primary_startup(
    vfs: &mut crate::devshell::vfs::Vfs,
    bin_path: &Path,
) -> io::Result<()> {
    let Some(meta) = load_guest_primary(bin_path)? else {
        return Ok(());
    };
    *vfs = crate::devshell::vfs::Vfs::new();
    let cwd = meta.logical_cwd.trim();
    if cwd.is_empty() {
        return Ok(());
    }
    if let Err(e) = vfs.mkdir(cwd) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("session logical_cwd mkdir {cwd}: {e}"),
        ));
    }
    vfs.set_cwd(cwd).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("session logical_cwd set_cwd {cwd}: {e}"),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{cwd_mutex, devshell_workspace_env_mutex};
    use std::fs;
    use std::io;

    fn tmp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xtask_devshell_{name}_{}_{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    struct EnvRestore {
        old: Option<String>,
    }

    impl EnvRestore {
        fn set_workspace_root(value: impl AsRef<std::ffi::OsStr>) -> Self {
            let old = std::env::var(ENV_DEVSHELL_WORKSPACE_ROOT).ok();
            std::env::set_var(ENV_DEVSHELL_WORKSPACE_ROOT, value);
            Self { old }
        }

        fn clear_workspace_root() -> Self {
            let old = std::env::var(ENV_DEVSHELL_WORKSPACE_ROOT).ok();
            std::env::remove_var(ENV_DEVSHELL_WORKSPACE_ROOT);
            Self { old }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.old {
                Some(s) => std::env::set_var(ENV_DEVSHELL_WORKSPACE_ROOT, s),
                None => std::env::remove_var(ENV_DEVSHELL_WORKSPACE_ROOT),
            }
        }
    }

    struct CurrentDirRestore {
        previous: PathBuf,
    }

    impl CurrentDirRestore {
        fn chdir(dir: &Path) -> io::Result<Self> {
            let previous = std::env::current_dir()?;
            std::env::set_current_dir(dir)?;
            Ok(Self { previous })
        }
    }

    impl Drop for CurrentDirRestore {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.previous);
        }
    }

    #[test]
    fn session_path_for_bin_replaces_extension() {
        let p = Path::new(".dev_shell.bin");
        assert_eq!(
            session_path_for_bin(p),
            PathBuf::from(".dev_shell.session.json")
        );
    }

    #[test]
    fn roundtrip_guest_primary_uses_cwd_cargo_devshell_when_no_workspace_env() {
        let _cwd_lock = cwd_mutex();
        let _workspace_env = devshell_workspace_env_mutex();
        let _env = EnvRestore::clear_workspace_root();
        let dir = tmp_dir("roundtrip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = dir.canonicalize().expect("canonicalize tmp");
        let _restore_dir = CurrentDirRestore::chdir(&dir).expect("chdir tmp");
        let bin_path = dir.join("state.bin");
        save_guest_primary(&bin_path, "/proj/foo").unwrap();
        let expected = dir.join(".cargo-devshell").join("session.json");
        assert!(expected.is_file(), "expected {}", expected.display());
        let meta = load_guest_primary(&bin_path).unwrap().expect("some");
        assert_eq!(meta.logical_cwd, "/proj/foo");
        assert_eq!(meta.format, FORMAT_DEVSHELL_SESSION_V1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_prefers_workspace_env_path() {
        let _cwd_lock = cwd_mutex();
        let _workspace_env = devshell_workspace_env_mutex();
        let dir = tmp_dir("ws_sess");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = dir.canonicalize().expect("canonicalize tmp");
        let _env = EnvRestore::set_workspace_root(&dir);
        let bin_path = dir.join("ignored.bin");
        save_guest_primary(&bin_path, "/x").unwrap();
        let expected = dir.join(".cargo-devshell").join("session.json");
        assert!(expected.is_file(), "expected {}", expected.display());
        let loaded = load_guest_primary(&bin_path).unwrap().expect("meta");
        assert_eq!(loaded.logical_cwd, "/x");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_startup_sets_cwd() {
        let _cwd_lock = cwd_mutex();
        let _workspace_env = devshell_workspace_env_mutex();
        let _env = EnvRestore::clear_workspace_root();
        let dir = tmp_dir("apply");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = dir.canonicalize().expect("canonicalize tmp");
        let _restore_dir = CurrentDirRestore::chdir(&dir).expect("chdir tmp");
        let bin_path = dir.join("x.bin");
        let session_path = session_path_for_bin(&bin_path);
        fs::write(
            &session_path,
            r#"{
            "format": "devshell_session_v1",
            "logical_cwd": "/a/b",
            "saved_at_unix_ms": 0
        }"#,
        )
        .unwrap();
        let mut vfs = crate::devshell::vfs::Vfs::new();
        vfs.mkdir("/a/b").unwrap();
        vfs.write_file("/a/b/f", b"x").unwrap();
        apply_guest_primary_startup(&mut vfs, &bin_path).unwrap();
        assert_eq!(vfs.cwd(), "/a/b");
        assert!(vfs.read_file("/a/b/f").is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
