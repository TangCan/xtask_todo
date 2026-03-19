//! Unit tests for VFS and path normalization.

use super::*;
use std::error::Error;

#[test]
fn vfs_error_invalid_path_display_and_source() {
    let mut vfs = Vfs::new();
    vfs.write_file("/f", b"").unwrap();
    let e = vfs.mkdir("/f/sub").unwrap_err();
    assert!(e.to_string().contains("invalid"));
    assert!(e.source().is_none());
}

#[test]
fn vfs_error_io_display_and_source() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/sub").unwrap();
    vfs.write_file("/sub/f", b"x").unwrap();
    let dir = std::env::temp_dir().join(format!("vfs_io_err_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file_as_host = dir.join("file");
    std::fs::write(&file_as_host, b"").unwrap();
    let e = vfs.copy_tree_to_host("/sub", &file_as_host).unwrap_err();
    assert!(e.to_string().contains("io:") || e.to_string().contains("io error"));
    assert!(e.source().is_some());
    let _ = std::fs::remove_file(&file_as_host);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn vfs_new_cwd_root() {
    let vfs = Vfs::new();
    assert_eq!(vfs.cwd(), "/");
}

#[test]
fn vfs_mkdir_and_list() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    vfs.mkdir("/foo/bar").unwrap();
    assert_eq!(vfs.list_dir("/").unwrap(), vec!["foo"]);
    assert_eq!(vfs.list_dir("/foo").unwrap(), vec!["bar"]);
}

#[test]
fn vfs_write_and_read_file() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/dir").unwrap();
    vfs.write_file("/dir/f", b"hello").unwrap();
    assert_eq!(vfs.read_file("/dir/f").unwrap(), b"hello");
}

#[test]
fn vfs_set_cwd() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/a").unwrap();
    vfs.set_cwd("/a").unwrap();
    assert_eq!(vfs.cwd(), "/a");
}

#[test]
fn vfs_touch() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    vfs.touch("/d/empty").unwrap();
    assert_eq!(vfs.read_file("/d/empty").unwrap(), b"");
}

#[test]
fn normalize_path_dot_dot() {
    assert_eq!(normalize_path("/a/b/.."), "/a");
    assert_eq!(normalize_path("a/../b"), "b");
}

#[test]
fn vfs_copy_tree_to_host() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/sub").unwrap();
    vfs.write_file("/sub/f.txt", b"data").unwrap();
    let dir = std::env::temp_dir().join(format!("vfs_export_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    vfs.copy_tree_to_host("/sub", &dir).unwrap();
    let sub_dir = dir.join("sub");
    let content = std::fs::read(sub_dir.join("f.txt")).unwrap();
    assert_eq!(content, b"data");
    let _ = std::fs::remove_file(sub_dir.join("f.txt"));
    let _ = std::fs::remove_dir(sub_dir);
    let _ = std::fs::remove_dir(dir);
}

#[test]
fn node_methods() {
    let dir = Node::Dir {
        name: "d".into(),
        children: vec![Node::File {
            name: "f".into(),
            content: vec![1, 2, 3],
        }],
    };
    assert_eq!(dir.name(), "d");
    assert!(dir.is_dir());
    assert!(!dir.is_file());
    let f = dir.child("f").unwrap();
    assert_eq!(f.name(), "f");
    assert!(f.is_file());
    assert!(!f.is_dir());
    assert!(dir.child("x").is_none());
    assert!(f.child("x").is_none());
}

#[test]
fn resolve_to_absolute_relative() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/a").unwrap();
    vfs.set_cwd("/a").unwrap();
    let abs = vfs.resolve_to_absolute("b");
    assert!(abs.contains('a'));
    assert!(abs.ends_with('b') || abs.contains('b'));
}

#[test]
fn resolve_to_absolute_dot_dot_from_subdir() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/aaa").unwrap();
    vfs.set_cwd("/aaa").unwrap();
    assert_eq!(vfs.cwd(), "/aaa");
    // cd .. from /aaa must resolve to /
    assert_eq!(vfs.resolve_to_absolute(".."), "/");
    vfs.set_cwd("..").unwrap();
    assert_eq!(vfs.cwd(), "/");
}

#[test]
fn normalize_path_windows_drive() {
    let out = normalize_path("C:\\foo\\bar");
    assert!(out.contains("foo"));
    assert!(out.contains("bar"));
    assert!(out.starts_with('/') || out == "foo/bar");
}

#[test]
fn mkdir_when_component_is_file_returns_err() {
    let mut vfs = Vfs::new();
    vfs.write_file("/f", b"").unwrap();
    assert!(vfs.mkdir("/f/sub").is_err());
}

#[test]
fn write_file_when_parent_is_file_returns_err() {
    let mut vfs = Vfs::new();
    vfs.write_file("/f", b"").unwrap();
    assert!(vfs.write_file("/f/child", b"x").is_err());
}

#[test]
fn write_file_when_parent_does_not_exist_returns_err() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    assert!(vfs.write_file("/d/nonexistent/sub", b"x").is_err());
}

#[test]
fn write_file_overwrite_existing() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    vfs.write_file("/d/f", b"first").unwrap();
    vfs.write_file("/d/f", b"second").unwrap();
    assert_eq!(vfs.read_file("/d/f").unwrap(), b"second");
}

#[test]
fn read_file_on_dir_errors() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    assert!(vfs.read_file("/d").is_err());
}

#[test]
fn list_dir_on_file_errors() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    vfs.write_file("/d/f", b"x").unwrap();
    assert!(vfs.list_dir("/d/f").is_err());
}

#[test]
fn set_cwd_to_file_errors() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/d").unwrap();
    vfs.write_file("/d/f", b"x").unwrap();
    assert!(vfs.set_cwd("/d/f").is_err());
}
