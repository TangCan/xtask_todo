//! `clean` subcommand - remove leftover xtask test directories from project root.

use argh::FromArgs;
use std::fs;

const PREFIXES: &[&str] = &["xtask_clippy_", "xtask_nongit_", "xtask_git_"];

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "clean")]
/// Remove `xtask_*` test directories (`xtask_clippy_*`, `xtask_nongit_*`, `xtask_git_*`) from project root
pub struct CleanArgs {}

/// Remove directories in `root` whose names start with any of `PREFIXES`.
///
/// # Errors
/// Returns an error if reading the directory or removing a matching directory fails.
pub fn cmd_clean(_args: CleanArgs) -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    let entries = fs::read_dir(&root)?;
    let mut removed = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if PREFIXES.iter().any(|p| name.starts_with(p)) {
            fs::remove_dir_all(&path)?;
            removed.push(path);
        }
    }
    for p in &removed {
        println!("removed {}", p.display());
    }
    if removed.is_empty() {
        println!("no xtask_* directories found in {}", root.display());
    }
    Ok(())
}
