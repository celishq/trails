//! A framed card of rows, as stats and report draw it.

use super::tree::{cut, visible};

/// The row that draws a line across a card.
pub(super) const RULE: &str = "\u{1}";

/// A card: its title in its top line, its rows inside its frame, and whether it is picked, which frames
/// it in heavy lines.
pub(super) struct Card {
    pub title: String,
    pub rows: Vec<String>,
    pub picked: bool,
}

impl Card {
    /// Its `height` lines, `width` wide: each row padded or cut to fit, and empty rows after them; the
    /// frame dim, or bold when picked, and a `RULE` row a line across it, in colour when `colour`.
    pub fn lines(&self, width: usize, height: usize, colour: bool) -> Vec<String> {
        let inside = width.saturating_sub(4);
        let [corner, side, edge, end, low, low_end, left, right] = match self.picked {
            true => ["┏", "┃", "━", "┓", "┗", "┛", "┣", "┫"],
            false => ["┌", "│", "─", "┐", "└", "┘", "├", "┤"],
        };
        let tone = if self.picked { "1" } else { "2" };
        let frame = |text: &str| paint(colour, tone, text);
        let title = cut(&self.title, width.saturating_sub(6));
        let rest = width.saturating_sub(visible(&title) + 5);
        let mut lines = vec![format!(
            "{} {} {}",
            frame(&format!("{corner}{edge}")),
            paint(colour, "1", &title),
            frame(&format!("{}{end}", edge.repeat(rest)))
        )];
        for r in 0..height.saturating_sub(2) {
            if self.rows.get(r).is_some_and(|r| r == RULE) {
                lines.push(frame(&format!("{left}{}{right}", edge.repeat(width - 2))));
                continue;
            }
            let row = self.rows.get(r).map_or(String::new(), |r| cut(r, inside));
            let pad = " ".repeat(inside - visible(&row));
            lines.push(format!("{} {row}{pad} {}", frame(side), frame(side)));
        }
        lines.push(frame(&format!("{low}{}{low_end}", edge.repeat(width - 2))));
        lines
    }
}

/// `cards` side by side, `per_row` a row and a space between two, each `width` wide and each row of
/// cards as tall as its tallest card.
pub(super) fn grid(cards: &[Card], per_row: usize, width: usize, colour: bool) -> Vec<String> {
    let mut lines = vec![];
    for row in cards.chunks(per_row.max(1)) {
        let height = row.iter().map(|c| c.rows.len()).max().unwrap_or(0) + 2;
        let drawn: Vec<Vec<String>> = row.iter().map(|c| c.lines(width, height, colour)).collect();
        for i in 0..height {
            let line: Vec<&str> = drawn.iter().map(|d| d[i].as_str()).collect();
            lines.push(line.join(" "));
        }
    }
    lines
}

/// `text` in the colour `code` when `colour`; else `text`.
pub(super) fn paint(colour: bool, code: &str, text: &str) -> String {
    match colour && !code.is_empty() {
        true => format!("\x1b[{code}m{text}\x1b[0m"),
        false => text.to_string(),
    }
}

#[cfg(test)]
mod tests;
