//! xtask - custom cargo tasks
//!
//! Run with: cargo xtask <command>

use argh::FromArgs;

fn main() {
    let cmd: XtaskCmd = argh::from_env();
    if let Err(e) = run(cmd) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run(cmd: XtaskCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.sub {
        XtaskSub::Run(args) => cmd_run(args),
        XtaskSub::Todo(args) => cmd_todo(args),
    }
}

#[derive(FromArgs)]
/// Cargo xtask - custom build/tooling tasks
struct XtaskCmd {
    #[argh(subcommand)]
    sub: XtaskSub,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum XtaskSub {
    Run(RunArgs),
    Todo(TodoArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run the main project (example task)
struct RunArgs {}

fn cmd_run(_args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("xtask run: placeholder - add your task logic here");
    Ok(())
}

#[derive(FromArgs)]
#[argh(subcommand, name = "todo")]
/// Demo todo list: create a few items, list, complete one, delete one
struct TodoArgs {}

fn cmd_todo(_args: TodoArgs) -> Result<(), Box<dyn std::error::Error>> {
    use todo::TodoList;

    let mut list = TodoList::new();
    let id1 = list.create("First task")?;
    let _id2 = list.create("Second task")?;
    let id3 = list.create("Third task")?;

    println!("Created 3 todos:");
    for t in list.list() {
        println!("  [{}] {} {}", t.id, if t.completed { "x" } else { " " }, t.title);
    }

    list.complete(id1)?;
    list.delete(id3)?;

    println!("\nAfter complete(id1) and delete(id3):");
    for t in list.list() {
        println!("  [{}] {} {}", t.id, if t.completed { "x" } else { " " }, t.title);
    }
    Ok(())
}
