//! `celis grep`: every trail of every project as lines, through the system `grep`.

mod cut;
mod kind;
mod row;
mod view;

use super::Command;
use celis_trails::Failure;
use cut::Cut;
use std::io::Write;
use std::process::Stdio;
use view::View;

impl Command {
    /// Gives `words` to the system `grep` with the view on its stdin, and prints the cut of what it
    /// prints. Writes no record.
    pub(super) fn grep(&self, words: &[String], out: &mut dyn Write) -> Result<(), Failure> {
        let view = match self.held()? {
            true => View::of(&self.trails()?),
            false => View::default(),
        };
        let Some(found) = run(words, view.text())? else {
            return super::put(out, "No line matches.");
        };
        if found.is_empty() {
            return Ok(());
        }
        let text = Cut::of(&found, &view).text(&view, &self.project().name, &found);
        super::put(out, &text)
    }

    /// Prints the `grep` searches that replace `celis past`, for a plugin from before. Writes no record.
    pub(super) fn past(&self, out: &mut dyn Write) -> Result<(), Failure> {
        let name = &self.project().name;
        let lines = [
            "celis past is gone; search the trails with celis grep:".to_string(),
            "  celis grep \" live {me} \"".to_string(),
            format!("  celis grep \"\\[{name}\\] rule [0-9a-f]* live {{project}} \""),
            "  celis grep -i -E \"<a word of the ask>|<another>\"".to_string(),
        ];
        super::put(out, &lines.join("\n"))
    }
}

/// What `grep` prints for `words` with `view` on its stdin; none when it exits 1, as it does when no line
/// matches. A `grep` that fails exits 2, with its own message.
fn run(words: &[String], view: String) -> Result<Option<String>, Failure> {
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
    let output = feed(words, view);
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    let output = output.map_err(anyhow::Error::from)?;
    match output.status.code() {
        Some(0) => Ok(Some(
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string(),
        )),
        Some(1) => Ok(None),
        _ => Err(Failure::quiet(2)),
    }
}

/// The output of `grep` run with `words` and `view` on its stdin, which it may stop reading.
fn feed(words: &[String], view: String) -> std::io::Result<std::process::Output> {
    let mut grep = std::process::Command::new("grep")
        .args(words)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = grep.stdin.take().expect("a piped stdin");
    let feed = std::thread::spawn(move || {
        let _ = stdin.write_all(view.as_bytes());
        let _ = stdin.write_all(b"\n");
    });
    let output = grep.wait_with_output();
    let _ = feed.join();
    output
}
