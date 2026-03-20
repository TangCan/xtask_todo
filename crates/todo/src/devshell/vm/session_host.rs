//! Host-only [`super::VmExecutionSession`]: same behavior as historical `sandbox::run_rust_tool` (temp export per command).

#![allow(clippy::pedantic, clippy::nursery)]

use std::process::ExitStatus;

use super::super::sandbox;
use super::super::vfs::Vfs;
use super::{VmError, VmExecutionSession};

/// Runs `rustup`/`cargo` via temp directory export + sync (no persistent VM).
#[derive(Debug, Default, Clone, Copy)]
pub struct HostSandboxSession;

impl HostSandboxSession {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl VmExecutionSession for HostSandboxSession {
    fn ensure_ready(&mut self, _vfs: &Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        Ok(())
    }

    fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError> {
        sandbox::run_rust_tool(vfs, vfs_cwd, program, args).map_err(VmError::Sandbox)
    }

    fn shutdown(&mut self, _vfs: &mut Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        Ok(())
    }
}
