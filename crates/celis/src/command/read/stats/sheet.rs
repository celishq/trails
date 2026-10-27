//! The sheet stats draws its board on: cards, wrapped runs of text, bars and big digits.

use super::measures::{trimmed, Cell, Measures};
use crate::terminal::canvas::{lines, paint, visible, Styled, BOLD, DIM, EDGE};
use crate::terminal::glyph::DIGITS;
use crate::terminal::grid::Grid;
use crate::terminal::unfold::faded_style;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use unicode_width::UnicodeWidthChar;

/// A run of texts kept together on one row.
pub(super) type Group = Vec<Span<'static>>;

/// The columns every legend item takes at least, the widest one, `■ first pass`, so the items of every
/// legend stand in the same columns.
pub(super) const LEGEND: u16 = 12;

/// The blocks that fill an eighth more of a cell, left to right.
const EIGHTHS: [&str; 8] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

/// The blocks that fill an eighth more of a cell, bottom to top.
pub(super) const SPARK: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// A sheet: the buffer it draws on, the columns a text keeps to inside a card, and how far what it
/// draws now has faded in.
pub(super) struct Sheet {
    pub buf: Buffer,
    pub clip: Option<(u16, u16)>,
    pub alpha: f64,
}

impl Sheet {
    pub fn new(width: u16, height: u16) -> Sheet {
        Sheet {
            buf: Buffer::empty(Rect::new(0, 0, width, height)),
            clip: None,
            alpha: 1.0,
        }
    }

    pub fn width(&self) -> u16 {
        self.buf.area.width
    }

    /// Writes `text` from `x` on row `y` in `look`, faded in, each character outside the clip left out;
    /// the column after it.
    pub fn put(&mut self, x: u16, y: u16, text: &str, look: Style) -> u16 {
        let look = match self.alpha < 1.0 {
            true => faded_style(look, self.alpha.max(0.0)),
            false => look,
        };
        if y >= self.buf.area.bottom() {
            let area = self.buf.area;
            self.buf.resize(Rect::new(0, 0, area.width, y + 32));
        }
        let area = self.buf.area;
        let mut x = x;
        for c in text.chars() {
            let wide = c.width().unwrap_or(0) as u16;
            let kept = self.clip.is_none_or(|(low, high)| (low..high).contains(&x));
            if kept && x < area.right() {
                self.buf[(x, y)].reset();
                self.buf.set_string(x, y, c.to_string(), look);
            }
            x += wide;
        }
        x
    }

    /// Writes `spans` from `x` on row `y`, each in its look; the column after the last.
    pub fn spans(&mut self, x: u16, y: u16, spans: &[Span]) -> u16 {
        spans
            .iter()
            .fold(x, |x, s| self.put(x, y, &s.content, s.style))
    }

    /// A box over `w` columns and `h` rows with round corners in `look`, `title` in its top border.
    pub fn boxed(&mut self, (x, y, w, h): (u16, u16, u16, u16), look: Style, title: &[Span]) {
        let line = "─".repeat(usize::from(w.saturating_sub(2)));
        self.put(x, y, &format!("╭{line}╮"), look);
        for r in 1..h.saturating_sub(1) {
            self.put(x, y + r, "│", look);
            self.put(x + w - 1, y + r, "│", look);
        }
        self.put(x, y + h.saturating_sub(1), &format!("╰{line}╯"), look);
        self.spans(x + 2, y, title);
    }

    /// A card over `w` columns and `h` rows, `name` upper case and what it shows in its top border, its
    /// inside the columns its texts keep to.
    pub fn card(&mut self, (x, y, w, h): (u16, u16, u16, u16), name: &str, says: &str) {
        self.clip = None;
        let mut title = vec![paint(BOLD, &format!(" {} ", name.to_uppercase()))];
        if !says.is_empty() && name.len() + says.len() + 8 < usize::from(w) {
            title.push(paint(DIM, &format!("· {says} ")));
        }
        self.boxed((x, y, w, h), EDGE, &title);
        self.clip = Some((x + 1, x + w - 1));
    }

    /// Writes `groups` from `x` on row `y` as `place` lays them out `w` wide; the rows used.
    pub fn flow(&mut self, x: u16, y: u16, w: u16, groups: &[Group]) -> u16 {
        let (placed, rows) = place(groups, w);
        for (dx, row, span) in placed {
            self.put(x + dx, y + row, &span.content, span.style);
        }
        rows
    }

    /// The legend of `kinds` from `x` on row `y` as a table of `measures`: each a square in its look
    /// and its name dim, every item `LEGEND` columns at least, so every legend's items stand in the same
    /// columns; the rows it takes.
    pub fn legend(
        &mut self,
        grid: Grid,
        (x, y, right): (u16, u16, u16),
        kinds: &[(&str, Style)],
    ) -> u16 {
        let cells: Vec<Cell> = kinds
            .iter()
            .map(|(name, look)| {
                (
                    vec![paint(*look, "■"), paint(DIM, &format!(" {name}"))],
                    vec![],
                )
            })
            .collect();
        self.measures(grid, (x, y, right), &cells, LEGEND)
    }

