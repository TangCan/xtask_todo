//! Unix socket / TCP / **Windows stdio via `podman machine ssh`** and `DEVSHELL_VM_SOCKET` parsing for β.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, ExitStatus};
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::PathBuf;

use super::super::config::ENV_DEVSHELL_VM_SOCKET;
#[cfg(windows)]
use super::super::podman_machine;
use super::super::VmError;

/// How to reach the β sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SocketSpec {
    #[cfg(unix)]
    Unix(PathBuf),
    /// `host:port`, e.g. `127.0.0.1:9847`
    Tcp(String),
    /// JSON lines over **`podman machine ssh`** stdin/stdout (Windows; no host TCP).
    #[cfg(windows)]
    Stdio,
}

pub(super) enum IpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    Tcp(TcpStream),
    #[cfg(windows)]
    StdioPipe(StdioPipe),
}

/// JSON line protocol over **`podman machine ssh`** pipes (single mutex for stdin + stdout reader).
pub(super) struct StdioPipe {
    inner: Arc<Mutex<StdioPipeInner>>,
}

struct StdioPipeInner {
    _child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl StdioPipe {
    pub(super) fn new(child: Child, stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let reader = BufReader::new(stdout);
        Self {
            inner: Arc::new(Mutex::new(StdioPipeInner {
                _child: child,
                stdin,
                reader,
            })),
        }
    }

    pub(super) fn read_json_line(&self) -> Result<serde_json::Value, VmError> {
        let mut out = String::new();
        let n = {
            let mut g = self.inner.lock().map_err(|e| VmError::Ipc(e.to_string()))?;
            g.reader
                .read_line(&mut out)
                .map_err(|e| VmError::Ipc(e.to_string()))?
        };
        if n == 0 || out.trim().is_empty() {
            return Err(VmError::Ipc(
                "beta sidecar (stdio) sent no JSON line (connection closed or empty response). \
                 Check podman machine start, devshell-vm Linux binary, or set DEVSHELL_VM_BACKEND=host."
                    .into(),
            ));
        }
        serde_json::from_str(out.trim()).map_err(|e| {
            VmError::Ipc(format!(
                "beta sidecar response is not JSON ({e}); first line prefix: {:?}",
                out.chars().take(80).collect::<String>()
            ))
        })
    }
}

impl Write for StdioPipe {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        g.stdin.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        g.stdin.flush()
    }
}

impl IpcStream {
    pub(super) fn try_clone(&self) -> std::io::Result<IpcStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => Ok(Self::Unix(u.try_clone()?)),
            Self::Tcp(t) => Ok(Self::Tcp(t.try_clone()?)),
            #[cfg(windows)]
            Self::StdioPipe(s) => Ok(Self::StdioPipe(StdioPipe {
                inner: Arc::clone(&s.inner),
            })),
        }
    }
}

impl Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.read(buf),
            Self::Tcp(t) => t.read(buf),
            #[cfg(windows)]
            Self::StdioPipe(s) => {
                let mut g = s
                    .inner
                    .lock()
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                g.reader.read(buf)
            }
        }
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.write(buf),
            Self::Tcp(t) => t.write(buf),
            #[cfg(windows)]
            Self::StdioPipe(s) => Write::write(s, buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.flush(),
            Self::Tcp(t) => t.flush(),
            #[cfg(windows)]
            Self::StdioPipe(s) => Write::flush(s),
        }
    }
}

pub(super) fn parse_devshell_vm_socket(raw: &str) -> Result<SocketSpec, VmError> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(VmError::Ipc(format!("{ENV_DEVSHELL_VM_SOCKET} is empty")));
    }
    #[cfg(windows)]
    if t.eq_ignore_ascii_case("stdio") {
        return Ok(SocketSpec::Stdio);
    }
    if let Some(rest) = t.strip_prefix("tcp://") {
        let addr = rest.trim();
        if addr.is_empty() {
            return Err(VmError::Ipc("tcp:// address is empty".into()));
        }
        return Ok(SocketSpec::Tcp(addr.to_string()));
    }
    if let Some(rest) = t.strip_prefix("tcp:") {
        let addr = rest.trim();
        if !addr.is_empty() && !addr.contains('\\') && !addr.starts_with('/') {
            return Ok(SocketSpec::Tcp(addr.to_string()));
        }
    }
    #[cfg(unix)]
    {
        Ok(SocketSpec::Unix(PathBuf::from(t)))
    }
    #[cfg(not(unix))]
    {
        Err(VmError::Ipc(
            "DEVSHELL_VM_SOCKET on Windows must be stdio or tcp:HOST:PORT (e.g. tcp:127.0.0.1:9847); see docs/devshell-vm-windows.md".into(),
        ))
    }
}

pub(super) fn connect_ipc(spec: &SocketSpec, workspace_root: &Path) -> Result<IpcStream, VmError> {
    #[cfg(not(windows))]
    let _ = workspace_root;
    match spec {
        #[cfg(unix)]
        SocketSpec::Unix(p) => UnixStream::connect(p).map(IpcStream::Unix).map_err(|e| {
            VmError::Ipc(format!(
                "connect {}: {e}; start: devshell-vm --serve-socket {}",
                p.display(),
                p.display()
            ))
        }),
        SocketSpec::Tcp(addr) => TcpStream::connect(addr).map(IpcStream::Tcp).map_err(|e| {
            let suffix = if cfg!(windows) {
                "\nIf nothing is listening: start devshell-vm --serve-tcp, or set DEVSHELL_VM_BACKEND=host"
            } else {
                ""
            };
            VmError::Ipc(format!(
                "connect tcp {addr}: {e}; start: devshell-vm --serve-tcp {addr}{suffix}"
            ))
        }),
        #[cfg(windows)]
        SocketSpec::Stdio => {
            let mut child = podman_machine::spawn_devshell_vm_stdio(workspace_root)?;
            let stdin = child.stdin.take().ok_or_else(|| {
                VmError::Ipc("podman machine ssh: missing stdin pipe".to_string())
            })?;
            let stdout = child.stdout.take().ok_or_else(|| {
                VmError::Ipc("podman machine ssh: missing stdout pipe".to_string())
            })?;
            Ok(IpcStream::StdioPipe(StdioPipe::new(child, stdin, stdout)))
        }
    }
}

pub(super) fn exit_status_from_code(code: i32) -> ExitStatus {
    let code = code.clamp(0, 255);
    #[cfg(windows)]
    {
        std::os::windows::process::ExitStatusExt::from_raw(code as u32)
    }
    #[cfg(not(windows))]
    {
        std::os::unix::process::ExitStatusExt::from_raw(code << 8)
    }
}
