//! What a watch draws on the terminal, and what it reads from it.

use super::tree::cut;
use super::wake::{keys, signals, Key, Wake};
use super::Command;
use celis_trails::{Failure, Link};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

impl Command {
    /// Runs `body` on the alternate screen, with the keys and signals it reads sent to the channel it
    /// gets; then sets the terminal back and leaves the alternate screen with nothing printed.
    pub(super) fn screen(
        &self,
        out: &mut dyn Write,
        body: impl FnOnce(&mut dyn Write, (Sender<Wake>, Receiver<Wake>)) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let (send, events) = channel();
        signals(send.clone());
        let typed = keys(send.clone());
        write!(out, "\x1b[?1049h\x1b[?25l").map_err(anyhow::Error::from)?;
        let done = body(out, (send, events));
        if let Some(terminal) = typed {
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &terminal) };
        }
        write!(out, "\x1b[?25h\x1b[?1049l")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        done
    }

    /// Sends a `Record` to `send` each time a record lands in the store or a linked folder, saying
    /// whether in a linked folder; the watcher that does it, kept while it should.
    pub(super) fn listen(&self, send: Sender<Wake>) -> Result<RecommendedWatcher, Failure> {
        let home = self.home()?;
        let shared = home.join("trails/shared");
        let links = Link::all(&home, &self.folder).into_iter().map(|l| l.folder);
        let ours = [
            shared.clone(),
            std::fs::canonicalize(&shared).unwrap_or_default(),
        ];
        let landed = move |event: notify::Result<notify::Event>| {
            let Ok(event) = event else { return };
            if !event.kind.is_access() {
                let linked = event
                    .paths
                    .iter()
                    .any(|p| !ours.iter().any(|o| p.starts_with(o)));
                let _ = send.send(Wake::Record(linked));
            }
        };
        let mut watcher = notify::recommended_watcher(landed).map_err(anyhow::Error::from)?;
        for folder in std::iter::once(shared).chain(links) {
            watcher
                .watch(&folder, RecursiveMode::NonRecursive)
                .map_err(anyhow::Error::from)?;
        }
        Ok(watcher)
    }
}

/// What a watch shows of its tree: the first line in view, whether the view moves to the selected
/// line on the next draw, the lines a page holds, the text on the screen, and what the last key did
/// when it has something to say.
#[derive(Default)]
pub(super) struct Screen {
    pub top: usize,
    pub follow: bool,
    pub page: usize,
    pub shown: String,
    pub said: Option<String>,
}

impl Screen {
    /// Draws `tree` over the screen from the top left, when it changed: each line cut short of the last
    /// column and cleared to its end, the rows above the last scrolled to show its `picked` line when the
    /// selection moved, `keys` or what the last key did on the last row, dim, and the rest cleared.
    pub fn draw(
        &mut self,
        out: &mut dyn Write,
        tree: &str,
        picked: Option<usize>,
        keys: &str,
    ) -> Result<(), Failure> {
        let size = super::size();
        let width = size.map(|s| usize::from(s.ws_col)).filter(|&w| w > 1);
        let width = width.map(|w| w - 1);
        let rows = size.map(|s| usize::from(s.ws_row)).filter(|&r| r > 0);
        let lines: Vec<String> = tree
            .lines()
            .map(|l| width.map_or(l.to_string(), |w| cut(l, w)))
            .collect();
        let lines = match rows
            .map(|r| r - 1)
            .filter(|&body| body >= 3 && lines.len() > body)
        {
            Some(body) => self.window(lines, body, picked),
            None => {
                self.top = 0;
                lines
            }
        };
        let said = self.said.as_deref().unwrap_or(keys);
        let said = width.map_or(said.to_string(), |w| cut(said, w));
        let said = format!("\x1b[2m{said}\x1b[0m");
        let keys = match rows {
            Some(rows) => format!("\x1b[J\x1b[{rows};1H{said}"),
            None => format!("\n{said}"),
        };
        let text = format!(
            "\x1b[?2026h\x1b[H{}\x1b[K{keys}\x1b[K\x1b[J\x1b[?2026l",
            lines.join("\x1b[K\n")
        );
        if text == self.shown {
            return Ok(());
        }
        write!(out, "{text}")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        self.shown = text;
        Ok(())
    }

    /// The `body` rows of `lines` in view from `top`: a first row for the lines above it and a last
    /// row for the lines below it, when there are any; the `picked` line in view when the selection
    /// moved.
    fn window(&mut self, lines: Vec<String>, body: usize, picked: Option<usize>) -> Vec<String> {
        let len = lines.len();
        let fit = |top: usize| {
            let up = usize::from(top > 0);
            let shown = body - up;
            shown - usize::from(top + shown < len)
        };
        self.top = self.top.min(len - (body - 1));
        if let (true, Some(picked)) = (std::mem::take(&mut self.follow), picked) {
            if picked < self.top {
                self.top = picked;
            }
            while picked >= self.top + fit(self.top) {
                self.top += 1;
            }
        }
        let shown = fit(self.top);
        self.page = shown;
        let mut view = Vec::new();
        if self.top > 0 {
            view.push(format!("  \x1b[2m↑ {} more\x1b[0m", self.top));
        }
        view.extend(lines[self.top..self.top + shown].iter().cloned());
        if self.top + shown < len {
            view.push(format!("  \x1b[2m↓ {} more\x1b[0m", len - self.top - shown));
        }
        view
    }

    /// Scrolls as `key` asks: a page up or down, or to the top or the end.
    pub fn press(&mut self, key: &Key) {
        let page = self.page.max(1);
        match key {
            Key::PageUp => self.top = self.top.saturating_sub(page),
            Key::PageDown => self.top += page,
            Key::Top => self.top = 0,
            Key::End => self.top = usize::MAX / 2,
            _ => return,
        }
        self.shown.clear();
    }
}

/// Copies `text` with the first of `pbcopy`, `wl-copy` and `xclip` that runs; `said` when one did, else
/// that none could.
pub(super) fn clipboard(text: &str, said: &str) -> String {
    let commands: [(&str, &[&str]); 3] = [
        ("pbcopy", &[]),
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
    ];
    for (name, args) in commands {
        let spawned = std::process::Command::new(name)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = spawned else { continue };
        let written = child
            .stdin
            .take()
            .is_some_and(|mut stdin| stdin.write_all(text.as_bytes()).is_ok());
        if child.wait().is_ok_and(|s| s.success()) && written {
            return said.to_string();
        }
    }
    "no pbcopy, wl-copy or xclip to copy with".to_string()
}

/// The time between two frames of the spinner.
pub(super) const TURN: Duration = Duration::from_millis(100);

/// The spinner's frame at this moment.
pub(super) fn spin() -> usize {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| (d.as_millis() / TURN.as_millis()) as usize)
}

/// The time from now until the second `at` since the epoch starts.
pub(super) fn until(at: u64) -> Duration {
    (std::time::UNIX_EPOCH + Duration::from_secs(at))
        .duration_since(std::time::SystemTime::now())
        .unwrap_or_default()
}
