//! `gh` subcommand - show GitHub Actions run log (e.g. `gh log` = latest run log).

use argh::FromArgs;
use std::process::Command;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "gh")]
/// GitHub CLI helpers (e.g. show latest Actions run log)
pub struct GhArgs {
    #[argh(subcommand)]
    pub sub: GhSub,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum GhSub {
    Log(GhLogArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "log")]
/// Show log of the most recent GitHub Actions run (equiv: gh run view $(gh run list --limit 1 --json databaseId -q '.[0].databaseId') --log)
pub struct GhLogArgs {}

/// Run gh subcommand.
///
/// # Errors
/// Returns an error if `gh` is not found, list returns no run, or view fails; prints message to stderr.
pub fn cmd_gh(args: &GhArgs) -> Result<(), Box<dyn std::error::Error>> {
    match &args.sub {
        GhSub::Log(_) => cmd_gh_log(),
    }
}

fn cmd_gh_log() -> Result<(), Box<dyn std::error::Error>> {
    let out = Command::new("gh")
        .args([
            "run",
            "list",
            "--limit",
            "1",
            "--json",
            "databaseId",
            "-q",
            ".[0].databaseId",
        ])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("gh: command not found");
            } else {
                eprintln!("gh: {e}");
            }
            return Err(e.into());
        }
    };

    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if msg.is_empty() {
            eprintln!("gh run list failed");
        } else {
            eprintln!("{msg}");
        }
        return Err("gh run list failed".into());
    }

    let id = String::from_utf8(out.stdout)
        .map_err(|_| "gh run list produced invalid UTF-8")?
        .trim()
        .to_string();

    if id.is_empty() {
        eprintln!("no runs found");
        return Err("no runs found".into());
    }

    let status = Command::new("gh")
        .args(["run", "view", &id, "--log"])
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        eprintln!("gh run view exited with code {code}");
        return Err(format!("gh run view failed (exit code {code})").into());
    }

    Ok(())
}
