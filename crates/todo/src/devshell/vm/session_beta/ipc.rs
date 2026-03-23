//! Unix socket / TCP stream and `DEVSHELL_VM_SOCKET` parsing for β.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::ExitStatus;

#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::PathBuf;

use super::super::config::ENV_DEVSHELL_VM_SOCKET;
use super::super::VmError;

/// How to reach the β sidecar.
#[derive(Debug, Clone)]
pub(super) enum SocketSpec {
    #[cfg(unix)]
    Unix(PathBuf),
    /// `host:port`, e.g. `127.0.0.1:9847`
    Tcp(String),
}

pub(super) enum IpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl IpcStream {
    pub(super) fn try_clone(&self) -> std::io::Result<IpcStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => Ok(Self::Unix(u.try_clone()?)),
            Self::Tcp(t) => Ok(Self::Tcp(t.try_clone()?)),
        }
    }
}

impl Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.read(buf),
            Self::Tcp(t) => t.read(buf),
        }
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.write(buf),
            Self::Tcp(t) => t.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.flush(),
            Self::Tcp(t) => t.flush(),
        }
    }
}

pub(super) fn parse_devshell_vm_socket(raw: &str) -> Result<SocketSpec, VmError> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(VmError::Ipc(format!("{ENV_DEVSHELL_VM_SOCKET} is empty")));
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
            "DEVSHELL_VM_SOCKET on Windows must be tcp:HOST:PORT (e.g. tcp:127.0.0.1:9847); see docs/devshell-vm-windows.md".into(),
        ))
    }
}

pub(super) fn connect_ipc(spec: &SocketSpec) -> Result<IpcStream, VmError> {
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
                "\nIf nothing is listening: install Podman (winget install -e --id Podman.Podman), \
                 run podman machine start, or cd to xtask_todo repo for auto-start. \
                 Host-only: set DEVSHELL_VM_BACKEND=host"
            } else {
                ""
            };
            VmError::Ipc(format!(
                "connect tcp {addr}: {e}; start: devshell-vm --serve-tcp {addr}{suffix}"
            ))
        }),
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
