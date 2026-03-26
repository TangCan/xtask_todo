use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Mutex, OnceLock, PoisonError};

use crate::devshell::vfs::Vfs;
use crate::devshell::vm::{
    session_beta::BetaSession, VmConfig, VmExecutionSession, ENV_DEVSHELL_VM_SOCKET,
};

fn vm_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn restore_var(key: &str, val: Option<std::ffi::OsString>) {
    match val {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

fn test_config() -> VmConfig {
    VmConfig {
        enabled: true,
        backend: "beta".to_string(),
        eager_start: false,
        lima_instance: "devshell-rust".to_string(),
    }
}

#[test]
fn ensure_ready_handshake_uses_single_json_line_and_accepts_handshake_ok() {
    let _g = vm_env_lock();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock sidecar");
    let addr = listener.local_addr().expect("local addr");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let reader_stream = stream.try_clone().expect("clone");
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read handshake");
        assert!(
            line.ends_with('\n'),
            "request must be newline-delimited JSON line: {line:?}"
        );
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("valid json");
        assert_eq!(
            req.get("op").and_then(serde_json::Value::as_str),
            Some("handshake")
        );
        assert_eq!(
            req.get("version").and_then(serde_json::Value::as_u64),
            Some(1)
        );
        writeln!(stream, "{{\"op\":\"handshake_ok\"}}").expect("write response");
        stream.flush().expect("flush");
    });

    let old_sock = std::env::var_os(ENV_DEVSHELL_VM_SOCKET);
    std::env::set_var(ENV_DEVSHELL_VM_SOCKET, format!("tcp:{addr}"));

    let mut beta = BetaSession::new(&test_config()).expect("new beta");
    let vfs = Vfs::new();
    beta.ensure_ready(&vfs, "/").expect("handshake ok");

    restore_var(ENV_DEVSHELL_VM_SOCKET, old_sock);
    server.join().expect("mock thread");
}

#[test]
fn ensure_ready_maps_error_frame_to_vmerror_ipc() {
    let _g = vm_env_lock();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock sidecar");
    let addr = listener.local_addr().expect("local addr");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let reader_stream = stream.try_clone().expect("clone");
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read handshake");
        writeln!(
            stream,
            "{{\"op\":\"error\",\"message\":\"handshake rejected\"}}"
        )
        .expect("write error");
        stream.flush().expect("flush");
    });

    let old_sock = std::env::var_os(ENV_DEVSHELL_VM_SOCKET);
    std::env::set_var(ENV_DEVSHELL_VM_SOCKET, format!("tcp:{addr}"));

    let mut beta = BetaSession::new(&test_config()).expect("new beta");
    let vfs = Vfs::new();
    let err = beta
        .ensure_ready(&vfs, "/")
        .expect_err("must fail on op:error");

    restore_var(ENV_DEVSHELL_VM_SOCKET, old_sock);
    server.join().expect("mock thread");

    let msg = err.to_string();
    assert!(
        msg.contains("handshake rejected"),
        "error frame message should surface in VmError::Ipc: {msg}"
    );
}

#[test]
fn ensure_ready_reports_non_json_response_prefix() {
    let _g = vm_env_lock();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock sidecar");
    let addr = listener.local_addr().expect("local addr");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let reader_stream = stream.try_clone().expect("clone");
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read handshake");
        writeln!(stream, "this is not json").expect("write non-json");
        stream.flush().expect("flush");
    });

    let old_sock = std::env::var_os(ENV_DEVSHELL_VM_SOCKET);
    std::env::set_var(ENV_DEVSHELL_VM_SOCKET, format!("tcp:{addr}"));

    let mut beta = BetaSession::new(&test_config()).expect("new beta");
    let vfs = Vfs::new();
    let err = beta
        .ensure_ready(&vfs, "/")
        .expect_err("must fail on non-json response");

    restore_var(ENV_DEVSHELL_VM_SOCKET, old_sock);
    server.join().expect("mock thread");

    let msg = err.to_string();
    assert!(
        msg.contains("not JSON") && msg.contains("first line prefix"),
        "parse failure should include prefix hint: {msg}"
    );
}
