//! `celis watch`: a trail's tree, printed again each time the store changes.

use super::tree::Tree;
use super::{Args, Command, Snapshot};
use crate::{Failure, State, Store};
use notify::{RecursiveMode, Watcher};
use std::io::Read;
use std::io::{IsTerminal, Write};
use std::os::fd::FromRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Prints the tree of the step `args` names, or with no id every active trail, then prints it again
    /// each time the tree changed, by a record or by the time, or the terminal's width changed. Runs until
    /// it is stopped.
    pub(super) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let store = self.store()?;
        let snapshot = Snapshot::of(&store);
        let steps = snapshot.steps();
        let id = match args.words.get(1) {
            Some(id) => Some(steps.find(id)?.id.clone()),
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
        let terminal = std::io::stdout().is_terminal();
        let mut shown = String::new();
        let mut width = self.columns();
        let mut store = Store::open(&self.home()?)?;
        loop {
            let tree = self.frame(&store, id.as_deref(), args, width)?;
            if !tree.is_empty() && tree != shown {
                let gap = match (terminal, shown.is_empty()) {
                    (true, _) => "\x1b[H\x1b[2J\x1b[3J",
                    (false, true) => "",
                    (false, false) => "\n",
                };
                write!(out, "{gap}{tree}")
                    .and_then(|_| out.flush())
                    .map_err(anyhow::Error::from)?;
                shown = tree;
            }
            match events.recv_timeout(Duration::from_millis(250)) {
                Ok(Wake::Record) => {
                    while events.recv_timeout(Duration::from_millis(50)).is_ok() {}
                    store = Store::open(&self.home()?)?;
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

    /// The tree of `store` that `watch` prints now, `width` wide, short or full as `args` asks: of the step
    /// `id`, or of every active trail of every project, newest first; empty with none.
    fn frame(
        &self,
        store: &Store,
        id: Option<&str>,
        args: &Args,
        width: Option<usize>,
    ) -> Result<String, Failure> {
        let snapshot = Snapshot::of(store);
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
        if !notes.is_empty() {
            let tree = Tree {
                steps: &steps,
                decisions: &decisions,
                styled: self.styled(),
                full: args.full,
                fold: false,
                all: id.is_none(),
                width,
            };
            tree.print(&notes, "trail", args.json, &mut printed)?;
        }
        Ok(String::from_utf8_lossy(&printed).into_owned())
    }
}

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
