//! One run of `celis`.

mod args;
mod read;
mod record;
mod setup;

use crate::failure::Failure;
use crate::help::{self, VERBS};
pub(crate) use args::Args;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Link, Project, Store, Trails};
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
        let colour = std::io::stdout().is_terminal() && self.colour();
        if let Some(page) = help::named(&self.args) {
            let _ = writeln!(out, "{}", help::text(&page, colour));
            return 0;
        }
        let json = Args::asks_json(&self.args);
        let args = Args::parse(&self.args);
        let words = args.as_ref().map_or(vec![], |a| a.words.clone());
        let mut failure = match args.and_then(|a| self.go(&a, out, err)) {
            Ok(()) => return 0,
            Err(failure) => failure,
        };
        if failure.code == 2 && !failure.message.is_empty() {
            let stop = if failure.message.ends_with('?') {
                ""
            } else {
                "."
            };
            let page = help::see(&self.args);
            failure.message = format!("{}{stop} See `{page}`.", failure.message);
        }
        self.log(&words, &failure);
        let line = if json {
            json!({"error": failure.name, "message": failure.message}).to_string()
        } else {
            format!("celis: {}", failure.message)
        };
        if !failure.message.is_empty() {
            let _ = writeln!(err, "{line}");
        }
        failure.code
    }

    fn go(&self, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> Result<(), Failure> {
        match args.words.first().map(String::as_str).unwrap_or("") {
            "step" => self.step(args, out, err, &self.project_of(args)?),
            "pattern" | "decide" => self.pattern(args, out, err, &self.project_of(args)?),
            "tree" => self.tree(args, out),
            "patterns" => self.patterns(args, out),
            "grep" => self.grep(&self.args[1..], out),
            "search" => self.search(args, out, err),
            "stats" => self.stats(args, out),
            "issue" => self.issue(args, out),
            "link" => self.link(args, out, &self.project()),
            "hook" => self.hook(args, out),
            "auth" => self.auth(args, out, err),
            "setup" => self.setup(args, out, err),
            "update" => self.update(args),
            "" => Err(Failure::usage("expected: celis <command>")),
            verb => Err(help::unknown(verb, verbs())),
        }
    }

    /// Adds the line of the failed run of `words` to `$CELIS_HOME/log`; a write that fails is let go.
    fn log(&self, words: &[String], failure: &Failure) {
        let Ok(home) = self.home() else {
            return;
        };
        let at = now();
        let line = format!(
            "{} {} {} {} · {}\n",
            crate::text::clock::date(at),
            crate::text::clock::clock(at, at),
            failure.code,
            ["celis".to_string()]
                .iter()
                .chain(words)
                .cloned()
                .collect::<Vec<_>>()
                .join(" "),
            failure.message
        );
        let file = std::fs::create_dir_all(&home).and_then(|()| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(home.join("log"))
        });
        let _ = file.and_then(|mut f| f.write_all(line.as_bytes()));
    }

    /// The project of the working folder.
    fn project(&self) -> Project {
        Project::at(&self.folder)
    }

    /// The project `--project` names, a project of the store with the name of its newest record, else the project of the working folder; any other id is a usage error.
    fn project_of(&self, args: &Args) -> Result<Project, Failure> {
        let project = self.project();
        let Some(id) = &args.project else {
            return Ok(project);
        };
        let anchor = self.trails()?.anchor(id).ok_or_else(|| {
            Failure::usage(format!("--project takes a project of the store, not {id}"))
        })?;
        Ok(project.with(anchor))
    }

    /// Whether a watch prints in colour: unless `NO_COLOR` is set.
    pub(crate) fn colour(&self) -> bool {
        self.env.get("NO_COLOR").is_none_or(|v| v.is_empty())
    }

    /// The width of the terminal stdout is, else `COLUMNS`; none when neither is known.
    pub(crate) fn columns(&self) -> Option<usize> {
        match size().map(|s| s.ws_col).filter(|&c| c > 0) {
            Some(columns) => Some(usize::from(columns)),
            None => self.env.get("COLUMNS").and_then(|c| c.parse().ok()),
        }
    }

    /// The agent session this run belongs to, from the first set variable of `$CLAUDE_CODE_SESSION_ID`,
    /// `$CODEX_THREAD_ID` and `$CELIS_SESSION_ID`.
    fn session(&self) -> Option<String> {
        [
            "CLAUDE_CODE_SESSION_ID",
            "CODEX_THREAD_ID",
            "CELIS_SESSION_ID",
        ]
        .iter()
        .filter_map(|name| self.env.get(*name))
        .find(|s| !s.is_empty())
        .cloned()
    }

    /// Whether the store exists.
    pub(crate) fn held(&self) -> Result<bool, Failure> {
        Ok(Store::exists(&self.home()?))
    }

    /// The file whose presence says the opening scene was shown: `$CELIS_OPENING`, else `opening` in
    /// `home`; none without a home.
    pub(crate) fn opening(&self) -> Option<PathBuf> {
        let named = self.env.get("CELIS_OPENING").map(PathBuf::from);
        named.or_else(|| self.home().ok().map(|home| home.join("opening")))
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

    /// The trails of the store, with the working folder's linked folders that are on, opened now; a check
    /// runs for `CELIS_CHECK_SECONDS`, 600 when not set.
    pub(crate) fn trails(&self) -> Result<Trails, Failure> {
        let limit = self
            .env
            .get("CELIS_CHECK_SECONDS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);
        let home = self.home()?;
        let links = Link::all(&home, &self.folder);
        let store = links.into_iter().fold(Store::open(&home)?, Store::with);
        Ok(Trails::open(store, now(), limit))
    }

    /// Prints the id of `note`: its first 8 characters on a terminal, else whole; `{"id"}` with
    /// `--json`.
    pub(crate) fn id(&self, out: &mut dyn Write, args: &Args, note: &Note) -> Result<(), Failure> {
        let line = match std::io::stdout().is_terminal() {
            true => short(&note.id),
            false => note.id.clone(),
        };
        show(out, args.json, "id", json!(note.id), &line)
    }
}

fn put(out: &mut dyn Write, line: &str) -> Result<(), Failure> {
    writeln!(out, "{line}")
        .map_err(anyhow::Error::from)
        .map_err(Failure::from)
}

/// Prints `{key: value}` as one JSON line with `json`, else `line`.
fn show(
    out: &mut dyn Write,
    json: bool,
    key: &str,
    value: serde_json::Value,
    line: &str,
) -> Result<(), Failure> {
    match json {
        true => put(out, &json!({ key: value }).to_string()),
        false => put(out, line),
    }
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

/// The size of the terminal stdout is; none off a terminal.
pub(crate) fn size() -> Option<libc::winsize> {
    if !std::io::stdout().is_terminal() {
        return None;
    }
    let mut size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let asked = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) };
    (asked == 0).then_some(size)
}
