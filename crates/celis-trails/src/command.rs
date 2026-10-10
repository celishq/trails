//! One run of `celis`.

mod args;
mod decision;
mod prompt;
mod read;
mod step;
mod tree;
mod watch;

use crate::help::{self, Verb, VERBS};
use crate::{Failure, Project, Store};
use args::Args;
use celis_wire::Note;
use serde_json::json;
use std::collections::HashMap;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

/// The folder under `$HOME` that `$CELIS_HOME` defaults to.
#[cfg(target_os = "macos")]
pub const HOME: &str = "Library/Application Support/Celis";
/// The folder under `$HOME` that `$CELIS_HOME` defaults to.
#[cfg(not(target_os = "macos"))]
pub const HOME: &str = ".local/share/celis";

/// One run of `celis`: its words, the environment and the working folder.
pub struct Command {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub folder: PathBuf,
}

impl Command {
    /// Runs, writing to `out` and `err`, or prints the help page its words name. Returns the exit code.
    pub fn run(&self, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
        if let Some(page) = help::named(&self.args) {
            let _ = writeln!(out, "{page}");
            return 0;
        }
        let json = Args::asks_json(&self.args);
        let mut failure = match Args::parse(&self.args).and_then(|a| self.go(&a, out, err)) {
            Ok(()) => return 0,
            Err(failure) => failure,
        };
        if failure.code == 2 {
            let words: Vec<&str> = self
                .args
                .iter()
                .map(String::as_str)
                .take_while(|a| *a != "--")
                .filter(|a| !a.starts_with("--"))
                .collect();
            let page = match Verb::of(&words) {
                Some(verb) => format!("celis {} --help", verb.command),
                None => "celis help".to_string(),
            };
            failure = failure.see(&page);
        }
        let line = if json {
            json!({"error": failure.name, "message": failure.message}).to_string()
        } else {
            format!("celis: {}", failure.message)
        };
        let _ = writeln!(err, "{line}");
        failure.code
    }

    fn go(&self, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> Result<(), Failure> {
        let now = now();
        match args.words.first().map(String::as_str).unwrap_or("") {
            "step" => self.step(args, out, err, &self.project(), now),
            "read" => self.read(args, out, &self.project()),
            "decide" => self.decide(args, out, &self.project(), now),
            "tree" => self.tree(args, out, &self.project()),
            "watch" => self.watch(args, out, &self.project()),
            "home" => self.store_path(args, out),
            "prompt" => self.prompt(args, out, &self.project(), now),
            "end" => self.end(args, out, &self.project(), now),
            "join" => self.join(args, out, now),
            "" => Err(Failure::usage("expected: celis <command>")),
            verb => Err(Failure::unknown(verb, verbs())),
        }
    }

    /// The project of the working folder.
    fn project(&self) -> Project {
        Project::at(&self.folder)
    }

    /// Prints the folder of the store. Writes nothing.
    fn store_path(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let path = Store::root(&self.home()?).display().to_string();
        match args.json {
            true => put(out, &json!({ "path": path }).to_string()),
            false => put(out, &path),
        }
    }

    /// Whether `tree` and `watch` print styled: on a terminal, or with `CLICOLOR_FORCE` set and not `0`;
    /// never with `NO_COLOR` set.
    fn styled(&self) -> bool {
        let set = |name: &str| self.env.get(name).is_some_and(|v| !v.is_empty());
        if set("NO_COLOR") {
            return false;
        }
        let forced =
            set("CLICOLOR_FORCE") && self.env.get("CLICOLOR_FORCE").is_some_and(|v| v != "0");
        forced || std::io::stdout().is_terminal()
    }

    /// The width of the terminal stdout is, else `COLUMNS`; none when neither is known.
    fn columns(&self) -> Option<usize> {
        if std::io::stdout().is_terminal() {
            let mut size = libc::winsize {
                ws_row: 0,
                ws_col: 0,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            let asked = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) };
            if asked == 0 && size.ws_col > 0 {
                return Some(usize::from(size.ws_col));
            }
        }
        self.env.get("COLUMNS").and_then(|c| c.parse().ok())
    }

    /// The agent session this run belongs to, from `$CLAUDE_CODE_SESSION_ID`.
    fn session(&self) -> Option<String> {
        self.env
            .get("CLAUDE_CODE_SESSION_ID")
            .filter(|s| !s.is_empty())
            .cloned()
    }

    /// Whether the store exists.
    fn held(&self) -> Result<bool, Failure> {
        Ok(Store::exists(&self.home()?))
    }

    /// `$CELIS_HOME`, else its default under `$HOME`.
    pub(crate) fn home(&self) -> Result<PathBuf, Failure> {
        let home = self
            .env
            .get("CELIS_HOME")
            .map(PathBuf::from)
            .or_else(|| self.env.get("HOME").map(|h| Path::new(h).join(HOME)));
        home.ok_or_else(|| Failure::from(anyhow::anyhow!("neither CELIS_HOME nor HOME is set")))
    }

    fn store(&self) -> Result<Store, Failure> {
        Store::open(&self.home()?)
    }

    fn id(&self, out: &mut dyn Write, args: &Args, note: &Note) -> Result<(), Failure> {
        match args.json {
            true => put(out, &json!({"id": note.id}).to_string()),
            false => put(out, &note.id),
        }
    }
}

fn put(out: &mut dyn Write, line: &str) -> Result<(), Failure> {
    writeln!(out, "{line}")
        .map_err(anyhow::Error::from)
        .map_err(Failure::from)
}

/// The first word of each command.
fn verbs() -> impl Iterator<Item = &'static str> {
    VERBS.iter().filter_map(|v| v.command.split(' ').next())
}

/// The seconds since the epoch now.
pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
