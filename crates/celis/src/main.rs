//! celis — the command line over the trail.

mod command;
mod help;

use std::io::Write;

fn main() {
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let before = args.iter().take_while(|a| *a != "--");
    if before.into_iter().any(|a| a == "--version") {
        let _ = writeln!(std::io::stdout(), "{}", version());
        return;
    }
    let command = command::Command {
        args,
        env: std::env::vars().collect(),
        folder: std::env::current_dir().unwrap_or_default(),
    };
    std::process::exit(command.run(&mut std::io::stdout(), &mut std::io::stderr()));
}

/// The version of this build: the release's `CELIS_VERSION` at build time, else the crate's.
const VERSION: &str = match option_env!("CELIS_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// `celis <version>`, then the first 8 characters of the commit the build came from, when it had one.
fn version() -> String {
    match option_env!("CELIS_COMMIT") {
        Some(commit) => format!("celis {VERSION} ({})", &commit[..commit.len().min(8)]),
        None => format!("celis {VERSION}"),
    }
}
