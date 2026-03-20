//! Merge `todo` release mount + `PATH` into Lima `lima.yaml` and render print-only fragments.

use std::fmt::Write as _;
use std::path::Path;

use serde_yaml::{Mapping, Value as YamlValue};

fn default_guest_path_value(guest_mount: &str) -> String {
    format!(
        "{guest_mount}:/host-cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    )
}

/// Merge `todo` release mount + `env.PATH` into Lima `lima.yaml` text. `host_release_abs` must be absolute.
/// Returns serialized YAML and whether anything was changed (skip write/restart when `false`).
pub(super) fn merge_todo_into_lima_yaml(
    content: &str,
    host_release_abs: &str,
    guest_mount: &str,
) -> Result<(String, bool), String> {
    let mut root: YamlValue =
        serde_yaml::from_str(content).map_err(|e| format!("parse lima.yaml: {e}"))?;
    let mapping = root
        .as_mapping_mut()
        .ok_or_else(|| "lima.yaml: root must be a mapping".to_string())?;

    let mut changed = false;

    // --- mounts ---
    let mounts_key = YamlValue::String("mounts".into());
    if !mapping.contains_key(&mounts_key) {
        mapping.insert(mounts_key.clone(), YamlValue::Sequence(Vec::new()));
        changed = true;
    }
    let mounts_val = mapping
        .get_mut(&mounts_key)
        .ok_or_else(|| "lima.yaml: internal mounts".to_string())?;
    let mounts_seq = mounts_val
        .as_sequence_mut()
        .ok_or_else(|| "lima.yaml: `mounts` must be a YAML sequence (list)".to_string())?;

    let mut already = false;
    for item in mounts_seq.iter() {
        let Some(m) = item.as_mapping() else {
            continue;
        };
        if let Some(YamlValue::String(loc)) = m.get(YamlValue::String("location".into())) {
            if loc == host_release_abs {
                already = true;
                break;
            }
        }
    }

    if !already {
        let mut mount = Mapping::new();
        mount.insert(
            YamlValue::String("location".into()),
            YamlValue::String(host_release_abs.to_string()),
        );
        mount.insert(
            YamlValue::String("mountPoint".into()),
            YamlValue::String(guest_mount.to_string()),
        );
        mount.insert(YamlValue::String("writable".into()), YamlValue::Bool(false));
        mounts_seq.push(YamlValue::Mapping(mount));
        changed = true;
    }

    // --- env.PATH ---
    let env_key = YamlValue::String("env".into());
    if !mapping.contains_key(&env_key) {
        mapping.insert(env_key.clone(), YamlValue::Mapping(Mapping::new()));
        changed = true;
    }
    let env_val = mapping
        .get_mut(&env_key)
        .ok_or_else(|| "lima.yaml: internal env".to_string())?;
    let env_map = env_val
        .as_mapping_mut()
        .ok_or_else(|| "lima.yaml: `env` must be a mapping".to_string())?;

    let path_key = YamlValue::String("PATH".into());
    let desired_prefix = default_guest_path_value(guest_mount);

    match env_map.get(&path_key) {
        Some(YamlValue::String(s)) => {
            let s = s.trim();
            if s.is_empty() {
                env_map.insert(path_key, YamlValue::String(desired_prefix));
                changed = true;
            } else if s == guest_mount
                || s.starts_with(&format!("{guest_mount}:"))
                || s.starts_with(&format!("{guest_mount}/"))
            {
                // Already prepended
            } else {
                let merged = format!("{guest_mount}:{s}");
                env_map.insert(path_key, YamlValue::String(merged));
                changed = true;
            }
        }
        Some(_) => {
            return Err("lima.yaml: env.PATH must be a string to merge safely".to_string());
        }
        None => {
            env_map.insert(path_key, YamlValue::String(desired_prefix));
            changed = true;
        }
    }

    let out = serde_yaml::to_string(&root).map_err(|e| format!("serialize lima.yaml: {e}"))?;
    Ok((out, changed))
}

/// YAML comment + mounts entry + env hint for merging `PATH` (for `--print-only` / `--write`).
pub(super) fn render_fragment(
    host_release_dir: &Path,
    guest_mount: &str,
) -> Result<String, String> {
    let abs = host_release_dir
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {e}", host_release_dir.display()))?;
    let loc = abs.to_string_lossy().replace('\\', "/");

    let mut s = String::new();
    s.push_str("# Generated for Lima: mount host `target/release` so guest can run `todo`.\n");
    s.push_str("# (Default `cargo xtask lima-todo` merges into ~/.lima/<instance>/lima.yaml automatically.)\n");
    s.push_str("# Merge `mounts:` entry below into ~/.lima/<instance>/lima.yaml\n");
    s.push_str("# and prepend guest_mount to env.PATH (see block at end).\n");
    s.push_str("#\n");
    s.push_str("# --- mounts: (append one list item) ---\n");
    let _ = writeln!(s, "  - location: \"{loc}\"");
    let _ = writeln!(s, "    mountPoint: {guest_mount}");
    s.push_str("    writable: false\n");
    s.push_str("#\n");
    s.push_str("# --- env: merge PATH (prepend; keep existing /host-cargo/bin if you use Rust mounts) ---\n");
    let _ = writeln!(s, "#   PATH: \"{}\"", default_guest_path_value(guest_mount));
    s.push_str("#\n");
    s.push_str("# Then: limactl stop <instance> && limactl start -y <instance>\n");
    s.push_str("# In guest: cd /workspace/<project> && todo list\n");
    Ok(s)
}
