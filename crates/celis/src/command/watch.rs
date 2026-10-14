//! `celis tree` on a terminal: the tree, drawn again each time the store or a linked folder changes.

use super::tree::{cut, Item, View, EMPTY, MORE};
use super::{Args, Command};
use celis_trails::{Failure, Link};
use notify::{RecursiveMode, Watcher};
use std::io::Read;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::process::Stdio;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Draws the tree `args` names on the alternate screen, then again when a record lands in the store
    /// or a linked folder, a time it shows changes, the spinner turns, a key is read, or the terminal's
    /// size changes, and sleeps between. With no store, it writes nothing and waits for one. On an
    /// interrupt or `q`, leaves the alternate screen with nothing printed, and ends.
    pub(super) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if let (false, Some(id)) = (self.held()?, args.words.get(1)) {
            return Err(Failure::no_step(id));
        }
        let (send, events) = channel();
        signals(send.clone());
        let typed = keys(send.clone());
        write!(out, "\x1b[?1049h\x1b[?25l").map_err(anyhow::Error::from)?;
        let watched = self.watched(args, out, (send, events));
        if let Some(terminal) = typed {
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &terminal) };
        }
        write!(out, "\x1b[?25h\x1b[?1049l")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        watched
    }

    /// The watch's loop: draws each tree it reads from the store, until an interrupt.
    fn watched(
        &self,
        args: &Args,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
    ) -> Result<(), Failure> {
        let mut screen = Screen::default();
        let empty = format!("{EMPTY}\n");
        if !self.held()? {
            screen.draw(out, &empty, None, &legend(None))?;
            while !self.held()? {
                if let Ok(Wake::Quit) = events.recv_timeout(Duration::from_millis(250)) {
                    return Ok(());
                }
            }
        }
        let mut trails = self.trails()?;
        if let Some(id) = args.words.get(1) {
            trails.steps().find(id)?;
        }
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
        let named = args.words.len() > 1;
        let mut view = View {
            watch: true,
            ..View::of(args)
        };
        let mut known: Vec<Item> = vec![];
        loop {
            trails.now = super::now();
            view.spin = Some(spin());
            let width = self.columns().map(|c| c.saturating_sub(1));
            let frame = self.frame(&trails, args, width, &view)?;
            if let Some(id) = view.selected.clone() {
                if !frame.order.iter().any(|i| i.id == id) {
                    view.selected = kept(&known, &frame.order, &id);
                    screen.follow = true;
                    continue;
                }
            }
            let tree = match frame.text.is_empty() && !named {
                true => empty.clone(),
                false => frame.text,
            };
            let selected = frame
                .order
                .iter()
                .find(|i| view.selected.as_ref() == Some(&i.id));
            let copy = selected.and_then(Item::copy);
            let chosen = selected.filter(|_| copy.is_some()).map(|i| i.id.clone());
            screen.draw(out, &tree, frame.picked, &legend(copy.as_deref()))?;
            let wait = frame.next.map(until);
            let wait = match frame.spun {
                true => wait.map_or(TURN, |w| w.min(TURN)),
                false => wait.unwrap_or(Duration::MAX),
            };
            let mut wake = events.recv_timeout(wait);
            let (mut landed, mut linked) = (false, false);
            loop {
                match wake {
                    Ok(Wake::Record(from)) => (landed, linked) = (true, linked || from),
                    Ok(Wake::Resize) => screen.shown.clear(),
                    Ok(Wake::Key(key)) => {
                        screen.said = match (&key, &copy, &chosen) {
                            (Key::Copy, Some(id), _) => {
                                Some(clipboard(id, &format!("copied {id}")))
                            }
                            (Key::Markdown, _, Some(id)) => {
                                let markdown = super::report::part(&trails, id)?;
                                Some(clipboard(&markdown, "copied markdown"))
                            }
                            _ => None,
                        };
                        let selected = view.selected.clone();
                        screen.press(&key);
                        press(key, &mut view, &frame.order, named);
                        screen.follow |= view.selected != selected;
                    }
                    Ok(Wake::Quit) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                    Err(RecvTimeoutError::Timeout) => break,
                }
                if !landed {
                    break;
                }
                wake = events.recv_timeout(Duration::from_millis(50));
            }
            match (landed, linked) {
                (_, true) => trails = self.trails()?,
                (true, false) => _ = trails.fresh(),
                _ => {}
            }
            known = frame.order;
        }
    }
}

