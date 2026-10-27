//! How a step looks in the views of v0.0.11: the terminal's own colours, the mark and looks of each
//! state, and the spinner's colour.

use super::canvas::{BOLD, DIM};
use super::look::Look;
use crate::progress::Phase;
use ratatui::style::{Color, Style};

pub(crate) const GREEN: Style = Style::new().fg(Color::Green);
pub(crate) const YELLOW: Style = Style::new().fg(Color::Yellow);
pub(crate) const MAGENTA: Style = Style::new().fg(Color::Magenta);
pub(crate) const RED: Style = Style::new().fg(Color::Red);

/// The look of the spinner: bright yellow.
pub(crate) const SPINNER: Style = Style::new().fg(Color::LightYellow);

/// How a step or a trail in `state` prints.
pub(crate) fn look(state: Phase) -> Look {
    let (mark, text, title) = match state {
        Phase::Open => ("○", Style::new(), Style::new()),
        Phase::Started => ("◐", YELLOW, BOLD),
        Phase::Done => ("✓", GREEN, Style::new()),
        Phase::Dropped => ("✗", RED, DIM),
        Phase::Stale => ("◌", RED, DIM),
    };
    Look { mark, text, title }
}
