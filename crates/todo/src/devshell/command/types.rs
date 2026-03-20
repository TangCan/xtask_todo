//! `RunResult`, `ExecContext`, and `BuiltinError` for command execution.

use super::super::vfs::Vfs;
use super::super::vm::SessionHolder;

/// Result of running a pipeline: continue the REPL loop or exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunResult {
    Continue,
    Exit,
}

/// Execution context: VFS and standard streams for one command.
pub struct ExecContext<'a> {
    pub vfs: &'a mut Vfs,
    pub stdin: &'a mut dyn std::io::Read,
    pub stdout: &'a mut dyn std::io::Write,
    pub stderr: &'a mut dyn std::io::Write,
    /// Rust tool execution session (host temp export or future VM).
    pub vm_session: &'a mut SessionHolder,
}

/// Error from builtin execution (redirect or VFS failure, unknown command).
#[derive(Debug)]
pub enum BuiltinError {
    UnknownCommand(String),
    RedirectRead,
    RedirectWrite,
    CdFailed,
    MkdirFailed,
    CatFailed,
    TouchFailed,
    LsFailed,
    ExportFailed,
    SaveFailed,
    TodoLoadFailed,
    TodoSaveFailed,
    TodoArgError,
    TodoDataError,
    /// rustup not found in PATH
    RustupNotFound,
    /// cargo not found in PATH
    CargoNotFound,
    /// Sandbox export (VFS to temp dir) failed
    SandboxExportFailed,
    /// Sandbox sync (temp dir back to VFS) failed
    SandboxSyncFailed,
    /// `cargo` / `rustup` subprocess exited with non-zero status (see stderr for tool output)
    RustToolNonZeroExit {
        program: String,
        code: Option<i32>,
    },
    /// VM workspace push/pull failed (see stderr for detail)
    VmWorkspaceSyncFailed,
    /// VM backend unavailable or misconfigured at startup
    VmSessionError(String),
}

impl std::fmt::Display for BuiltinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand(name) => write!(f, "unknown command: {name}"),
            Self::RedirectRead => f.write_str("redirect read failed"),
            Self::RedirectWrite => f.write_str("redirect write failed"),
            Self::CdFailed => f.write_str("cd failed"),
            Self::MkdirFailed => f.write_str("mkdir failed"),
            Self::CatFailed => f.write_str("cat failed"),
            Self::TouchFailed => f.write_str("touch failed"),
            Self::LsFailed => f.write_str("ls failed"),
            Self::ExportFailed => f.write_str("export failed"),
            Self::SaveFailed => f.write_str("save failed"),
            Self::TodoLoadFailed => f.write_str("todo load failed"),
            Self::TodoSaveFailed => f.write_str("todo save failed"),
            Self::TodoArgError => f.write_str("todo argument error"),
            Self::TodoDataError => f.write_str("todo data error"),
            Self::RustupNotFound => f.write_str("rustup not found in PATH"),
            Self::CargoNotFound => f.write_str("cargo not found in PATH"),
            Self::SandboxExportFailed => f.write_str("sandbox export failed"),
            Self::SandboxSyncFailed => f.write_str("sandbox sync failed"),
            Self::RustToolNonZeroExit { program, code } => match code {
                Some(c) => write!(f, "{program} exited with status {c}"),
                None => write!(f, "{program} exited with non-zero status"),
            },
            Self::VmWorkspaceSyncFailed => f.write_str("vm workspace sync failed"),
            Self::VmSessionError(msg) => write!(f, "{msg}"),
        }
    }
}
