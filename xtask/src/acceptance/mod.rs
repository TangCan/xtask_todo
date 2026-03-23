//! `cargo xtask acceptance` — run checks aligned with [docs/acceptance.md](../../docs/acceptance.md) and write a Markdown report.

mod checks;
mod report;

#[cfg(test)]
mod tests;

use std::fs;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use argh::FromArgs;

use checks::run_all_checks;
use report::{build_report, manual_skip_rows};

/// Outcome of a single automated check.
#[derive(Debug, Clone)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    /// Not run (missing tool, target, etc.).
    Skip(String),
}

/// One row in the automated section of the report.
#[derive(Debug)]
pub struct AutomatedCheck {
    pub id: &'static str,
    pub description: &'static str,
    pub command: String,
    pub status: CheckStatus,
}

/// Run acceptance automation and write report.
///
/// # Errors
/// Returns an error message if any **non-skipped** check fails or if I/O fails.
pub fn cmd_acceptance(args: AcceptanceArgs) -> Result<(), String> {
    let root = workspace_root()?;
    let checks = run_all_checks(&root);
    let manual = manual_skip_rows();
    let report = build_report(&root, &checks, &manual);

    if args.stdout_only {
        print!("{report}");
    } else {
        let out = args
            .output
            .unwrap_or_else(|| root.join("docs/acceptance-report.md"));
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&out).map_err(|e| e.to_string())?;
        IoWrite::write_all(&mut f, report.as_bytes()).map_err(|e| e.to_string())?;
        eprintln!("Wrote acceptance report to {}", out.display());
        // Short summary to stderr
        let fails = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Fail(_)))
            .count();
        let skips = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Skip(_)))
            .count();
        eprintln!(
            "Summary: {} checks, {} passed, {} failed, {} skipped (automated)",
            checks.len(),
            checks.len() - fails - skips,
            fails,
            skips
        );
    }

    let any_fail = checks
        .iter()
        .any(|c| matches!(c.status, CheckStatus::Fail(_)));
    if any_fail {
        return Err(
            "one or more acceptance checks failed — see report or run with RUST_BACKTRACE=1"
                .to_string(),
        );
    }
    Ok(())
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "acceptance")]
/// Run automated acceptance checks from docs/acceptance.md and write a Markdown report
pub struct AcceptanceArgs {
    /// write report to PATH (default: <workspace>/docs/acceptance-report.md)
    #[argh(option, short = 'o')]
    pub output: Option<PathBuf>,
    /// print report to stdout only; do not write a file
    #[argh(switch)]
    pub stdout_only: bool,
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut dir = cwd.clone();
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
            if text.contains("[workspace]") && text.contains("members") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(cwd)
}
