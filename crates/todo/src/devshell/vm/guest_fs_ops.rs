//! Guest filesystem operations for Mode P ([`GuestFsOps`]).
//!
//! See `docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md` §4.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

#[cfg(any(unix, feature = "beta-vm"))]
use super::VmError;
#[cfg(unix)]
use super::{GammaSession, VmConfig};

/// Errors from [`GuestFsOps`] (guest path / remote command).
#[derive(Debug)]
pub enum GuestFsError {
    /// Path escapes the workspace mount or is not absolute.
    InvalidPath(String),
    /// No such file or directory.
    NotFound(String),
    /// Expected a directory.
    NotADirectory(String),
    /// Expected a regular file.
    IsADirectory(String),
    /// Guest command failed (non-zero exit).
    GuestCommand { status: Option<i32>, stderr: String },
    /// VM / `limactl` (Unix γ) or β IPC failure.
    #[cfg(any(unix, feature = "beta-vm"))]
    Vm(VmError),
    /// I/O or UTF-8 issues in the mock implementation.
    Internal(String),
}

impl fmt::Display for GuestFsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(s) => write!(f, "invalid guest path: {s}"),
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::NotADirectory(s) => write!(f, "not a directory: {s}"),
            Self::IsADirectory(s) => write!(f, "is a directory: {s}"),
            Self::GuestCommand { stderr, .. } => write!(f, "guest command failed: {stderr}"),
            #[cfg(any(unix, feature = "beta-vm"))]
            Self::Vm(e) => write!(f, "{e}"),
            Self::Internal(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for GuestFsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(any(unix, feature = "beta-vm"))]
            Self::Vm(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(any(unix, feature = "beta-vm"))]
impl From<VmError> for GuestFsError {
    fn from(e: VmError) -> Self {
        Self::Vm(e)
    }
}

/// Operations on the **guest** filesystem (Mode P). Paths are **guest** absolute paths (e.g. `/workspace/foo`).
pub trait GuestFsOps {
    /// List **non-hidden** names in `guest_path` (like `ls -1A`).
    ///
    /// # Errors
    /// Returns [`GuestFsError`] when `guest_path` is invalid or guest listing fails.
    fn list_dir(&mut self, guest_path: &str) -> Result<Vec<String>, GuestFsError>;

    /// Read a file by guest path.
    ///
    /// # Errors
    /// Returns [`GuestFsError`] when `guest_path` is invalid or reading fails.
    fn read_file(&mut self, guest_path: &str) -> Result<Vec<u8>, GuestFsError>;

    /// Write or replace a file.
    ///
    /// # Errors
    /// Returns [`GuestFsError`] when path validation or write fails.
    fn write_file(&mut self, guest_path: &str, data: &[u8]) -> Result<(), GuestFsError>;

    /// Create a directory (and parents), like `mkdir -p`.
    ///
    /// # Errors
    /// Returns [`GuestFsError`] when path validation or directory creation fails.
    fn mkdir(&mut self, guest_path: &str) -> Result<(), GuestFsError>;

    /// Remove a file or directory tree (`rm -rf` semantics on Lima; mock removes subtree).
    ///
    /// # Errors
    /// Returns [`GuestFsError`] when path validation or removal fails.
    fn remove(&mut self, guest_path: &str) -> Result<(), GuestFsError>;
}

// --- path helpers (shared) -------------------------------------------------

/// Lexically normalize an absolute Unix-style path (no symlink resolution).
#[must_use]
pub fn normalize_guest_path(path: &str) -> Option<String> {
    let mut stack: Vec<&str> = Vec::new();
    for part in path.trim().split('/').filter(|s| !s.is_empty()) {
        match part {
            "." => {}
            ".." => {
                stack.pop();
            }
            p => stack.push(p),
        }
    }
    if stack.is_empty() {
        Some("/".to_string())
    } else {
        Some(format!("/{}", stack.join("/")))
    }
}

/// Guest directory for the current logical cwd (γ layout: `guest_mount` + last segment of `logical_cwd`).
/// Same rule as `guest_dir_for_cwd_inner` in `session_gamma` / push layout.
#[must_use]
pub fn guest_project_dir_on_guest(guest_mount: &str, logical_cwd: &str) -> String {
    let trimmed = logical_cwd.trim_matches('/');
    let base = guest_mount.trim_end_matches('/');
    if trimmed.is_empty() {
        base.to_string()
    } else {
        let last = trimmed.split('/').next_back().unwrap_or(".");
        format!("{base}/{last}")
    }
}

/// True if `path` is `mount` or a strict descendant (after normalization). `mount` is e.g. `/workspace`.
#[must_use]
pub fn guest_path_is_under_mount(mount: &str, path: &str) -> bool {
    let Some(m) = normalize_guest_path(mount) else {
        return false;
    };
    let Some(p) = normalize_guest_path(path) else {
        return false;
    };
    let m = m.trim_end_matches('/').to_string();
    p == m || p.starts_with(&format!("{m}/"))
}

// --- Mock (tests + future harness) -----------------------------------------

#[derive(Debug, Clone)]
enum MockNode {
    File(Vec<u8>),
    Dir,
}