/// The line on the last row of a watch with the keys it reads, `c` with the id `copy` it copies.
fn legend(copy: Option<&str>) -> String {
    let copy = copy.map_or(String::new(), |id| {
        format!(" · c copy {id} · m copy markdown")
    });
    format!("↑↓ select · ←→ close open · * open all under{copy} · PgUp PgDn scroll · f full · a all · q quit")
}

/// What a watch shows of its tree: the first line in view, whether the view moves to the selected
/// line on the next draw, the lines a page holds, the text on the screen, and what the last key did
/// when it has something to say.
#[derive(Default)]
struct Screen {
    top: usize,
    follow: bool,
    page: usize,
    shown: String,
    said: Option<String>,
}

impl Screen {
    /// Draws `tree` over the screen from the top left, when it changed: each line cut short of the last
    /// column and cleared to its end, the rows above the last scrolled to show its `picked` line when the
    /// selection moved, `keys` or what the last key did on the last row, dim, and the rest cleared.
    fn draw(
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
    fn press(&mut self, key: &Key) {
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

/// A key a watch reads.
enum Key {
    Up,
    Down,
    Enter,
    Right,
    Left,
    Star,
    PageUp,
    PageDown,
    Top,
    End,
    Full,
    All,
    Copy,
    Markdown,
}

/// Changes `view` as `key` asks: moves the selection along `order`, opens or closes the selected trail,
/// rule or step, or everything under it too, selects the one it is under, or turns full on or off, and
/// all when the watch `named` no id.
fn press(key: Key, view: &mut View, order: &[Item], named: bool) {
    let at = view
        .selected
        .as_ref()
        .and_then(|s| order.iter().position(|o| &o.id == s));
    let last = order.len().saturating_sub(1);
    match (key, at.map(|at| &order[at])) {
        (Key::Down, None) => view.selected = order.first().map(|o| o.id.clone()),
        (Key::Down, Some(_)) => view.selected = at.map(|at| order[(at + 1).min(last)].id.clone()),
        (Key::Up, Some(_)) => view.selected = at.map(|at| order[at.saturating_sub(1)].id.clone()),
        (Key::Enter, Some(item)) => turn(view, &item.id, item.open, !item.open),
        (Key::Right, Some(item)) => turn(view, &item.id, item.open, true),
        (Key::Left, Some(item)) if item.open => turn(view, &item.id, true, false),
        (Key::Left, Some(item)) => {
            if let Some(parent) = &item.parent {
                view.selected = Some(parent.clone());
            }
        }
        (Key::Star, Some(item)) => {
            let opened = |view: &View, id: &String| view.full != view.flipped.contains(id);
            let open = !(item.open && item.under.iter().all(|id| opened(view, id)));
            turn(view, &item.id, item.open, open);
            for id in &item.under {
                let now = opened(view, id);
                turn(view, id, now, open);
            }
        }
        (Key::Up | Key::Enter | Key::Right | Key::Left | Key::Star, None) => {}
        (Key::PageUp | Key::PageDown | Key::Top | Key::End | Key::Copy | Key::Markdown, _) => {}
        (Key::Full, _) => view.full = !view.full,
        (Key::All, _) => view.all = !view.all && !named,
    }
}

/// The line to select once the selected `id` left `order`: the nearest line it was under in `known`
/// that is still in `order`, else the first.
fn kept(known: &[Item], order: &[Item], id: &str) -> Option<String> {
    let mut at = Some(id.to_string());
    while let Some(id) = at {
        if order.iter().any(|i| i.id == id) {
            return Some(id);
        }
        at = known
            .iter()
            .find(|i| i.id == id)
            .and_then(|i| i.parent.clone());
    }
    order.first().map(|i| i.id.clone())
}

/// Opens the line `id` in `view` when `open`, else closes it; it is open when `now`. `Show more` opens
/// as every trail by day.
fn turn(view: &mut View, id: &str, now: bool, open: bool) {
    if id == MORE {
        view.all = open;
        return;
    }
    if now != open && !view.flipped.remove(id) {
        view.flipped.insert(id.to_string());
    }
}

/// When stdin is a terminal, turns its echo and line mode off and sends each key it reads to `send`,
/// `q` as a `Quit`; the terminal's settings before, to set back.
fn keys(send: Sender<Wake>) -> Option<libc::termios> {
    let mut before: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut before) } != 0 {
        return None;
    }
    let mut raw = before;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) };
    std::thread::spawn(move || {
        let mut typed = Vec::new();
        for byte in std::io::stdin().lock().bytes() {
            let Ok(byte) = byte else { break };
            typed.push(byte);
            let wake = match typed.as_slice() {
                [0x1b] | [0x1b, b'['] | [0x1b, b'[', b'5' | b'6'] => continue,
                [0x1b, b'[', b'5', b'~'] => Wake::Key(Key::PageUp),
                [0x1b, b'[', b'6', b'~'] | [b' '] => Wake::Key(Key::PageDown),
                [b'g'] => Wake::Key(Key::Top),
                [b'G'] => Wake::Key(Key::End),
                [0x1b, b'[', b'A'] => Wake::Key(Key::Up),
                [0x1b, b'[', b'B'] => Wake::Key(Key::Down),
                [0x1b, b'[', b'C'] => Wake::Key(Key::Right),
                [0x1b, b'[', b'D'] => Wake::Key(Key::Left),
                [b'*'] => Wake::Key(Key::Star),
                [b'\r'] | [b'\n'] => Wake::Key(Key::Enter),
                [b'f'] => Wake::Key(Key::Full),
                [b'a'] => Wake::Key(Key::All),
                [b'c'] => Wake::Key(Key::Copy),
                [b'm'] => Wake::Key(Key::Markdown),
                [b'q'] => Wake::Quit,
                _ => {
                    typed.clear();
                    continue;
                }
            };
            typed.clear();
            if send.send(wake).is_err() {
                break;
            }
        }
    });
    Some(before)
}

