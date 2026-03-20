use std::fs;
use std::path::{Path, PathBuf};

use base64::prelude::*;

use crate::server::{handle_line, ServerState, SessionCtx};

fn line_session_start(staging: &Path) -> String {
    serde_json::json!({
        "op": "session_start",
        "session_id": "t",
        "staging_dir": staging.to_str().unwrap(),
        "guest_workspace": "/workspace",
    })
    .to_string()
}

#[test]
fn handle_handshake() {
    let mut st = ServerState::default();
    let out = handle_line(r#"{"op":"handshake","version":1}"#, &mut st);
    assert!(out.contains("handshake_ok"));
}

#[test]
fn handle_exec_fail_flag() {
    let mut st = ServerState::default();
    let out = handle_line(
        r#"{"op":"exec","session_id":"s","argv":["cargo","--devshell-vm-test-fail"]}"#,
        &mut st,
    );
    assert!(out.contains("\"exit_code\":1"));
}

#[test]
fn handle_guest_fs_list_dir_stub_without_session() {
    let mut st = ServerState::default();
    let out = handle_line(
        r#"{"op":"guest_fs","session_id":"s","operation":"list_dir","guest_path":"/workspace"}"#,
        &mut st,
    );
    assert!(out.contains("guest_fs_ok"));
    assert!(out.contains("stub-entry"));
}

#[test]
fn handle_guest_fs_read_missing_stub() {
    let mut st = ServerState::default();
    let out = handle_line(
        r#"{"op":"guest_fs","session_id":"s","operation":"read_file","guest_path":"/workspace/__missing__/x"}"#,
        &mut st,
    );
    assert!(out.contains("guest_fs_error"));
    assert!(out.contains("not_found"));
}

#[test]
fn guest_fs_reads_host_file_after_session_start() {
    let dir = std::env::temp_dir().join(format!(
        "devshell_vm_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("hello.txt"), b"hello-beta").unwrap();

    let mut st = ServerState::default();
    let _ = handle_line(&line_session_start(&dir), &mut st);

    let out = handle_line(
        r#"{"op":"guest_fs","session_id":"t","operation":"read_file","guest_path":"/workspace/hello.txt"}"#,
        &mut st,
    );
    assert!(out.contains("guest_fs_ok"));
    assert!(out.contains("content_base64"));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let b64 = v.get("content_base64").and_then(|x| x.as_str()).unwrap();
    let bytes = BASE64_STANDARD.decode(b64).unwrap();
    assert_eq!(bytes, b"hello-beta");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn map_guest_to_host_rejects_escape() {
    let ctx = SessionCtx {
        staging_root: PathBuf::from("/tmp"),
        guest_mount: "/workspace".to_string(),
    };
    assert!(ctx.map_guest_to_host("/workspace/../etc/passwd").is_err());
}
