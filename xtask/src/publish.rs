//! `publish` subcommand - bump version, publish to crates.io, tag, and push to GitHub.

use argh::FromArgs;
use std::fs;
use std::path::Path;
use std::process::Command;

const CRATE_CARGO: &str = "crates/todo/Cargo.toml";
const PACKAGE: &str = "xtask-todo-lib";

/// Bump patch version (e.g. 0.1.2 -> 0.1.3) in crates/todo/Cargo.toml and return the new version.
fn bump_version_in_cargo_toml(workspace_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = workspace_root.join(CRATE_CARGO);
    let content = fs::read_to_string(&path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version = ") {
            let rest = trimmed
                .strip_prefix("version = ")
                .ok_or("version line format")?
                .trim();
            let quote_char = rest.chars().next().ok_or("version value")?;
            if quote_char != '"' && quote_char != '\'' {
                continue;
            }
            let rest = &rest[1..];
            let end = rest.find(quote_char).ok_or("version end quote")?;
            let version = rest[..end].trim();
            let parts: Vec<u32> = version
                .split('.')
                .map(|s: &str| s.parse::<u32>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "version must be major.minor.patch")?;
            if parts.len() != 3 {
                return Err("version must be major.minor.patch".into());
            }
            let new_version = format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1);
            let old_version_str = format!("{quote_char}{version}{quote_char}");
            let new_version_str = format!("{quote_char}{new_version}{quote_char}");
            let new_content = content.replace(
                &format!("version = {old_version_str}"),
                &format!("version = {new_version_str}"),
            );
            if new_content == content {
                return Err("version replacement failed".into());
            }
            fs::write(&path, &new_content)?;
            return Ok(new_version);
        }
    }
    Err("no version = \"...\" found in crates/todo/Cargo.toml".into())
}

fn run(cmd: &mut Command, step: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = cmd.status()?;
    if !status.success() {
        let code = status.code().unwrap_or(1);
        return Err(std::io::Error::other(format!("{step} failed with exit code {code}")).into());
    }
    Ok(())
}

/// Publish subcommand: bump version, publish to crates.io, tag, push to GitHub.
#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "publish")]
/// Bump patch version, publish xtask-todo-lib to crates.io, create tag, push branch and tag to GitHub.
pub struct PublishArgs {}

/// Run publish: bump version -> commit -> cargo publish -> tag -> push branch and tag.
///
/// # Errors
/// Fails if version bump, git, or cargo publish fails.
pub fn cmd_publish(_args: &PublishArgs) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = std::env::current_dir().map_err(|e| format!("current_dir: {e}"))?;
    let cargo_path = workspace_root.join(CRATE_CARGO);
    if !cargo_path.exists() {
        return Err(format!("{CRATE_CARGO} not found (run from workspace root)").into());
    }

    let new_version = bump_version_in_cargo_toml(&workspace_root)?;
    let tag = format!("{PACKAGE}-v{new_version}");
    println!("Bumped to {new_version} (tag: {tag})");

    run(
        Command::new("git")
            .args(["add", CRATE_CARGO])
            .current_dir(&workspace_root),
        "git add",
    )?;
    run(
        Command::new("git")
            .args(["commit", "-m", &format!("Release {PACKAGE} v{new_version}")])
            .current_dir(&workspace_root),
        "git commit",
    )?;
    run(
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["publish", "-p", PACKAGE, "--registry", "crates-io"])
            .current_dir(&workspace_root),
        "cargo publish",
    )?;
    run(
        Command::new("git")
            .args(["tag", &tag])
            .current_dir(&workspace_root),
        "git tag",
    )?;
    run(
        Command::new("git")
            .args(["push", "origin", "HEAD"])
            .current_dir(&workspace_root),
        "git push branch",
    )?;
    run(
        Command::new("git")
            .args(["push", "origin", &tag])
            .current_dir(&workspace_root),
        "git push tag",
    )?;
    println!("Done. GitHub Release will be created by workflow for tag {tag}.");
    Ok(())
}
