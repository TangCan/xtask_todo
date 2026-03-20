//! Virtual filesystem for the devshell.

mod copy_to_host;
mod error;
mod node;
mod path;
mod tree;

#[cfg(test)]
mod tests;

pub use error::VfsError;
pub use node::Node;
pub use path::{normalize_path, resolve_path_with_cwd};
pub use tree::Vfs;
