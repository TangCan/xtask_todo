// Example crate: allow some pedantic/nursery lints to avoid large refactors.
#![allow(
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

pub mod command;
pub mod completion;
pub mod parser;
pub mod repl;
pub mod serialization;
pub mod todo_io;
pub mod vfs;
