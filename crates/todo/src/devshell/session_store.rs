//! Session persistence for devshell (design §10).
//!
//! - **Mode S:** legacy [`.dev_shell.bin`] snapshot via [`crate::devshell::serialization`].
//! - **Mode P (guest-primary):** the guest workspace is authoritative; we **do not** write the legacy
//!   bin format on exit. Instead we persist **metadata only** (JSON) beside the bin path:
//!   **`{stem}.session.json`** when the bin path ends in `.bin` (e.g. `.dev_shell.bin` →
//!   `.dev_shell.session.json`). The file records `logical_cwd` for the next REPL start; the in-memory
//!   VFS tree is not a second source of truth for the project tree in Mode P.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// JSON `format` field for [`GuestPrimarySessionV1`].
pub const FORMAT_DEVSHELL_SESSION_V1: &str = "devshell_session_v1";

/// Metadata persisted for guest-primary sessions (no legacy VFS snapshot).
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

/// Save guest-primary session metadata (JSON). Does **not** write `.dev_shell.bin`.
///
/// # Errors
/// I/O errors from writing the JSON file.
pub fn save_guest_primary(bin_path: &Path, logical_cwd: &str) -> io::Result<()> {
    let meta = GuestPrimarySessionV1::new(logical_cwd.to_string());
    let text = serde_json::to_string_pretty(&meta).map_err(|e| io::Error::other(e.to_string()))?;
    std::fs::write(session_path_for_bin(bin_path), text)
}

/// Load guest-primary metadata if the companion file exists and is valid v1.
///
/// Returns `Ok(None)` if the file is missing or not a recognized format.
///
/// # Errors
/// I/O errors from reading the file, or invalid JSON (wrapped as `InvalidData`).
pub fn load_guest_primary(bin_path: &Path) -> io::Result<Option<GuestPrimarySessionV1>> {
    let p = session_path_for_bin(bin_path);
    if !p.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&p)?;
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
    // set_cwd requires the path to exist as a directory in VFS — mkdir the chain first.
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
    use std::fs;

    fn tmp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xtask_devshell_{name}_{}_{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
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
    fn roundtrip_guest_primary_session() {
        let dir = tmp_dir("roundtrip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let bin_path = dir.join("state.bin");
        save_guest_primary(&bin_path, "/proj/foo").unwrap();
        let meta = load_guest_primary(&bin_path).unwrap().expect("some");
        assert_eq!(meta.logical_cwd, "/proj/foo");
        assert_eq!(meta.format, FORMAT_DEVSHELL_SESSION_V1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_startup_sets_cwd() {
        let dir = tmp_dir("apply");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
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