    /// Writes `cells` from `x` on row `y` as `Measures` lays them out before `right`, each at least `least`
    /// columns: each lead, and its change a gap after the widest lead of its column; the rows it takes.
    pub fn measures(
        &mut self,
        grid: Grid,
        (x, y, right): (u16, u16, u16),
        cells: &[Cell],
        least: u16,
    ) -> u16 {
        let table = Measures::of(grid, (x, right), cells, least);
        for (((lead, change), (at, row)), from) in
            cells.iter().zip(&table.places).zip(&table.changes)
        {
            self.spans(*at, y + row, lead);
            self.spans(*from, y + row, &trimmed(change));
        }
        table.rows
    }

    /// A solid bar in `look` to `share` of `cells`, its end cut to an eighth of a cell.
    pub fn bar(&mut self, x: u16, y: u16, share: f64, cells: u16, look: Style) {
        let eighths = (share.clamp(0.0, 1.0) * f64::from(cells) * 8.0).round_ties_even() as usize;
        let mut bar = "█".repeat(eighths / 8);
        if !eighths.is_multiple_of(8) {
            bar.push_str(EIGHTHS[eighths % 8]);
        }
        self.put(x, y, &bar, look);
    }

    /// One solid bar of `parts`, each a share and its look, `cells` wide, each part a cell at least when
    /// it holds half a share or more, the largest taking what rounding leaves.
    pub fn stack(&mut self, x: u16, y: u16, parts: &[(f64, Style)], cells: u16) {
        let total: f64 = parts.iter().map(|p| p.0).sum();
        let total = if total > 0.0 { total } else { 1.0 };
        let mut widths: Vec<i64> = parts
            .iter()
            .map(|(p, _)| match *p >= 0.5 {
                true => ((p / total * f64::from(cells)).round_ties_even() as i64).max(1),
                false => 0,
            })
            .collect();
        let most = (0..widths.len()).max_by_key(|&i| (widths[i], std::cmp::Reverse(i)));
        if let Some(most) = most {
            widths[most] += i64::from(cells) - widths.iter().sum::<i64>();
        }
        let mut at = x;
        for (width, (_, look)) in widths.iter().zip(parts) {
            let width = (*width).max(0) as usize;
            at = self.put(at, y, &"█".repeat(width), *look);
        }
    }

    /// `text` as big digits three rows high in bold `look`, a digit each four columns; the column after.
    pub fn big(&mut self, x: u16, y: u16, text: &str, look: Style) -> u16 {
        for (n, c) in text.chars().enumerate() {
            let rows = match c {
                '%' => ["   ", "   ", " % "],
                ':' => ["   ", " ▀ ", " ▀ "],
                '-' => ["   ", "▀▀▀", "   "],
                c => c.to_digit(10).map_or(["   "; 3], |d| DIGITS[d as usize]),
            };
            for (r, row) in rows.iter().enumerate() {
                self.put(x + n as u16 * 4, y + r as u16, row, look.patch(BOLD));
            }
        }
        x + text.chars().count() as u16 * 4
    }

    /// Its rows from `from` up to `to` as lines, a blank one for each row past what it drew.
    pub fn lines(&self, from: u16, to: u16) -> Vec<Styled> {
        let w = usize::from(self.width());
        let rows = to.saturating_sub(from);
        let mut part = Buffer::empty(Rect::new(0, 0, self.width(), rows));
        let start = (usize::from(from) * w).min(self.buf.content.len());
        let end = (usize::from(to) * w).min(self.buf.content.len());
        part.content[..end - start].clone_from_slice(&self.buf.content[start..end]);
        lines(&part)
    }
}

/// Where each text of `groups` goes from the left of a row `w` wide: a group that would pass `w` moves
/// whole to the next row, which drops a lone separator leading it (blank, `·` or `→`) and the spaces
/// before its first text; each text with its column and row, and the rows used.
pub(super) fn place(groups: &[Group], w: u16) -> (Vec<(u16, u16, Span<'static>)>, u16) {
    let (mut x, mut row) = (0u16, 0u16);
    let mut placed = vec![];
    for group in groups {
        let mut parts: Vec<Span<'static>> = group.clone();
        let text: String = parts.iter().map(|p| p.content.as_ref()).collect();
        let size = visible(text.trim_end()) as u16;
        if x > 0 && x + size > w {
            (x, row) = (0, row + 1);
            if parts.len() > 1 && ["", "·", "→"].contains(&parts[0].content.trim()) {
                parts.remove(0);
            }
            let first = parts[0].content.trim_start().to_string();
            parts[0] = Span::styled(first, parts[0].style);
        }
        for part in parts {
            let wide = visible(&part.content) as u16;
            placed.push((x, row, part));
            x += wide;
        }
    }
    (placed, row + 1)
}

#[cfg(test)]
mod tests;
