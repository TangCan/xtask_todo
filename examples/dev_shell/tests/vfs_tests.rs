use dev_shell::vfs::{normalize_path, Node, Vfs};

#[test]
fn normalize_path_unix_style() {
    assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
    assert_eq!(normalize_path("foo/bar"), "foo/bar");
}

#[test]
fn normalize_path_windows_backslash() {
    assert_eq!(normalize_path("foo\\bar"), "foo/bar");
    assert_eq!(normalize_path("C:\\foo\\bar"), "/foo/bar");
}

#[test]
fn resolve_absolute_path_root() {
    let vfs = Vfs::new();
    let n = vfs.resolve_absolute("/").unwrap();
    assert!(n.is_dir());
}

#[test]
fn resolve_absolute_path_missing_returns_err() {
    let vfs = Vfs::new();
    assert!(vfs.resolve_absolute("/foo").is_err());
}

#[test]
fn mkdir_creates_path() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo/bar").unwrap();
    let n = vfs.resolve_absolute("/foo/bar").unwrap();
    assert!(n.is_dir());
}

#[test]
fn write_file_creates_file() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    vfs.write_file("/foo/f", b"hello").unwrap();
    let n = vfs.resolve_absolute("/foo/f").unwrap();
    match &n {
        Node::File { content, .. } => assert_eq!(content.as_slice(), b"hello"),
        _ => panic!(),
    }
}
