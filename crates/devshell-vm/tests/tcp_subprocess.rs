//! TCP end-to-end: spawn `devshell-vm --serve-tcp` as a **subprocess** (avoids in-process
//! `accept`/`connect` scheduling deadlocks). Only on **Unix** (`exec` uses `true`).

#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

struct KillOnDrop(Option<std::process::Child>);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn free_localhost_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.local_addr().expect("local_addr").port()
}

#[test]
fn devshell_vm_tcp_handshake_session_exec() {
    let bin = env!("CARGO_BIN_EXE_devshell-vm");
    let port = free_localhost_port();
    let addr_s = format!("127.0.0.1:{port}");

    // Release port; child process will bind.
    thread::sleep(Duration::from_millis(50));

    let child = KillOnDrop(Some(
        Command::new(bin)
            .args(["--serve-tcp", &addr_s])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn devshell-vm --serve-tcp"),
    ));

    thread::sleep(Duration::from_millis(150));

    let dir = std::env::temp_dir().join(format!(
        "devshell_vm_tcp_e2e_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir staging");

    let staging = dir.to_str().expect("utf8 staging");
    let session_start = serde_json::json!({
        "op": "session_start",
        "session_id": "t",
        "staging_dir": staging,
        "guest_workspace": "/workspace",
    })
    .to_string();

    let mut client = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let mut reader = BufReader::new(client.try_clone().unwrap());

    writeln!(client, r#"{{"op":"handshake","version":1}}"#).unwrap();
    client.flush().unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("handshake_ok"), "{line}");

    writeln!(client, "{session_start}").unwrap();
    client.flush().unwrap();
    line.clear();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("session_ok"), "{line}");

    writeln!(
        client,
        r#"{{"op":"exec","session_id":"t","guest_cwd":"/workspace","argv":["true"]}}"#
    )
    .unwrap();
    client.flush().unwrap();
    line.clear();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("exec_result"), "{line}");
    assert!(line.contains("\"exit_code\":0"), "{line}");

    drop(client);
    drop(child);

    let _ = std::fs::remove_dir_all(&dir);
}
