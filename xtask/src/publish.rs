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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn run_success() {
        let mut cmd = Command::new("true");
        assert!(run(&mut cmd, "true").is_ok());
    }

    #[test]
    fn run_failure() {
        let mut cmd = Command::new("false");
        let err = run(&mut cmd, "step").unwrap_err();
        assert!(err.to_string().contains("step"));
        assert!(err.to_string().contains("exit code"));
    }

    #[test]
    fn bump_version_in_cargo_toml_bumps_patch() {
        let dir = std::env::temp_dir().join(format!("xtask_publish_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("crates/todo"));
        let cargo = r#"[package]
name = "todo"
version = "0.1.2"
edition = "2021"
"#;
        std::fs::write(dir.join("crates/todo/Cargo.toml"), cargo).unwrap();
        let new_ver = bump_version_in_cargo_toml(&dir).unwrap();
        assert_eq!(new_ver, "0.1.3");
        let content = std::fs::read_to_string(dir.join("crates/todo/Cargo.toml")).unwrap();
        assert!(content.contains("version = \"0.1.3\""));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bump_version_in_cargo_toml_single_quotes() {
        let dir =
            std::env::temp_dir().join(format!("xtask_publish_test_sq_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("crates/todo"));
        std::fs::write(dir.join("crates/todo/Cargo.toml"), "version = '1.0.0'\n").unwrap();
        let new_ver = bump_version_in_cargo_toml(&dir).unwrap();
        assert_eq!(new_ver, "1.0.1");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bump_version_in_cargo_toml_no_version_line_err() {
        let dir =
            std::env::temp_dir().join(format!("xtask_publish_test_nv_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("crates/todo"));
        std::fs::write(
            dir.join("crates/todo/Cargo.toml"),
            "[package]\nname = \"x\"\n",
        )
        .unwrap();
        let err = bump_version_in_cargo_toml(&dir).unwrap_err();
        assert!(err.to_string().contains("no version"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bump_version_in_cargo_toml_invalid_triple_err() {
        let dir =
            std::env::temp_dir().join(format!("xtask_publish_test_bad_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("crates/todo"));
        std::fs::write(dir.join("crates/todo/Cargo.toml"), "version = \"1.0\"\n").unwrap();
        let err = bump_version_in_cargo_toml(&dir).unwrap_err();
        assert!(err.to_string().contains("major.minor.patch"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn cmd_publish_fails_when_crate_cargo_missing() {
        let dir = std::env::temp_dir().join(format!("xtask_publish_cmd_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = cmd_publish(&PublishArgs {});
        std::env::set_current_dir(&cwd).unwrap();
        let _ = std::fs::remove_dir_all(dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    /// Full `cmd_publish` success path with fake git and cargo (Unix only).
    #[test]
    #[cfg(unix)]
    fn cmd_publish_succeeds_with_fake_git_and_cargo() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("xtask_publish_ok_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("crates/todo"));
        let _ = std::fs::create_dir_all(dir.join("bin"));
        let bin = dir.join("bin");
        for (name, script) in [
            ("git", "#!/bin/sh\nexit 0\n"),
            ("cargo", "#!/bin/sh\nexit 0\n"),
        ] {
            let p = bin.join(name);
            let mut f = std::fs::File::create(&p).unwrap();
            f.write_all(script.as_bytes()).unwrap();
            f.sync_all().unwrap();
            drop(f);
            let mut perms = std::fs::metadata(&p).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&p, perms).unwrap();
        }
        std::fs::write(
            dir.join("crates/todo/Cargo.toml"),
            "[package]\nname = \"todo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        let cwd = std::env::current_dir().unwrap();
        let old_path = std::env::var_os("PATH");
        let old_cargo = std::env::var_os("CARGO");
        let bin_abs = std::fs::canonicalize(&bin).unwrap();
        let mut path_vec: Vec<_> =
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        path_vec.insert(0, bin_abs.clone());
        std::env::set_var("PATH", std::env::join_paths(path_vec).unwrap());
        std::env::set_var("CARGO", bin_abs.join("cargo"));
        std::env::set_current_dir(&dir).unwrap();
        let result = cmd_publish(&PublishArgs {});
        std::env::set_current_dir(&cwd).unwrap();
        if let Some(p) = old_path {
            std::env::set_var("PATH", p);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(c) = old_cargo {
            std::env::set_var("CARGO", c);
        } else {
            std::env::remove_var("CARGO");
        }
        let _ = std::fs::remove_dir_all(dir);
        assert!(
            result.is_ok(),
            "cmd_publish with fake git/cargo: {result:?}"
        );
    }
}
