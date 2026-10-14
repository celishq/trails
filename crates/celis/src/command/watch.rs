//! `celis tree` on a terminal: the tree, drawn again each time the store changes.

use super::tree::{cut, View, EMPTY};
use super::{Args, Command};
use celis_trails::Failure;
use notify::{RecursiveMode, Watcher};
use std::io::Read;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Draws the tree `args` names on the alternate screen, then again when a record lands, a time it
    /// shows changes, the spinner turns, a key is read, or the terminal's size changes, and sleeps
    /// between. With no store, it writes nothing and waits for one. On an interrupt or `q`, prints the
    /// last tree on the main screen and ends.
    pub(super) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if let (false, Some(id)) = (self.held()?, args.words.get(1)) {
            return Err(Failure::no_step(id));
        }
        let (send, events) = channel();
        signals(send.clone());
        let typed = keys(send.clone());
        write!(out, "\x1b[?1049h\x1b[?25l").map_err(anyhow::Error::from)?;
        let mut shown = String::new();
        let watched = self.watched(args, out, (send, events), &mut shown);
        if let Some(terminal) = typed {
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &terminal) };
        }
        let shown = shown.strip_suffix(KEYS).unwrap_or(&shown);
        write!(out, "\x1b[?25h\x1b[?1049l{shown}")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)?;
        watched
    }

    /// The watch's loop: draws each tree it reads from the store into `shown`, until an interrupt.
    fn watched(
        &self,
        args: &Args,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
        shown: &mut String,
    ) -> Result<(), Failure> {
        if !self.held()? {
            draw(out, &format!("{EMPTY}\n"), shown)?;
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
        let landed = move |event: notify::Result<notify::Event>| {
            if event.is_ok_and(|e| !e.kind.is_access()) {
                let _ = send.send(Wake::Record);
            }
        };
        let mut watcher = notify::recommended_watcher(landed).map_err(anyhow::Error::from)?;
        watcher
            .watch(
                &self.home()?.join("trails/shared"),
                RecursiveMode::NonRecursive,
            )
            .map_err(anyhow::Error::from)?;
        let mut view = View::of(args);
        loop {
            trails.now = super::now();
            view.spin = Some(spin());
            let frame = self.frame(&trails, args, self.columns(), &view)?;
            let mut tree = frame.text;
            if tree.is_empty() && args.words.len() < 2 {
                tree = format!("{EMPTY}\n");
            }
            draw(out, &format!("{tree}{KEYS}"), shown)?;
            let order = frame.order;
            let wait = frame.next.map(until);
            let wait = match frame.spun {
                true => wait.map_or(TURN, |w| w.min(TURN)),
                false => wait.unwrap_or(Duration::MAX),
            };
            let mut wake = events.recv_timeout(wait);
            let mut landed = false;
            loop {
                match wake {
                    Ok(Wake::Record) => landed = true,
                    Ok(Wake::Resize) => shown.clear(),
                    Ok(Wake::Key(key)) => press(key, &mut view, &order),
                    Ok(Wake::Quit) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                    Err(RecvTimeoutError::Timeout) => break,
                }
                if !landed {
                    break;
                }
                wake = events.recv_timeout(Duration::from_millis(50));
            }
            if landed {
                trails = self.trails()?;
            }
        }
    }
}

/// The line that ends each tree of a watch, dim, with the keys it reads.
const KEYS: &str = "\x1b[2m↑↓ select · enter open · f full · a all · q quit\x1b[0m\n";

/// A key a watch reads.
enum Key {
    Up,
    Down,
    Enter,
    Full,
    All,
}

/// Changes `view` as `key` asks: moves the selection along `order`, flips the selected trail, or turns
/// full or all.
fn press(key: Key, view: &mut View, order: &[String]) {
    let at = view
        .selected
        .as_ref()
        .and_then(|s| order.iter().position(|o| o == s));
    let moved = match (key, at) {
        (Key::Down, None) => order.first(),
        (Key::Down, Some(at)) => order.get(at + 1).or(order.get(at)),
        (Key::Up, Some(at)) => order.get(at.saturating_sub(1)),
        (Key::Up, None) => None,
        (Key::Enter, _) => {
            if let Some(id) = view.selected.clone() {
                if !view.flipped.remove(&id) {
                    view.flipped.insert(id);
                }
            }
            return;
        }
        (Key::Full, _) => return view.full = !view.full,
        (Key::All, _) => return view.all = !view.all,
    };
    if let Some(id) = moved {
        view.selected = Some(id.clone());
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
                [0x1b] | [0x1b, b'['] => continue,
                [0x1b, b'[', b'A'] => Wake::Key(Key::Up),
                [0x1b, b'[', b'B'] => Wake::Key(Key::Down),
                [b'\r'] | [b'\n'] => Wake::Key(Key::Enter),
                [b'f'] => Wake::Key(Key::Full),
                [b'a'] => Wake::Key(Key::All),
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

/// The time between two frames of the spinner.
const TURN: Duration = Duration::from_millis(100);

/// The spinner's frame at this moment.
fn spin() -> usize {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| (d.as_millis() / TURN.as_millis()) as usize)
}

/// Draws `tree` over the screen from the top left, when it is not what `shown` holds: each line cut to the
/// width and cleared to its end, the rest of the screen cleared, and the tree cut to the screen's height.
fn draw(out: &mut dyn Write, tree: &str, shown: &mut String) -> Result<(), Failure> {
    if tree.is_empty() || tree == shown {
        return Ok(());
    }
    let size = super::size();
    let width = size.map(|s| usize::from(s.ws_col)).filter(|&w| w > 0);
    let rows = size.map(|s| usize::from(s.ws_row)).filter(|&r| r > 0);
    let mut lines: Vec<String> = tree
        .lines()
        .map(|l| width.map_or(l.to_string(), |w| cut(l, w)))
        .collect();
    let keys = lines.pop_if(|l| l.contains("↑↓ select"));
    if let Some(rows) = rows.map(|r| r.saturating_sub(usize::from(keys.is_some())).max(1)) {
        if lines.len() > rows {
            let more = lines.len() - rows + 1;
            lines.truncate(rows - 1);
            lines.push(format!("… {more} more lines"));
        }
    }
    lines.extend(keys);
    write!(
        out,
        "\x1b[?2026h\x1b[H{}\x1b[K\x1b[J\x1b[?2026l",
        lines.join("\x1b[K\n")
    )
    .and_then(|_| out.flush())
    .map_err(anyhow::Error::from)?;
    *shown = tree.to_string();
    Ok(())
}

/// The time from now until the second `at` since the epoch starts.
fn until(at: u64) -> Duration {
    (std::time::UNIX_EPOCH + Duration::from_secs(at))
        .duration_since(std::time::SystemTime::now())
        .unwrap_or_default()
}

/// What wakes the watch before its next tick.
enum Wake {
    Record,
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
