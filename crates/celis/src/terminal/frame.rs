//! A frame of rows, as stats and report draw their cards in.

use super::canvas::{clip, cut, lines, Styled, BOLD, DIM};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Widget};

/// The row that draws a line across a frame.
pub(crate) const PATTERN: &str = "\u{1}";

/// A frame: its title in its top line, its rows inside it, and whether it is picked, which draws it in
/// heavy lines.
pub(crate) struct Frame {
    pub title: String,
    pub rows: Vec<Styled>,
    pub picked: bool,
}

impl Frame {
    /// Its `height` lines, `width` wide: each row padded or cut to fit, and empty rows after them; the
    /// frame dim, or bold when picked, and a `RULE` row a line across it.
    pub fn lines(&self, width: usize, height: usize) -> Vec<Styled> {
        let mut buf = Buffer::empty(Rect::new(0, 0, size(width), size(height)));
        self.render(buf.area, &mut buf);
        lines(&buf)
    }

    /// Draws it over `area` of `buf`.
    pub(crate) fn render(&self, area: Rect, buf: &mut Buffer) {
        let (border, frame, [left, edge, right]) = match self.picked {
            true => (BorderType::Thick, BOLD, ["┣", "━", "┫"]),
            false => (BorderType::Plain, DIM, ["├", "─", "┤"]),
        };
        Block::bordered()
            .border_type(border)
            .border_style(frame)
            .render(area, buf);
        let width = usize::from(area.width);
        let title = cut(&self.title, width.saturating_sub(6));
        buf.set_string(area.x + 2, area.y, " ", Style::reset());
        buf.set_string(area.x + 3, area.y, &title, BOLD);
        let x = area.x + 3 + title.chars().count() as u16;
        buf.set_string(x, area.y, " ", Style::reset());
        let inside = width.saturating_sub(4);
        for (n, row) in self.rows.iter().enumerate() {
            let y = area.y + 1 + n as u16;
            if y + 1 >= area.bottom() {
                break;
            }
            match row.to_string() == PATTERN {
                true => {
                    let line = format!("{left}{}{right}", edge.repeat(width - 2));
                    buf.set_string(area.x, y, line, frame);
                }
                false => {
                    buf.set_line(area.x + 2, y, &clip(row, inside), size(inside));
                }
            }
        }
    }
}

/// `n` as a size on the screen.
fn size(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
