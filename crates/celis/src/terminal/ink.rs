//! How a card prints its text: dim, bold, and its numbers big or plain.

use super::canvas::{paint, width, Styled, BOLD, DIM};
use super::glyph::digits;
use ratatui::style::Style;
use ratatui::text::Span;

/// How a card prints: its numbers as plain text, and in colour.
#[derive(Clone, Copy)]
pub(crate) struct Ink {
    pub plain: bool,
    pub colour: bool,
}

impl Ink {
    pub fn paint(self, style: Style, text: &str) -> Span<'static> {
        paint(self.colour, style, text)
    }

    pub fn dim(self, text: &str) -> Span<'static> {
        self.paint(DIM, text)
    }

    /// `text` big and bold: three rows of digits, or one row when plain.
    pub fn big(self, text: &str) -> Vec<Span<'static>> {
        match self.plain {
            true => vec![self.paint(BOLD, text)],
            false => digits(text).iter().map(|r| self.paint(BOLD, r)).collect(),
        }
    }

    /// `text` big, each of its rows followed by the line of `side` beside it, `gap` spaces after the
    /// widest row.
    pub fn beside(self, text: &str, side: &[Styled], gap: usize) -> Vec<Styled> {
        let big: Vec<Styled> = self.big(text).into_iter().map(Styled::from).collect();
        let room = big.iter().map(width).max().unwrap_or(0) + gap;
        big.into_iter()
            .enumerate()
            .map(|(i, r)| {
                let mut row = super::canvas::pad(r, room);
                row.spans
                    .extend(side.get(i).map_or(vec![], |s| s.spans.clone()));
                row
            })
            .collect()
    }
}
