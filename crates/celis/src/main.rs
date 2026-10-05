//! celis — the command line over an account and cairns, and over a vault in the `vault` build.

mod command;
mod failure;
mod help;
mod home;

use command::Command;
use std::io::Write;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("init") {
        args.insert(0, "cairn".into());
    }
    if let Some(page) = help::page(&args) {
        let _ = writeln!(std::io::stdout(), "{page}");
        return;
    }
    if args.first().map(String::as_str) == Some("cairn") {
        let cairn = celis_cairns::Command {
            args: args[1..].to_vec(),
            env: std::env::vars().collect(),
            folder: std::env::current_dir().unwrap_or_default(),
        };
        std::process::exit(cairn.run(&mut std::io::stdout(), &mut std::io::stderr()));
    }
    let json = args.iter().any(|a| a == "--json");
    match Command::parse(&args).and_then(|command| command.run()) {
        Ok(bytes) => {
            let _ = std::io::stdout().write_all(&bytes);
        }
        Err(failure) => {
            eprintln!("{}", failure.render(json));
            std::process::exit(failure.code);
        }
    }
}
