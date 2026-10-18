//! A `celis` command that runs on a terminal, and what it prints there.

use super::Repo;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

/// What starts each tree: a frame drawn from the top left.
pub const FRAME: &str = "\x1b[?2026h\x1b[H";

/// A running `celis tree` on a terminal, and the text it printed so far.
pub struct Watch {
    pub child: Child,
    lead: i32,
    bytes: Receiver<Vec<u8>>,
    pub seen: Vec<u8>,
}

impl Watch {
    /// Runs `celis <args>` on a new terminal in the main checkout of `repo`, at about noon.
    pub fn start(repo: &Repo, args: &[&str]) -> Watch {
        let (mut lead, mut follow) = (0, 0);
        let opened = unsafe {
            libc::openpty(
                &mut lead,
                &mut follow,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(opened, 0, "openpty");
        let side = |fd| unsafe { Stdio::from_raw_fd(libc::dup(fd)) };
        let child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .args(args)
            .current_dir(repo.path())
            .env_clear()
            .env("PATH", path(repo))
            .env("CELIS_HOME", repo.home())
            .env("TZ", super::noon())
            .stdin(side(follow))
            .stdout(side(follow))
            .stderr(side(follow))
            .spawn()
            .unwrap();
        unsafe { libc::close(follow) };
        let (send, bytes) = channel();
        let mut terminal = unsafe { std::fs::File::from_raw_fd(libc::dup(lead)) };
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(n) = terminal.read(&mut buffer) {
                if n == 0 || send.send(buffer[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Watch {
            child,
            lead,
            bytes,
            seen: Vec::new(),
        }
    }

    /// All the text printed so far, after a wait of `wait` for more.
    pub fn shown(&mut self, wait: Duration) -> String {
        let end = Instant::now() + wait;
        while let Ok(chunk) = self
            .bytes
            .recv_timeout(end.saturating_duration_since(Instant::now()))
        {
            self.seen.extend(chunk);
        }
        plain(&String::from_utf8_lossy(&self.seen))
    }

    /// The trees printed so far, one string each with no line or frame codes, after a wait of `wait`
    /// for more.
    pub fn trees(&mut self, wait: Duration) -> Vec<String> {
        self.shown(wait)
            .split(FRAME)
            .skip(1)
            .map(|t| {
                t.replace("\x1b[K", "")
                    .replace("\x1b[J", "")
                    .replace("\x1b[?2026l", "")
            })
            .collect()
    }

    /// Waits until the newest tree holds `text`; the time it took.
    pub fn until(&mut self, text: &str) -> Duration {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if self
                .trees(Duration::from_millis(50))
                .last()
                .is_some_and(|t| t.contains(text))
            {
                return start.elapsed();
            }
        }
        panic!("no {text:?} in {:?}", self.shown(Duration::ZERO));
    }

    /// The newest tree with its colour codes, after a wait of `wait` for more.
    pub fn coloured(&mut self, wait: Duration) -> String {
        self.shown(wait);
        let text = String::from_utf8_lossy(&self.seen).into_owned();
        text.rsplit(FRAME).next().unwrap_or("").to_string()
    }

    /// The lines of the newest tree, each with no codes, above the key line.
    pub fn rows(&mut self) -> Vec<String> {
        let last = self.trees(Duration::ZERO).pop().unwrap_or_default();
        let body = last.split("\x1b[").next().unwrap_or("").to_string();
        body.split("\r\n").map(str::to_string).collect()
    }

    /// Waits until the newest tree no longer holds `text`.
    pub fn without(&mut self, text: &str) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if self
                .trees(Duration::from_millis(50))
                .last()
                .is_some_and(|t| !t.contains(text))
            {
                return;
            }
        }
        panic!("still {text:?} in {:?}", self.shown(Duration::ZERO));
    }

    /// Types `keys` on the terminal.
    pub fn press(&self, keys: &str) {
        let written = unsafe { libc::write(self.lead, keys.as_ptr().cast(), keys.len()) };
        assert_eq!(written, keys.len() as isize);
    }

    /// Turns the terminal `columns` wide, and signals it.
    pub fn resize(&self, columns: u16) {
        self.size(24, columns);
    }

    /// Turns the terminal `rows` high and `columns` wide, and signals it.
    pub fn size(&self, rows: u16, columns: u16) {
        let size = libc::winsize {
            ws_row: rows,
            ws_col: columns,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        assert_eq!(
            unsafe { libc::ioctl(self.lead, libc::TIOCSWINSZ, &size) },
            0
        );
        let pid = libc::pid_t::try_from(self.child.id()).unwrap();
        assert_eq!(unsafe { libc::kill(pid, libc::SIGWINCH) }, 0);
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        unsafe { libc::close(self.lead) };
    }
}

/// The watch's `PATH`: the home's `bin` folder, then the test's own.
fn path(repo: &Repo) -> String {
    let own = std::env::var("PATH").unwrap_or_default();
    format!("{}:{own}", repo.home().join("bin").display())
}

/// `text` with no colour codes.
pub fn plain(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("\x1b[") {
        out.push_str(&rest[..at]);
        let code = &rest[at + 2..];
        match code.find(|c: char| c.is_ascii_alphabetic()) {
            Some(end) if code.as_bytes()[end] == b'm' => rest = &code[end + 1..],
            _ => {
                out.push_str("\x1b[");
                rest = code;
            }
        }
    }
    out + rest
}
