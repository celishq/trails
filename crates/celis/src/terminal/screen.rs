//! What a watch draws on the terminal, and what it reads from it.

use super::canvas::{clip, line, Styled, DIM};
use super::legend::legend;
use super::remark::Remark;
use super::wake::{keys, signals, Key, Wake};
use crate::command::Command;
use crate::failure::Failure;
use celis_trails::Link;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::text::Span;
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

impl Command {
    /// Runs `body` on the alternate screen with autowrap off, with the keys and signals it reads sent to
    /// the channel it gets, once it asked the terminal its background; then sets the terminal back and
    /// leaves the alternate screen with nothing printed.
    pub(crate) fn screen(
        &self,
        out: &mut dyn Write,
        body: impl FnOnce(&mut dyn Write, (Sender<Wake>, Receiver<Wake>)) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let (send, events) = channel();
        signals(send.clone());
        super::background::ask();
        let typed = keys(send.clone());
        write!(out, "\x1b[?1049h\x1b[?7l\x1b[?25l").map_err(anyhow::Error::from)?;
        let done = body(out, (send, events));
        if let Some(terminal) = typed {
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &terminal) };
        }
        write!(out, "\x1b[?25h\x1b[?7h\x1b[?1049l")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        done
    }

    /// Sends to `send` a `Record` with the files each time records land in the store or a linked folder,
    /// or a `Rescan` when the system lost track of them; the watcher that does it, kept while it should.
    pub(crate) fn listen(&self, send: Sender<Wake>) -> Result<RecommendedWatcher, Failure> {
        let home = self.home()?;
        let shared = home.join("trails/shared");
        let links = Link::all(&home, &self.folder).into_iter().map(|l| l.folder);
        let landed = move |event: notify::Result<notify::Event>| {
            let Ok(event) = event else { return };
            let wake = match event.need_rescan() {
                true => Wake::Rescan,
                false if event.kind.is_access() => return,
                false => Wake::Record(event.paths),
            };
            let _ = send.send(wake);
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

/// What a watch shows of its tree: whether in colour, the first line in view, whether the view moves to
/// the selected line on the next draw, the lines a page holds, the rows on the screen, what the last
/// key did when it has something to say, when it was first drawn, when it first drew the checks it
/// draws now, the remark it shows with when it changed to it, the remark it showed before, every
/// remark it showed and since when it shows the newest of them, whether the terminal
/// was resized since it last drew, the second it started, whether it held the scene when it
/// started, and when it last wrote a frame.
#[derive(Default)]
pub(crate) struct Screen {
    pub colour: bool,
    pub top: usize,
    pub follow: bool,
    pub page: usize,
    pub shown: Vec<String>,
    pub said: Option<String>,
    pub opened: Option<std::time::Instant>,
    pub checked: Option<std::time::Instant>,
    pub remark: Option<(Remark, Option<std::time::Instant>)>,
    pub former: Option<Remark>,
    pub seen: Vec<Remark>,
    pub since: Option<std::time::Instant>,
    pub resized: bool,
    pub started: Option<u64>,
    pub played: Option<bool>,
    pub wrote: Option<std::time::Instant>,
}

impl Screen {
    /// Draws `head` then `lines` over the screen from the top left, the whole screen cleared first after
    /// a resize, each row only when it changed: each line cut short of the last column and cleared to its
    /// end, the rows under `head` and above the last scrolled to show its `picked` line when the
    /// selection moved, the key line `keys`, its keys bold, or what the last key did on the last row, dim,
    /// and the rest cleared.
    pub fn draw(
        &mut self,
        out: &mut dyn Write,
        head: &[Styled],
        lines: &[Styled],
        picked: Option<usize>,
        keys: &str,
    ) -> Result<(), Failure> {
        let size = crate::command::size();
        let width = size.map(|s| usize::from(s.ws_col)).filter(|&w| w > 1);
        let width = width.map(|w| w - 1);
        let rows = size.map(|s| usize::from(s.ws_row)).filter(|&r| r > 0);
        let mut screen = head.to_vec();
        match rows
            .map(|r| r.saturating_sub(1 + head.len()))
            .filter(|&body| body >= 3 && lines.len() > body)
        {
            Some(body) => screen.extend(self.window(lines, body, picked)),
            None => {
                self.top = 0;
                screen.extend_from_slice(lines);
            }
        }
        let last = rows.map(|r| r - 1).filter(|&k| k >= screen.len());
        screen.resize(last.unwrap_or(screen.len()), Styled::default());
        screen.push(match &self.said {
            Some(said) => Styled::from(Span::styled(said.to_string(), DIM)),
            None => legend(keys),
        });
        let screen: Vec<String> = screen
            .iter()
            .map(|l| match width {
                Some(w) => line(&clip(l, w), self.colour),
                None => line(l, self.colour),
            })
            .collect();
        if screen == self.shown {
            return Ok(());
        }
        let mut text = String::from("\x1b[?2026h");
        if std::mem::take(&mut self.resized) {
            text += "\x1b[H\x1b[2J";
        }
        for (n, row) in screen.iter().enumerate() {
            if self.shown.get(n) != Some(row) {
                text += &format!("\x1b[{};1H{row}\x1b[K", n + 1);
            }
        }
        if rows.is_none() && (self.shown.is_empty() || screen.len() < self.shown.len()) {
            text += &format!("\x1b[{};1H\x1b[J", screen.len() + 1);
        }
        text += "\x1b[?2026l";
        write!(out, "{text}")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        self.shown = screen;
        self.wrote = Some(std::time::Instant::now());
        Ok(())
    }

    /// The `body` rows of `lines` in view from `top`: a first row for the lines above it and a last
    /// row for the lines below it, when there are any; the `picked` line in view when the selection
    /// moved, from the first line when it fits there.
    fn window(&mut self, lines: &[Styled], body: usize, picked: Option<usize>) -> Vec<Styled> {
        let len = lines.len();
        let fit = |top: usize| {
            let up = usize::from(top > 0);
            let shown = body - up;
            shown - usize::from(top + shown < len)
        };
        self.top = self.top.min(len - (body - 1));
        if let (true, Some(picked)) = (std::mem::take(&mut self.follow), picked) {
            if picked < fit(0) {
                self.top = 0;
            } else if picked < self.top {
                self.top = picked;
            }
            while picked >= self.top + fit(self.top) {
                self.top += 1;
            }
        }
        let shown = fit(self.top);
        self.page = shown;
        let more = |text: String| Styled::from(vec![Span::raw("  "), Span::styled(text, DIM)]);
        let mut view = Vec::new();
        if self.top > 0 {
            view.push(more(format!("↑ {} more", self.top)));
        }
        view.extend_from_slice(&lines[self.top..self.top + shown]);
        if self.top + shown < len {
            view.push(more(format!("↓ {} more", len - self.top - shown)));
        }
        view
    }

    /// Scrolls as `key` asks: a page up or down, or to the top or the end.
    pub fn press(&mut self, key: &Key) {
        let page = self.page.max(1);
        match key {
            Key::PageUp => self.top = self.top.saturating_sub(page),
            Key::PageDown | Key::Space => self.top += page,
            Key::Top => self.top = 0,
            Key::End => self.top = usize::MAX / 2,
            _ => return,
        }
        self.shown.clear();
    }
}

/// Copies `text` with the first of `pbcopy`, `wl-copy` and `xclip` that runs; `said` when one did, else
/// that none could.
pub(crate) fn clipboard(text: &str, said: &str) -> String {
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

/// The time from now until the second `at` since the epoch starts.
pub(crate) fn until(at: u64) -> Duration {
    (std::time::UNIX_EPOCH + Duration::from_secs(at))
        .duration_since(std::time::SystemTime::now())
        .unwrap_or_default()
}
