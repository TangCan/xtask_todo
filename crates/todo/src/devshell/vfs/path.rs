//! Logical path normalization and resolution against cwd.

/// Resolve `path` against `cwd` to an absolute logical path (same rules as [`super::Vfs::resolve_to_absolute`]).
#[must_use]
pub fn resolve_path_with_cwd(cwd: &str, path: &str) -> String {
    let path = path.trim();
    let path_normalized = normalize_path(path);
    // 绝对路径：直接归一化后返回
    if path_normalized.starts_with('/') {
        return path_normalized;
    }
    if path_normalized == "/" {
        return cwd.to_string();
    }
    // 相对路径：先与 cwd 拼接再归一化，避免单独 ".." 被归一成 "." 导致无法退回根目录
    let base = cwd.trim_end_matches('/');
    let p = path.trim_start_matches('/');
    let combined = if base.is_empty() {
        format!("/{p}")
    } else {
        format!("{base}/{p}")
    };
    let result = normalize_path(&combined);
    if result.is_empty() || result == "." {
        "/".to_string()
    } else if result.starts_with('/') {
        result
    } else {
        format!("/{result}")
    }
}

/// Normalize a path to Unix style: backslash -> slash, strip Windows drive,
/// resolve . and .., preserve absolute vs relative.
#[must_use]
pub fn normalize_path(input: &str) -> String {
    let s = input.replace('\\', "/");

    // Strip Windows drive letter prefix (e.g. C:) and treat as absolute.
    let (rest, absolute) = if s.len() >= 2
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && s.chars().nth(1) == Some(':')
    {
        (&s[2..], true)
    } else {
        (s.as_str(), s.starts_with('/'))
    };

    let rest = rest.trim_start_matches('/');
    let mut out: Vec<&str> = Vec::new();
    for p in rest.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            _ => out.push(p),
        }
    }

    if absolute {
        "/".to_string() + &out.join("/")
    } else if out.is_empty() {
        ".".to_string()
    } else {
        out.join("/")
    }
}
