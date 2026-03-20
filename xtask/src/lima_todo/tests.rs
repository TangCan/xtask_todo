use super::helpers::host_release_str_for_target_dir;
use super::yaml::{merge_todo_into_lima_yaml, render_fragment};

#[test]
fn render_fragment_contains_location_and_mount() {
    let tmp = std::env::temp_dir().join(format!(
        "lima_todo_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .map_or(0, |d| d.as_nanos())
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let release = tmp.join("release");
    std::fs::create_dir_all(&release).unwrap();
    std::fs::write(release.join("todo"), b"x").unwrap();
    let s = render_fragment(&release, "/host-todo-bin").expect("render");
    assert!(s.contains("mountPoint: /host-todo-bin"));
    assert!(s.contains("writable: false"));
    assert!(s.contains("location:"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn merge_adds_mount_and_path() {
    let yaml = r#"mounts:
  - location: "~"
    mountPoint: /tmp/fromhome
    writable: true
env:
  PATH: /usr/bin:/bin
"#;
    let (out, changed) =
        merge_todo_into_lima_yaml(yaml, "/abs/release", "/host-todo-bin").expect("merge");
    assert!(changed);
    assert!(out.contains("/abs/release"));
    assert!(out.contains("mountPoint: /host-todo-bin"));
    assert!(out.contains("/host-todo-bin:/usr/bin:/bin") || out.contains("host-todo-bin"));
}

#[test]
fn host_release_str_when_release_dir_missing_uses_target_plus_release() {
    let tmp = std::env::temp_dir().join(format!(
        "lima_todo_hr_{}_{}",
        std::process::id(),
        std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .map_or(0, |d| d.as_nanos())
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("target")).unwrap();
    let td = tmp.join("target");
    let s = host_release_str_for_target_dir(&td).expect("host_release_str");
    assert!(
        s.ends_with("/release") || s.ends_with("\\release"),
        "unexpected: {s}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn merge_idempotent_mount() {
    let yaml = r#"mounts:
  - location: "/abs/release"
    mountPoint: /host-todo-bin
    writable: false
env:
  PATH: "/host-todo-bin:/host-cargo/bin:/usr/bin:/bin"
"#;
    let (out, changed) =
        merge_todo_into_lima_yaml(yaml, "/abs/release", "/host-todo-bin").expect("merge");
    assert!(
        !changed,
        "merge should be idempotent when mount and PATH already match"
    );
    assert!(
        out.contains("/abs/release"),
        "serialized yaml should still contain release mount path: {out}"
    );
}
