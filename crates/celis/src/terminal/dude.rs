//! The celis dude: the app's logo outline in 14 by 6 quadrant pixels, 7 by 3 cells, with its two eyes
//! two pixels wide and two tall, blinking.

use super::canvas::Styled;
use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// The cells across the dude.
pub(crate) const COLUMNS: usize = 7;

/// The dude with its eyes open: `█` a body cell, a quadrant glyph the part of one, a space the screen.
const OPEN: [&str; 3] = ["▄████▙▖", "██ █ ██", " ▀████▀"];

/// The dude with its eyes shut.
const SHUT: [&str; 3] = ["▄████▙▖", "███████", " ▀████▀"];

/// The ticks of one blink, and the ticks between blinks.
const BLINK: (usize, usize) = (2, 34);

/// The dude's rows at the spinner's frame `tick`, blinking once each `BLINK` ticks.
pub(crate) fn dude(tick: usize) -> [Styled; 3] {
    let shut = tick % BLINK.1 >= BLINK.1 - BLINK.0;
    let rows = if shut { SHUT } else { OPEN };
    rows.map(|row| Styled::from(row.chars().map(cell).collect::<Vec<_>>()))
}

/// The span of one cell of the dude: a body cell as a white background, a part of one as a white glyph.
fn cell(glyph: char) -> Span<'static> {
    match glyph {
        '█' => Span::styled(" ", Style::new().bg(Color::White)),
        ' ' => Span::raw(" "),
        half => Span::styled(half.to_string(), Style::new().fg(Color::White)),
    }
}

#[cfg(test)]
mod tests;