/// Copies `text` with the first of `pbcopy`, `wl-copy` and `xclip` that runs; `said` when one did, else
/// that none could.
fn clipboard(text: &str, said: &str) -> String {
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
const TURN: Duration = Duration::from_millis(100);

/// The spinner's frame at this moment.
fn spin() -> usize {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| (d.as_millis() / TURN.as_millis()) as usize)
}

/// The time from now until the second `at` since the epoch starts.
fn until(at: u64) -> Duration {
    (std::time::UNIX_EPOCH + Duration::from_secs(at))
        .duration_since(std::time::SystemTime::now())
        .unwrap_or_default()
}

/// What wakes the watch before its next tick: a record, in a linked folder or not.
enum Wake {
    Record(bool),
    Resize,
    Key(Key),
    Quit,
}

/// The pipe the signal handler writes to; -1 before `signals` opens it.
static SIGNALLED: AtomicI32 = AtomicI32::new(-1);

/// Sends a `Resize` to `send` each time the terminal's size changes, and a `Quit` on an interrupt or a
/// stop, the moment the signal arrives.
fn signals(send: Sender<Wake>) {
    let mut fds = [0; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return;
    }
    SIGNALLED.store(fds[1], Ordering::Relaxed);
    let handler = signalled as extern "C" fn(libc::c_int);
    for signal in [libc::SIGWINCH, libc::SIGINT, libc::SIGTERM] {
        unsafe { libc::signal(signal, handler as libc::sighandler_t) };
    }
    let mut pipe = unsafe { std::fs::File::from_raw_fd(fds[0]) };
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        while pipe.read(&mut byte).is_ok_and(|n| n > 0) {
            let wake = match i32::from(byte[0]) {
                libc::SIGWINCH => Wake::Resize,
                _ => Wake::Quit,
            };
            if send.send(wake).is_err() {
                break;
            }
        }
    });
}

/// Writes the number of the signal to the pipe; it does only what a signal handler may.
extern "C" fn signalled(signal: libc::c_int) {
    let fd = SIGNALLED.load(Ordering::Relaxed);
    let byte = [u8::try_from(signal).unwrap_or(0)];
    unsafe { libc::write(fd, byte.as_ptr().cast(), 1) };
}