/// In-memory [`GuestFsOps`] for unit tests (no VM).
#[derive(Debug, Default)]
pub struct MockGuestFsOps {
    nodes: HashMap<String, MockNode>,
}

impl MockGuestFsOps {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn norm_key(path: &str) -> Result<String, GuestFsError> {
        normalize_guest_path(path).ok_or_else(|| GuestFsError::InvalidPath(path.to_string()))
    }

    fn ensure_parent_dirs(&mut self, path: &str) -> Result<(), GuestFsError> {
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            let parent_s = parent.to_string_lossy();
            if parent_s.is_empty() || parent_s == "/" {
                return Ok(());
            }
            let pk = Self::norm_key(&parent_s)?;
            if !self.nodes.contains_key(&pk) {
                self.mkdir(&pk)?;
            }
        }
        Ok(())
    }

    fn direct_child_names(&self, dir: &str) -> Result<Vec<String>, GuestFsError> {
        let d = Self::norm_key(dir)?;
        if !matches!(self.nodes.get(&d), Some(MockNode::Dir)) {
            return Err(GuestFsError::NotADirectory(d));
        }
        let prefix = if d == "/" {
            "/".to_string()
        } else {
            format!("{d}/")
        };
        let mut names = std::collections::HashSet::new();
        for key in self.nodes.keys() {
            if key == &d {
                continue;
            }
            if !key.starts_with(&prefix) {
                continue;
            }
            let rest = &key[prefix.len()..];
            if let Some(first) = rest.split('/').next() {
                if !first.is_empty() {
                    names.insert(first.to_string());
                }
            }
        }
        let mut v: Vec<String> = names.into_iter().collect();
        v.sort();
        Ok(v)
    }
}

impl GuestFsOps for MockGuestFsOps {
    fn list_dir(&mut self, guest_path: &str) -> Result<Vec<String>, GuestFsError> {
        self.direct_child_names(guest_path)
    }

    fn read_file(&mut self, guest_path: &str) -> Result<Vec<u8>, GuestFsError> {
        let k = Self::norm_key(guest_path)?;
        match self.nodes.get(&k) {
            Some(MockNode::File(b)) => Ok(b.clone()),
            Some(MockNode::Dir) => Err(GuestFsError::IsADirectory(k)),
            None => Err(GuestFsError::NotFound(k)),
        }
    }

    fn write_file(&mut self, guest_path: &str, data: &[u8]) -> Result<(), GuestFsError> {
        let k = Self::norm_key(guest_path)?;
        self.ensure_parent_dirs(&k)?;
        if matches!(self.nodes.get(&k), Some(MockNode::Dir)) {
            return Err(GuestFsError::IsADirectory(k));
        }
        self.nodes.insert(k, MockNode::File(data.to_vec()));
        Ok(())
    }

    fn mkdir(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        let k = Self::norm_key(guest_path)?;
        if matches!(self.nodes.get(&k), Some(MockNode::File(_))) {
            return Err(GuestFsError::InvalidPath(format!(
                "mkdir: file exists at {k}"
            )));
        }
        if k == "/" {
            self.nodes.entry("/".to_string()).or_insert(MockNode::Dir);
            return Ok(());
        }
        let chunks: Vec<&str> = k.split('/').filter(|s| !s.is_empty()).collect();
        let mut cur = String::new();
        for (i, seg) in chunks.iter().enumerate() {
            cur = if i == 0 {
                format!("/{seg}")
            } else {
                format!("{cur}/{seg}")
            };
            if matches!(self.nodes.get(&cur), Some(MockNode::File(_))) {
                return Err(GuestFsError::InvalidPath(format!(
                    "mkdir: file in the way: {cur}"
                )));
            }
            self.nodes.entry(cur.clone()).or_insert(MockNode::Dir);
        }
        Ok(())
    }

    fn remove(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        let k = Self::norm_key(guest_path)?;
        if !self.nodes.contains_key(&k) {
            return Err(GuestFsError::NotFound(k));
        }
        let to_remove: Vec<String> = self
            .nodes
            .keys()
            .filter(|key| *key == &k || key.starts_with(&format!("{k}/")))
            .cloned()
            .collect();
        for key in to_remove {
            self.nodes.remove(&key);
        }
        Ok(())
    }
}

// --- Lima (Unix): γ [`GammaSession`] + `limactl shell` -----------------------

/// Validate `guest_path` is absolute and under `mount` (γ/β shared).
pub fn validate_guest_path_under_mount(
    mount: &str,
    guest_path: &str,
) -> Result<String, GuestFsError> {
    let Some(p) = normalize_guest_path(guest_path) else {
        return Err(GuestFsError::InvalidPath(guest_path.to_string()));
    };
    if !guest_path_is_under_mount(mount, &p) {
        return Err(GuestFsError::InvalidPath(format!(
            "path not under guest mount {mount}: {p}"
        )));
    }
    Ok(p)
}

#[cfg(unix)]
fn gamma_validate_guest_path(g: &GammaSession, guest_path: &str) -> Result<String, GuestFsError> {
    validate_guest_path_under_mount(g.guest_mount(), guest_path)
}

