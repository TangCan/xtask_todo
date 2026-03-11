//! `clippy` subcommand - run clippy on the workspace.

use argh::FromArgs;
use std::ffi::OsString;
use std::process::{Command, Stdio};

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "clippy")]
/// Run clippy on the workspace (pedantic + nursery, -D warnings)
pub struct ClippyArgs {}

/// Maps a command status to Result. Used by `cmd_clippy` and tests.
pub fn status_to_result(
    status: std::process::ExitStatus,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(1);
        Err(std::io::Error::other(format!("{name} exited with code {code}")).into())
    }
}

/// Run clippy on the workspace.
///
/// # Errors
/// Returns an error if clippy exits with a non-zero status.
pub fn cmd_clippy(_args: ClippyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cmd = Command::new(cargo);
    cmd.args([
        "clippy",
        "--all-targets",
        "--",
        "-W",
        "clippy::pedantic",
        "-W",
        "clippy::nursery",
        "-D",
        "warnings",
    ]);
    if std::env::var_os("XTASK_CLIPPY_QUIET").is_some() {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = cmd.status()?;
    status_to_result(status, "clippy")
}
