//! `ghcr` subcommand — latest **devshell-vm** OCI tag and `podman pull` line (for Windows β fallback).

use argh::FromArgs;
use semver::Version;
use serde_json::Value;

/// crates.io and GitHub require a descriptive User-Agent.
const HTTP_USER_AGENT: &str = concat!(
    "xtask-todo-ghcr/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/TangCan/xtask_todo)"
);

/// Align with `crates/todo` `default_container_image()` (`ghcr.io/tangcan/xtask_todo/devshell-vm:v…`).
const CRATE_NAME: &str = "xtask-todo-lib";
const GH_OWNER_LOWER: &str = "tangcan";
/// API path uses the canonical owner/repo casing from `repository` in Cargo.toml.
const GITHUB_REPO_PATH: &str = "TangCan/xtask_todo";
const PACKAGE_NAME: &str = "devshell-vm";
const RELEASE_TAG_PREFIX: &str = "xtask-todo-lib-v";

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "ghcr")]
/// Latest published devshell-vm OCI image tag and podman pull command (GitHub Releases + crates.io APIs).
pub struct GhcrArgs {
    /// source: auto (default), releases, crates-io, or github-packages
    #[argh(option, short = 's', long = "source")]
    pub source: Option<String>,
}

/// Run `ghcr` subcommand.
///
/// # Errors
/// Returns an error if HTTP fails or JSON is unexpected.
pub fn cmd_ghcr(args: &GhcrArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mode = args
        .source
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("auto");

    let (resolved_from, tag) = match mode {
        "crates-io" => (
            "crates.io (max_version)".to_string(),
            tag_from_crates_io_max_version()?,
        ),
        "releases" => (
            "GitHub Releases (latest)".to_string(),
            tag_from_github_releases_latest()?,
        ),
        "github-packages" => (
            "GitHub Container Registry (package versions API)".to_string(),
            tag_from_github_package_versions()?,
        ),
        "auto" => match tag_from_github_releases_latest() {
            Ok(t) => ("GitHub Releases (latest tag)".to_string(), t),
            Err(e_rel) => {
                eprintln!("Note: GitHub releases/latest unavailable ({e_rel}); using crates.io.");
                (
                    "crates.io (max_version)".to_string(),
                    tag_from_crates_io_max_version()?,
                )
            }
        },
        _ => {
            return Err(
                "invalid --source (use auto, releases, crates-io, or github-packages)".into(),
            );
        }
    };

    let image = format!(
        "ghcr.io/{}/{}/{}:{}",
        GH_OWNER_LOWER, "xtask_todo", PACKAGE_NAME, tag
    );

    println!("Resolved from: {resolved_from}");
    println!("Latest image tag (semver): {tag}");
    println!();
    println!("Full reference (matches `cargo-devshell` default when DEVSHELL_VM_CONTAINER_IMAGE is unset):");
    println!("  {image}");
    println!();
    println!("Verify with Podman:");
    println!("  podman pull {image}");
    println!();
    println!(
        "If `podman pull` fails with 404, the OCI job may have failed or the image is not public."
    );
    println!("Confirm: GitHub → Actions → Release → devshell-vm-oci job succeeded for this tag.");

    Ok(())
}

/// `GET /repos/{owner}/{repo}/releases/latest` — public, no token; tag matches Release + OCI workflow.
fn tag_from_github_releases_latest() -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO_PATH}/releases/latest");
    let body = http_get_text(&url, "application/vnd.github+json")?;
    parse_github_releases_latest_json(&body)
}

/// Parse GitHub `releases/latest` JSON body (unit-tested without network).
fn parse_github_releases_latest_json(body: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let tag_name = v
        .get("tag_name")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "GitHub releases: missing tag_name".to_string())?;
    let rest = tag_name.strip_prefix(RELEASE_TAG_PREFIX).ok_or_else(|| {
        format!("GitHub releases: expected tag prefix {RELEASE_TAG_PREFIX}, got {tag_name}")
    })?;
    validate_semver(rest)?;
    Ok(format!("v{rest}"))
}

fn tag_from_crates_io_max_version() -> Result<String, String> {
    let url = format!("https://crates.io/api/v1/crates/{CRATE_NAME}");
    let body = http_get_text(&url, "application/json")?;
    parse_crates_io_max_version_json(&body)
}

fn parse_crates_io_max_version_json(body: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let max = v
        .get("crate")
        .and_then(|c| c.get("max_version"))
        .and_then(|x| x.as_str())
        .ok_or_else(|| "crates.io: missing crate.max_version".to_string())?;
    validate_semver(max)?;
    Ok(format!("v{max}"))
}

/// Optional; GitHub often returns 401 without `GITHUB_TOKEN` — set env var for this source.
fn tag_from_github_package_versions() -> Result<String, String> {
    let url = format!(
        "https://api.github.com/users/{GH_OWNER_LOWER}/packages/container/{PACKAGE_NAME}/versions?per_page=100"
    );
    let body = http_get_github_maybe_authenticated(&url)?;
    best_semver_tag_from_package_versions_json(&body)
}