#[cfg(unix)]
fn map_shell_output(out: std::process::Output) -> Result<Vec<u8>, GuestFsError> {
    if out.status.success() {
        return Ok(out.stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Err(GuestFsError::GuestCommand {
        status: out.status.code(),
        stderr,
    })
}

/// [`GuestFsOps`] on the live γ session (used by REPL dispatch in guest-primary mode).
#[cfg(unix)]
impl GuestFsOps for GammaSession {
    fn list_dir(&mut self, guest_path: &str) -> Result<Vec<String>, GuestFsError> {
        let p = gamma_validate_guest_path(self, guest_path)?;
        let out = self.limactl_shell_output(&p, "ls", &["-1A".to_string()])?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            return Err(GuestFsError::GuestCommand {
                status: out.status.code(),
                stderr,
            });
        }
        let s = String::from_utf8_lossy(&out.stdout);
        let names: Vec<String> = s
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(std::string::ToString::to_string)
            .collect();
        Ok(names)
    }

    fn read_file(&mut self, guest_path: &str) -> Result<Vec<u8>, GuestFsError> {
        let p = gamma_validate_guest_path(self, guest_path)?;
        let path = Path::new(&p);
        let parent = path
            .parent()
            .and_then(|x| x.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("/");
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| GuestFsError::InvalidPath(p.clone()))?;
        let out = self.limactl_shell_output(parent, "cat", &[name.to_string()])?;
        map_shell_output(out)
    }

    fn write_file(&mut self, guest_path: &str, data: &[u8]) -> Result<(), GuestFsError> {
        let p = gamma_validate_guest_path(self, guest_path)?;
        let out = self.limactl_shell_stdin(
            "/",
            "dd",
            &[
                "if=/dev/stdin".to_string(),
                "status=none".to_string(),
                format!("of={p}"),
            ],
            data,
        )?;
        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(GuestFsError::GuestCommand {
                status: out.status.code(),
                stderr,
            })
        }
    }

    fn mkdir(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        let p = gamma_validate_guest_path(self, guest_path)?;
        let out = self.limactl_shell_output("/", "mkdir", &["-p".to_string(), p])?;
        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(GuestFsError::GuestCommand {
                status: out.status.code(),
                stderr,
            })
        }
    }

    fn remove(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        let p = gamma_validate_guest_path(self, guest_path)?;
        let out = self.limactl_shell_output("/", "rm", &["-rf".to_string(), p])?;
        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(GuestFsError::GuestCommand {
                status: out.status.code(),
                stderr,
            })
        }
    }
}

/// Owns a γ [`GammaSession`] for [`GuestFsOps`] tests and harnesses (delegates to [`GuestFsOps`] for [`GammaSession`]).
#[cfg(unix)]
pub struct LimaGuestFsOps {
    session: GammaSession,
}

#[cfg(unix)]
impl LimaGuestFsOps {
    /// Build from VM config (does not start the VM until first operation).
    ///
    /// # Errors
    /// Same as [`GammaSession::new`] (e.g. `limactl` missing).
    pub fn new(config: &VmConfig) -> Result<Self, VmError> {
        Ok(Self {
            session: GammaSession::new(config)?,
        })
    }
}

#[cfg(unix)]
impl GuestFsOps for LimaGuestFsOps {
    fn list_dir(&mut self, guest_path: &str) -> Result<Vec<String>, GuestFsError> {
        GuestFsOps::list_dir(&mut self.session, guest_path)
    }

    fn read_file(&mut self, guest_path: &str) -> Result<Vec<u8>, GuestFsError> {
        GuestFsOps::read_file(&mut self.session, guest_path)
    }

    fn write_file(&mut self, guest_path: &str, data: &[u8]) -> Result<(), GuestFsError> {
        GuestFsOps::write_file(&mut self.session, guest_path, data)
    }

    fn mkdir(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        GuestFsOps::mkdir(&mut self.session, guest_path)
    }

    fn remove(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        GuestFsOps::remove(&mut self.session, guest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_guest_path_dotdot() {
        assert_eq!(normalize_guest_path("/a/b/../c").as_deref(), Some("/a/c"));
        assert_eq!(normalize_guest_path("/").as_deref(), Some("/"));
    }

    #[test]
    fn under_mount() {
        assert!(guest_path_is_under_mount("/workspace", "/workspace/foo"));
        assert!(!guest_path_is_under_mount("/workspace", "/etc/passwd"));
        assert!(!guest_path_is_under_mount(
            "/workspace",
            "/workspace/../etc/passwd"
        ));
    }

    #[test]
    fn mock_mkdir_write_list_read_remove() {
        let mut m = MockGuestFsOps::new();
        m.mkdir("/workspace/p").unwrap();
        m.write_file("/workspace/p/a.txt", b"hi").unwrap();
        let names = m.list_dir("/workspace/p").unwrap();
        assert!(names.contains(&"a.txt".to_string()));
        assert_eq!(m.read_file("/workspace/p/a.txt").unwrap(), b"hi");
        m.remove("/workspace/p").unwrap();
        assert!(m.read_file("/workspace/p/a.txt").is_err());
    }
}
