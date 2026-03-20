//! Sandbox: export VFS to a temp dir for isolated execution (e.g. rustup/cargo), then sync back.
//!
//! Task 1: export VFS subtree (cwd) to host temp dir with unique name and 0o700.
//! Task 2: run a subprocess with cwd set to the export dir (path-based; fd-only is future).

use std::path::{Path, PathBuf};
use std::process::Command;

use super::vfs::Vfs;

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

/// Errors from sandbox export/sync.
#[derive(Debug)]
pub enum SandboxError {
    /// Failed to create temp dir or set permissions.
    ExportFailed(std::io::Error),
    /// VFS copy to host failed.
    CopyFailed(super::vfs::VfsError),
    /// Sync from host back to VFS failed.
    SyncBackFailed(std::io::Error),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExportFailed(e) => write!(f, "export failed: {e}"),
            Self::CopyFailed(e) => write!(f, "copy to host failed: {e}"),
            Self::SyncBackFailed(e) => write!(f, "sync back failed: {e}"),
        }
    }
}

impl std::error::Error for SandboxError {}

/// Export the VFS subtree at `vfs_path` (e.g. current cwd) to a new temporary directory.
///
/// The directory lives under the system temp dir with a unique name (`devshell_<pid>_<nanos>`)
/// and on Unix has mode `0o700`. Returns the path to the created directory;
/// the caller is responsible for cleanup (remove dir when done).
///
/// # Errors
/// Returns `SandboxError::ExportFailed` if the temp dir cannot be created or permissions set.
/// Returns `SandboxError::CopyFailed` if VFS copy to host fails.
pub fn export_vfs_to_temp_dir(vfs: &Vfs, vfs_path: &str) -> Result<PathBuf, SandboxError> {
    let temp_base = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = format!("devshell_{}_{}", std::process::id(), nanos);
    let path = temp_base.join(name);

    std::fs::create_dir(&path).map_err(SandboxError::ExportFailed)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)
            .map_err(SandboxError::ExportFailed)?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&path, perms).map_err(SandboxError::ExportFailed)?;
    }

    vfs.copy_tree_to_host(vfs_path, &path)
        .map_err(SandboxError::CopyFailed)?;

    Ok(path)
}

/// Run a subprocess with cwd set to `export_dir`.
///
/// Child inherits the process stdin/stdout/stderr (redirects for this builtin can be added later).
/// Returns the process exit status; the caller should then sync back and remove the dir.
///
/// # Errors
/// Returns `SandboxError::ExportFailed` if spawning the process fails (e.g. program not found).
pub fn run_in_export_dir<P: AsRef<Path>>(
    export_dir: &Path,
    program: P,
    args: &[String],
) -> Result<std::process::ExitStatus, SandboxError> {
    let mut child = Command::new(program.as_ref())
        .args(args)
        .current_dir(export_dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(SandboxError::ExportFailed)?;

    child.wait().map_err(SandboxError::ExportFailed)
}

/// Export VFS subtree at `vfs_path`, run `program` with `args` in that dir, sync changes back, then cleanup.
/// Returns the child's exit status. Caller should check `status.success()`.
///
/// # Errors
/// Returns `SandboxError` if binary not in PATH (`ExportFailed` with a message), export fails, spawn fails, or sync fails.
pub fn run_rust_tool(
    vfs: &mut Vfs,
    vfs_path: &str,
    program: &str,
    args: &[String],
) -> Result<std::process::ExitStatus, SandboxError> {
    let program_path = find_in_path(program).ok_or_else(|| {
        SandboxError::ExportFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{program} not found in PATH"),
        ))
    })?;

    let export_dir = export_vfs_to_temp_dir(vfs, vfs_path)?;
    let work_dir = host_export_root(&export_dir, vfs_path);

    let status = run_in_export_dir(&work_dir, &program_path, args);
    let sync_result = sync_host_dir_to_vfs(&export_dir, vfs_path, vfs);
    let _ = std::fs::remove_dir_all(&export_dir);

    sync_result?;
    status
}

/// Host path that corresponds to the root of the exported VFS subtree.
///
/// `copy_tree_to_host` places the resolved node (with its name) directly under `export_dir`,
/// so for `/projects/hello` we get `export_dir/hello`, not `export_dir/projects/hello`.
fn host_export_root(export_dir: &Path, vfs_path: &str) -> PathBuf {
    let trimmed = vfs_path.trim_matches('/');
    if trimmed.is_empty() {
        export_dir.to_path_buf()
    } else {
        let last = trimmed.split('/').next_back().unwrap_or(".");
        export_dir.join(last)
    }
}

