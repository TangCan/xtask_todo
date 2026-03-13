//! .bin format: magic "DEVS" + version + cwd + root node tree.

use std::io::{Cursor, Read, Write};

use crate::vfs::{Node, Vfs};

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
            Error::InvalidMagic => write!(f, "invalid magic"),
            Error::InvalidVersion => write!(f, "invalid version"),
            Error::Truncated => write!(f, "truncated data"),
            Error::InvalidUtf8(e) => write!(f, "invalid utf-8: {}", e),
            Error::Io(e) => write!(f, "io error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidUtf8(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
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
            write_u16_le(w, name_bytes.len() as u16)?;
            w.write_all(name_bytes)?;
            write_u32_le(w, children.len() as u32)?;
            for child in children {
                serialize_node(w, child)?;
            }
        }
        Node::File { name, content } => {
            w.write_all(&[NODE_FILE])?;
            let name_bytes = name.as_bytes();
            write_u16_le(w, name_bytes.len() as u16)?;
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
            let mut children = Vec::with_capacity(child_count as usize);
            for _ in 0..child_count {
                children.push(deserialize_node(r)?);
            }
            Ok(Node::Dir { name, children })
        }
        NODE_FILE => {
            let content_len = read_u64_le(r)?;
            let mut content = vec![0u8; content_len as usize];
            r.read_exact(&mut content)?;
            Ok(Node::File { name, content })
        }
        _ => Err(Error::Truncated),
    }
}

/// Serialize VFS to .bin format: DEVS magic + version 1 + cwd + root node tree.
pub fn serialize(vfs: &Vfs) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    out.write_all(MAGIC)?;
    out.write_all(&[VERSION])?;
    let cwd = vfs.cwd().as_bytes();
    write_u32_le(&mut out, cwd.len() as u32)?;
    out.write_all(cwd)?;
    serialize_node(&mut out, vfs.root())?;
    Ok(out)
}

/// Deserialize VFS from .bin format.
pub fn deserialize(bytes: &[u8]) -> Result<Vfs, Error> {
    let mut r = Cursor::new(bytes);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)
        .map_err(|_| Error::Truncated)?;
    if &magic != MAGIC {
        return Err(Error::InvalidMagic);
    }
    let mut ver = [0u8; 1];
    r.read_exact(&mut ver).map_err(|_| Error::Truncated)?;
    if ver[0] != VERSION {
        return Err(Error::InvalidVersion);
    }
    let cwd_len = read_u32_le(&mut r)?;
    let mut cwd_buf = vec![0u8; cwd_len as usize];
    r.read_exact(&mut cwd_buf)?;
    let cwd = String::from_utf8(cwd_buf).map_err(Error::InvalidUtf8)?;
    let root = deserialize_node(&mut r)?;
    Ok(Vfs::from_parts(root, cwd))
}

/// Save VFS to a .bin file.
pub fn save_to_file(vfs: &Vfs, path: &std::path::Path) -> std::io::Result<()> {
    let bytes = serialize(vfs).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, bytes)
}

/// Load VFS from a .bin file.
pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Vfs> {
    let bytes = std::fs::read(path)?;
    deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
