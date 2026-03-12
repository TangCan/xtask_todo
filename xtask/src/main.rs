//! xtask - custom cargo tasks
//!
//! Run with: `cargo xtask <command>`

fn main() {
    if let Err(e) = xtask::run() {
        eprintln!("error: {}", e.message);
        std::process::exit(e.code);
    }
}