/// Sync the host export directory back into the VFS at `vfs_path`.
///
/// Walks the host subtree and creates/overwrites files and dirs in the VFS.
/// Does not remove VFS nodes that no longer exist on host (add/update only).
///
/// # Errors
/// Returns `SandboxError::SyncBackFailed` on host read or VFS write failure.
pub fn sync_host_dir_to_vfs(
    export_dir: &Path,
    vfs_path: &str,
    vfs: &mut Vfs,
) -> Result<(), SandboxError> {
    let host_root = host_export_root(export_dir, vfs_path);
    if !host_root.is_dir() {
        return Ok(());
    }
    sync_host_dir_to_vfs_recursive(&host_root, vfs_path, vfs)
}

fn sync_host_dir_to_vfs_recursive(
    host_dir: &Path,
    vfs_prefix: &str,
    vfs: &mut Vfs,
) -> Result<(), SandboxError> {
    let entries = std::fs::read_dir(host_dir).map_err(SandboxError::SyncBackFailed)?;
    for entry in entries {
        let entry = entry.map_err(SandboxError::SyncBackFailed)?;
        let name = entry.file_name();
        let name_str = name.to_str().ok_or_else(|| {
            SandboxError::SyncBackFailed(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "non-UTF8 path",
            ))
        })?;
        let vfs_path_here = if vfs_prefix == "/" || vfs_prefix.is_empty() {
            format!("/{name_str}")
        } else {
            format!("{vfs_prefix}/{name_str}")
        };

        if entry.path().is_dir() {
            vfs.mkdir(&vfs_path_here)
                .map_err(|e| SandboxError::SyncBackFailed(std::io::Error::other(e.to_string())))?;
            sync_host_dir_to_vfs_recursive(&entry.path(), &vfs_path_here, vfs)?;
        } else {
            let content = std::fs::read(entry.path()).map_err(SandboxError::SyncBackFailed)?;
            vfs.write_file(&vfs_path_here, &content)
                .map_err(|e| SandboxError::SyncBackFailed(std::io::Error::other(e.to_string())))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::vfs::Vfs;
    use super::{export_vfs_to_temp_dir, sync_host_dir_to_vfs};

    #[test]
    fn export_empty_cwd_creates_dir() {
        let vfs = Vfs::new();
        let path = export_vfs_to_temp_dir(&vfs, "/").unwrap();
        assert!(path.is_dir());
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("devshell_"));
        let _ = std::fs::remove_dir(path);
    }

    #[test]
    fn export_with_files_and_dirs() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/proj").unwrap();
        vfs.set_cwd("/proj").unwrap();
        vfs.write_file("/proj/Cargo.toml", b"[package]\nname = \"foo\"\n")
            .unwrap();
        vfs.mkdir("/proj/src").unwrap();
        vfs.write_file("/proj/src/main.rs", b"fn main() {}\n")
            .unwrap();
        let path = export_vfs_to_temp_dir(&vfs, "/proj").unwrap();
        assert!(path.is_dir());
        // copy_tree_to_host exports the node at /proj into path, so content is path/proj/...
        let proj = path.join("proj");
        let cargo = proj.join("Cargo.toml");
        let main = proj.join("src/main.rs");
        assert!(cargo.is_file(), "proj/Cargo.toml should exist");
        assert!(main.is_file(), "proj/src/main.rs should exist");
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("foo"));
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn sync_host_to_vfs_adds_files() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/proj").unwrap();
        let path = export_vfs_to_temp_dir(&vfs, "/proj").unwrap();
        let proj_host = path.join("proj");
        std::fs::write(proj_host.join("new.txt"), b"hello").unwrap();
        sync_host_dir_to_vfs(&path, "/proj", &mut vfs).unwrap();
        let content = vfs.read_file("/proj/new.txt").unwrap();
        assert_eq!(content, b"hello");
        let _ = std::fs::remove_dir_all(path);
    }

    /// Regression: cwd for cargo must be `export_dir/<last-segment>`, not `export_dir/<full-vfs-path>`.
    /// Otherwise `cargo run` fails with ENOENT on `current_dir` (misreported as `CargoNotFound`).
    #[test]
    fn nested_vfs_path_host_uses_leaf_dir_not_full_path() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/projects").unwrap();
        vfs.mkdir("/projects/hello").unwrap();
        vfs.write_file(
            "/projects/hello/Cargo.toml",
            b"[package]\nname = \"hello\"\n",
        )
        .unwrap();

        let export = export_vfs_to_temp_dir(&vfs, "/projects/hello").unwrap();
        let hello = export.join("hello");
        let wrong = export.join("projects").join("hello");
        assert!(
            hello.join("Cargo.toml").is_file(),
            "expected export at export_dir/hello/, matching copy_tree_to_host"
        );
        assert!(
            !wrong.join("Cargo.toml").is_file(),
            "must not use export_dir/projects/hello (that path is empty)"
        );

        std::fs::write(hello.join("synced.txt"), b"ok").unwrap();
        sync_host_dir_to_vfs(&export, "/projects/hello", &mut vfs).unwrap();
        assert_eq!(vfs.read_file("/projects/hello/synced.txt").unwrap(), b"ok");

        let _ = std::fs::remove_dir_all(export);
    }
}
