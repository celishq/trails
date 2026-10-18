//! How a step looks in every view: the mark and looks of its state, the signs of its flags, and the
//! spinner of a working one.

use super::canvas::{Styled, BOLD, DIM, GREEN, RED, YELLOW};
use crate::progress::Flags;
use celis_trails::State;
use ratatui::style::{Color, Style};
use std::time::Duration;

/// How a step in a state prints: its mark, the look of its mark, and the look of its title.
pub(crate) struct Look {
    pub mark: &'static str,
    pub text: Style,
    pub title: Style,
}

/// How a step or a trail in `state` prints.
pub(crate) fn look(state: State) -> Look {
    let (mark, text, title) = match state {
        State::Open => ("○", Style::new(), Style::new()),
        State::Working => ("◐", YELLOW, BOLD),
        State::Done => ("✓", GREEN, Style::new()),
        State::Dropped => ("✗", RED, DIM),
        State::Expired => ("◌", RED, DIM),
    };
    Look { mark, text, title }
}

/// The signs of `flags` with their looks: `↻` when revised, then `~` when changed.
pub(crate) fn signs(flags: Flags) -> Vec<(&'static str, Style)> {
    let all = [(flags.revised, "↻", GREEN), (flags.changed, "~", YELLOW)];
    all.into_iter()
        .filter(|(on, ..)| *on)
        .map(|(_, sign, style)| (sign, style))
        .collect()
}

/// The frames of the spinner of a working step.
const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// What stands for the spinner in a text until a view draws its frame.
pub(crate) const SPUN: &str = "\u{E000}";

/// The look of the spinner: bright yellow.
pub(crate) const SPINNER: Style = Style::new().fg(Color::LightYellow);

/// The time between two frames of the spinner.
pub(crate) const TURN: Duration = Duration::from_millis(100);

/// The spinner at its frame `frame`.
pub(crate) fn frame(frame: usize) -> &'static str {
    SPIN[frame % SPIN.len()]
}

/// `lines` with each spinner in them at its frame `frame`.
pub(crate) fn spinner(lines: &[Styled], frame: usize) -> Vec<Styled> {
    let mut lines = lines.to_vec();
    for span in lines.iter_mut().flat_map(|l| l.spans.iter_mut()) {
        if span.content.contains(SPUN) {
            span.content = span.content.replace(SPUN, self::frame(frame)).into();
        }
    }
    lines
}

/// The spinner's frame at this moment.
pub(crate) fn spin() -> usize {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| (d.as_millis() / TURN.as_millis()) as usize)
}
