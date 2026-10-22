//! How a step looks in the views of v0.0.11: the terminal's own colours, the mark and looks of each
//! state, the signs of its flags, and the spinner's colour.

use super::canvas::{BOLD, DIM};
use super::look::Look;
use crate::progress::Flags;
use celis_trails::State;
use ratatui::style::{Color, Style};

pub(crate) const GREEN: Style = Style::new().fg(Color::Green);
pub(crate) const YELLOW: Style = Style::new().fg(Color::Yellow);
pub(crate) const MAGENTA: Style = Style::new().fg(Color::Magenta);
pub(crate) const RED: Style = Style::new().fg(Color::Red);

/// The look of the spinner: bright yellow.
pub(crate) const SPINNER: Style = Style::new().fg(Color::LightYellow);

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
