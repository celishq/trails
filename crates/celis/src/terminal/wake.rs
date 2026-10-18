//! What wakes a watch: a record that lands, the store read again, a key, a new size, or an interrupt.

use crate::failure::Failure;
use celis_trails::Trails;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::Sender;

/// What wakes the watch before its next tick: the files of records that landed, a change it must read
/// the whole store for, or the store read again.
pub(crate) enum Wake {
    Record(Vec<PathBuf>),
    Rescan,
    Store(Box<Result<Trails, Failure>>),
    Resize,
    Key(Key),
    Quit,
}

/// The pipe the signal handler writes to; -1 before `signals` opens it.
static SIGNALLED: AtomicI32 = AtomicI32::new(-1);

/// Sends a `Resize` to `send` each time the terminal's size changes, and a `Quit` on an interrupt or a
/// stop, the moment the signal arrives.
pub(crate) fn signals(send: Sender<Wake>) {
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

/// A key a watch reads.
pub(crate) enum Key {
    Up,
    Down,
    Enter,
    Right,
    Left,
    Star,
    PageUp,
    PageDown,
    Space,
    Tab,
    BackTab,
    Back,
    Forth,
    Escape,
    Top,
    End,
    Full,
    All,
    Copy,
    Markdown,
}

impl Key {
    /// Whether it changes what a page shows, so the keys after it wait for the page drawn again.
    pub(crate) fn lays_out(&self) -> bool {
        matches!(
            self,
            Key::Enter | Key::Right | Key::Left | Key::Star | Key::Full | Key::All
        )
    }
}

/// When stdin is a terminal, turns its echo and line mode off and sends each key it reads to `send`,
/// `q` as a `Quit` and an escape that comes alone as an `Escape`; the terminal's settings before, to
/// set back.
pub(crate) fn keys(send: Sender<Wake>) -> Option<libc::termios> {
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
        let mut stdin = std::io::stdin().lock();
        let mut chunk = [0u8; 64];
        let mut typed = Vec::new();
        while let Ok(read @ 1..) = stdin.read(&mut chunk) {
            for (n, byte) in chunk[..read].iter().enumerate() {
                typed.push(*byte);
                let alone = n + 1 == read;
                let wake = match typed.as_slice() {
                    [0x1b] if alone => Wake::Key(Key::Escape),
                    [0x1b] | [0x1b, b'['] | [0x1b, b'[', b'5' | b'6'] => continue,
                    [0x1b, b'[', b'5', b'~'] => Wake::Key(Key::PageUp),
                    [0x1b, b'[', b'6', b'~'] => Wake::Key(Key::PageDown),
                    [0x1b, b'[', b'Z'] => Wake::Key(Key::BackTab),
                    [b'\t'] => Wake::Key(Key::Tab),
                    [b' '] => Wake::Key(Key::Space),
                    [b'['] => Wake::Key(Key::Back),
                    [b']'] => Wake::Key(Key::Forth),
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
                    return;
                }
            }
        }
    });
    Some(before)
}
