//! `RunResult`, `ExecContext`, and `BuiltinError` for command execution.

use super::super::vfs::Vfs;

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
}
