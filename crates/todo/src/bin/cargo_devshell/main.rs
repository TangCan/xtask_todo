//! Binary for `cargo devshell`: same package as the lib, one Cargo.toml publishes both.
#![allow(
    dead_code,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::too_many_lines,
    clippy::result_unit_err,
    clippy::cast_possible_truncation,
    clippy::branches_sharing_code,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_identity
)]

mod command;
mod completion;
mod parser;
mod repl;
mod serialization;
mod todo_io;
mod vfs;

use std::cell::RefCell;
use std::io::{self, BufReader, IsTerminal, Write};
use std::rc::Rc;

use crate::vfs::Vfs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.as_slice() {
        [] | [_] => std::path::Path::new(".dev_shell.bin"),
        [_, path] => std::path::Path::new(path),
        _ => {
            let _ = writeln!(io::stderr(), "usage: dev_shell [path]");
            std::process::exit(1);
        }
    };

    let vfs = match serialization::load_from_file(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if args.len() > 1 {
                    let _ = writeln!(io::stderr(), "File not found, starting with empty VFS");
                }
            } else {
                let _ = writeln!(io::stderr(), "Failed to load {}: {}", path.display(), e);
            }
            Vfs::new()
        }
    };
    let vfs = Rc::new(RefCell::new(vfs));

    let is_tty = io::stdin().is_terminal();
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let _ = repl::run(&vfs, is_tty, path, &mut stdin, &mut stdout, &mut stderr);
}
