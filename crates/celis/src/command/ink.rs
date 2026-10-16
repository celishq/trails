//! How a card prints its text: dim, bold, and its numbers big or plain.

use super::card::paint;
use super::glyph::digits;
use super::tree::visible;

/// How a card prints: its numbers as plain text, and in colour.
#[derive(Clone, Copy)]
pub(super) struct Ink {
    pub plain: bool,
    pub colour: bool,
}

impl Ink {
    pub fn paint(self, code: &str, text: &str) -> String {
        paint(self.colour, code, text)
    }

    pub fn dim(self, text: &str) -> String {
        self.paint("2", text)
    }

    /// `text` big and bold: three rows of digits, or one row when plain.
    pub fn big(self, text: &str) -> Vec<String> {
        match self.plain {
            true => vec![self.paint("1", text)],
            false => digits(text).iter().map(|r| self.paint("1", r)).collect(),
        }
    }

    /// `text` big, each of its rows followed by the line of `side` beside it, `gap` spaces after the
    /// widest row.
    pub fn beside(self, text: &str, side: &[String], gap: usize) -> Vec<String> {
        let big = self.big(text);
        let room = big.iter().map(|r| visible(r)).max().unwrap_or(0) + gap;
        big.iter()
            .enumerate()
            .map(|(i, r)| {
                let pad = " ".repeat(room - visible(r));
                format!("{r}{pad}{}", side.get(i).map_or("", String::as_str))
            })
            .collect()
    }
}
