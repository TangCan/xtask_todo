//! `fmt` subcommand - run rustfmt on the workspace (equivalent to `cargo fmt`).

use argh::FromArgs;
use std::ffi::OsString;
use std::process::{Command, Stdio};

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "fmt")]
/// Run cargo fmt on the workspace
pub struct FmtArgs {}

/// Run cargo fmt (format all packages).
///
/// # Errors
/// Returns an error if cargo fmt exits with a non-zero status.
pub fn cmd_fmt(_args: FmtArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cmd = Command::new(cargo);
    cmd.arg("fmt");
    if std::env::var_os("XTASK_FMT_QUIET").is_some() {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(1);
        Err(std::io::Error::other(format!("cargo fmt exited with code {code}")).into())
    }
}
