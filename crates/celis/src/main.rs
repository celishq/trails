//! celis — the command line over the trail.

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let before = args.iter().take_while(|a| *a != "--");
    if before.into_iter().any(|a| a == "--version") {
        let _ = writeln!(std::io::stdout(), "{}", version());
        return;
    }
    let command = celis_trails::Command {
        args,
        env: std::env::vars().collect(),
        folder: std::env::current_dir().unwrap_or_default(),
    };
    std::process::exit(command.run(&mut std::io::stdout(), &mut std::io::stderr()));
}

/// `celis <version>`, then the first 8 characters of the commit the build came from, when it had one.
fn version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    match option_env!("CELIS_COMMIT") {
        Some(commit) => format!("celis {version} ({})", &commit[..commit.len().min(8)]),
        None => format!("celis {version}"),
    }
}