/// Pick highest `v*` semver tag from GitHub package versions API JSON (unit-tested without network).
fn best_semver_tag_from_package_versions_json(body: &str) -> Result<String, String> {
    let arr: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let versions = arr
        .as_array()
        .ok_or_else(|| "GitHub packages: expected JSON array".to_string())?;

    let mut best: Option<(Version, String)> = None;
    for ver in versions {
        let tags = ver
            .get("metadata")
            .and_then(|m| m.get("container"))
            .and_then(|c| c.get("tags"))
            .and_then(|t| t.as_array())
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str());

        for t in tags {
            if let Some(rest) = t.strip_prefix('v') {
                if let Ok(v) = Version::parse(rest) {
                    let replace = match &best {
                        None => true,
                        Some((bv, _)) => v > *bv,
                    };
                    if replace {
                        best = Some((v, t.to_string()));
                    }
                }
            }
        }
    }

    best.map(|(_, tag)| tag).ok_or_else(|| {
        "no v* semver tags found on GHCR package (or set GITHUB_TOKEN if you see 401)".to_string()
    })
}

fn http_get_github_maybe_authenticated(url: &str) -> Result<String, String> {
    let mut req = ureq::get(url)
        .set("User-Agent", HTTP_USER_AGENT)
        .set("Accept", "application/vnd.github+json");
    if let Ok(tok) = std::env::var("GITHUB_TOKEN") {
        let t = tok.trim();
        if !t.is_empty() {
            req = req.set("Authorization", &format!("Bearer {t}"));
        }
    }
    let resp = req.call().map_err(|e| format!("GET {url}: {e}"))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(format!(
            "GET {url}: HTTP {status} (for unauthenticated access, try: cargo xtask ghcr --source auto)"
        ));
    }
    resp.into_string()
        .map_err(|e| format!("read body {url}: {e}"))
}

fn http_get_text(url: &str, accept: &str) -> Result<String, String> {
    let resp = ureq::get(url)
        .set("User-Agent", HTTP_USER_AGENT)
        .set("Accept", accept)
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;

    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(format!("GET {url}: HTTP {status}"));
    }

    resp.into_string()
        .map_err(|e| format!("read body {url}: {e}"))
}

fn validate_semver(s: &str) -> Result<(), String> {
    Version::parse(s).map_err(|_| format!("not a semver: {s}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_ghcr_invalid_source_errors() {
        let r = cmd_ghcr(&GhcrArgs {
            source: Some("nope".into()),
        });
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("invalid --source"));
    }

    #[test]
    fn validate_semver_accepts_and_rejects() {
        assert!(validate_semver("0.1.22").is_ok());
        assert!(validate_semver("not-a-version").is_err());
    }

    #[test]
    fn parse_github_releases_latest_json_ok() {
        let j = r#"{"tag_name":"xtask-todo-lib-v0.1.22"}"#;
        assert_eq!(parse_github_releases_latest_json(j).unwrap(), "v0.1.22");
    }

    #[test]
    fn parse_github_releases_latest_json_wrong_prefix() {
        let j = r#"{"tag_name":"v0.1.0"}"#;
        let e = parse_github_releases_latest_json(j).unwrap_err();
        assert!(e.contains("expected tag prefix"));
    }

    #[test]
    fn parse_github_releases_latest_json_missing_tag_name() {
        let j = r"{}";
        assert!(parse_github_releases_latest_json(j)
            .unwrap_err()
            .contains("tag_name"));
    }

    #[test]
    fn parse_crates_io_max_version_json_ok() {
        let j = r#"{"crate":{"max_version":"0.2.0"}}"#;
        assert_eq!(parse_crates_io_max_version_json(j).unwrap(), "v0.2.0");
    }

    #[test]
    fn parse_crates_io_max_version_json_missing() {
        let j = r#"{"crate":{}}"#;
        assert!(parse_crates_io_max_version_json(j)
            .unwrap_err()
            .contains("max_version"));
    }

    #[test]
    fn best_semver_tag_from_package_versions_prefers_highest() {
        let j = r#"[
          {"metadata":{"container":{"tags":["v0.1.0","latest"]}}},
          {"metadata":{"container":{"tags":["v0.1.22"]}}}
        ]"#;
        assert_eq!(
            best_semver_tag_from_package_versions_json(j).unwrap(),
            "v0.1.22"
        );
    }

    #[test]
    fn best_semver_tag_from_package_versions_empty_array_errors() {
        let e = best_semver_tag_from_package_versions_json("[]").unwrap_err();
        assert!(e.contains("no v* semver"));
    }

    #[test]
    fn best_semver_tag_from_package_versions_not_array_errors() {
        assert!(best_semver_tag_from_package_versions_json("{}")
            .unwrap_err()
            .contains("array"));
    }
}
