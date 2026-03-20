//! .bin format: magic "DEVS" + version + cwd + root node tree.

use std::io::{Cursor, Read, Write};

use super::vfs::{Node, Vfs};

const MAGIC: &[u8; 4] = b"DEVS";
const VERSION: u8 = 1;

const NODE_DIR: u8 = 0;
const NODE_FILE: u8 = 1;

#[derive(Debug)]
pub enum Error {
    InvalidMagic,
    InvalidVersion,
    Truncated,
    InvalidUtf8(std::string::FromUtf8Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "invalid magic"),
            Self::InvalidVersion => write!(f, "invalid version"),
            Self::Truncated => write!(f, "truncated data"),
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8: {e}"),
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

fn write_u32_le(w: &mut impl Write, n: u32) -> std::io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn write_u16_le(w: &mut impl Write, n: u16) -> std::io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn write_u64_le(w: &mut impl Write, n: u64) -> std::io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn read_u32_le(r: &mut impl Read) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u16_le(r: &mut impl Read) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u64_le(r: &mut impl Read) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn serialize_node(w: &mut impl Write, node: &Node) -> std::io::Result<()> {
    match node {
        Node::Dir { name, children } => {
            w.write_all(&[NODE_DIR])?;
            let name_bytes = name.as_bytes();
            let len_u16 = u16::try_from(name_bytes.len())
                .map_err(|_| std::io::Error::other("name len overflow"))?;
            write_u16_le(w, len_u16)?;
            w.write_all(name_bytes)?;
            let len_u32 = u32::try_from(children.len())
                .map_err(|_| std::io::Error::other("children len overflow"))?;
            write_u32_le(w, len_u32)?;
            for child in children {
                serialize_node(w, child)?;
            }
        }
        Node::File { name, content } => {
            w.write_all(&[NODE_FILE])?;
            let name_bytes = name.as_bytes();
            let len_u16 = u16::try_from(name_bytes.len())
                .map_err(|_| std::io::Error::other("name len overflow"))?;
            write_u16_le(w, len_u16)?;
            w.write_all(name_bytes)?;
            write_u64_le(w, content.len() as u64)?;
            w.write_all(content)?;
        }
    }
    Ok(())
}

fn deserialize_node(r: &mut impl Read) -> Result<Node, Error> {
    let mut tag = [0u8; 1];
    r.read_exact(&mut tag).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Error::Truncated
        } else {
            Error::Io(e)
        }
    })?;
    let name_len = read_u16_le(r)?;
    let mut name_buf = vec![0u8; name_len as usize];
    r.read_exact(&mut name_buf)?;
    let name = String::from_utf8(name_buf).map_err(Error::InvalidUtf8)?;

    match tag[0] {
        NODE_DIR => {
            let child_count = read_u32_le(r)?;
            let n = usize::try_from(child_count).map_err(|_| Error::Truncated)?;
            let mut children = Vec::with_capacity(n);
            for _ in 0..child_count {
                children.push(deserialize_node(r)?);
            }
            Ok(Node::Dir { name, children })
        }
        NODE_FILE => {
            let content_len = read_u64_le(r)?;
            let n = usize::try_from(content_len).map_err(|_| Error::Truncated)?;
            let mut content = vec![0u8; n];
            r.read_exact(&mut content)?;
            Ok(Node::File { name, content })
        }
        _ => Err(Error::Truncated),
    }
}

/// Serialize VFS to .bin format: DEVS magic + version 1 + cwd + root node tree.
///
/// # Errors
/// Returns `Error::Io` on write failure or if cwd/name/children length overflows the wire format.
pub fn serialize(vfs: &Vfs) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    out.write_all(MAGIC)?;
    out.write_all(&[VERSION])?;
    let cwd = vfs.cwd().as_bytes();
    let cwd_len = u32::try_from(cwd.len())
        .map_err(|_| Error::Io(std::io::Error::other("cwd len overflow")))?;
    write_u32_le(&mut out, cwd_len)?;
    out.write_all(cwd)?;
    serialize_node(&mut out, vfs.root())?;
    Ok(out)
}

/// Deserialize VFS from .bin format.
///
/// # Errors
/// Returns `Error` on invalid magic/version, truncated data, or invalid UTF-8.
pub fn deserialize(bytes: &[u8]) -> Result<Vfs, Error> {
    let mut r = Cursor::new(bytes);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic).map_err(|_| Error::Truncated)?;
    if &magic != MAGIC {
        return Err(Error::InvalidMagic);
    }
    let mut ver = [0u8; 1];
    r.read_exact(&mut ver).map_err(|_| Error::Truncated)?;
    if ver[0] != VERSION {
        return Err(Error::InvalidVersion);
    }
    let cwd_len = read_u32_le(&mut r)?;
    let cwd_len_usize = usize::try_from(cwd_len).map_err(|_| Error::Truncated)?;
    let mut cwd_buf = vec![0u8; cwd_len_usize];
    r.read_exact(&mut cwd_buf)?;
    let cwd = String::from_utf8(cwd_buf).map_err(Error::InvalidUtf8)?;
    let root = deserialize_node(&mut r)?;
    Ok(Vfs::from_parts(root, cwd))
}

