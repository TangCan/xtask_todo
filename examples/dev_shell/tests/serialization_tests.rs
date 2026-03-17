use xtask_todo_devshell::serialization;
use xtask_todo_devshell::vfs::Vfs;

#[test]
fn roundtrip_empty_vfs() {
    let vfs = Vfs::new();
    let bytes = serialization::serialize(&vfs).unwrap();
    assert!(bytes.starts_with(b"DEVS"));
    let loaded = serialization::deserialize(&bytes).unwrap();
    assert!(loaded.resolve_absolute("/").is_ok());
}

#[test]
fn roundtrip_with_tree() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo/bar").unwrap();
    vfs.write_file("/foo/f", b"hi").unwrap();
    let bytes = serialization::serialize(&vfs).unwrap();
    assert!(bytes.starts_with(b"DEVS"));
    let loaded = serialization::deserialize(&bytes).unwrap();
    // Tree matches: /foo/bar is a dir, /foo/f has content "hi"
    let root = loaded.resolve_absolute("/").unwrap();
    assert!(root.is_dir());
    let foo = root.child("foo").expect("foo exists");
    assert!(foo.is_dir());
    assert!(foo.child("bar").is_some());
    assert!(foo.child("bar").unwrap().is_dir());
    assert!(foo.child("f").is_some());
    assert!(foo.child("f").unwrap().is_file());
    let content = loaded.read_file("/foo/f").unwrap();
    assert_eq!(content, b"hi");
}
