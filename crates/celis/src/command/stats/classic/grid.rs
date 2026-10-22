//! Frames side by side, as the cards of stats draw them.

use crate::terminal::canvas::{lines, Styled};
use crate::terminal::frame::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// `frames` side by side, `per_row` a row and a space between two, each `width` wide and each row of
/// frames as tall as its tallest frame.
pub(crate) fn grid(frames: &[Frame], per_row: usize, width: usize) -> Vec<Styled> {
    let mut drawn = vec![];
    for row in frames.chunks(per_row.max(1)) {
        let height = row.iter().map(|c| c.rows.len()).max().unwrap_or(0) + 2;
        let all = row.len() * (width + 1) - 1;
        let mut buf = Buffer::empty(Rect::new(0, 0, size(all), size(height)));
        for (n, card) in row.iter().enumerate() {
            let x = size(n * (width + 1));
            card.render(Rect::new(x, 0, size(width), size(height)), &mut buf);
        }
        drawn.extend(lines(&buf));
    }
    drawn
}

/// `n` as a size on the screen.
fn size(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