/// Save VFS to a .bin file.
///
/// # Errors
/// Returns I/O error on serialize or write failure.
pub fn save_to_file(vfs: &Vfs, path: &std::path::Path) -> std::io::Result<()> {
    let bytes = serialize(vfs).map_err(std::io::Error::other)?;
    std::fs::write(path, bytes)
}

/// Load VFS from a .bin file.
///
/// # Errors
/// Returns I/O error on read failure or invalid/corrupt .bin data.
pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Vfs> {
    let bytes = std::fs::read(path)?;
    deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_vfs() {
        let vfs = Vfs::new();
        let bytes = serialize(&vfs).unwrap();
        let vfs2 = deserialize(&bytes).unwrap();
        assert_eq!(vfs.cwd(), vfs2.cwd());
    }

    #[test]
    fn roundtrip_with_dir_and_file() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/foo").unwrap();
        vfs.write_file("/foo/bar", b"content").unwrap();
        let bytes = serialize(&vfs).unwrap();
        let vfs2 = deserialize(&bytes).unwrap();
        assert_eq!(vfs2.read_file("/foo/bar").unwrap(), b"content");
    }

    #[test]
    fn invalid_magic() {
        let bytes = b"XXXX\x01\x00\x00\x00\x01/\x00\x00\x00\x00";
        assert!(matches!(deserialize(bytes), Err(Error::InvalidMagic)));
    }

    #[test]
    fn error_display() {
        assert_eq!(Error::InvalidMagic.to_string(), "invalid magic");
        assert_eq!(Error::InvalidVersion.to_string(), "invalid version");
        assert_eq!(Error::Truncated.to_string(), "truncated data");
        assert!(Error::Io(std::io::Error::other("e"))
            .to_string()
            .contains("io error"));
        let utf8_err = Vec::<u8>::from([0xff, 0xfe]);
        let e = String::from_utf8(utf8_err).unwrap_err();
        assert!(Error::InvalidUtf8(e).to_string().contains("utf-8"));
    }

    #[test]
    fn error_source() {
        use std::error::Error as _;
        let utf8_err = Vec::<u8>::from([0xff, 0xfe]);
        let e = Error::InvalidUtf8(String::from_utf8(utf8_err).unwrap_err());
        assert!(e.source().is_some());
        let e = Error::Io(std::io::Error::other("test"));
        assert!(e.source().is_some());
        assert!(Error::InvalidMagic.source().is_none());
    }

    #[test]
    fn deserialize_invalid_tag() {
        let mut bytes = serialize(&Vfs::new()).unwrap();
        let root_tag_offset = 4 + 1 + 4 + 1; // MAGIC + VERSION + cwd_len + cwd
        bytes[root_tag_offset] = 0xff; // invalid node tag
        assert!(matches!(deserialize(&bytes), Err(Error::Truncated)));
    }

    #[test]
    fn invalid_version() {
        let mut bytes = vec![b'D', b'E', b'V', b'S', 99, 0, 0, 0, 1, b'/'];
        bytes.extend_from_slice(&0u32.to_le_bytes());
        assert!(matches!(deserialize(&bytes), Err(Error::InvalidVersion)));
    }

    #[test]
    fn load_from_file_nonexistent() {
        let path = std::path::Path::new("/nonexistent_devshell_path_12345");
        let r = load_from_file(path);
        assert!(r.is_err());
    }

    #[test]
    fn error_from_io() {
        let e: Error = std::io::Error::other("test").into();
        assert!(matches!(e, Error::Io(_)));
    }

    /// Truncated data after header: no byte for root node tag; `deserialize_node` `read_exact` fails (Truncated or Io).
    #[test]
    fn deserialize_truncated_after_cwd() {
        let bytes = vec![b'D', b'E', b'V', b'S', 1, 0, 0, 0, 1, b'/']; // MAGIC + VERSION + cwd_len=1 + cwd="/"
        let r = deserialize(&bytes);
        assert!(r.is_err(), "expected Err (Truncated or Io)");
    }

    /// Truncated inside `deserialize_node`: root node tag and `name_len` read, then `child_count` read fails (Truncated or Io).
    #[test]
    fn deserialize_truncated_inside_node() {
        // MAGIC + VERSION + cwd_len=0 + NODE_DIR=0 + name_len=0 (2 bytes) -> then read_u32_le needs 4 bytes, we have none
        let bytes: Vec<u8> = [b'D', b'E', b'V', b'S', 1, 0, 0, 0, 0, 0, 0, 0]
            .into_iter()
            .collect();
        let r = deserialize(&bytes);
        assert!(r.is_err(), "expected Err (Truncated or Io)");
    }

    /// First byte of a node: EOF maps to `Error::Truncated` (covers `UnexpectedEof` branch in `deserialize_node`).
    #[test]
    fn deserialize_node_tag_unexpected_eof_is_truncated() {
        let mut r = Cursor::new(&[]);
        let e = deserialize_node(&mut r).unwrap_err();
        assert!(matches!(e, Error::Truncated));
    }

    /// Non-EOF I/O errors from reading the node tag map to `Error::Io`.
    #[test]
    fn deserialize_node_tag_other_io_error_is_io() {
        struct FailRead;
        impl std::io::Read for FailRead {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected read failure"))
            }
        }
        let e = deserialize_node(&mut FailRead).unwrap_err();
        assert!(matches!(e, Error::Io(_)));
    }
}
