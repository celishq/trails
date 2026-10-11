//! `celis watch`: a trail's tree, printed again each time the store changes.

use super::tree::Tree;
use super::{Args, Command, Snapshot};
use crate::{Failure, State};
use notify::{RecursiveMode, Watcher};
use std::io::Read;
use std::io::{IsTerminal, Write};
use std::os::fd::FromRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Prints the tree of the step `args` names, or with no id every active trail, then prints it again
    /// when a record lands, a time it shows changes, or the terminal's width changes, and sleeps between.
    /// With no store, it writes nothing and waits for one. Runs until it is stopped.
    pub(super) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let terminal = std::io::stdout().is_terminal();
        let mut shown = String::new();
        if !self.held()? {
            if let Some(id) = args.words.get(1) {
                return Err(Failure::no_step(id));
            }
            if !args.json {
                show(out, &format!("{EMPTY}\n"), &mut shown, terminal)?;
            }
            while !self.held()? {
                std::thread::sleep(Duration::from_millis(250));
            }
        }
        let mut snapshot = self.snapshot()?;
        let id = match args.words.get(1) {
            Some(id) => Some(snapshot.steps().find(id)?.id.clone()),
            None => None,
        };
        let (send, events) = channel();
        resizes(send.clone());
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
        let mut width = self.columns();
        loop {
            snapshot.now = super::now();
            let (tree, next) = self.frame(&snapshot, id.as_deref(), args, width)?;
            show(out, &tree, &mut shown, terminal)?;
            let wake = match next {
                Some(at) => events.recv_timeout(until(at)),
                None => events.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match wake {
                Ok(Wake::Record) => {
                    while events.recv_timeout(Duration::from_millis(50)).is_ok() {}
                    snapshot = self.snapshot()?;
                }
                Ok(Wake::Resize) => {}
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
            if self.columns() != width {
                width = self.columns();
                shown.clear();
            }
        }
    }

    /// The tree of `snapshot` that `watch` prints now, `width` wide, short or full as `args` asks: of the
    /// step `id`, or of every active trail of every project, newest first; with none, what to do to start
    /// one. Then the time it changes next, if any.
    fn frame(
        &self,
        snapshot: &Snapshot,
        id: Option<&str>,
        args: &Args,
        width: Option<usize>,
    ) -> Result<(String, Option<u64>), Failure> {
        let steps = snapshot.steps();
        let decisions = snapshot.decisions();
        let notes: Vec<_> = match id {
            Some(id) => steps.find(id).ok().into_iter().collect(),
            None => steps
                .roots()
                .into_iter()
                .rev()
                .filter(|n| steps.state(n) == State::Active)
                .collect(),
        };
        let mut printed = Vec::new();
        let mut next = None;
        if !notes.is_empty() {
            let tree = Tree {
                steps: &steps,
                decisions: &decisions,
                colour: self.colour(),
                full: args.full,
                fold: false,
                all: id.is_none(),
                width,
                next: Default::default(),
            };
            tree.print(&notes, "trail", args.json, &mut printed)?;
            next = tree.next.get();
        } else if id.is_none() && !args.json {
            return Ok((format!("{EMPTY}\n"), None));
        }
        Ok((String::from_utf8_lossy(&printed).into_owned(), next))
    }
}

/// Prints `tree` when it is not empty and not what `shown` holds: on a terminal from the top left of a
/// cleared screen, else after an empty line.
fn show(
    out: &mut dyn Write,
    tree: &str,
    shown: &mut String,
    terminal: bool,
) -> Result<(), Failure> {
    if tree.is_empty() || tree == shown {
        return Ok(());
    }
    let gap = match (terminal, shown.is_empty()) {
        (true, _) => "\x1b[H\x1b[2J\x1b[3J",
        (false, true) => "",
        (false, false) => "\n",
    };
    write!(out, "{gap}{tree}")
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

/// What `watch` with no id prints while no trail is active.
const EMPTY: &str = "No active trail. Start one in your agent with /celis:trail <what you want>,\nand it shows here as the agent works.";

/// What wakes `watch` before its next tick.
enum Wake {
    Record,
    Resize,
}

/// The pipe the resize signal writes to; -1 before `resizes` opens it.
static RESIZED: AtomicI32 = AtomicI32::new(-1);

/// Sends a `Resize` to `send` each time the terminal's size changes, the moment the signal arrives.
fn resizes(send: Sender<Wake>) {
    let mut fds = [0; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return;
    }
    RESIZED.store(fds[1], Ordering::Relaxed);
    let handler = resized as extern "C" fn(libc::c_int);
    unsafe { libc::signal(libc::SIGWINCH, handler as libc::sighandler_t) };
    let mut pipe = unsafe { std::fs::File::from_raw_fd(fds[0]) };
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        while pipe.read(&mut byte).is_ok_and(|n| n > 0) && send.send(Wake::Resize).is_ok() {}
    });
}

/// Writes one byte to the resize pipe; it does only what a signal handler may.
extern "C" fn resized(_: libc::c_int) {
    let fd = RESIZED.load(Ordering::Relaxed);
    unsafe { libc::write(fd, [0u8].as_ptr().cast(), 1) };
}
