//! `guest_fs` JSON op handlers: host-backed (session) and stub (no session).

use std::path::Path;

use base64::prelude::*;

use crate::server::SessionCtx;

pub fn guest_fs_on_host(
    ctx: &SessionCtx,
    v: &serde_json::Value,
    sid: &serde_json::Value,
) -> String {
    let guest_path = v.get("guest_path").and_then(|x| x.as_str()).unwrap_or("");
    let operation = v.get("operation").and_then(|x| x.as_str()).unwrap_or("");
    let host = match ctx.map_guest_to_host(guest_path) {
        Ok(p) => p,
        Err(msg) => {
            return serde_json::json!({
                "op": "guest_fs_error",
                "code": "invalid_path",
                "message": msg,
            })
            .to_string();
        }
    };

    match operation {
        "list_dir" => guest_fs_host_list_dir(&host, sid),
        "read_file" => guest_fs_host_read_file(&host, sid, guest_path),
        "write_file" => guest_fs_host_write_file(v, &host, sid),
        "mkdir" => guest_fs_host_mkdir(&host, sid),
        "remove" => guest_fs_host_remove(&host, sid, guest_path),
        _ => serde_json::json!({
            "op": "guest_fs_error",
            "code": "unknown",
            "message": operation,
        })
        .to_string(),
    }
}

fn guest_fs_host_list_dir(host: &Path, sid: &serde_json::Value) -> String {
    match std::fs::read_dir(host) {
        Ok(rd) => {
            let mut names: Vec<String> = rd
                .filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            serde_json::json!({
                "op": "guest_fs_ok",
                "session_id": sid,
                "names": names,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "op": "guest_fs_error",
            "code": "not_found",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

fn guest_fs_host_read_file(host: &Path, sid: &serde_json::Value, guest_path: &str) -> String {
    match std::fs::read(host) {
        Ok(bytes) => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
            "content_base64": BASE64_STANDARD.encode(&bytes),
        })
        .to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::json!({
            "op": "guest_fs_error",
            "code": "not_found",
            "message": guest_path,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "op": "guest_fs_error",
            "code": "io_error",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

fn guest_fs_host_write_file(v: &serde_json::Value, host: &Path, sid: &serde_json::Value) -> String {
    let b64 = v
        .get("content_base64")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let data = match BASE64_STANDARD.decode(b64) {
        Ok(d) => d,
        Err(e) => {
            return serde_json::json!({
                "op": "guest_fs_error",
                "code": "invalid_base64",
                "message": e.to_string(),
            })
            .to_string();
        }
    };
    if let Some(parent) = host.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(host, data) {
        Ok(()) => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "op": "guest_fs_error",
            "code": "io_error",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

fn guest_fs_host_mkdir(host: &Path, sid: &serde_json::Value) -> String {
    match std::fs::create_dir_all(host) {
        Ok(()) => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "op": "guest_fs_error",
            "code": "io_error",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

fn guest_fs_host_remove(host: &Path, sid: &serde_json::Value, guest_path: &str) -> String {
    let meta = std::fs::metadata(host);
    let r = match meta {
        Ok(m) if m.is_dir() => std::fs::remove_dir_all(host),
        Ok(_) => std::fs::remove_file(host),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return serde_json::json!({
                "op": "guest_fs_error",
                "code": "not_found",
                "message": guest_path,
            })
            .to_string();
        }
        Err(e) => Err(e),
    };
    match r {
        Ok(()) => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "op": "guest_fs_error",
            "code": "io_error",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

pub fn guest_fs_stub(sid: &serde_json::Value, v: &serde_json::Value) -> String {
    let guest_path = v.get("guest_path").and_then(|x| x.as_str()).unwrap_or("");
    let operation = v.get("operation").and_then(|x| x.as_str()).unwrap_or("");
    if !guest_path.starts_with("/workspace") && guest_path != "/" {
        return serde_json::json!({
            "op": "guest_fs_error",
            "code": "invalid_path",
            "message": guest_path,
        })
        .to_string();
    }
    match operation {
        "list_dir" => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
            "names": ["stub-entry"],
        })
        .to_string(),
        "read_file" => {
            if guest_path.contains("__missing__") {
                serde_json::json!({
                    "op": "guest_fs_error",
                    "code": "not_found",
                    "message": guest_path,
                })
                .to_string()
            } else {
                serde_json::json!({
                    "op": "guest_fs_ok",
                    "session_id": sid,
                    "content_base64": "YmV0YS1zdHViCg==",
                })
                .to_string()
            }
        }
        "write_file" | "mkdir" | "remove" => serde_json::json!({
            "op": "guest_fs_ok",
            "session_id": sid,
        })
        .to_string(),
        _ => serde_json::json!({
            "op": "guest_fs_error",
            "code": "unknown",
            "message": operation,
        })
        .to_string(),
    }
}
