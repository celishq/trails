//! The terminal's own background, asked once with OSC 11, so every fade ends on the screen's colour.

use std::io::{IsTerminal, Write};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// The background when the terminal does not say.
const FALLBACK: (u8, u8, u8) = (35, 38, 49);

/// The longest the terminal has to answer.
const WAIT: Duration = Duration::from_millis(300);

/// What the terminal said its background is.
static SAID: OnceLock<(u8, u8, u8)> = OnceLock::new();

/// The terminal's background: what it said when `ask` ran, else `FALLBACK`.
pub(crate) fn background() -> (u8, u8, u8) {
    SAID.get().copied().unwrap_or(FALLBACK)
}

/// Asks the terminal its background with OSC 11 and keeps its answer, waiting `WAIT` at most; does
/// nothing off a terminal or once it has an answer.
pub(crate) fn ask() {
    let terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if !terminal || SAID.get().is_some() {
        return;
    }
    let mut before: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut before) } != 0 {
        return;
    }
    let mut raw = before;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 0;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) };
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x1b]11;?\x07").and_then(|()| out.flush());
    let said = answer();
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &before) };
    if let Some(colour) = parse(&said) {
        let _ = SAID.set(colour);
    }
}

/// The bytes the terminal answers on stdin, up to its BEL or ST, or what came within `WAIT`.
fn answer() -> Vec<u8> {
    let end = Instant::now() + WAIT;
    let mut said = vec![];
    while !said.ends_with(b"\x07") && !said.ends_with(b"\x1b\\") {
        let left = end.saturating_duration_since(Instant::now()).as_millis();
        let mut poll = libc::pollfd {
            fd: libc::STDIN_FILENO,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll, 1, left.try_into().unwrap_or(0)) };
        if ready <= 0 {
            break;
        }
        let mut chunk = [0u8; 64];
        let read =
            unsafe { libc::read(libc::STDIN_FILENO, chunk.as_mut_ptr().cast(), chunk.len()) };
        let Ok(read @ 1..) = usize::try_from(read) else {
            break;
        };
        said.extend_from_slice(&chunk[..read]);
    }
    said
}

/// The colour in an OSC 11 answer `said`, its `rgb:` parts cut to their first two hex digits.
fn parse(said: &[u8]) -> Option<(u8, u8, u8)> {
    let text = String::from_utf8_lossy(said);
    let (_, rgb) = text.split_once("rgb:")?;
    let mut parts = rgb
        .split(['/', '\x07', '\x1b'])
        .map(|p| u8::from_str_radix(p.get(..2)?, 16).ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}

#[cfg(test)]
mod tests;
