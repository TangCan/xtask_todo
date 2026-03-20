//! Read text from the **host filesystem** in a Windows-friendly way.
//!
//! Windows editors often save scripts as **UTF-8 with BOM** or **UTF-16 LE with BOM** (“Unicode” in
//! Notepad). `std::fs::read_to_string` assumes UTF-8 only, which can yield mojibake or load failures.
//! VFS export/sync for `cargo`/`rustup` already uses binary [`std::fs::read`] / [`write`]; this module
//! targets **text** loads: `-f` scripts, `source` / `.` from disk, nested `source` in `.dsh`, and
//! `.todo.json`.

use std::io;
use std::path::Path;

/// Strip a leading UTF-8 BOM (`EF BB BF`) if present.
#[must_use]
pub fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Decode bytes from a host text file: UTF-16 LE/BE (with BOM), else UTF-8 (optional BOM).
///
/// # Errors
/// Returns [`io::Error`] with kind [`io::ErrorKind::InvalidData`] if decoding fails.
pub fn decode_host_text_bytes(bytes: &[u8]) -> Result<String, io::Error> {
    use io::ErrorKind::InvalidData;

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let rest = &bytes[2..];
        if !rest.len().is_multiple_of(2) {
            return Err(io::Error::new(
                InvalidData,
                "invalid UTF-16LE: odd byte length after BOM",
            ));
        }
        let u16s: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&u16s).map_err(|e| io::Error::new(InvalidData, e));
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        let rest = &bytes[2..];
        if !rest.len().is_multiple_of(2) {
            return Err(io::Error::new(
                InvalidData,
                "invalid UTF-16BE: odd byte length after BOM",
            ));
        }
        let u16s: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&u16s).map_err(|e| io::Error::new(InvalidData, e));
    }

    let b = strip_utf8_bom(bytes);
    String::from_utf8(b.to_vec()).map_err(|e| io::Error::new(InvalidData, e))
}

/// Read a host file and decode as for [`decode_host_text_bytes`].
///
/// # Errors
/// I/O errors from [`std::fs::read`], or [`io::ErrorKind::InvalidData`] if text is not valid.
pub fn read_host_text(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    decode_host_text_bytes(&bytes)
}

/// Decode VFS file bytes as script/JSON text (UTF-8 with optional BOM, or UTF-16 with BOM).
#[must_use]
pub fn script_text_from_vfs_bytes(bytes: &[u8]) -> Option<String> {
    decode_host_text_bytes(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_plain() {
        assert_eq!(decode_host_text_bytes(b"hello").unwrap(), "hello");
    }

    #[test]
    fn utf8_bom_stripped() {
        let mut v = vec![0xEF, 0xBB, 0xBF];
        v.extend_from_slice(b"echo ok");
        assert_eq!(decode_host_text_bytes(&v).unwrap(), "echo ok");
    }

    #[test]
    fn utf16le_bom_hello() {
        // "Hi" in UTF-16 LE: H=0x48, i=0x69
        let bytes: Vec<u8> = vec![0xFF, 0xFE, 0x48, 0x00, 0x69, 0x00];
        assert_eq!(decode_host_text_bytes(&bytes).unwrap(), "Hi");
    }

    #[test]
    fn utf16be_bom_hi() {
        let bytes: Vec<u8> = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x69];
        assert_eq!(decode_host_text_bytes(&bytes).unwrap(), "Hi");
    }

    #[test]
    fn strip_utf8_bom_only() {
        assert_eq!(strip_utf8_bom(b"a"), b"a");
        assert_eq!(strip_utf8_bom(&[0xEF, 0xBB, 0xBF]), b"");
    }
}
