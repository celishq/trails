//! `celis grep`: every trail of every project as lines, through the system `grep`.

mod found;

use crate::command::read::view::View;
use crate::command::Command;
use crate::failure::Failure;
use found::Found;
use std::io::Write;
use std::process::Stdio;

impl Command {
    /// Gives `words` to the system `grep` with the view on its stdin, and prints what it found,
    /// grouped by trail. Writes no record.
    pub(crate) fn grep(&self, words: &[String], out: &mut dyn Write) -> Result<(), Failure> {
        let view = match self.held()? {
            true => View::of(&self.trails()?),
            false => View::default(),
        };
        let Some(found) = run(words, view.text())? else {
            return crate::command::put(out, "No line matches.");
        };
        if found.is_empty() {
            return Ok(());
        }
        match Found::of(&found, &view) {
            Some(lines) => crate::command::put(out, &lines.text(&self.project().anchor.name)),
            None => crate::command::put(out, &found),
        }
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
                .trim_end_matches('\n')
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
